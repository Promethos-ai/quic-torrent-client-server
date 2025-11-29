# Verify Output Setup - Check all required files and directories for output testing

Write-Host "========================================"
Write-Host "Output Setup Verification"
Write-Host "========================================"
Write-Host ""

$allGood = $true

# Check directories
Write-Host "Checking directories..."
$dirs = @(
    "downloaded\soak",
    "seed_refs",
    "soak_logs"
)

foreach ($dir in $dirs) {
    if (Test-Path $dir) {
        Write-Host "  [OK] $dir" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $dir (missing)" -ForegroundColor Red
        $allGood = $false
    }
}

Write-Host ""
Write-Host "Checking seed files (local)..."
$seedFiles = @("hello_world.txt", "small.txt", "medium.bin", "large.bin", "data.json", "log.txt")
foreach ($file in $seedFiles) {
    $path = Join-Path "seed" $file
    if (Test-Path $path) {
        $size = (Get-Item $path).Length
        Write-Host "  [OK] $file ($size bytes)" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $file (missing)" -ForegroundColor Red
        $allGood = $false
    }
}

Write-Host ""
Write-Host "Checking torrent files..."
$torrentFiles = @("hello_world.txt.torrent", "small.txt.torrent", "medium.bin.torrent", "large.bin.torrent", "data.json.torrent", "log.txt.torrent")
foreach ($file in $torrentFiles) {
    $path = Join-Path "seed" $file
    if (Test-Path $path) {
        $size = (Get-Item $path).Length
        Write-Host "  [OK] $file ($size bytes)" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $file (missing)" -ForegroundColor Red
        $allGood = $false
    }
}

Write-Host ""
Write-Host "Checking reference files (for hash comparison)..."
foreach ($file in $seedFiles) {
    $path = Join-Path "seed_refs" $file
    if (Test-Path $path) {
        $size = (Get-Item $path).Length
        Write-Host "  [OK] $file ($size bytes)" -ForegroundColor Green
    } else {
        Write-Host "  [WARN] $file (missing - will be fetched during soak test)" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Checking client executable..."
if (Test-Path "target\release\client.exe") {
    Write-Host "  [OK] target\release\client.exe" -ForegroundColor Green
} else {
    Write-Host "  [FAIL] target\release\client.exe (missing - run 'cargo build --release')" -ForegroundColor Red
    $allGood = $false
}

Write-Host ""
Write-Host "========================================"
if ($allGood) {
    Write-Host "[SUCCESS] All required files and directories are present!" -ForegroundColor Green
    Write-Host ""
    Write-Host "You can now run:"
    Write-Host "  .\test_output.ps1"
    Write-Host "or"
    Write-Host "  .\soak_test.ps1 -Hours 12"
} else {
    Write-Host "[FAIL] Some required files are missing" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please ensure:"
    Write-Host "  1. All seed files exist in seed/"
    Write-Host "  2. All torrent files exist in seed/"
    Write-Host "  3. Client is built: cargo build --release"
    Write-Host "  4. Reference files are fetched (or will be auto-fetched during soak test)"
}
Write-Host ""

