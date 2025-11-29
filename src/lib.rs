//! # QUIC Torrent Client & Server
//!
//! A minimal BitTorrent tracker and client implementation using QUIC protocol.
//! - QUIC tracker server
//! - QUIC client for downloading files
//! - JSON message format over QUIC streams

use std::collections::BTreeMap;

/// Encodes a byte string into bencode format.
pub fn bencode_string(s: &[u8]) -> Vec<u8> {
    let mut result = format!("{}:", s.len()).into_bytes();
    result.extend_from_slice(s);
    result
}

/// Encodes an integer into bencode format.
pub fn bencode_int(i: i64) -> Vec<u8> {
    format!("i{}e", i).into_bytes()
}

/// Encodes a dictionary into bencode format.
pub fn bencode_dict(d: &BTreeMap<Vec<u8>, BencodeValue>) -> Vec<u8> {
    let mut result = vec![b'd'];
    for (key, value) in d {
        result.extend_from_slice(&bencode_string(key));
        result.extend_from_slice(&value.encode());
    }
    result.push(b'e');
    result
}

/// Represents a bencode value (string, integer, or dictionary).
#[derive(Clone)]
pub enum BencodeValue {
    String(Vec<u8>),
    Int(i64),
    Dict(BTreeMap<Vec<u8>, BencodeValue>),
}

impl BencodeValue {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            BencodeValue::String(s) => bencode_string(s),
            BencodeValue::Int(i) => bencode_int(*i),
            BencodeValue::Dict(d) => bencode_dict(d),
        }
    }
}

/// Decodes a bencode value from bytes.
pub fn decode_bencode(data: &[u8]) -> Result<(BencodeValue, usize), Box<dyn std::error::Error>> {
    if data.is_empty() {
        return Err("Empty data".into());
    }
    
    match data[0] {
        b'i' => {
            let end = data.iter().position(|&b| b == b'e')
                .ok_or("Missing 'e' terminator for integer")?;
            let num_str = std::str::from_utf8(&data[1..end])?;
            let num = num_str.parse::<i64>()?;
            Ok((BencodeValue::Int(num), end + 1))
        }
        b'd' => {
            let mut map = BTreeMap::new();
            let mut pos = 1;
            
            while pos < data.len() && data[pos] != b'e' {
                let (key, key_len) = decode_bencode(&data[pos..])?;
                let key_bytes = match key {
                    BencodeValue::String(s) => s,
                    _ => return Err("Dictionary key must be a string".into()),
                };
                pos += key_len;
                
                let (value, value_len) = decode_bencode(&data[pos..])?;
                pos += value_len;
                
                map.insert(key_bytes, value);
            }
            
            if pos >= data.len() || data[pos] != b'e' {
                return Err("Missing 'e' terminator for dictionary".into());
            }
            
            Ok((BencodeValue::Dict(map), pos + 1))
        }
        b'0'..=b'9' => {
            let colon_pos = data.iter().position(|&b| b == b':')
                .ok_or("Missing ':' in string encoding")?;
            let len_str = std::str::from_utf8(&data[0..colon_pos])?;
            let len = len_str.parse::<usize>()?;
            
            let start = colon_pos + 1;
            let end = start + len;
            
            if end > data.len() {
                return Err("String length exceeds available data".into());
            }
            
            let string_data = data[start..end].to_vec();
            Ok((BencodeValue::String(string_data), end))
        }
        _ => Err(format!("Unknown bencode type: {}", data[0] as char).into()),
    }
}

#[macro_use]
pub mod logger;
pub mod quic_utils;
pub mod quic_tracker;
pub mod quic_client;
pub mod messages;
pub mod client;
pub mod console_client;
pub mod ai_processor;
pub mod work_distribution;

// Logging macros
#[macro_export]
macro_rules! log_server {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::logger::get_logger() {
            let _ = logger.server_log(&format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_server_received {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::logger::get_logger() {
            let _ = logger.server_received(&format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_client {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::logger::get_logger() {
            let _ = logger.client_log(&format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_client_sent {
    ($($arg:tt)*) => {
        if let Some(logger) = $crate::logger::get_logger() {
            let _ = logger.client_sent(&format!($($arg)*));
        }
    };
}




