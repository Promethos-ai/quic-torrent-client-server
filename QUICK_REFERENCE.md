# QUIC Torrent System - Quick Reference Guide

## Quick Start

### Server (Ubuntu)

```bash
# Install dependencies
cd /home/dbertrand
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Start server
python3 quic_tracker_server.py 7001
```

### Client (Windows)

```powershell
# Build client
cd quic-torrent-client-server
cargo build --release

# Test connection
.\target\release\client.exe download seed\hello_world.txt.torrent output.txt 162.221.207.169 7001

# Run test suite
.\test_output.ps1
```

---

## Key Files

### Server (Python)
- `wireshark-smarty/quic_tracker_server.py` - Main server implementation
- `wireshark-smarty/byte_level_alpn_fix.py` - **CRITICAL:** ALPN compatibility fix
- `wireshark-smarty/requirements.txt` - Python dependencies

### Client (Rust)
- `src/quic_client.rs` - QUIC connection management
- `src/quic_utils.rs` - QUIC configuration
- `src/client.rs` - High-level client API

### Test Scripts
- `test_output.ps1` - Quick test (all files once)
- `soak_test.ps1` - Long-duration test (12 hours)

---

## Critical Configuration

### ALPN Protocol

**MUST be bytes, not strings:**

```python
# CORRECT (bytes):
alpn_protocols = [b"h3"]

# WRONG (strings):
alpn_protocols = ["h3"]  # Will fail!
```

### Server Configuration

```python
configuration = QuicConfiguration(
    is_client=False,
    alpn_protocols=[b"h3"],  # Bytes!
    certificate=cert,
    private_key=private_key,
)
```

### Client Configuration

```rust
crypto.alpn_protocols = vec![b"h3".to_vec()];  // Vec<Vec<u8>>
```

---

## Common Issues

### Connection Timeout

**Symptom:** `ConnectionLost(TimedOut)`

**Causes:**
1. Server not running
2. Firewall blocking UDP port 7001
3. ALPN negotiation failing (check logs for "BYTE LEVEL" messages)

**Fix:**
- Verify server is running: `ps aux | grep quic_tracker_server`
- Check firewall: `sudo ufw allow 7001/udp`
- Verify ALPN patch is loaded (check server logs)

### ALPN Negotiation Failure

**Symptom:** Handshake never completes

**Fix:**
- Ensure `byte_level_alpn_fix.py` is imported before aioquic
- Verify ALPN is configured as bytes: `[b"h3"]` not `["h3"]`
- Check server logs for "BYTE LEVEL" messages

### File Not Found

**Symptom:** `FILE_NOT_FOUND` error

**Fix:**
- Verify file exists in server's `seed/` directory
- Check file permissions
- Verify filename matches exactly (case-sensitive)

---

## Message Formats

### TrackerAnnounceRequest

```json
{
  "info_hash": "abc123...",
  "peer_id": "-QC0001-1234567890123456",
  "port": 6881,
  "uploaded": 0,
  "downloaded": 0,
  "left": 1024,
  "event": "started",
  "ip": "192.168.1.100"
}
```

### FileRequest

```json
{
  "file": "hello_world.txt"
}
```

### Error Response

```json
{
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

---

## Testing

### Quick Test

```powershell
.\test_output.ps1
```

Tests all files once, verifies hash integrity.

### Soak Test

```powershell
.\soak_test.ps1 -Hours 12
```

Runs continuous testing for 12 hours, monitors reliability.

### Monitor Test Progress

```powershell
Get-Content soak_logs\summary.log -Tail 20 -Wait
```

---

## Server Management

### Check Server Status

```bash
# Check if running
ps aux | grep quic_tracker_server

# Check port
sudo netstat -tuln | grep 7001

# View logs
tail -50 /home/dbertrand/quic_tracker_server.log
```

### Start Server

```bash
cd /home/dbertrand
source venv/bin/activate
nohup python3 quic_tracker_server.py 7001 > quic_server.log 2>&1 &
```

### Stop Server

```bash
pkill -f quic_tracker_server
```

---

## Network Configuration

- **Protocol:** QUIC (UDP)
- **Port:** 7001
- **ALPN:** `h3` (HTTP/3)
- **Encryption:** TLS 1.3 (built into QUIC)
- **Message Format:** JSON over bidirectional streams

---

## Security Notes

- **Passwords:** Never hardcode passwords in scripts
- **Use environment variables:** `$env:SSH_PASSWORD` or SSH keys
- **Certificates:** Self-signed for development, use CA-signed for production
- **Firewall:** Only open necessary ports (7001/udp)

---

## Troubleshooting Checklist

- [ ] Server is running (`ps aux | grep quic_tracker_server`)
- [ ] Port 7001 is open (`netstat -tuln | grep 7001`)
- [ ] Firewall allows UDP 7001 (`sudo ufw allow 7001/udp`)
- [ ] ALPN patch is loaded (check server logs for "BYTE LEVEL")
- [ ] ALPN configured as bytes (`[b"h3"]` not `["h3"]`)
- [ ] Files exist in `seed/` directory
- [ ] Client can resolve server hostname/IP
- [ ] Network connectivity (ping server)

---

## Documentation

- **Full Documentation:** `PROJECT_DOCUMENTATION.md`
- **ALPN Fix Details:** `wireshark-smarty/ALPN_FIX_TECHNICAL_DETAILS.md`
- **Server Config:** `wireshark-smarty/SERVER_CONFIG.txt`

---

**Last Updated:** November 29, 2025

