# Test Rust Client Connection to Ubuntu Server
# This script tests the connection between the Rust client and Python server

param(
    [string]$Server = "162.221.207.169",
    [int]$Port = 7001
)

$ErrorActionPreference = "Continue"

Write-Host "========================================"
Write-Host "Rust Client → Ubuntu Server Connection Test"
Write-Host "========================================"
Write-Host "Server: $Server`:$Port"
Write-Host ""

# Step 1: Verify client executable exists
Write-Host "Step 1: Checking Rust client executable..."
if (-not (Test-Path "target\release\client.exe")) {
    Write-Host "  [FAIL] Client executable not found" -ForegroundColor Red
    Write-Host "  Building client..."
    cargo build --release
    if (-not (Test-Path "target\release\client.exe")) {
        Write-Host "  [FAIL] Build failed" -ForegroundColor Red
        exit 1
    }
}
Write-Host "  [OK] Client executable found" -ForegroundColor Green
$exe = Get-Item "target\release\client.exe"
Write-Host "    Path: $($exe.FullName)"
Write-Host "    Size: $([math]::Round($exe.Length / 1MB, 2)) MB"
Write-Host ""

# Step 2: Verify torrent file exists
Write-Host "Step 2: Checking torrent file..."
$torrentFile = "seed\hello_world.txt.torrent"
if (-not (Test-Path $torrentFile)) {
    Write-Host "  [FAIL] Torrent file not found: $torrentFile" -ForegroundColor Red
    Write-Host "  Please ensure seed files are present"
    exit 1
}
Write-Host "  [OK] Torrent file found: $torrentFile" -ForegroundColor Green
Write-Host ""

# Step 3: Test network connectivity
Write-Host "Step 3: Testing network connectivity..."
$ping = Test-Connection -ComputerName $Server -Count 1 -Quiet -ErrorAction SilentlyContinue
if ($ping) {
    Write-Host "  [OK] Server is reachable (ping)" -ForegroundColor Green
} else {
    Write-Host "  [WARN] Server ping failed (may be firewall, but QUIC may still work)" -ForegroundColor Yellow
}
Write-Host ""

# Step 4: Test QUIC connection
Write-Host "Step 4: Testing QUIC connection..."
Write-Host "  Downloading hello_world.txt from server..."
$testFile = "test_ubuntu_connection_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"

$startTime = Get-Date
$result = .\target\release\client.exe download $torrentFile $testFile $Server $Port 2>&1
$endTime = Get-Date
$duration = ($endTime - $startTime).TotalSeconds

Write-Host ""
Write-Host "Connection attempt completed in $([math]::Round($duration, 2)) seconds"
Write-Host ""

# Step 5: Analyze results
if (Test-Path $testFile) {
    $size = (Get-Item $testFile).Length
    $content = Get-Content $testFile -Raw -ErrorAction SilentlyContinue
    
    Write-Host "  [SUCCESS] Connection successful!" -ForegroundColor Green
    Write-Host "  Downloaded file: $testFile"
    Write-Host "  File size: $size bytes"
    if ($content) {
        Write-Host "  File content: $content"
    }
    Write-Host ""
    Write-Host "========================================"
    Write-Host "✓ Rust client CAN connect to Ubuntu server!" -ForegroundColor Green
    Write-Host "========================================"
    Write-Host ""
    Write-Host "The cross-platform QUIC connection is working correctly."
    Write-Host ""
    
    # Clean up
    Remove-Item $testFile -Force -ErrorAction SilentlyContinue
    
    exit 0
} elseif ($result -match "Error:.*TimedOut|Connection.*timeout|TimedOut") {
    Write-Host "  [TIMEOUT] Connection timed out" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Possible causes:"
    Write-Host "  1. Server is not running on Ubuntu"
    Write-Host "  2. Port 7001/UDP is blocked by firewall"
    Write-Host "  3. Server is not listening on $Server`:$Port"
    Write-Host ""
    Write-Host "To fix:"
    Write-Host "  1. SSH to Ubuntu server: ssh dbertrand@$Server"
    Write-Host "  2. Start server: cd /home/dbertrand && python3 quic_tracker_server.py 7001"
    Write-Host "  3. Verify port is open: sudo netstat -tuln | grep 7001"
    Write-Host ""
    Write-Host "The client code is configured correctly - server needs to be running."
    exit 1
} elseif ($result -match "Error:") {
    Write-Host "  [ERROR] Connection error detected" -ForegroundColor Red
    Write-Host ""
    $errorLines = $result | Select-String "Error:" | Select-Object -First 3
    foreach ($line in $errorLines) {
        Write-Host "    $line" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "Check server logs and client.log for details."
    exit 1
} else {
    Write-Host "  [UNKNOWN] Unexpected result" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Client output:"
    $result | Select-Object -Last 10 | ForEach-Object { Write-Host "    $_" }
    exit 1
}

