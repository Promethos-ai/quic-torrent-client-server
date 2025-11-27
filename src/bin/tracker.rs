//! # Tracker Server Binary
//!
//! Deployable BitTorrent tracker server.
//! Usage: cargo run --bin tracker [--quic] [port]
//!   --quic: Use QUIC protocol (default: HTTP)
//!   port: Server port (default: 7000 for HTTP, 7001 for QUIC)

use quic_torrent_client_server::quic_tracker;
use quic_torrent_client_server::logger;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the directory where the executable is located
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent()
        .ok_or("Cannot get executable directory")?;
    
    // Determine project root: if we're in target/release/, go up 2 levels
    // Otherwise assume executable directory is project root (for deployed scenarios)
    let project_dir = {
        let exe_str = exe_dir.to_string_lossy().to_lowercase();
        if exe_str.contains("target") && exe_str.contains("release") {
            // We're in target/release/, go up to project root
            exe_dir.parent()
                .and_then(|p| p.parent())
                .unwrap_or(exe_dir)
        } else {
            // Use executable directory (for deployed/service scenarios)
            exe_dir
        }
    };
    
    // Change to the project directory so relative paths work
    if let Err(e) = env::set_current_dir(&project_dir) {
        eprintln!("Warning: Failed to change to project directory {}: {}", project_dir.display(), e);
        eprintln!("Continuing with current directory...");
    }
    
    // Use absolute path for log file in project directory
    let log_path = project_dir.join("tracker.log");
    if let Err(e) = logger::init_logger(log_path.to_str().unwrap_or("tracker.log")) {
        eprintln!("Error initializing logger: {}", e);
        return Err(e.into());
    }
    
    // Stop any running tracker processes on startup (but not this one)
    let current_pid = std::process::id();
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Get all tracker.exe processes and kill only those that aren't us
        if let Ok(output) = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq tracker.exe", "/FO", "CSV", "/NH"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    // CSV format: "tracker.exe","PID","Session Name","Session#","Mem Usage"
                    if let Some(pid_start) = line.find("\",\"") {
                        if let Some(pid_end) = line[pid_start + 3..].find("\"") {
                            if let Ok(pid) = line[pid_start + 3..pid_start + 3 + pid_end].parse::<u32>() {
                                if pid != current_pid {
                                    let _ = Command::new("taskkill")
                                        .args(["/F", "/PID", &pid.to_string()])
                                        .output();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        // On Unix, pkill by default won't kill the current process
        let _ = Command::new("pkill")
            .args(["-f", "tracker"])
            .output();
    }
    
    let args: Vec<String> = env::args().collect();
    
    // Check for --quic flag
    let use_quic = args.iter().any(|arg| arg == "--quic" || arg == "-q");
    
    // Get port (default: 7000 for HTTP, 7001 for QUIC)
    let default_port = if use_quic { 7001 } else { 7000 };
    let port = args.iter()
        .filter(|arg| !arg.starts_with("--") && !arg.starts_with("-"))
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(default_port);
    
    println!("========================================");
    println!("BitTorrent Tracker Server");
    println!("========================================");
    println!("Protocol: QUIC");
    println!("Executable directory: {}", exe_dir.display());
    println!("Working directory: {}", project_dir.display());
    println!("Stopped any existing tracker processes");
    println!("Starting tracker on port {}...", port);
    println!("Logging to: {}", log_path.display());
    println!("========================================");
    
    // Log startup
    quic_torrent_client_server::log_server!("Starting QUIC tracker server on port {}", port);
    
    // Run QUIC tracker
    let result = quic_tracker::run_quic_tracker(port).await;
    
    match result {
        Ok(()) => {
            println!("Tracker stopped normally");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Tracker error: {}", e);
            eprintln!("{}", error_msg);
            quic_torrent_client_server::log_server!("{}", error_msg);
            Err(e)
        }
    }
}

