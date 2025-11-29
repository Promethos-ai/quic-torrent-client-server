//! # QUIC Tracker Server
//!
//! Implements a BitTorrent tracker using the QUIC protocol with JSON message format.
//! QUIC provides:
//! - Built-in encryption (TLS 1.3)
//! - Multiplexing (multiple streams per connection)
//! - Connection migration
//! - Reduced latency compared to TCP

use quinn::{Endpoint, ServerConfig};
use crate::quic_utils::create_server_config;
use crate::messages::{TrackerAnnounceRequest, TrackerAnnounceResponse, PeerInfo, FileRequest, FileResponse, ErrorResponse, AiRequest, AiResponse, ResponseMetadata};
use crate::ai_processor::{AiProcessor, AiProcessingConfig};
use crate::work_distribution::{WorkDistributionManager, NodeCapability};

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
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Debug)]
pub struct Peer {
    pub peer_id: String,
    pub ip: String,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
}

#[derive(Default)]
pub struct TrackerState {
    peers: HashMap<String, Vec<Peer>>, // info_hash -> peers
}

/// Handles a QUIC connection from a client.
///
/// QUIC supports multiple bidirectional streams per connection.
/// This function:
/// 1. Accepts bidirectional streams from the connection
/// 2. Reads JSON requests from the stream
/// 3. Processes announce or file requests
/// 4. Sends JSON responses back through the stream
pub async fn handle_quic_connection(
    connection: quinn::Connection,
    state: Arc<RwLock<TrackerState>>,
    ai_processor: Option<Arc<RwLock<AiProcessor>>>,
    work_dist: Option<Arc<WorkDistributionManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let remote_addr = connection.remote_address();
    crate::log_server!("New QUIC connection established from: {}", remote_addr);
    
    while let Ok(stream) = connection.accept_bi().await {
        crate::log_server!("New bidirectional stream opened from: {}", remote_addr);
        let (mut send, mut recv) = stream;
        let state = Arc::clone(&state);
        let ai_proc = ai_processor.clone();
        let work_dist_clone = work_dist.clone();
        
        tokio::spawn(async move {
            // Read request
            let mut buffer = Vec::new();
            loop {
                let mut chunk = vec![0u8; 4096];
                match recv.read(&mut chunk).await {
                    Ok(Some(size)) => {
                        buffer.extend_from_slice(&chunk[..size]);
                    }
                    Ok(None) => break,
                    Err(e) => {
                        crate::log_server!("ERROR: Error reading QUIC stream from {}: {}", remote_addr, e);
                        return;
                    }
                }
            }
            
            if buffer.is_empty() {
                crate::log_server!("Empty request received from: {}", remote_addr);
                return;
            }
            
            crate::log_server!("Received {} bytes from: {}", buffer.len(), remote_addr);
            
            // Try to parse as JSON request
            let request_str = match String::from_utf8(buffer) {
                Ok(s) => s,
                Err(e) => {
                    crate::log_server!("ERROR: Invalid UTF-8 in request from {}: {}", remote_addr, e);
                    let error = ErrorResponse {
                        error: "Invalid UTF-8 encoding".to_string(),
                        code: Some("INVALID_ENCODING".to_string()),
                    };
                    let _ = send.write_all(&serde_json::to_string(&error).unwrap().into_bytes()).await;
                    return;
                }
            };
            
            // Detect and log request type
            let request_type = detect_request_type(&request_str);
            crate::log_server!("[REQUEST] REQUEST_TYPE: {} - from: {}", request_type, remote_addr);
            crate::log_server!("[REQUEST] JSON payload: {}", request_str);
            
            // Route to appropriate processing module based on request type
            crate::log_server!("[ROUTING] Incoming request detected - type: {}, from: {}", request_type, remote_addr);
            
            // Try to parse as announce request -> Tracker Module
            if let Ok(announce_req) = serde_json::from_str::<TrackerAnnounceRequest>(&request_str) {
                crate::log_server!("[ROUTING] Request type: TrackerAnnounceRequest");
                crate::log_server!("[ROUTING] Routing to: quic_tracker::handle_announce_request()");
                crate::log_server!("[ROUTING] Processing module: Tracker Module");
                crate::log_server_received!("Parsed TrackerAnnounceRequest from: {}", remote_addr);
                handle_announce_request(announce_req, state, &mut send).await;
            }
            // Try to parse as file request -> File Serving Module
            else if let Ok(file_req) = serde_json::from_str::<FileRequest>(&request_str) {
                crate::log_server!("[ROUTING] Request type: FileRequest");
                crate::log_server!("[ROUTING] Routing to: quic_tracker::handle_file_request()");
                crate::log_server!("[ROUTING] Processing module: File Serving Module");
                crate::log_server_received!("Parsed FileRequest from: {} - file: '{}'", remote_addr, file_req.file);
                handle_file_request(file_req, &mut send).await;
            }
            // Try to parse as AI request -> AI Processing Module
            else if let Ok(ai_req) = serde_json::from_str::<AiRequest>(&request_str) {
                crate::log_server!("[ROUTING] Request type: AiRequest");
                crate::log_server!("[ROUTING] Routing to: quic_tracker::handle_ai_request()");
                crate::log_server!("[ROUTING] Processing module: AI Processing Module");
                crate::log_server!("[ROUTING] Handler function: handle_ai_request() -> ai_processor::process_query_sync()");
                handle_ai_request(ai_req, ai_proc, work_dist_clone, &mut send).await;
            }
            else {
                // Unknown request type -> Error Handler Module
                crate::log_server!("[ROUTING] Request type: UnknownRequest");
                crate::log_server!("[ROUTING] Routing to: Error Handler (send_error)");
                crate::log_server!("[ROUTING] Processing module: Error Handler Module");
                crate::log_server!("ERROR: Unknown request type from: {} - request_len={}", remote_addr, request_str.len());
                let error = ErrorResponse {
                    error: "Unknown request type".to_string(),
                    code: Some("UNKNOWN_REQUEST".to_string()),
                };
                let _ = send.write_all(&serde_json::to_string(&error).unwrap().into_bytes()).await;
            }
        });
    }
    
    Ok(())
}

