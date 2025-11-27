# QUIC Torrent Client & Server

A minimal BitTorrent tracker and client implementation using the QUIC protocol (HTTP/3 over UDP) with JSON message format.

## Features

- **QUIC Tracker Server**: BitTorrent tracker using QUIC protocol
- **QUIC Client**: Download files via QUIC with console interface
- **JSON Messages**: All communication uses JSON over QUIC streams
- **TLS 1.3 Encryption**: Built-in encryption via QUIC
- **Comprehensive Logging**: Real-time console and file logging

## Building

```bash
cargo build --release
```

## Running the Server

```bash
# Start QUIC tracker on port 7001
cargo run --release --bin tracker -- --quic 7001
```

The server will:
- Create a `seed/` directory if it doesn't exist
- Seed with `hello_world.txt` if the seed directory is empty
- Listen for QUIC connections on the specified port
- Log all activity to `tracker.log` and console

## Running the Client

### Console Mode (Interactive)
```bash
cargo run --release --bin client
```

Then use commands:
- `/download <torrent_file> [output_file] [server] [port]` - Download a file
- `/help` - Show available commands
- Type a query directly to send an AI query (if AI service is configured)

### Direct Download
```bash
cargo run --release --bin client download seed/hello_world.txt.torrent downloaded/hello_world.txt 127.0.0.1 7001
```

## Example Usage

1. Start the tracker server:
   ```bash
   cargo run --release --bin tracker -- --quic 7001
   ```

2. In another terminal, download a file:
   ```bash
   cargo run --release --bin client download seed/hello_world.txt.torrent downloaded/hello_world.txt 127.0.0.1 7001
   ```

3. Verify the downloaded file matches the seed:
   ```bash
   # On Linux/Mac
   diff seed/hello_world.txt downloaded/hello_world.txt
   
   # On Windows PowerShell
   Compare-Object (Get-Content seed/hello_world.txt) (Get-Content downloaded/hello_world.txt)
   ```

## Project Structure

```
quic-torrent-client-server/
├── src/
│   ├── bin/
│   │   ├── tracker.rs      # Tracker server binary
│   │   └── client.rs       # Client binary
│   ├── lib.rs              # Library exports and bencode
│   ├── quic_tracker.rs     # QUIC tracker implementation
│   ├── quic_client.rs      # QUIC client implementation
│   ├── quic_utils.rs       # QUIC utilities (certificates, config)
│   ├── messages.rs         # JSON message definitions
│   ├── client.rs           # Client download logic
│   ├── console_client.rs   # Interactive console interface
│   └── logger.rs           # Logging system
├── seed/                   # Seed files directory
├── downloaded/             # Downloaded files directory
└── Cargo.toml
```

## Protocol

The implementation uses QUIC (HTTP/3 over UDP) with:
- **TLS 1.3** encryption (built into QUIC)
- **JSON messages** for all communication
- **Bidirectional streams** for request/response
- **Self-signed certificates** for development (accepts all certs in client)

## License

MIT OR Apache-2.0

