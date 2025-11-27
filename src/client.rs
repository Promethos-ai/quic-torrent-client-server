//! # BitTorrent Client
//!
//! Example client implementation for downloading files.

use std::fs;
use rand;
use sha1::{Sha1, Digest};
use crate::{decode_bencode, BencodeValue};

#[derive(Clone)]
pub struct TorrentFile {
    pub announce: String,
    pub info_hash: String,
    pub piece_length: usize,
    pub pieces: Vec<Vec<u8>>,
    pub length: usize,
    pub name: String,
}

impl TorrentFile {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let torrent_data = fs::read(path)?;
        let (torrent_value, _) = decode_bencode(&torrent_data)?;
        
        let torrent_dict = match torrent_value {
            BencodeValue::Dict(d) => d,
            _ => return Err("Torrent file must be a dictionary".into()),
        };
        
        // Get announce URL
        let announce_bytes = torrent_dict.get(b"announce".as_slice())
            .ok_or("Missing 'announce' field")?;
        let announce = match announce_bytes {
            BencodeValue::String(s) => String::from_utf8(s.clone())?,
            _ => return Err("'announce' must be a string".into()),
        };
        
        // Get info dictionary
        let info_value = torrent_dict.get(b"info".as_slice())
            .ok_or("Missing 'info' field")?;
        let info_dict = match info_value {
            BencodeValue::Dict(d) => d,
            _ => return Err("'info' must be a dictionary".into()),
        };
        
        // Calculate info hash (SHA-1 of bencoded info dictionary)
        let info_bencoded = info_value.encode();
        let mut hasher = Sha1::new();
        hasher.update(&info_bencoded);
        let info_hash = hex::encode(hasher.finalize());
        
        // Get name
        let name_bytes = info_dict.get(b"name".as_slice())
            .ok_or("Missing 'name' field in info")?;
        let name = match name_bytes {
            BencodeValue::String(s) => String::from_utf8(s.clone())?,
            _ => return Err("'name' must be a string".into()),
        };
        
        // Get length
        let length_value = info_dict.get(b"length".as_slice())
            .ok_or("Missing 'length' field in info")?;
        let length = match length_value {
            BencodeValue::Int(i) => *i as usize,
            _ => return Err("'length' must be an integer".into()),
        };
        
        // Get piece length
        let piece_length_value = info_dict.get(b"piece length".as_slice())
            .ok_or("Missing 'piece length' field in info")?;
        let piece_length = match piece_length_value {
            BencodeValue::Int(i) => *i as usize,
            _ => return Err("'piece length' must be an integer".into()),
        };
        
        // Get pieces (concatenated SHA-1 hashes, each 20 bytes)
        let pieces_bytes = info_dict.get(b"pieces".as_slice())
            .ok_or("Missing 'pieces' field in info")?;
        let pieces_data = match pieces_bytes {
            BencodeValue::String(s) => s.clone(),
            _ => return Err("'pieces' must be a string".into()),
        };
        
        let mut pieces = Vec::new();
        for chunk in pieces_data.chunks(20) {
            if chunk.len() == 20 {
                pieces.push(chunk.to_vec());
            }
        }
        
        Ok(Self {
            announce,
            info_hash,
            piece_length,
            pieces,
            length,
            name,
        })
    }
}


#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
}

pub async fn download_file(
    torrent_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    download_file_quic_torrent(torrent_path, output_path, "127.0.0.1", 7001).await
}


/// Announces to a QUIC tracker server.
///
/// # Arguments
/// * `server` - Tracker server hostname or IP address
/// * `port` - Tracker server port (default 7001 for QUIC tracker)
/// * `info_hash` - The torrent info hash
/// * `peer_id` - Unique peer identifier
/// * `peer_port` - Port this peer is listening on
/// * `uploaded` - Bytes uploaded
/// * `downloaded` - Bytes downloaded
/// * `left` - Bytes remaining to download
///
/// # Returns
/// List of peer information
pub async fn announce_to_quic_tracker(
    server: &str,
    port: u16,
    info_hash: &str,
    peer_id: &str,
    peer_port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
) -> Result<Vec<PeerInfo>, Box<dyn std::error::Error>> {
    crate::log_client!("[announce_to_quic_tracker] ENTRY - server={}, port={}, info_hash={}, peer_id={}, peer_port={}, uploaded={}, downloaded={}, left={}", 
        server, port, info_hash, peer_id, peer_port, uploaded, downloaded, left);
    crate::log_client_sent!("Sending QUIC announce request to {}:{} - info_hash={}, peer_id={}", 
        server, port, info_hash, peer_id);
    
    let request = crate::messages::TrackerAnnounceRequest {
        info_hash: info_hash.to_string(),
        peer_id: peer_id.to_string(),
        port: peer_port,
        uploaded: Some(uploaded),
        downloaded: Some(downloaded),
        left: Some(left),
        event: Some("started".to_string()),
        ip: None,
    };
    
    crate::log_client!("[announce_to_quic_tracker] Created TrackerAnnounceRequest, creating QUIC client");
    let client = crate::quic_client::QuicClient::new()?;
    crate::log_client!("[announce_to_quic_tracker] QUIC client created, sending message to {}:{}", server, port);
    
    let response: crate::messages::TrackerAnnounceResponse = 
        client.send_message(server, port, &request).await?;
    
    crate::log_client!("[announce_to_quic_tracker] Received response - peers_count={}, complete={}, incomplete={}, interval={}", 
        response.peers.len(), response.complete, response.incomplete, response.interval);
    crate::log_client!("Received QUIC announce response: {} peers", response.peers.len());
    
    // Convert to PeerInfo format
    let peers: Vec<PeerInfo> = response.peers
        .into_iter()
        .map(|p| PeerInfo {
            ip: p.ip.clone(),
            port: p.port,
        })
        .collect();
    
    crate::log_client!("[announce_to_quic_tracker] EXIT - success=true, peers_count={}", peers.len());
    crate::log_client!("[announce_to_quic_tracker] Return: Vec<PeerInfo> with {} peers", peers.len());
    
    Ok(peers)
}