async fn handle_announce_request(
    req: TrackerAnnounceRequest,
    state: Arc<RwLock<TrackerState>>,
    send: &mut quinn::SendStream,
) {
    crate::log_server!("[HANDLER] Function: quic_tracker::handle_announce_request()");
    crate::log_server!("[HANDLER] Module: Tracker Module");
    crate::log_server!("[HANDLER] Processing TrackerAnnounceRequest");
    
    let info_hash = req.info_hash.clone();
    let peer_ip = req.ip.unwrap_or_else(|| "127.0.0.1".to_string());
    
    crate::log_server_received!("Received QUIC announce request from peer_id: {}, info_hash: {}, ip: {}, port: {}, uploaded: {}, downloaded: {}, left: {}, event: {:?}", 
        req.peer_id, info_hash, peer_ip, req.port, 
        req.uploaded.unwrap_or(0), req.downloaded.unwrap_or(0), req.left.unwrap_or(0), req.event);
    
    let peer = Peer {
        peer_id: req.peer_id.clone(),
        ip: peer_ip,
        port: req.port,
        uploaded: req.uploaded.unwrap_or(0),
        downloaded: req.downloaded.unwrap_or(0),
        left: req.left.unwrap_or(0),
    };

    // Update peer list
    {
        let mut state = state.write().unwrap();
        let peers = state.peers.entry(info_hash.clone()).or_insert_with(Vec::new);
        let was_present = peers.iter().any(|p| p.peer_id == peer.peer_id);
        peers.retain(|p| p.peer_id != peer.peer_id);
        if req.event.as_deref() != Some("stopped") {
            peers.push(peer.clone());
            if was_present {
                crate::log_server!("Peer updated (QUIC): peer_id={}, info_hash={}, ip={}, port={}, uploaded={}, downloaded={}, left={}", 
                    peer.peer_id, info_hash, peer.ip, peer.port, peer.uploaded, peer.downloaded, peer.left);
            } else {
                crate::log_server!("Peer registered (QUIC): peer_id={}, info_hash={}, ip={}, port={}, uploaded={}, downloaded={}, left={}", 
                    peer.peer_id, info_hash, peer.ip, peer.port, peer.uploaded, peer.downloaded, peer.left);
            }
        } else {
            crate::log_server!("Peer unregistered (QUIC): peer_id={}, info_hash={}, ip={}, port={}", 
                peer.peer_id, info_hash, peer.ip, peer.port);
        }
        let total_peers = peers.len();
        crate::log_server!("Peer count for info_hash {}: {} peers", info_hash, total_peers);
    }

    // Build response
    let peers_list = {
        let state = state.read().unwrap();
        state.peers.get(&info_hash).cloned().unwrap_or_default()
    };
    
    let peer_infos: Vec<PeerInfo> = peers_list
        .iter()
        .filter(|p| p.peer_id != req.peer_id)
        .map(|p| PeerInfo {
            ip: p.ip.clone(),
            port: p.port,
        })
        .collect();
    
    let response = TrackerAnnounceResponse {
        interval: 60,
        peers: peer_infos.clone(),
        complete: peers_list.iter().filter(|p| p.left == 0).count() as u64,
        incomplete: peers_list.iter().filter(|p| p.left > 0).count() as u64,
    };
    
    let json_response = match serde_json::to_string(&response) {
        Ok(json) => json,
        Err(e) => {
            crate::log_server!("ERROR: Error serializing QUIC announce response: {}", e);
            let error = ErrorResponse {
                error: "Internal server error".to_string(),
                code: Some("SERIALIZATION_ERROR".to_string()),
            };
            serde_json::to_string(&error).unwrap()
        }
    };
    
    crate::log_server!("Sending QUIC announce response: {} peers, {} complete, {} incomplete, interval={}s", 
        peer_infos.len(),
        response.complete,
        response.incomplete,
        response.interval);
    
    let _ = send.write_all(json_response.as_bytes()).await;
    let _ = send.finish().await;
}

