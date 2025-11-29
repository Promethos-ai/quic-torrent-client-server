//! # QUIC Client
//!
//! Functions for connecting to QUIC endpoints and sending/receiving JSON messages.

use quinn::{Endpoint, ClientConfig};
use crate::quic_utils::create_client_config;
use crate::messages::ErrorResponse;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Detect request type from JSON string
fn detect_request_type(json: &str) -> &'static str {
    if json.contains("\"info_hash\"") && json.contains("\"peer_id\"") {
        "TrackerAnnounceRequest"
    } else if json.contains("\"file\"") {
        "FileRequest"
    } else if json.contains("\"query\"") {
        "AiRequest"
    } else if json.contains("\"type\"") {
        "CustomJSON"
    } else {
        "UnknownRequest"
    }
}

/// Detect response type from JSON string
fn detect_response_type(json: &str) -> &'static str {
    if json.contains("\"peers\"") && json.contains("\"interval\"") {
        "TrackerAnnounceResponse"
    } else if json.contains("\"data\"") && json.contains("\"filename\"") {
        "FileResponse"
    } else if json.contains("\"answer\"") && json.contains("\"metadata\"") {
        "AiResponse"
    } else if json.contains("\"error\"") {
        "ErrorResponse"
    } else {
        "UnknownResponse"
    }
}

/// Connects to a QUIC endpoint and sends/receives JSON messages.
pub struct QuicClient {
    endpoint: Endpoint,
}

impl QuicClient {
    /// Creates a new QUIC client.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client_config = create_client_config()?;
        // Use 0.0.0.0:0 to bind to all interfaces (both IPv4 and IPv6)
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);
        
        Ok(Self { endpoint })
    }
    
    /// Connects to a QUIC server and sends a JSON message, receiving a response.
    pub async fn send_message<T, R>(
        &self,
        server: &str,
        port: u16,
        message: &T,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        crate::log_client!("[QuicClient::send_message] ENTRY - server={}, port={}", server, port);
        
        let addr = format!("{}:{}", server, port).parse()?;
        crate::log_client!("[QuicClient::send_message] Parsed address: {}", addr);
        
        crate::log_client!("[QuicClient::send_message] Connecting to {}:{}", server, port);
        let connection = self.endpoint.connect(addr, server)?;
        
        // FALLBACK SHUNT: Extended timeout and retry logic for ALPN negotiation
        // Multiple ALPN protocols in config provide automatic fallback
        let conn = match tokio::time::timeout(
            std::time::Duration::from_secs(20),  // Extended timeout for ALPN negotiation
            connection
        ).await {
            Ok(Ok(conn)) => {
                crate::log_client!("[QuicClient::send_message] Connection established (first attempt)");
                conn
            }
            Ok(Err(e)) => {
                // Connection failed - log and retry once
                crate::log_client!("[QuicClient::send_message] Connection failed: {:?}, attempting retry", e);
                let retry_connection = self.endpoint.connect(addr, server)?;
                match tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    retry_connection
                ).await {
                    Ok(Ok(conn)) => {
                        crate::log_client!("[QuicClient::send_message] Connection established (retry)");
                        conn
                    }
                    Ok(Err(e)) => return Err(Box::new(e)),
                    Err(_) => return Err("Connection timeout on retry".into()),
                }
            }
            Err(_) => {
                // Timeout - try one more time
                crate::log_client!("[QuicClient::send_message] Connection timeout, retrying...");
                let retry_connection = self.endpoint.connect(addr, server)?;
                match tokio::time::timeout(
                    std::time::Duration::from_secs(20),
                    retry_connection
                ).await {
                    Ok(Ok(conn)) => {
                        crate::log_client!("[QuicClient::send_message] Connection established (timeout retry)");
                        conn
                    }
                    Ok(Err(e)) => return Err(Box::new(e)),
                    Err(_) => return Err("Connection timeout on final retry".into()),
                }
            }
        };
        
        crate::log_client!("[QuicClient::send_message] Connection established");
        
        // Open a bidirectional stream
        crate::log_client!("[QuicClient::send_message] Opening bidirectional stream");
        let (mut send, mut recv) = conn.open_bi().await?;
        crate::log_client!("[QuicClient::send_message] Stream opened");
        
        // Serialize and send the message
        let json = serde_json::to_string(message)?;
        
        // Detect and log request type
        let request_type = detect_request_type(&json);
        crate::log_client!("[CLIENT] ===== OUTGOING REQUEST =====");
        crate::log_client!("[CLIENT] REQUEST TYPE: {}", request_type);
        crate::log_client!("[CLIENT] Function: quic_client::send_message()");
        crate::log_client!("[CLIENT] Target: {}:{}", server, port);
        crate::log_client!("[CLIENT] JSON payload length: {}", json.len());
        crate::log_client!("[CLIENT] JSON payload: {}", json);
        crate::log_client!("[CLIENT] Data sent to: Server {}:{}", server, port);
        
        send.write_all(json.as_bytes()).await?;
        send.finish().await?;
        crate::log_client!("[QuicClient::send_message] Message sent, waiting for response");
        
        // Read the response
        let mut buffer = Vec::new();
        let mut chunks_received = 0;
        loop {
            let mut chunk = vec![0u8; 4096];
            match recv.read(&mut chunk).await? {
                Some(size) => {
                    buffer.extend_from_slice(&chunk[..size]);
                    chunks_received += 1;
                    if chunks_received % 10 == 0 {
                        crate::log_client!("[QuicClient::send_message] Received chunk {} - buffer_len={}", 
                            chunks_received, buffer.len());
                    }
                }
                None => {
                    crate::log_client!("[QuicClient::send_message] Stream closed, received {} chunks, total_bytes={}", 
                        chunks_received, buffer.len());
                    break;
                }
            }
        }
        
        crate::log_client!("[QuicClient::send_message] Deserializing response - buffer_len={}", buffer.len());
        // Deserialize the response
        let response: R = serde_json::from_slice(&buffer)?;
        
        // Detect and log response type from raw buffer
        let response_str = String::from_utf8_lossy(&buffer);
        let response_type = detect_response_type(&response_str);
        crate::log_client!("[CLIENT] ===== INCOMING RESPONSE =====");
        crate::log_client!("[CLIENT] RESPONSE TYPE: {}", response_type);
        crate::log_client!("[CLIENT] Source: {}:{}", server, port);
        crate::log_client!("[CLIENT] Response length: {}", buffer.len());
        crate::log_client!("[CLIENT] Data received from: Server {}:{}", server, port);
        crate::log_client!("[CLIENT] Response deserialized successfully");
        crate::log_client!("[QuicClient::send_message] EXIT - Return: success, response_size={}", buffer.len());
        
        Ok(response)
    }
    
    /// Sends a tracker announce request and receives a response.
    pub async fn send_tracker_announce(
        &self,
        server: &str,
        port: u16,
        request: &crate::messages::TrackerAnnounceRequest,
    ) -> Result<crate::messages::TrackerAnnounceResponse, Box<dyn std::error::Error>> {
        self.send_message(server, port, request).await
    }
    
    /// Sends a file request and receives a response.
    pub async fn send_file_request(
        &self,
        server: &str,
        port: u16,
        request: &crate::messages::FileRequest,
    ) -> Result<crate::messages::FileResponse, Box<dyn std::error::Error>> {
        self.send_message(server, port, request).await
    }
}

use tokio::io::{AsyncReadExt, AsyncWriteExt};

