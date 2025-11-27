//! # Console Client
//!
//! Interactive console interface with input and output areas for the client.

use crate::client;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Console client with input and output areas.
pub struct ConsoleClient {
    output_buffer: Arc<Mutex<Vec<String>>>,
    max_output_lines: usize,
}

impl ConsoleClient {
    pub fn new(max_output_lines: usize) -> Self {
        Self {
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            max_output_lines,
        }
    }
    
    /// Adds a line to the output buffer.
    pub async fn add_output(&self, line: String) {
        let mut buffer = self.output_buffer.lock().await;
        buffer.push(line);
        
        // Keep only the last max_output_lines
        if buffer.len() > self.max_output_lines {
            buffer.remove(0);
        }
    }
    
    /// Clears the output buffer.
    pub async fn clear_output(&self) {
        let mut buffer = self.output_buffer.lock().await;
        buffer.clear();
    }
    
    /// Gets all output lines.
    pub async fn get_output(&self) -> Vec<String> {
        let buffer = self.output_buffer.lock().await;
        buffer.clone()
    }
    
    /// Displays the console UI with output area and input prompt.
    pub fn display_ui(&self, output_lines: &[String]) {
        // Clear screen using ANSI escape codes (works on modern Windows 10+ and Unix)
        print!("\x1B[2J\x1B[1;1H");
        
        // Header
        println!("╔══════════════════════════════════════════════════════════════════════════════╗");
        println!("║                    BitTorrent & AI Client - Console                          ║");
        println!("╚══════════════════════════════════════════════════════════════════════════════╝");
        println!();
        
        // Output area
        println!("┌─────────────────────────────────── OUTPUT ───────────────────────────────────┐");
        if output_lines.is_empty() {
            println!("│                                                                              │");
            println!("│  No output yet. Type a command to get started.                              │");
            println!("│  Type '/help' for available commands.                                        │");
            println!("│                                                                              │");
        } else {
            // Show last 20 lines (most recent first)
            let display_lines: Vec<_> = output_lines.iter().rev().take(20).collect();
            for line in display_lines {
                // Wrap long lines
                for wrapped in wrap_text(line, 78) {
                    // Truncate if still too long
                    let display = if wrapped.len() > 76 {
                        &wrapped[..73]
                    } else {
                        &wrapped
                    };
                    println!("│  {:<76} │", display);
                }
            }
        }
        println!("└──────────────────────────────────────────────────────────────────────────────┘");
        println!();
        
        // Commands help (compact)
        println!("Commands: /help | /download | /clear | /quit");
        println!();
    }
    
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > width {
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    
    lines
}

/// Runs the interactive console client.
pub async fn run_console() -> Result<(), Box<dyn std::error::Error>> {
    let console = Arc::new(ConsoleClient::new(100));
    
    // Initial welcome message
    console.add_output("Welcome to BitTorrent Client Console!".to_string()).await;
    console.add_output("Type '/help' for available commands".to_string()).await;
    
    // Initial display
    let output = console.get_output().await;
    console.display_ui(&output);
    
    // Use tokio for async input
    use tokio::io::{AsyncBufReadExt, BufReader};
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    
    loop {
        // Input prompt
        print!("> ");
        io::stdout().flush()?;
        
        // Read input asynchronously
        match reader.next_line().await {
            Ok(Some(line)) => {
                let input = line.trim().to_string();
                
                if input.is_empty() {
                    // Refresh display on empty input
                    let output = console.get_output().await;
                    console.display_ui(&output);
                    continue;
                }
                
                // Handle commands
                if input.starts_with('/') {
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    let command = parts[0];
                    
                    match command {
                        "/quit" | "/exit" => {
                            console.add_output("Exiting...".to_string()).await;
                            let output = console.get_output().await;
                            console.display_ui(&output);
                            break;
                        }
                        "/help" | "/?" => {
                            let help = r#"Available Commands:
  /help                    - Show this help message
  /download <torrent> [output] [server] [port]
                          - Download a file using torrent (QUIC protocol)
  /clear                   - Clear output area
  /quit | /exit           - Exit the console"#;
                            console.add_output(help.to_string()).await;
                        }
                        "/clear" => {
                            console.clear_output().await;
                            console.add_output("Output cleared.".to_string()).await;
                        }
                        "/download" => {
                            let args: Vec<String> = parts.iter().skip(1).map(|s| s.to_string()).collect();
                            handle_download_command(&console, &args).await?;
                        }
                        _ => {
                            console.add_output(format!("Unknown command: {}. Type /help for available commands.", command)).await;
                        }
                    }
                } else {
                    // Unknown input - show help
                    console.add_output(format!("Unknown input: '{}'. Type /help for available commands.", input)).await;
                }
                
                // Refresh display
                let output = console.get_output().await;
                console.display_ui(&output);
            }
            Ok(None) => {
                // EOF
                console.add_output("Exiting...".to_string()).await;
                let output = console.get_output().await;
                console.display_ui(&output);
                break;
            }
            Err(e) => {
                console.add_output(format!("Error reading input: {}", e)).await;
                let output = console.get_output().await;
                console.display_ui(&output);
                break;
            }
        }
    }
    
    Ok(())
}

async fn handle_download_command(
    console: &Arc<ConsoleClient>,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    console.add_output("Starting download...".to_string()).await;
    
    // Filter out any flags (QUIC is always used now)
    let filtered_args: Vec<&String> = args.iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .collect();
    
    let torrent_path = if let Some(path) = filtered_args.get(0) {
        let mut path_str = path.to_string();
        // Auto-add .torrent extension if not present
        if !path_str.ends_with(".torrent") {
            path_str.push_str(".torrent");
        }
        path_str
    } else {
        console.add_output("Error: Torrent file path required".to_string()).await;
        console.add_output("Usage: /download <torrent_file> [output_file] [server] [port]".to_string()).await;
        console.add_output("Example: /download seed\\hello_world.txt.torrent".to_string()).await;
        console.add_output("Example: /download seed\\hello_world.txt (auto-adds .torrent)".to_string()).await;
        console.add_output("Example: /download seed\\file.torrent downloaded\\file.txt 192.168.1.100 7001".to_string()).await;
        return Ok(());
    };
    
    // Validate torrent file exists
    if !std::path::Path::new(&torrent_path).exists() {
        console.add_output(format!("Error: Torrent file not found: {}", torrent_path)).await;
        return Ok(());
    }
    
    let output_path = if let Some(path) = filtered_args.get(1) {
        path.to_string()
    } else {
        let torrent_name = std::path::Path::new(&torrent_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("downloaded_file");
        format!("downloaded/{}", torrent_name)
    };
    
    let default_tracker = "127.0.0.1".to_string();
    let default_port = 7001u16; // QUIC default port
    
    let tracker_server = filtered_args.get(2)
        .map(|s| s.as_str())
        .unwrap_or(&default_tracker);
    let tracker_port = filtered_args.get(3)
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    
    console.add_output("Protocol: QUIC".to_string()).await;
    console.add_output(format!("Torrent: {}", torrent_path)).await;
    console.add_output(format!("Output: {}", output_path)).await;
    console.add_output(format!("Tracker: {}:{}", tracker_server, tracker_port)).await;
    
    let result = client::download_file_quic_torrent(
        &torrent_path,
        &output_path,
        tracker_server,
        tracker_port,
    ).await;
    
    match result {
        Ok(_) => {
            console.add_output("✓ Download completed successfully!".to_string()).await;
        }
        Err(e) => {
            console.add_output(format!("✗ Download failed: {}", e)).await;
        }
    }
    
    Ok(())
}


