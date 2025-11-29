# Random Client Test - Run client 10 times with random file requests
# Tests both announce and file download functionality

param(
    [string]$Server = "162.221.207.169",
    [int]$Port = 7001,
    [int]$Iterations = 10
)

$ErrorActionPreference = "Continue"

Write-Host "========================================"
Write-Host "Random Client Test - $Iterations Iterations"
Write-Host "========================================"
Write-Host "Server: $Server`:$Port"
Write-Host ""

# Get available torrent files
$torrentFiles = @(
    "seed\hello_world.txt.torrent",
    "seed\small.txt.torrent",
    "seed\medium.bin.torrent",
    "seed\data.json.torrent",
    "seed\log.txt.torrent"
)

# Filter to only existing files
$availableTorrents = $torrentFiles | Where-Object { Test-Path $_ }

if ($availableTorrents.Count -eq 0) {
    Write-Host "[FAIL] No torrent files found in seed directory" -ForegroundColor Red
    exit 1
}

Write-Host "Available files: $($availableTorrents.Count)"
$availableTorrents | ForEach-Object { Write-Host "  - $_" }
Write-Host ""

# Create output directory
$outDir = "downloaded\random_test"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Results tracking
$results = @()
$successCount = 0
$failCount = 0

Write-Host "Starting $Iterations random tests..."
Write-Host ""

for ($i = 1; $i -le $Iterations; $i++) {
    # Randomly select a torrent file
    $randomIndex = Get-Random -Minimum 0 -Maximum $availableTorrents.Count
    $selectedTorrent = $availableTorrents[$randomIndex]
    $fileName = [System.IO.Path]::GetFileNameWithoutExtension($selectedTorrent)
    
    # Create unique output filename
    $outputFile = Join-Path $outDir "${i}_${fileName}_$(Get-Date -Format 'HHmmss').txt"
    
    Write-Host "[$i/$Iterations] Testing: $fileName" -ForegroundColor Cyan
    Write-Host "  Torrent: $selectedTorrent"
    Write-Host "  Output: $outputFile"
    Write-Host "  Server: $Server`:$Port"
    
    # Run client
    $startTime = Get-Date
    $result = .\target\release\client.exe download $selectedTorrent $outputFile $Server $Port 2>&1
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    # Check result
    $success = $false
    $errorMsg = ""
    
    if (Test-Path $outputFile) {
        $fileSize = (Get-Item $outputFile).Length
        if ($fileSize -gt 0) {
            $success = $true
            $successCount++
            Write-Host "  [OK] Download successful!" -ForegroundColor Green
            Write-Host "    Size: $fileSize bytes"
            Write-Host "    Duration: $([math]::Round($duration, 2))s"
        } else {
            $failCount++
            $errorMsg = "File created but empty"
            Write-Host "  [FAIL] File is empty" -ForegroundColor Red
            Remove-Item $outputFile -Force -ErrorAction SilentlyContinue
        }
    } elseif ($result -match "Error:.*TimedOut|Connection.*timeout") {
        $failCount++
        $errorMsg = "Connection timeout"
        Write-Host "  [TIMEOUT] Connection failed" -ForegroundColor Yellow
    } elseif ($result -match "Error:") {
        $failCount++
        $errorMsg = ($result | Select-String "Error:" | Select-Object -First 1).Line.Trim()
        Write-Host "  [FAIL] Error: $errorMsg" -ForegroundColor Red
    } else {
        $failCount++
        $errorMsg = "Unknown failure"
        Write-Host "  [FAIL] Unknown error" -ForegroundColor Red
    }
    
    $results += @{
        Iteration = $i
        File = $fileName
        Success = $success
        Duration = $duration
        Error = $errorMsg
        Size = if ($success) { (Get-Item $outputFile).Length } else { 0 }
    }
    
    Write-Host ""
    
    # Small delay between requests
    Start-Sleep -Milliseconds 500
}

Write-Host "========================================"
Write-Host "Test Results Summary"
Write-Host "========================================"
Write-Host ""
Write-Host "Total tests: $Iterations"
Write-Host "  Success: $successCount" -ForegroundColor Green
Write-Host "  Failed:  $failCount" -ForegroundColor $(if ($failCount -gt 0) { "Red" } else { "Green" })
Write-Host ""

Write-Host "Detailed Results:"
Write-Host ""
foreach ($r in $results) {
    $status = if ($r.Success) { "[OK]" } else { "[FAIL]" }
    $color = if ($r.Success) { "Green" } else { "Red" }
    Write-Host "$status Iteration $($r.Iteration): $($r.File) - $([math]::Round($r.Duration, 2))s" -ForegroundColor $color
    if (-not $r.Success -and $r.Error) {
        Write-Host "    Error: $($r.Error)" -ForegroundColor Yellow
    } elseif ($r.Success) {
        Write-Host "    Size: $($r.Size) bytes" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "Output files: $outDir"
Write-Host ""

if ($successCount -eq $Iterations) {
    Write-Host "[SUCCESS] All tests passed!" -ForegroundColor Green
    exit 0
} elseif ($successCount -gt 0) {
    Write-Host "[PARTIAL] Some tests passed ($successCount/$Iterations)" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "[FAIL] All tests failed" -ForegroundColor Red
    exit 1
}

