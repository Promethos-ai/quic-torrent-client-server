//! # Message Types
//!
//! JSON message structures for QUIC communication between client and services.

use serde::{Deserialize, Serialize};

/// AI query request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// The user's query text
    pub query: String,
    /// Optional conversation context (previous messages)
    pub context: Option<Vec<MessageContext>>,
    /// Optional parameters for AI processing
    pub parameters: Option<AiParameters>,
}

/// Context for a message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContext {
    /// Role: "user", "assistant", or "system"
    pub role: String,
    /// Message content
    pub content: String,
}

/// Parameters for AI processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiParameters {
    /// Temperature for sampling (0.0 to 2.0)
    pub temperature: Option<f64>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
    /// Top-p sampling parameter
    pub top_p: Option<f64>,
}

/// AI query response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// The generated answer
    pub answer: String,
    /// Metadata about the processing
    pub metadata: Option<ResponseMetadata>,
}

/// Metadata about the AI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Number of tokens used for input
    pub input_tokens: Option<usize>,
    /// Number of tokens generated
    pub output_tokens: Option<usize>,
    /// Total tokens used
    pub total_tokens: Option<usize>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
}

/// Error response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Optional error code
    pub code: Option<String>,
}

/// Tracker announce request (JSON format for QUIC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerAnnounceRequest {
    pub info_hash: String,
    pub peer_id: String,
    pub port: u16,
    pub uploaded: Option<u64>,
    pub downloaded: Option<u64>,
    pub left: Option<u64>,
    pub event: Option<String>,
    pub ip: Option<String>,
}

/// Tracker announce response (JSON format for QUIC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerAnnounceResponse {
    pub interval: u64,
    pub peers: Vec<PeerInfo>,
    pub complete: u64,
    pub incomplete: u64,
}

/// Peer information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
}

/// File request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequest {
    pub file: String,
}

/// File response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResponse {
    pub data: Vec<u8>,
    pub filename: String,
    pub size: usize,
}


