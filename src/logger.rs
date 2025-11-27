//! # Logging System
//!
//! Provides comprehensive logging with timestamps and two-column format.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use chrono::Local;

/// Logger that writes to a file with formatted output.
pub struct TorrentLogger {
    file: Arc<Mutex<std::fs::File>>,
}

impl TorrentLogger {
    pub fn new(log_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn write_log(&self, side: &str, message: &str, arrow: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        
        // Format log line
        let log_line = format!("{:<25} | {:<50} {} {:<50}", 
            timestamp, side, arrow, message);
        
        // Write to file
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", log_line)?;
        file.flush()?;
        drop(file); // Release lock before printing
        
        // Also print to console for real-time display with timestamp
        let time_str = now.format("%H:%M:%S%.3f");
        if side == "SERVER" {
            if arrow == "←" {
                println!("[{}] [SERVER ←] {}", time_str, message);
            } else {
                println!("[{}] [SERVER →] {}", time_str, message);
            }
        } else if side == "CLIENT" {
            if arrow == "→" {
                println!("[{}] [CLIENT →] {}", time_str, message);
            } else {
                println!("[{}] [CLIENT ←] {}", time_str, message);
            }
        } else if !message.is_empty() {
            println!("[{}] {}", time_str, message);
        }
        
        Ok(())
    }

    pub fn server_log(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_log("SERVER", message, "→")
    }

    pub fn client_log(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.write_log("", "", "←");
        self.write_log("CLIENT", message, "")
    }

    pub fn server_received(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_log("SERVER", message, "←")
    }

    pub fn client_sent(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.write_log("", "", "→");
        self.write_log("CLIENT", message, "")
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_LOGGER: Arc<Mutex<Option<Arc<TorrentLogger>>>> = Arc::new(Mutex::new(None));
}

pub fn init_logger(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create/truncate log file and write header
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(log_file)?;
    writeln!(file, "{:<25} | {:<50} {} {:<50}", "TIMESTAMP", "SERVER", "", "CLIENT")?;
    writeln!(file, "{}", "-".repeat(130))?;
    drop(file); // Close file before creating logger
    
    // Now create the logger (which will append)
    let logger = Arc::new(TorrentLogger::new(log_file)?);
    
    *GLOBAL_LOGGER.lock().unwrap() = Some(logger.clone());
    Ok(())
}

pub fn get_logger() -> Option<Arc<TorrentLogger>> {
    GLOBAL_LOGGER.lock().unwrap().clone()
}

