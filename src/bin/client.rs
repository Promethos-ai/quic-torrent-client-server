//! # BitTorrent Client Binary
//!
//! Comprehensive client that supports all functions:
//! - Download files via QUIC
//! - Send AI queries to AI service
//! - Process AI queries locally
//!
//! Usage:
//!   cargo run --bin client download [torrent_file] [output_file] [tracker_server] [tracker_port]
//!   cargo run --bin client ai-query [server] [port] [query]
//!   cargo run --bin client ai-local [query]

use quic_torrent_client_server::client;
use quic_torrent_client_server::logger;
use std::env;
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init_logger("client.log")?;
    
    let args: Vec<String> = env::args().collect();
    
    // If no arguments provided, show complete instructions
    if args.len() == 1 {
        print_complete_instructions();
        return Ok(());
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "console" | "interactive" => {
            // Run interactive console
            quic_torrent_client_server::console_client::run_console().await?;
        }
        "download" => {
            handle_download(&args[2..]).await?;
        }
        _ => {
            println!("Unknown command: {}", command);
            print_usage();
        }
    }
    
    Ok(())
}

fn print_complete_instructions() {
    println!("========================================");
    println!("QUIC BitTorrent Client - Connection Instructions");
    println!("========================================");
    println!();
    println!("TEST UBUNTU SERVER CONNECTION:");
    println!("  Server IP:    162.221.207.169");
    println!("  Server Port:  7001");
    println!("  Protocol:     QUIC (HTTP/3 over UDP)");
    println!("  Encryption:   TLS 1.3 (built into QUIC)");
    println!();
    println!("========================================");
    println!("AVAILABLE COMMANDS:");
    println!("========================================");
    println!();
    println!("1. DOWNLOAD A FILE:");
    println!("   cargo run --bin client download [file] [output] [server] [port]");
    println!();
    println!("   Examples:");
    println!("   # Download hello_world.txt from test server:");
    println!("   cargo run --bin client download hello_world.txt output.txt 162.221.207.169 7001");
    println!();
    println!("   # Download with defaults (uses 127.0.0.1:7001):");
    println!("   cargo run --bin client download hello_world.txt output.txt");
    println!();
    println!("2. SEND AI QUERY:");
    println!("   Use the random_json_test binary for AI queries:");
    println!("   cargo run --release --bin random_json_test -- 162.221.207.169 7001 10");
    println!();
    println!("3. INTERACTIVE CONSOLE:");
    println!("   cargo run --bin client console");
    println!("   cargo run --bin client interactive");
    println!();
    println!("========================================");
    println!("QUICK START - CONNECT TO TEST SERVER:");
    println!("========================================");
    println!();
    println!("Step 1: Test connection with a file download");
    println!("   cargo run --bin client download hello_world.txt test_output.txt 162.221.207.169 7001");
    println!();
    println!("Step 2: Test AI query (requires random_json_test binary)");
    println!("   cargo run --release --bin random_json_test -- 162.221.207.169 7001 5");
    println!();
    println!("Step 3: Test tracker announce");
    println!("   cargo run --release --bin random_json_test -- 162.221.207.169 7001 10");
    println!();
    println!("========================================");
    println!("SERVER INFORMATION:");
    println!("========================================");
    println!();
    println!("Test Ubuntu Server:");
    println!("  - IP Address:  162.221.207.169");
    println!("  - Port:         7001");
    println!("  - Protocol:     QUIC");
    println!("  - Status:       Python QUIC tracker server with AI capabilities");
    println!("  - Features:     File serving, Tracker announce, AI processing");
    println!();
    println!("Available Files on Server:");
    println!("  - hello_world.txt");
    println!("  - small.txt");
    println!("  - medium.bin");
    println!("  - data.json");
    println!("  - log.txt");
    println!("  (Files are in ~/seed/ directory on server)");
    println!();
    println!("========================================");
    println!("TROUBLESHOOTING:");
    println!("========================================");
    println!();
    println!("If connection fails:");
    println!("  1. Verify server is running: Check server logs on Ubuntu");
    println!("  2. Check firewall: Ensure UDP port 7001 is open");
    println!("  3. Verify network: ping 162.221.207.169");
    println!("  4. Check certificates: Client uses self-signed certs (auto-accepted)");
    println!("  5. View logs: Check client.log for detailed error messages");
    println!();
    println!("For more information, see:");
    println!("  - README.md");
    println!("  - CLIENT_DEPLOYMENT.md");
    println!("  - QUICK_REFERENCE.md");
    println!();
    println!("========================================");
}

fn print_usage() {
    println!("========================================");
    println!("BitTorrent Client - All Functions");
    println!("========================================");
    println!();
    println!("Usage: cargo run --bin client [command]");
    println!();
    println!("Commands:");
    println!("  (no command) | console | interactive");
    println!("    Start interactive console with input/output areas (default)");
    println!();
    println!("  download [torrent_file] [output_file] [tracker_server] [tracker_port]");
    println!("    Download a file using a torrent (QUIC protocol)");
    println!("    tracker_server: Server IP or hostname (default: 127.0.0.1)");
    println!("    tracker_port: Server port (default: 7001)");
    println!("    Example: download seed\\file.torrent downloaded\\file.txt 192.168.1.100 7001");
    println!();
    println!("========================================");
}

async fn handle_download(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Filter out any flags (QUIC is always used now)
    let filtered_args: Vec<&String> = args.iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .collect();
    
    let default_torrent = "test.torrent".to_string();
    let torrent_path = filtered_args.get(0)
        .map(|s| s.to_string())
        .unwrap_or(default_torrent);
    
    // Default output path
    let output_path = if let Some(path) = filtered_args.get(1) {
        path.to_string()
    } else {
        let torrent_name = std::path::Path::new(&torrent_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("downloaded_file");
        format!("downloaded/{}", torrent_name)
    };
    
    // Default tracker settings (QUIC port)
    let default_tracker = "127.0.0.1".to_string();
    let default_port = 7001u16;
    
    let tracker_server = filtered_args.get(2)
        .map(|s| s.as_str())
        .unwrap_or(&default_tracker);
    let tracker_port = filtered_args.get(3)
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    
    println!("========================================");
    println!("BitTorrent Client - Download (QUIC)");
    println!("========================================");
    println!("Protocol: QUIC");
    println!("Torrent file: {}", torrent_path);
    println!("Output file: {}", output_path);
    println!("Tracker: {}:{}", tracker_server, tracker_port);
    println!("Logging to: client.log");
    println!("========================================");
    
    client::download_file_quic_torrent(
        &torrent_path,
        &output_path,
        tracker_server,
        tracker_port,
    ).await?;
    
    Ok(())
}


