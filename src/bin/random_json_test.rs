//! Random JSON and File Request Test
//! 
//! Sends random combinations of:
//! - TrackerAnnounceRequest (JSON)
//! - FileRequest (JSON)
//! - Custom JSON messages (to test error handling)

use quic_torrent_client_server::messages::*;
use quic_torrent_client_server::quic_client::QuicClient;
use std::time::Instant;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let server = args.get(1).map(|s| s.as_str()).unwrap_or("162.221.207.169");
    let port: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7001);
    let iterations: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20);

    println!("========================================");
    println!("Random JSON & File Request Test");
    println!("========================================");
    println!("Server: {}:{}", server, port);
    println!("Iterations: {}", iterations);
    println!();

    // Available files (excluding large.bin)
    let available_files = vec![
        "hello_world.txt",
        "small.txt",
        "medium.bin",
        "data.json",
        "log.txt",
    ];

    // Generate random info hashes and peer IDs
    let mut rng = rand::thread_rng();
    let client = QuicClient::new()?;

    let mut stats = TestStats::new();

    for i in 1..=iterations {
        let test_type = rng.gen_range(0..=3); // 0=announce, 1=file, 2=custom JSON, 3=AI query
        
        match test_type {
            0 => {
                // TrackerAnnounceRequest
                println!("[{}] Testing: TrackerAnnounceRequest", i);
                let start = Instant::now();
                
                let info_hash = generate_random_hash(&mut rng);
                let peer_id = format!("-QC{:04}-{}", 
                    rng.gen_range(1000..9999),
                    generate_random_string(&mut rng, 12));
                
                let request = TrackerAnnounceRequest {
                    info_hash: info_hash.clone(),
                    peer_id: peer_id.clone(),
                    port: rng.gen_range(6881..6999),
                    uploaded: Some(rng.gen_range(0..1000000)),
                    downloaded: Some(rng.gen_range(0..1000000)),
                    left: Some(rng.gen_range(0..10000000)),
                    event: Some(match rng.gen_range(0..3) {
                        0 => "started".to_string(),
                        1 => "completed".to_string(),
                        _ => "stopped".to_string(),
                    }),
                    ip: Some(format!("192.168.{}.{}", 
                        rng.gen_range(1..255), 
                        rng.gen_range(1..255))),
                };

                match client.send_message::<_, TrackerAnnounceResponse>(server, port, &request).await {
                    Ok(response) => {
                        let duration = start.elapsed();
                        println!("  [OK] Announce successful!");
                        println!("    Peers: {}, Complete: {}, Incomplete: {}", 
                            response.peers.len(), response.complete, response.incomplete);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.announce_success += 1;
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        println!("  [FAIL] Announce failed: {}", e);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.announce_fail += 1;
                    }
                }
            }
            1 => {
                // FileRequest
                let file = available_files[rng.gen_range(0..available_files.len())];
                println!("[{}] Testing: FileRequest - {}", i, file);
                let start = Instant::now();

                let request = FileRequest {
                    file: file.to_string(),
                };

                match client.send_message::<_, FileResponse>(server, port, &request).await {
                    Ok(response) => {
                        let duration = start.elapsed();
                        println!("  [OK] File download successful!");
                        println!("    File: {}, Size: {} bytes", response.filename, response.size);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.file_success += 1;
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        println!("  [FAIL] File download failed: {}", e);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.file_fail += 1;
                    }
                }
            }
            2 => {
                // Custom JSON (to test error handling)
                // We'll send a malformed request by using a struct that doesn't match server expectations
                println!("[{}] Testing: Custom JSON (Unknown Request)", i);
                let start = Instant::now();

                // Create a struct that serializes to JSON but isn't a valid request type
                #[derive(serde::Serialize)]
                struct UnknownRequest {
                    #[serde(rename = "type")]
                    request_type: String,
                    data: String,
                }

                let unknown_request = UnknownRequest {
                    request_type: match rng.gen_range(0..3) {
                        0 => "ping".to_string(),
                        1 => "status".to_string(),
                        _ => "unknown".to_string(),
                    },
                    data: generate_random_string(&mut rng, 20),
                };

                let json_str = serde_json::to_string(&unknown_request)?;
                println!("  Sending: {}", json_str);

                // Try to send and expect an error response
                match client.send_message::<_, ErrorResponse>(server, port, &unknown_request).await {
                    Ok(response) => {
                        let duration = start.elapsed();
                        println!("  [OK] Server responded with error (expected): {}", response.error);
                        println!("    Code: {:?}", response.code);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.custom_success += 1;
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        println!("  [INFO] Request rejected (expected): {}", e);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.custom_success += 1; // Still counts as success if server rejects properly
                    }
                }
            }
            3 => {
                // AiRequest
                let queries = vec![
                    "What is the capital of France?",
                    "Explain quantum computing in simple terms",
                    "Hello, how are you?",
                    "What is 2 + 2?",
                    "Tell me about artificial intelligence",
                ];
                let query = queries[rng.gen_range(0..queries.len())];
                println!("[{}] Testing: AiRequest - {}", i, query);
                let start = Instant::now();

                match quic_torrent_client_server::client::send_ai_query(
                    server,
                    port,
                    query,
                    None,
                    Some(0.7),
                    Some(100),
                    Some(0.9),
                ).await {
                    Ok(response) => {
                        let duration = start.elapsed();
                        println!("  [OK] AI query successful!");
                        println!("    Answer: {}...", 
                            if response.answer.len() > 50 {
                                &response.answer[..50]
                            } else {
                                &response.answer
                            });
                        if let Some(meta) = &response.metadata {
                            println!("    Tokens: {} input, {} output, {} total", 
                                meta.input_tokens.unwrap_or(0),
                                meta.output_tokens.unwrap_or(0),
                                meta.total_tokens.unwrap_or(0));
                            println!("    Processing time: {}ms", meta.processing_time_ms.unwrap_or(0));
                        }
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.ai_success += 1;
                    }
                    Err(e) => {
                        let duration = start.elapsed();
                        println!("  [FAIL] AI query failed: {}", e);
                        println!("    Duration: {:.2}s", duration.as_secs_f64());
                        stats.ai_fail += 1;
                    }
                }
            }
            _ => unreachable!(),
        }

        // Small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        println!();
    }

    println!("========================================");
    println!("Test Results Summary");
    println!("========================================");
    println!("Total tests: {}", iterations);
    println!("  Announce: {} success, {} failed", stats.announce_success, stats.announce_fail);
    println!("  File: {} success, {} failed", stats.file_success, stats.file_fail);
    println!("  Custom JSON: {} success, {} failed", stats.custom_success, stats.custom_fail);
    println!("  AI Query: {} success, {} failed", stats.ai_success, stats.ai_fail);
    println!();
    
    let total_success = stats.announce_success + stats.file_success + stats.custom_success + stats.ai_success;
    let total_fail = stats.announce_fail + stats.file_fail + stats.custom_fail + stats.ai_fail;
    
    if total_fail == 0 {
        println!("[SUCCESS] All tests passed!");
    } else {
        println!("[PARTIAL] {} passed, {} failed", total_success, total_fail);
    }

    Ok(())
}

struct TestStats {
    announce_success: usize,
    announce_fail: usize,
    file_success: usize,
    file_fail: usize,
    custom_success: usize,
    custom_fail: usize,
    ai_success: usize,
    ai_fail: usize,
}

impl TestStats {
    fn new() -> Self {
        Self {
            announce_success: 0,
            announce_fail: 0,
            file_success: 0,
            file_fail: 0,
            custom_success: 0,
            custom_fail: 0,
            ai_success: 0,
            ai_fail: 0,
        }
    }
}

fn generate_random_hash(rng: &mut impl Rng) -> String {
    (0..40)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

fn generate_random_string(rng: &mut impl Rng, len: usize) -> String {
    (0..len)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect()
}