async fn handle_file_request(
    req: FileRequest,
    send: &mut quinn::SendStream,
) {
    crate::log_server!("[HANDLER] Function: quic_tracker::handle_file_request()");
    crate::log_server!("[HANDLER] Module: File Serving Module");
    crate::log_server!("[HANDLER] Processing FileRequest");
    crate::log_server_received!("Received QUIC file download request: file='{}'", req.file);
    
    // Get current working directory
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let seed_dir = current_dir.join("seed");
    
    // Sanitize filename to prevent directory traversal
    let sanitized = req.file.replace("..", "").replace("\\", "").replace("/", "");
    let file_path = if sanitized.is_empty() {
        crate::log_server!("No filename specified, using default: hello_world.txt");
        seed_dir.join("hello_world.txt")
    } else {
        seed_dir.join(&sanitized)
    };
    
    crate::log_server!("Resolved file path: {}", file_path.display());
    
    // Check file size limit (5MB max for JSON transfer to avoid timeouts)
    const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5MB
    let file_metadata = match fs::metadata(&file_path) {
        Ok(meta) => meta,
        Err(e) => {
            let error_msg = format!("File not found or unreadable: {} - {}", file_path.display(), e);
            crate::log_server!("ERROR: {}", error_msg);
            let error = ErrorResponse {
                error: format!("File not found: {}", file_path.display()),
                code: Some("FILE_NOT_FOUND".to_string()),
            };
            let json_error = serde_json::to_string(&error).unwrap();
            let _ = send.write_all(json_error.as_bytes()).await;
            let _ = send.finish().await;
            return;
        }
    };
    
    if file_metadata.len() > MAX_FILE_SIZE {
        crate::log_server!("WARNING: File too large for JSON transfer: {} ({} bytes > {} bytes)", 
            req.file, file_metadata.len(), MAX_FILE_SIZE);
        let error = ErrorResponse {
            error: format!("File too large: {} ({} bytes). Maximum size: {} bytes. Use chunked transfer for larger files.", 
                req.file, file_metadata.len(), MAX_FILE_SIZE),
            code: Some("FILE_TOO_LARGE".to_string()),
        };
        let json_error = serde_json::to_string(&error).unwrap();
        let _ = send.write_all(json_error.as_bytes()).await;
        let _ = send.finish().await;
        return;
    }
    
    match fs::read(&file_path) {
        Ok(data) => {
            // Warn about large file transfers (>1MB) that may be slow
            if data.len() > 1024 * 1024 {
                crate::log_server!("WARNING: Large file transfer: {} ({} bytes) - JSON serialization may be slow", 
                    req.file, data.len());
            }
            
            crate::log_server!("File found: {} ({} bytes), sending via QUIC", file_path.display(), data.len());
            
            let response = FileResponse {
                data: data.clone(),
                filename: req.file.clone(),
                size: data.len(),
            };
            
            let json_response = match serde_json::to_string(&response) {
                Ok(json) => json,
                Err(e) => {
                    crate::log_server!("ERROR: Error serializing QUIC file response: {}", e);
                    let error = ErrorResponse {
                        error: "Internal server error".to_string(),
                        code: Some("SERIALIZATION_ERROR".to_string()),
                    };
                    serde_json::to_string(&error).unwrap()
                }
            };
            
            let _ = send.write_all(json_response.as_bytes()).await;
            let _ = send.finish().await;
            crate::log_server!("File sent successfully via QUIC: {} ({} bytes)", req.file, data.len());
        }
        Err(e) => {
            let error_msg = format!("File not found or unreadable: {} - {}", file_path.display(), e);
            crate::log_server!("ERROR: {}", error_msg);
            let error = ErrorResponse {
                error: format!("File not found: {}", file_path.display()),
                code: Some("FILE_NOT_FOUND".to_string()),
            };
            let json_error = serde_json::to_string(&error).unwrap();
            crate::log_server!("Sending error response via QUIC: {}", error.error);
            let _ = send.write_all(json_error.as_bytes()).await;
            let _ = send.finish().await;
        }
    }
}

