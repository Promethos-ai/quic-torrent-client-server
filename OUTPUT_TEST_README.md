# Output Test Files

This document describes all files and directories needed to test the output/download portion of the QUIC torrent client.

## Test Scripts

### `test_output.ps1`
Comprehensive test script that:
- Downloads all test files from the server
- Verifies file integrity using SHA256 hash comparison
- Compares downloaded files against reference files
- Generates detailed test reports
- Saves logs for each download attempt

**Usage:**
```powershell
.\test_output.ps1
.\test_output.ps1 -Server 162.221.207.169 -Port 7001
```

### `verify_output_setup.ps1`
Verification script that checks:
- All required directories exist
- All seed files are present
- All torrent files are present
- Reference files are available
- Client executable is built

**Usage:**
```powershell
.\verify_output_setup.ps1
```

### `soak_test.ps1`
Long-running soak test that:
- Continuously downloads files for a specified duration (default: 12 hours)
- Verifies file integrity on each download
- Logs all results to `soak_logs/summary.log`
- Saves per-iteration logs

**Usage:**
```powershell
.\soak_test.ps1
.\soak_test.ps1 -Hours 24
```

## Required Directories

### `downloaded/soak/`
Output directory for files downloaded during the soak test. Files are named with iteration numbers:
- `000001_hello_world.txt`
- `000002_small.txt`
- etc.

### `downloaded/test_output/`
Output directory for files downloaded during the `test_output.ps1` script.

### `seed_refs/`
Reference files fetched from the server for hash comparison. These are the "ground truth" files used to verify downloaded files are correct.

### `soak_logs/`
Log directory for the soak test:
- `summary.log` - Summary of all test iterations
- `iter_XXXXXX_<filename>.log` - Per-iteration client output logs

### `test_output_logs/`
Log directory for the `test_output.ps1` script:
- `<filename>.log` - Client output for each test file

## Test Files

### Seed Files (in `seed/`)
- `hello_world.txt` (17 bytes)
- `small.txt` (35 bytes)
- `medium.bin` (500,000 bytes)
- `large.bin` (20,000,000 bytes)
- `data.json` (39 bytes)
- `log.txt` (1,500,000 bytes)

### Torrent Files (in `seed/`)
- `hello_world.txt.torrent`
- `small.txt.torrent`
- `medium.bin.torrent`
- `large.bin.torrent`
- `data.json.torrent`
- `log.txt.torrent`

### Reference Files (in `seed_refs/`)
These are fetched from the server and used for hash comparison:
- `hello_world.txt`
- `small.txt`
- `medium.bin`
- `large.bin`
- `data.json`
- `log.txt`

## Hash Verification

All tests use SHA256 hashing to verify file integrity:
- Downloaded files are hashed
- Reference files are hashed
- Hashes are compared byte-for-byte
- Mismatches are reported as failures

## Quick Start

1. **Verify setup:**
   ```powershell
   .\verify_output_setup.ps1
   ```

2. **Run quick test:**
   ```powershell
   .\test_output.ps1
   ```

3. **Run long soak test:**
   ```powershell
   .\soak_test.ps1 -Hours 12
   ```

## Server Configuration

Default server settings:
- **Server:** `162.221.207.169`
- **Port:** `7001`
- **Protocol:** QUIC

These can be overridden in the test scripts using parameters.

## Expected Results

### Successful Test
- All files download successfully
- All hash comparisons pass
- No errors in logs
- Output files match reference files exactly

### Failed Test
- Download errors (connection, timeout, etc.)
- Hash mismatches (file corruption)
- Missing files
- Client errors

All failures are logged with detailed error messages and saved to log files for analysis.


