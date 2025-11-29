# QUIC Torrent Client-Server System

Cross-platform BitTorrent tracker implementation using QUIC protocol, connecting Rust client (Windows) to Python server (Ubuntu).

## Overview

This project implements a complete BitTorrent tracker system over QUIC (HTTP/3), enabling:
- Peer announcements (BitTorrent tracker protocol)
- File transfers over QUIC streams
- JSON message format
- Cross-platform compatibility (Rust ↔ Python)

## Key Features

- **QUIC Protocol:** Modern, encrypted, low-latency transport
- **Cross-Platform:** Rust client (Windows) ↔ Python server (Ubuntu)
- **BitTorrent Compatible:** Standard tracker announce protocol
- **File Serving:** Server acts as seeder for files
- **Comprehensive Testing:** Soak tests and integrity verification

## Architecture

```
Rust Client (Windows)          Python Server (Ubuntu)
     │                              │
     │  QUIC (UDP, TLS 1.3)        │
     │  ALPN: h3                    │
     │  JSON Messages               │
     ├──────────────────────────────►│
     │                              │
     │  TrackerAnnounceRequest      │
     │  FileRequest                 │
     │                              │
     │◄─────────────────────────────┤
     │  TrackerAnnounceResponse     │
     │  FileResponse                │
```

## Critical Component: ALPN Byte-Level Fix

**IMPORTANT:** This project includes a critical fix for ALPN (Application-Layer Protocol Negotiation) compatibility between Rust's `quinn` library and Python's `aioquic` library.

**Location:** `wireshark-smarty/byte_level_alpn_fix.py`

**Why It's Needed:**
- Rust uses `Vec<Vec<u8>>` (bytes) for ALPN
- Python's `aioquic` normalizes to strings, breaking byte comparison
- The fix patches `aioquic` at runtime to perform byte-to-byte comparison

**See:** `wireshark-smarty/ALPN_FIX_TECHNICAL_DETAILS.md` for complete technical details.

## Documentation

- **[Complete Project Documentation](PROJECT_DOCUMENTATION.md)** - Comprehensive guide covering all aspects
- **[Monkey Patching Documentation](../wireshark-smarty/MONKEY_PATCHING_README.md)** - Complete guide to all runtime patches
- **[ALPN Fix Technical Details](../wireshark-smarty/ALPN_FIX_TECHNICAL_DETAILS.md)** - Deep dive into the ALPN compatibility fix
- **[Quick Reference Guide](QUICK_REFERENCE.md)** - Quick start and troubleshooting

## Quick Start

### Prerequisites

**Server (Ubuntu):**
- Python 3.8+
- `aioquic` or `qh3` library
- `cryptography` library

**Client (Windows):**
- Rust 1.70+
- `quinn` library
- `rustls` library

### Server Setup

```bash
cd /home/dbertrand
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
python3 quic_tracker_server.py 7001
```

### Client Setup

```powershell
cd quic-torrent-client-server
cargo build --release
.\target\release\client.exe download seed\hello_world.txt.torrent output.txt 162.221.207.169 7001
```

## Testing

### Quick Test

```powershell
.\test_output.ps1
```

Tests all files once and verifies hash integrity.

### Soak Test

```powershell
.\soak_test.ps1 -Hours 12
```

Runs continuous testing for 12 hours.

## Project Structure

```
quic-torrent-client-server/
├── src/
│   ├── quic_client.rs      # QUIC connection management
│   ├── quic_utils.rs        # QUIC configuration
│   ├── client.rs            # High-level client API
│   └── messages.rs          # Message type definitions
├── wireshark-smarty/
│   ├── quic_tracker_server.py    # Python server implementation
│   ├── byte_level_alpn_fix.py    # CRITICAL: ALPN compatibility fix
│   └── requirements.txt           # Python dependencies
├── PROJECT_DOCUMENTATION.md      # Complete documentation
├── QUICK_REFERENCE.md             # Quick reference guide
└── README.md                      # This file
```

## Protocol Details

- **Transport:** QUIC (UDP-based)
- **Security:** TLS 1.3 (built into QUIC)
- **ALPN:** `h3` (HTTP/3)
- **Message Format:** JSON over bidirectional QUIC streams
- **Port:** 7001 (UDP)

## Message Types

### TrackerAnnounceRequest
Announces a peer to the tracker (BitTorrent protocol).

### FileRequest
Requests a file download from the server.

### CapabilityFrame
Application-layer capability negotiation.

## Security

- **Passwords:** Never hardcoded - use environment variables or SSH keys
- **Certificates:** Self-signed for development (use CA-signed for production)
- **Firewall:** Only open necessary ports (7001/udp)

## Troubleshooting

See [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for troubleshooting checklist.

Common issues:
- Connection timeouts → Check server status and firewall
- ALPN negotiation failures → Verify `byte_level_alpn_fix.py` is loaded
- File not found → Verify files exist in server's `seed/` directory

## License

[Add your license here]

## Contributors

[Add contributors here]

---

**Last Updated:** November 29, 2025