async fn handle_ai_request(
    req: AiRequest,
    ai_processor: Option<Arc<RwLock<AiProcessor>>>,
    work_dist: Option<Arc<WorkDistributionManager>>,
    send: &mut quinn::SendStream,
) {
    crate::log_server!("[HANDLER] Function: quic_tracker::handle_ai_request()");
    crate::log_server!("[HANDLER] Module: AI Processing Module");
    crate::log_server!("[HANDLER] Processing AiRequest");
    crate::log_server!("[HANDLER] Handler chain: handle_ai_request() -> ai_processor::process_query_sync()");
    
    // Try to process locally first if AI processor is available
    // Note: We process in a separate block to ensure lock is dropped before await
    let local_response = {
        if let Some(ai_proc) = &ai_processor {
            crate::log_server!("[AI_PROCESSOR] process_query() called - query_len={}, context={}", 
                req.query.len(), req.context.is_some());
            
            if let Some(params) = &req.parameters {
                crate::log_server!("[AI_PROCESSOR] Parameters: temperature={:?}, max_tokens={:?}, top_p={:?}",
                    params.temperature, params.max_tokens, params.top_p);
            }
            
            crate::log_server!("[AI_PROCESSOR] Entering ai_processor.process_query() function");
            
            // Clone data needed for processing
            let query = req.query.clone();
            let context = req.context.clone();
            let temp = req.parameters.as_ref().and_then(|p| p.temperature);
            let max_tok = req.parameters.as_ref().and_then(|p| p.max_tokens);
            let top_p_val = req.parameters.as_ref().and_then(|p| p.top_p);
            
            // Process and get result (lock is held only during the sync call)
            let (answer, metadata) = {
                crate::log_server!("[AI_PROCESSOR] Calling function: ai_processor::process_query_sync()");
                crate::log_server!("[AI_PROCESSOR] Module: AI Processing Module");
                let mut proc = ai_proc.write().unwrap();
                proc.process_query_sync(
                    &query,
                    context.as_deref(),
                    temp,
                    max_tok,
                    top_p_val,
                ).unwrap_or_else(|e| {
                    crate::log_server!("ERROR: AI processing failed: {}", e);
                    (format!("Error: {}", e), ResponseMetadata {
                        input_tokens: None,
                        output_tokens: None,
                        total_tokens: None,
                        processing_time_ms: None,
                    })
                })
            };
            
            crate::log_server!("[AI_PROCESSOR] process_query() returned - answer_len={}, tokens={:?}",
                answer.len(), metadata.total_tokens);
            
            Some(AiResponse {
                answer,
                metadata: Some(metadata),
            })
        } else {
            None
        }
    };
    
    // If local processing succeeded, send response
    if let Some(response) = local_response {
        let json_response = match serde_json::to_string(&response) {
            Ok(json) => json,
            Err(e) => {
                crate::log_server!("ERROR: Error serializing AI response: {}", e);
                let error = ErrorResponse {
                    error: "Internal server error".to_string(),
                    code: Some("SERIALIZATION_ERROR".to_string()),
                };
                serde_json::to_string(&error).unwrap()
            }
        };
        
        crate::log_server!("[HANDLER] handle_ai_request() -> sending AiResponse");
        let _ = send.write_all(json_response.as_bytes()).await;
        let _ = send.finish().await;
        crate::log_server!("[HANDLER] handle_ai_request() completed");
        return;
    }
    
    // If local processing failed or not available, try work delegation
    if let Some(work_dist_manager) = work_dist {
        crate::log_server!("[WORK_DIST] Attempting work delegation for AI request");
        match work_dist_manager.delegate_ai_work(&req, &NodeCapability::AiProcessing).await {
            Ok(response) => {
                let json_response = match serde_json::to_string(&response) {
                    Ok(json) => json,
                    Err(e) => {
                        crate::log_server!("ERROR: Error serializing delegated AI response: {}", e);
                        let error = ErrorResponse {
                            error: "Internal server error".to_string(),
                            code: Some("SERIALIZATION_ERROR".to_string()),
                        };
                        serde_json::to_string(&error).unwrap()
                    }
                };
                
                crate::log_server!("[WORK_DIST] Delegated work completed successfully");
                let _ = send.write_all(json_response.as_bytes()).await;
                let _ = send.finish().await;
                return;
            }
            Err(e) => {
                crate::log_server!("ERROR: Work delegation failed: {}", e);
            }
        }
    }
    
    // If all else fails, return error
    let error = ErrorResponse {
        error: "AI processing not available".to_string(),
        code: Some("AI_UNAVAILABLE".to_string()),
    };
    let json_error = serde_json::to_string(&error).unwrap();
    let _ = send.write_all(json_error.as_bytes()).await;
    let _ = send.finish().await;
}

