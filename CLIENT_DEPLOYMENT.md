# Client Deployment Guide

## Moving `client.exe` to Another Machine

The Rust QUIC client is **self-contained** and requires **NO external files** to run. All necessary components are compiled into the binary.

## What's Included in `client.exe`

### ✅ Built-In (No Files Needed)

1. **Certificate Handling**
   - Client uses `AcceptAllVerifier` - accepts any certificate (including self-signed)
   - No CA certificates or certificate files required
   - Certificate validation is disabled for development use
   - See: `src/quic_utils.rs::create_client_config()`

2. **ALPN Configuration**
   - Multiple ALPN protocols built-in: `h3`, `h2`, `http/1.1`, `doq`
   - Automatic fallback mechanism
   - No configuration files needed

3. **Message Serialization**
   - JSON message structures compiled into binary
   - No external schema files needed

## Files You DON'T Need

- ❌ Certificate files (`.pem`, `.der`, `.crt`)
- ❌ Configuration files
- ❌ Seed files (only needed for server)
- ❌ Torrent files (optional, only if using torrent-based downloads)
- ❌ Log files (created automatically if logging is enabled)

## What You DO Need

### Required

1. **The Binary**
   - `target/release/client.exe` (or `target/release/random_json_test.exe` for testing)
   - That's it! The binary is fully self-contained.

2. **Network Access**
   - UDP port access (QUIC uses UDP)
   - Ability to connect to server IP:port (default: `162.221.207.169:7001`)

### Optional (For Logging)

If you want logging, the client will create:
- `client.log` in the current working directory (if logging is enabled)

## How to Deploy

### Step 1: Copy the Binary

```powershell
# Copy just the executable
Copy-Item "target\release\client.exe" "C:\path\to\destination\client.exe"
```

Or for testing:
```powershell
Copy-Item "target\release\random_json_test.exe" "C:\path\to\destination\random_json_test.exe"
```

### Step 2: Run on New Machine

```powershell
# Basic usage
.\client.exe

# Or with test binary
.\random_json_test.exe 162.221.207.169 7001 10
```

## Code Reference: Where Payload Data Comes From

### Certificate Handling
**Location:** `src/quic_utils.rs::create_client_config()`

```rust
pub fn create_client_config() -> Result<ClientConfig, Box<dyn std::error::Error>> {
    // Uses AcceptAllVerifier - accepts ANY certificate
    // No external certificate files needed!
    struct AcceptAllVerifier;
    impl ServerCertVerifier for AcceptAllVerifier {
        fn verify_server_cert(...) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())  // Always accepts
        }
        // ... other methods
    }
    
    // Empty root certificate store - no CA certs needed
    let mut tls_config = TlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    
    // ALPN protocols hardcoded in binary
    tls_config.alpn_protocols = vec![
        b"h3".to_vec(),
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
        b"doq".to_vec(),
    ];
    
    // Disable certificate validation
    tls_config.dangerous().set_certificate_verifier(Arc::new(AcceptAllVerifier));
    
    // ... rest of config
}
```

### Message Structures
**Location:** `src/messages.rs`

All message types are compiled into the binary:
- `TrackerAnnounceRequest`
- `FileRequest`
- `AiRequest`
- `TrackerAnnounceResponse`
- `FileResponse`
- `AiResponse`
- `ErrorResponse`

### Client Initialization
**Location:** `src/quic_client.rs::QuicClient::new()`

```rust
pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let client_config = create_client_config()?;  // No files needed
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    Ok(Self { endpoint })
}
```

## Testing on New Machine

### Quick Test

```powershell
# Test connection to server
.\random_json_test.exe 162.221.207.169 7001 5
```

### Expected Output

```
========================================
Random JSON & File Request Test
========================================
Server: 162.221.207.169:7001
Iterations: 5

[1] Testing: AiRequest - ...
  [OK] AI query successful!
    Answer length: 123, Tokens: Some(45)
    Duration: 0.15s

[2] Testing: FileRequest - hello_world.txt
  [OK] File download successful!
    Size: 13 bytes
    Duration: 0.12s

...
```

## Troubleshooting

### "Connection timeout"
- Check firewall allows UDP outbound
- Verify server is running: `162.221.207.169:7001`
- Check network connectivity

### "Certificate error"
- Should not happen - client accepts all certificates
- If it does, check `src/quic_utils.rs` for `AcceptAllVerifier`

### "ALPN negotiation failed"
- Should not happen - multiple ALPN protocols built-in
- Check server is using compatible ALPN

## Summary

**The client is 100% self-contained:**
- ✅ No certificate files needed
- ✅ No configuration files needed
- ✅ No external dependencies
- ✅ Just copy `client.exe` and run it!

**Payload data sources:**
- Certificates: Generated/validated in-memory (accepts all)
- ALPN: Hardcoded in binary (`h3`, `h2`, `http/1.1`, `doq`)
- Messages: Compiled structs in binary
- Configuration: All defaults, no files needed

