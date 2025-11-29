param(
    [int]$Hours = 12
)

$server = "162.221.207.169"
$port   = 7001

# Torrent → seed file mapping (excluding large files)
$tests = @(
    @{ Torrent="seed\hello_world.txt.torrent"; SeedFile="hello_world.txt" },
    @{ Torrent="seed\small.txt.torrent";       SeedFile="small.txt"       },
    @{ Torrent="seed\medium.bin.torrent";      SeedFile="medium.bin"      },
    @{ Torrent="seed\data.json.torrent";       SeedFile="data.json"       },
    @{ Torrent="seed\log.txt.torrent";        SeedFile="log.txt"         }
)

$logDir = "soak_logs"
$outDir = "downloaded\soak"
New-Item -ItemType Directory -Force -Path $logDir,$outDir | Out-Null

$summaryLog = Join-Path $logDir "summary.log"

function Get-FileHashHex([string]$path) {
    if (-not (Test-Path $path)) { return $null }
    (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLower()
}

# Fetch reference copies from server for integrity comparison
# NOTE: Requires SSH password to be set in environment variable SSH_PASSWORD
# or use SSH key authentication instead
Write-Host "Fetching reference seed files from server for hash comparison..."
$refDir = "seed_refs"
New-Item -ItemType Directory -Force -Path $refDir | Out-Null

# Use SSH key authentication if available, otherwise prompt for password
$usePassword = $env:SSH_PASSWORD
foreach ($t in $tests) {
    $seed     = $t.SeedFile
    $localRef = Join-Path $refDir $seed
    # Use ${} to avoid PowerShell treating ":" after $server as a drive separator
    $remote   = "dbertrand@${server}:/home/dbertrand/seed/$seed"
    if ($usePassword) {
        & pscp -pw $usePassword $remote $localRef 2>&1 | Out-Null
    } else {
        # Try without password (SSH key)
        & pscp $remote $localRef 2>&1 | Out-Null
    }
}

$stopTime = (Get-Date).AddHours($Hours)
$iter = 0

Add-Content $summaryLog "=== Soak test start: $(Get-Date) for $Hours hours ==="

while ((Get-Date) -lt $stopTime) {
    foreach ($t in $tests) {
        $iter++
        $torrent  = $t.Torrent
        $seedFile = $t.SeedFile
        $ts       = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        $safeName = $seedFile.Replace("\","_")
        $outFile  = Join-Path $outDir ("{0:000000}_{1}" -f $iter,$safeName)

        Write-Host "[$ts] ITER $iter - Downloading $seedFile via $torrent → $outFile"

        $cmd = ".\target\release\client.exe download `"$torrent`" `"$outFile`" $server $port"
        $out = Invoke-Expression "$cmd 2>&1"

        # Per-iteration client log
        $iterLog = Join-Path $logDir ("iter_{0:000000}_{1}.log" -f $iter,$safeName)
        $out | Set-Content $iterLog

        $status = "OK"
        $reason = ""

        # Check for different types of errors (case-insensitive, multiline)
        $outString = $out | Out-String
        $isConnectionError = $outString -match "(?i)(Error:.*TimedOut|Connection.*timeout|Connection.*failed|TimedOut|timeout.*connection)"
        $isServerError = $outString -match "(?i)(Error:.*server|connection refused|server.*error)"
        $isFileError = $outString -match "(?i)(Error:.*file|file.*not found|not found)"
        $hasError = $outString -match "(?i)Error:"
        $fileExists = Test-Path $outFile

        if (-not $fileExists) {
            if ($isConnectionError) {
                $status = "RETRY"
                $reason = "connection timeout - will retry"
            }
            elseif ($isServerError) {
                $status = "RETRY"
                $reason = "server error - will retry"
            }
            elseif ($hasError) {
                # Try to extract the actual error message
                $errorMsg = ($out | Select-String -Pattern '(?i)Error:.*' | ForEach-Object { $_.Line.Trim() } | Select-Object -First 1)
                if (-not $errorMsg) {
                    $errorMsg = "unknown error"
                }
                $status = "FAIL"
                $reason = "client error: $errorMsg"
            }
            else {
                $status = "FAIL"
                $reason = "missing output file"
            }
        }
        elseif ($hasError -and -not $isConnectionError -and -not $isServerError) {
            # File exists but there was an error (might be partial download)
            $status = "WARN"
            $reason = "error during download but file exists"
        }
        else {
            # File exists, verify hash
            $localHash = Get-FileHashHex $outFile
            $refPath   = Join-Path $refDir $seedFile
            $refHash   = Get-FileHashHex $refPath

            if (-not $refHash) {
                $status = "WARN"
                $reason = "no reference hash for $seedFile"
            }
            elseif ($localHash -ne $refHash) {
                $status = "FAIL"
                $reason = "hash mismatch"
            }
        }

        $line = "{0} ITER={1} FILE={2} STATUS={3} REASON={4}" -f $ts,$iter,$seedFile,$status,$reason
        Add-Content $summaryLog $line
        
        # Color code output
        $color = switch ($status) {
            "OK" { "Green" }
            "WARN" { "Yellow" }
            "RETRY" { "Cyan" }
            "FAIL" { "Red" }
            default { "White" }
        }
        Write-Host $line -ForegroundColor $color

        # For RETRY status, wait longer before next attempt
        if ($status -eq "RETRY") {
            Start-Sleep -Seconds 5
        } else {
            Start-Sleep -Seconds 2
        }

        if ((Get-Date) -ge $stopTime) { break }
    }
}

Add-Content $summaryLog "=== Soak test end: $(Get-Date) ==="
Write-Host "Soak test complete. Summary: $summaryLog"