/// Starts the QUIC tracker server.
///
/// This function:
/// 1. Creates a QUIC server configuration with self-signed certificate
/// 2. Creates a QUIC endpoint listening on the specified port
/// 3. Accepts incoming QUIC connections
/// 4. Spawns tasks to handle each connection
///
/// # Arguments
/// * `port` - UDP port to listen on (QUIC uses UDP, not TCP)
///
/// # Returns
/// * `Ok(())` if server starts successfully
/// * `Err` if binding fails or certificate generation fails
pub async fn run_quic_tracker(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    run_quic_tracker_with_ai(port, true, true).await
}

/// Starts the QUIC tracker server with AI capabilities and work distribution
pub async fn run_quic_tracker_with_ai(
    port: u16,
    enable_ai: bool,
    enable_work_dist: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(TrackerState::default()));
    
    // Initialize AI processor if enabled
    let ai_processor = if enable_ai {
        crate::log_server!("[AI_PROCESSOR] Initializing AI processor");
        Some(Arc::new(RwLock::new(AiProcessor::new(None))))
    } else {
        None
    };
    
    // Initialize work distribution manager if enabled
    let work_dist = if enable_work_dist {
        crate::log_server!("[WORK_DIST] Initializing work distribution manager");
        Some(Arc::new(WorkDistributionManager::new()))
    } else {
        None
    };
    
    // Create seed directory if it doesn't exist
    let current_dir = std::env::current_dir()?;
    let seed_dir = current_dir.join("seed");
    fs::create_dir_all(&seed_dir)?;
    crate::log_server!("Seed directory ready: {}", seed_dir.display());
    
    // Seed the server with Hello World file on startup if seed folder is empty
    let seed_file = seed_dir.join("hello_world.txt");
    if !fs::metadata(&seed_file).is_ok() {
        crate::log_server!("Seeding server with initial file: {}", seed_file.display());
        fs::write(&seed_file, "Hello World!")?;
        println!("✓ Seeded server with '{}' containing: Hello World!", seed_file.display());
    }
    
    // Create server configuration
    let server_config = create_server_config()?;
    
    // Create QUIC endpoint and bind to UDP port
    // Note: QUIC uses UDP, not TCP!
    let endpoint = Endpoint::server(server_config, format!("0.0.0.0:{}", port).parse()?)?;
    
    println!("========================================");
    println!("QUIC Tracker Server Started");
    println!("========================================");
    println!("Protocol: QUIC (HTTP/3 over UDP)");
    println!("Listening on: quic://0.0.0.0:{}", port);
    println!("Transport: UDP (not TCP)");
    println!("Encryption: TLS 1.3 (built into QUIC)");
    println!("Message Format: JSON");
    println!("Server can also serve files (acts as peer)");
    println!("Logging to: tracker.log");
    println!("========================================");
    
    crate::log_server!("Server started and listening for QUIC connections on port {}", port);
    
    // Accept incoming QUIC connections
    // Each connection can have multiple streams
    while let Some(conn) = endpoint.accept().await {
        crate::log_server!("Incoming QUIC connection attempt...");
        let connection = match conn.await {
            Ok(conn) => {
                let remote = conn.remote_address();
                crate::log_server!("QUIC connection accepted from: {}", remote);
                conn
            },
            Err(e) => {
                crate::log_server!("ERROR: Connection error: {}", e);
                continue;
            }
        };
        let state = Arc::clone(&state);
        
        // Spawn a task to handle this connection
        // QUIC allows multiple streams per connection, so we handle them all
        let ai_proc = ai_processor.clone();
        let work_dist_clone = work_dist.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_quic_connection(connection, state, ai_proc, work_dist_clone).await {
                crate::log_server!("ERROR: QUIC connection handler error: {}", e);
            }
        });
    }
    
    Ok(())
}

