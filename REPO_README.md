# RUST Client and Random Payload to External Ubuntu Server

A Rust QUIC client that sends random payloads (JSON messages, file requests, AI queries) to an external Ubuntu Python server over QUIC protocol.

## Overview

This repository contains a self-contained Rust QUIC client that communicates with a remote Ubuntu server running a Python QUIC tracker server. The client supports:

- **Tracker Announce Requests** - BitTorrent tracker protocol
- **File Requests** - Download files from server
- **AI Query Requests** - Send AI queries and receive responses
- **Random Payload Testing** - Automated testing with random request types

## Key Features

- ✅ **100% Self-Contained** - No external files needed (certificates, configs built-in)
- ✅ **QUIC Protocol** - Modern, encrypted, low-latency transport (UDP + TLS 1.3)
- ✅ **Cross-Platform** - Rust client (Windows) ↔ Python server (Ubuntu)
- ✅ **Multiple Request Types** - Tracker, File, AI, and custom JSON messages
- ✅ **Comprehensive Logging** - Detailed request/response logging with component tracking

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Network access to server (default: `162.221.207.169:7001`)

### Build

```bash
cargo build --release
```

### Run Tests

```bash
# Run random payload tests
cargo run --release --bin random_json_test -- 162.221.207.169 7001 10

# Or use the compiled binary
.\target\release\random_json_test.exe 162.221.207.169 7001 10
```

### Run Client

```bash
# Interactive console
cargo run --release --bin client

# Or use the compiled binary
.\target\release\client.exe
```

## Deployment

The client is **fully self-contained** - just copy the binary and run it!

See [CLIENT_DEPLOYMENT.md](CLIENT_DEPLOYMENT.md) for complete deployment instructions.

### What's Included in the Binary

- ✅ Certificate handling (accepts all certificates, including self-signed)
- ✅ ALPN protocols (`h3`, `h2`, `http/1.1`, `doq`)
- ✅ Message serialization (all JSON message types)
- ✅ No external dependencies or files needed

## Project Structure

```
quic-torrent-client-server/
├── src/
│   ├── bin/
│   │   ├── client.rs              # Main client binary
│   │   ├── random_json_test.rs    # Random payload test binary
│   │   └── tracker.rs             # Tracker server binary
│   ├── quic_client.rs             # QUIC client implementation
│   ├── quic_utils.rs              # Certificate & config utilities
│   ├── messages.rs                 # JSON message structures
│   ├── client.rs                  # Client functions
│   ├── ai_processor.rs            # AI processing stubs
│   └── work_distribution.rs       # Work distribution system
├── CLIENT_DEPLOYMENT.md           # Deployment guide
├── README.md                      # Main documentation
└── Cargo.toml                     # Rust dependencies
```

## Request Types

### TrackerAnnounceRequest
```json
{
  "info_hash": "abc123...",
  "peer_id": "-ST0001-123456789",
  "port": 6881,
  "uploaded": 0,
  "downloaded": 0,
  "left": 20000000,
  "event": "started"
}
```

### FileRequest
```json
{
  "file": "hello_world.txt"
}
```

### AiRequest
```json
{
  "query": "What is the capital of France?",
  "context": [
    {"role": "user", "content": "Hello"},
    {"role": "assistant", "content": "Hi there!"}
  ],
  "parameters": {
    "temperature": 0.7,
    "max_tokens": 512,
    "top_p": 0.9
  }
}
```

## Server Connection

Default server: `162.221.207.169:7001`

The client accepts self-signed certificates automatically (development mode).

## Documentation

- **[CLIENT_DEPLOYMENT.md](CLIENT_DEPLOYMENT.md)** - Complete deployment guide
- **[README.md](README.md)** - Full project documentation
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Quick start and troubleshooting

## License

MIT OR Apache-2.0

## Repository

GitHub: `github.com/promethos-ai/RUST-client-and-random-payload-to-external-ubuntu-server`

