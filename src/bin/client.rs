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
    
    // Default to console mode if no command provided
    let command = if args.len() < 2 {
        "console".to_string()
    } else {
        args[1].clone()
    };
    
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