/// Downloads a file from a QUIC tracker server.
///
/// # Arguments
/// * `server` - Server hostname or IP address
/// * `port` - Server port (default 7001 for QUIC tracker)
/// * `filename` - Name of the file to download
/// * `output_path` - Where to save the downloaded file
///
/// # Returns
/// Ok(()) on success
pub async fn download_file_quic(
    server: &str,
    port: u16,
    filename: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::log_client!("[download_file_quic] ENTRY - server={}, port={}, filename={}, output_path={}", 
        server, port, filename, output_path);
    crate::log_client_sent!("Requesting file download via QUIC from {}:{} - file: {}", 
        server, port, filename);
    
    let request = crate::messages::FileRequest {
        file: filename.to_string(),
    };
    
    crate::log_client!("[download_file_quic] Created FileRequest, creating QUIC client");
    let client = crate::quic_client::QuicClient::new()?;
    crate::log_client!("[download_file_quic] QUIC client created, sending file request to {}:{}", server, port);
    
    let response: crate::messages::FileResponse = 
        client.send_message(server, port, &request).await?;
    
    crate::log_client!("[download_file_quic] Received file response - size={}, filename={}", 
        response.size, response.filename);
    crate::log_client!("Received file via QUIC: {} bytes", response.size);
    
    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        crate::log_client!("[download_file_quic] Creating output directory: {:?}", parent);
        fs::create_dir_all(parent)?;
    }
    
    crate::log_client!("[download_file_quic] Writing file to: {}", output_path);
    fs::write(output_path, &response.data)?;
    crate::log_client!("[download_file_quic] File written successfully - {} bytes", response.data.len());
    crate::log_client!("File saved successfully to: {}", output_path);
    
    crate::log_client!("[download_file_quic] EXIT - success=true, file_size={}", response.size);
    
    Ok(())
}

/// Downloads a file using a torrent file via QUIC.
///
/// This is the QUIC version of download_file, using QUIC for all communication.
pub async fn download_file_quic_torrent(
    torrent_path: &str,
    output_path: &str,
    tracker_server: &str,
    tracker_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::log_client!("[download_file_quic_torrent] ENTRY - torrent_path={}, output_path={}, tracker_server={}, tracker_port={}", 
        torrent_path, output_path, tracker_server, tracker_port);
    
    crate::log_client!("[download_file_quic_torrent] Parsing torrent file: {}", torrent_path);
    let torrent = TorrentFile::from_file(torrent_path)?;
    
    crate::log_client!("[download_file_quic_torrent] Torrent parsed - name={}, info_hash={}, length={}, piece_length={}, pieces_count={}", 
        torrent.name, torrent.info_hash, torrent.length, torrent.piece_length, torrent.pieces.len());
    crate::log_client!("Starting QUIC download: file={}, info_hash={}, size={} bytes", 
        torrent.name, torrent.info_hash, torrent.length);
    println!("Downloading torrent via QUIC: {}", torrent.name);
    println!("Info hash: {}", torrent.info_hash);
    println!("File size: {} bytes", torrent.length);
    
    let peer_id = format!("-ST0001-{}", rand::random::<u64>());
    crate::log_client!("[download_file_quic_torrent] Generated peer_id: {}", peer_id);
    crate::log_client!("Generated peer_id: {}", peer_id);
    
    // Announce to QUIC tracker
    crate::log_client!("[download_file_quic_torrent] Announcing to QUIC tracker: {}:{}", tracker_server, tracker_port);
    println!("Announcing to QUIC tracker: {}:{}", tracker_server, tracker_port);
    let _peers = announce_to_quic_tracker(
        tracker_server,
        tracker_port,
        &torrent.info_hash,
        &peer_id,
        6881,
        0,
        0,
        torrent.length as u64,
    ).await?;
    crate::log_client!("[download_file_quic_torrent] Announce complete - peers_count={}", _peers.len());
    println!("Announced successfully");
    
    // Extract filename
    let filename = std::path::Path::new(&torrent.name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&torrent.name);
    
    crate::log_client!("[download_file_quic_torrent] Extracted filename: {}", filename);
    
    // Download file via QUIC
    crate::log_client!("[download_file_quic_torrent] Starting file download via QUIC");
    println!("Downloading file from QUIC server: {}:{}", tracker_server, tracker_port);
    download_file_quic(tracker_server, tracker_port, filename, output_path).await?;
    
    crate::log_client!("[download_file_quic_torrent] Download complete");
    println!("File saved to: {}", output_path);
    println!("Download complete!");
    
    crate::log_client!("[download_file_quic_torrent] EXIT - success=true, output_path={}", output_path);
    
    Ok(())
}

