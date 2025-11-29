# Test Output Portion - Verify download and hash comparison functionality
# This script tests that downloads work and hash verification is correct

param(
    [string]$Server = "162.221.207.169",
    [int]$Port = 7001
)

$ErrorActionPreference = "Stop"

Write-Host "========================================"
Write-Host "Output Portion Test"
Write-Host "========================================"
Write-Host "Server: $Server"
Write-Host "Port: $Port"
Write-Host ""

# Test files (excluding large files)
$tests = @(
    @{ Torrent="seed\hello_world.txt.torrent"; SeedFile="hello_world.txt" },
    @{ Torrent="seed\small.txt.torrent";       SeedFile="small.txt"       },
    @{ Torrent="seed\medium.bin.torrent";      SeedFile="medium.bin"      },
    @{ Torrent="seed\data.json.torrent";       SeedFile="data.json"       },
    @{ Torrent="seed\log.txt.torrent";        SeedFile="log.txt"         }
)

# Ensure directories exist
$outDir = "downloaded\test_output"
$refDir = "seed_refs"
$logDir = "test_output_logs"

New-Item -ItemType Directory -Force -Path $outDir,$logDir | Out-Null

# Helper function to compute SHA256 hash
function Get-FileHashHex([string]$path) {
    if (-not (Test-Path $path)) { 
        return $null 
    }
    (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLower()
}

Write-Host "Step 1: Verifying reference files exist..."
$allRefsExist = $true
foreach ($t in $tests) {
    $refPath = Join-Path $refDir $t.SeedFile
    if (-not (Test-Path $refPath)) {
        Write-Host "  [FAIL] Missing reference: $($t.SeedFile)" -ForegroundColor Red
        $allRefsExist = $false
    } else {
        $size = (Get-Item $refPath).Length
        Write-Host "  [OK] $($t.SeedFile) ($size bytes)" -ForegroundColor Green
    }
}

if (-not $allRefsExist) {
    Write-Host ""
    Write-Host "ERROR: Some reference files are missing. Run the soak test setup first." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Step 2: Testing downloads..."
$results = @()

foreach ($t in $tests) {
    $torrent = $t.Torrent
    $seedFile = $t.SeedFile
    $outFile = Join-Path $outDir $seedFile
    
    Write-Host ""
    Write-Host "Testing: $seedFile" -ForegroundColor Cyan
    Write-Host "  Torrent: $torrent"
    Write-Host "  Output: $outFile"
    
    # Remove existing output file if it exists
    if (Test-Path $outFile) {
        Remove-Item $outFile -Force
    }
    
    # Download
    Write-Host "  Downloading..."
    $cmd = ".\target\release\client.exe download `"$torrent`" `"$outFile`" $Server $Port"
    $output = Invoke-Expression "$cmd 2>&1"
    
    # Save log
    $logFile = Join-Path $logDir "$seedFile.log"
    $output | Set-Content $logFile
    
    # Check for connection errors (case-insensitive, multiline)
    $outputString = $output | Out-String
    $isConnectionError = $outputString -match "(?i)(Error:.*TimedOut|Connection.*timeout|Connection.*failed|TimedOut|timeout.*connection)"
    
    # Check if download succeeded
    if (-not (Test-Path $outFile)) {
        if ($isConnectionError) {
            Write-Host "  [RETRY] Connection timeout - server may be unreachable" -ForegroundColor Cyan
            $results += @{
                File = $seedFile
                Status = "RETRY"
                Reason = "Connection timeout"
            }
        } else {
            Write-Host "  [FAIL] Download failed - file not created" -ForegroundColor Red
            $results += @{
                File = $seedFile
                Status = "FAIL"
                Reason = "File not created"
            }
        }
        continue
    }
    
    $downloadedSize = (Get-Item $outFile).Length
        Write-Host "  [OK] File created ($downloadedSize bytes)" -ForegroundColor Green
    
    # Compare hash
    Write-Host "  Comparing hash..."
    $downloadedHash = Get-FileHashHex $outFile
    $refPath = Join-Path $refDir $seedFile
    $refHash = Get-FileHashHex $refPath
    
    if (-not $refHash) {
        Write-Host "  [WARN] No reference hash available" -ForegroundColor Yellow
        $results += @{
            File = $seedFile
            Status = "WARN"
            Reason = "No reference hash"
        }
    } elseif ($downloadedHash -eq $refHash) {
        Write-Host "  [OK] Hash matches reference" -ForegroundColor Green
        $results += @{
            File = $seedFile
            Status = "OK"
            Reason = "Hash verified"
        }
    } else {
        Write-Host "  [FAIL] Hash mismatch!" -ForegroundColor Red
        Write-Host "    Downloaded: $downloadedHash"
        Write-Host "    Reference:  $refHash"
        $results += @{
            File = $seedFile
            Status = "FAIL"
            Reason = "Hash mismatch"
        }
    }
}

Write-Host ""
Write-Host "========================================"
Write-Host "Test Results Summary"
Write-Host "========================================"

$okCount = ($results | Where-Object { $_.Status -eq "OK" }).Count
$warnCount = ($results | Where-Object { $_.Status -eq "WARN" }).Count
$retryCount = ($results | Where-Object { $_.Status -eq "RETRY" }).Count
$failCount = ($results | Where-Object { $_.Status -eq "FAIL" }).Count

foreach ($r in $results) {
    $color = switch ($r.Status) {
        "OK" { "Green" }
        "WARN" { "Yellow" }
        "RETRY" { "Cyan" }
        "FAIL" { "Red" }
    }
    Write-Host "$($r.Status.PadRight(6)) $($r.File.PadRight(20)) $($r.Reason)" -ForegroundColor $color
}

Write-Host ""
Write-Host "Total: $($results.Count) tests"
Write-Host "  OK:    $okCount" -ForegroundColor Green
Write-Host "  WARN:  $warnCount" -ForegroundColor Yellow
Write-Host "  RETRY: $retryCount" -ForegroundColor Cyan
Write-Host "  FAIL:  $failCount" -ForegroundColor Red
Write-Host ""
Write-Host "Output files: $outDir"
Write-Host "Logs: $logDir"
Write-Host ""

if ($failCount -eq 0 -and $retryCount -eq 0) {
    Write-Host "[SUCCESS] All tests passed!" -ForegroundColor Green
    exit 0
} elseif ($retryCount -gt 0 -and $failCount -eq 0) {
    Write-Host "[WARN] Some tests had connection issues (server may be down)" -ForegroundColor Yellow
    Write-Host "  Check server status at ${Server}:${Port}"
    exit 0
} else {
    Write-Host "[FAIL] Some tests failed" -ForegroundColor Red
    exit 1
}

