# QUIC Torrent Client & Server - Usage Guide

Complete guide for running and using the QUIC Torrent tracker server and client.

## Table of Contents

- [Building](#building)
- [Running the Server](#running-the-server)
- [Running the Client](#running-the-client)
- [Examples](#examples)
- [Network Configuration](#network-configuration)
- [Troubleshooting](#troubleshooting)

## Building

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Build Commands

```bash
# Build in release mode (optimized)
cargo build --release

# Build only the tracker server
cargo build --release --bin tracker

# Build only the client
cargo build --release --bin client
```

The executables will be in `target/release/`:
- `tracker.exe` (Windows) or `tracker` (Linux/Mac) - Server
- `client.exe` (Windows) or `client` (Linux/Mac) - Client

## Running the Server

### Basic Usage

```bash
# Start QUIC tracker on default port 7001
cargo run --release --bin tracker -- --quic 7001

# Or use the built executable
./target/release/tracker --quic 7001
```

### Server Options

- `--quic` or `-q`: Use QUIC protocol (required)
- Port number: Specify the port (default: 7001 for QUIC)

### Server Behavior

- Creates a `seed/` directory if it doesn't exist
- Automatically seeds `hello_world.txt` if seed directory is empty
- Listens on `0.0.0.0` (all network interfaces) for the specified port
- Logs all activity to `tracker.log` and console
- Displays real-time connection and request information

### Example Server Output

```
========================================
BitTorrent Tracker Server
========================================
Protocol: QUIC
Executable directory: E:\rust\quic-torrent-client-server
Working directory: E:\rust\quic-torrent-client-server
Stopped any existing tracker processes
Starting tracker on port 7001...
Logging to: tracker.log
========================================
========================================
QUIC Tracker Server Started
========================================
Protocol: QUIC (HTTP/3 over UDP)
Listening on: quic://0.0.0.0:7001
Transport: UDP (not TCP)
Encryption: TLS 1.3 (built into QUIC)
Message Format: JSON
Server can also serve files (acts as peer)
Logging to: tracker.log
========================================
```

## Running the Client

### Console Mode (Interactive)

```bash
# Start interactive console
cargo run --release --bin client

# Or use the built executable
./target/release/client
```

In console mode, you can use commands:
- `/download <torrent_file> [output_file] [server] [port]` - Download a file
- `/help` - Show available commands
- `/clear` - Clear output area
- `/quit` - Exit the console

### Direct Download Mode

```bash
# Basic download (uses default server 127.0.0.1:7001)
cargo run --release --bin client download seed/hello_world.txt.torrent downloaded/hello_world.txt

# Download from specific server
cargo run --release --bin client download seed/hello_world.txt.torrent downloaded/hello_world.txt 192.168.1.229 7001

# Full syntax
cargo run --release --bin client download <torrent_file> <output_file> <server_ip> <port>
```

### Client Arguments

1. `torrent_file` - Path to the .torrent file (required)
2. `output_file` - Where to save the downloaded file (optional, defaults to `downloaded/<filename>`)
3. `server_ip` - Tracker server IP address (optional, defaults to `127.0.0.1`)
4. `port` - Tracker server port (optional, defaults to `7001`)

## Examples

### Example 1: Local Testing (Same Machine)

**Terminal 1 - Start Server:**
```bash
cd quic-torrent-client-server
cargo run --release --bin tracker -- --quic 7001
```

**Terminal 2 - Download File:**
```bash
cd quic-torrent-client-server
cargo run --release --bin client download seed/hello_world.txt.torrent downloaded/hello_world.txt 127.0.0.1 7001
```

**Expected Output:**
```
========================================
BitTorrent Client - Download (QUIC)
========================================
Protocol: QUIC
Torrent file: seed/hello_world.txt.torrent
Output file: downloaded/hello_world.txt
Tracker: 127.0.0.1:7001
Logging to: client.log
========================================
Downloading torrent via QUIC: hello_world.txt
Info hash: a557c977d3a06f16d32f44b70ed472dc8bb433a7
File size: 17 bytes
Announcing to QUIC tracker: 127.0.0.1:7001
Announced successfully
Downloading file from QUIC server: 127.0.0.1:7001
Received file via QUIC: 17 bytes
File saved successfully to: downloaded/hello_world.txt
Download complete!
```

### Example 2: Network Testing (Different Machines)

**On Server Machine (192.168.1.229):**
```bash
# Start server
./target/release/tracker --quic 7001
```

**On Client Machine:**
```bash
# Download from remote server
./target/release/client download seed/hello_world.txt.torrent downloaded/remote_file.txt 192.168.1.229 7001
```

### Example 3: Interactive Console Mode

```bash
# Start client in console mode
./target/release/client

# In the console, type:
/download seed/hello_world.txt.torrent downloaded/test.txt 192.168.1.229 7001

# Or use defaults (127.0.0.1:7001)
/download seed/hello_world.txt.torrent
```

### Example 4: Multiple Downloads

```bash
# Download multiple files in sequence
./target/release/client download seed/file1.torrent downloaded/file1.txt 192.168.1.229 7001
./target/release/client download seed/file2.torrent downloaded/file2.txt 192.168.1.229 7001
./target/release/client download seed/file3.torrent downloaded/file3.txt 192.168.1.229 7001
```

### Example 5: Using PowerShell Scripts (Windows)

**Start Server:**
```powershell
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd 'E:\rust\quic-torrent-client-server'; .\target\release\tracker.exe --quic 7001"
```

**Run Client:**
```powershell
.\target\release\client.exe download seed\hello_world.txt.torrent downloaded\test.txt 192.168.1.229 7001
```

## Network Configuration

### Finding Your Server IP Address

**Windows (PowerShell):**
```powershell
Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike "127.*" -and $_.IPAddress -notlike "169.254.*" } | Select-Object IPAddress, InterfaceAlias
```

**Linux/Mac:**
```bash
# Linux
ip addr show | grep "inet " | grep -v 127.0.0.1

# Mac
ifconfig | grep "inet " | grep -v 127.0.0.1
```

### Firewall Configuration

**Windows Firewall:**
```powershell
# Allow UDP port 7001 (QUIC uses UDP)
New-NetFirewallRule -DisplayName "QUIC Tracker" -Direction Inbound -Protocol UDP -LocalPort 7001 -Action Allow
```

**Linux (ufw):**
```bash
sudo ufw allow 7001/udp
```

**Linux (iptables):**
```bash
sudo iptables -A INPUT -p udp --dport 7001 -j ACCEPT
```

### Server Binding

The server binds to `0.0.0.0` by default, which means it listens on all network interfaces:
- `127.0.0.1` (localhost)
- Your local subnet IP (e.g., `192.168.1.229`)
- All other network interfaces

This allows clients from:
- The same machine (localhost)
- The same local network
- Other networks (if port-forwarded)

## Troubleshooting

### Connection Timeout

**Problem:** Client can't connect to server

**Solutions:**
1. Verify server is running:
   ```bash
   # Check if process is running
   ps aux | grep tracker  # Linux/Mac
   Get-Process tracker     # Windows
   ```

2. Check server is listening:
   ```bash
   # Linux/Mac
   netstat -an | grep 7001
   
   # Windows
   netstat -an | findstr 7001
   ```

3. Verify firewall allows UDP port 7001

4. Check IP address is correct:
   ```bash
   # Test connectivity
   ping 192.168.1.229
   ```

### File Not Found Error

**Problem:** `Error: File not found or unreadable`

**Solutions:**
1. Ensure seed file exists in `seed/` directory
2. Check file permissions
3. Verify torrent file points to correct filename

### Hash Mismatch

**Problem:** Downloaded file hash doesn't match seed file

**Solutions:**
1. Verify seed file hasn't been modified
2. Check network connection stability
3. Re-download the file

### Port Already in Use

**Problem:** `Address already in use` or `Port already in use`

**Solutions:**
1. Stop existing tracker process:
   ```bash
   # Windows
   taskkill /F /IM tracker.exe
   
   # Linux/Mac
   pkill tracker
   ```

2. Use a different port:
   ```bash
   ./target/release/tracker --quic 7002
   ```

### Certificate Errors

**Problem:** QUIC connection fails with certificate errors

**Note:** The client is configured to accept self-signed certificates for development. If you see certificate errors, it may indicate:
1. Network connectivity issues
2. Firewall blocking UDP packets
3. Server not running

## Verification

### Verify File Integrity

**Windows (PowerShell):**
```powershell
$seedHash = (Get-FileHash seed/hello_world.txt -Algorithm SHA256).Hash
$downloadedHash = (Get-FileHash downloaded/hello_world.txt -Algorithm SHA256).Hash
if ($seedHash -eq $downloadedHash) {
    Write-Host "Files match!" -ForegroundColor Green
} else {
    Write-Host "Files don't match!" -ForegroundColor Red
}
```

**Linux/Mac:**
```bash
sha256sum seed/hello_world.txt downloaded/hello_world.txt
# Compare the hashes
```

### Test Multiple Downloads

**Windows (PowerShell):**
```powershell
for ($i = 1; $i -le 10; $i++) {
    .\target\release\client.exe download seed\hello_world.txt.torrent "downloaded\test_$i.txt" 192.168.1.229 7001
}
```

**Linux/Mac:**
```bash
for i in {1..10}; do
    ./target/release/client download seed/hello_world.txt.torrent "downloaded/test_$i.txt" 192.168.1.229 7001
done
```

## Log Files

- **Server logs:** `tracker.log` - All server activity with timestamps
- **Client logs:** `client.log` - All client activity with timestamps

Both logs show:
- Connection events
- Request/response details
- File operations
- Errors and warnings

## Performance Notes

- **Connection time:** ~10-800ms (first connection slower, subsequent faster)
- **File transfer:** Depends on file size and network speed
- **Concurrent connections:** Server supports multiple simultaneous clients
- **QUIC benefits:** Lower latency, built-in encryption, multiplexing

## Security Notes

- Uses self-signed certificates for development
- Accepts all certificates in client (development mode only)
- For production, use proper TLS certificates
- QUIC provides TLS 1.3 encryption by default

## Next Steps

- Create your own torrent files for files in the `seed/` directory
- Deploy server on a dedicated machine
- Configure firewall rules for your network
- Set up automatic startup (systemd service, Windows service, etc.)




