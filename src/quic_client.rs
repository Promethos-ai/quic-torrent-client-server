//! # QUIC Client
//!
//! Functions for connecting to QUIC endpoints and sending/receiving JSON messages.

use quinn::{Endpoint, ClientConfig};
use crate::quic_utils::create_client_config;
use crate::messages::ErrorResponse;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

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
        let conn = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            connection
        ).await??;
        crate::log_client!("[QuicClient::send_message] Connection established");
        
        // Open a bidirectional stream
        crate::log_client!("[QuicClient::send_message] Opening bidirectional stream");
        let (mut send, mut recv) = conn.open_bi().await?;
        crate::log_client!("[QuicClient::send_message] Stream opened");
        
        // Serialize and send the message
        let json = serde_json::to_string(message)?;
        crate::log_client!("[QuicClient::send_message] Serialized message - json_len={}", json.len());
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
        crate::log_client!("[QuicClient::send_message] Response deserialized successfully");
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

