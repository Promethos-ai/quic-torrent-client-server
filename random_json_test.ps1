# Random JSON and File Request Test
# Tests server with random combinations of:
# - TrackerAnnounceRequest (JSON)
# - FileRequest (JSON)  
# - Custom JSON messages (to test error handling)

param(
    [string]$Server = "162.221.207.169",
    [int]$Port = 7001,
    [int]$Iterations = 20
)

$ErrorActionPreference = "Continue"

Write-Host "========================================"
Write-Host "Random JSON & File Request Test"
Write-Host "========================================"
Write-Host "Server: ${Server}:${Port}"
Write-Host "Iterations: $Iterations"
Write-Host ""

# Build the test binary if needed
$binPath = "target\release\random_json_test.exe"
if (-not (Test-Path $binPath)) {
    Write-Host "Building random_json_test..." -ForegroundColor Yellow
    cargo build --release --bin random_json_test
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed!" -ForegroundColor Red
        exit 1
    }
}

# Run the test
Write-Host "Running random JSON test..." -ForegroundColor Green
Write-Host ""

& $binPath $Server $Port $Iterations

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Test completed!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[FAILED] Test exited with code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

