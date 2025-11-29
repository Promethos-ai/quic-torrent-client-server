# View Server Logs - Real-time or recent logs from Ubuntu server

param(
    [switch]$Follow,
    [int]$Lines = 50
)

$ErrorActionPreference = "Continue"

$server = "162.221.207.169"
$logFile = "/home/dbertrand/quic_tracker_server.log"
$pass = $env:SSH_PASSWORD

if (-not $pass) {
    Write-Host "=== SERVER LOG VIEWER ===" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "SSH_PASSWORD environment variable not set."
    Write-Host ""
    Write-Host "To view server logs, you have two options:"
    Write-Host ""
    Write-Host "Option 1: Set SSH_PASSWORD and use this script"
    Write-Host "  `$env:SSH_PASSWORD = 'your_password'"
    Write-Host "  .\view_server_logs.ps1"
    Write-Host "  .\view_server_logs.ps1 -Follow  # For live tail"
    Write-Host ""
    Write-Host "Option 2: SSH directly to server"
    Write-Host "  ssh dbertrand@$server"
    Write-Host "  tail -f $logFile  # Live logs"
    Write-Host "  tail -50 $logFile  # Last 50 lines"
    Write-Host ""
    Write-Host "Option 3: Use SSH key (if configured)"
    Write-Host "  plink -ssh dbertrand@$server 'tail -f $logFile'"
    Write-Host ""
    exit 0
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server Log Viewer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server: $server"
Write-Host "Log File: $logFile"
Write-Host ""

if ($Follow) {
    Write-Host "Following log file (live updates)..." -ForegroundColor Yellow
    Write-Host "Press Ctrl+C to stop"
    Write-Host ""
    
    # Use plink to tail -f the log file
    # Note: plink doesn't support -f directly, so we'll poll
    $lastLineCount = 0
    while ($true) {
        $logs = echo y | plink -ssh -batch dbertrand@$server -pw $pass "tail -n +$lastLineCount $logFile 2>/dev/null" 2>&1
        if ($logs) {
            $newLines = $logs | Where-Object { $_ -and $_.Trim() -ne "" }
            foreach ($line in $newLines) {
                if ($line -match "ERROR|Exception|Traceback") {
                    Write-Host "  $line" -ForegroundColor Red
                } elseif ($line -match "WARNING|WARN") {
                    Write-Host "  $line" -ForegroundColor Yellow
                } elseif ($line -match "BYTE LEVEL|ALPN|negotiate") {
                    Write-Host "  $line" -ForegroundColor Cyan
                } elseif ($line -match "Received|Sending|File|Connection|Stream") {
                    Write-Host "  $line" -ForegroundColor Green
                } else {
                    Write-Host "  $line"
                }
                $lastLineCount++
            }
        }
        Start-Sleep -Seconds 2
    }
} else {
    Write-Host "Fetching last $Lines lines..."
    Write-Host ""
    
    $logs = echo y | plink -ssh -batch dbertrand@$server -pw $pass "tail -$Lines $logFile 2>/dev/null" 2>&1
    
    if ($logs) {
        $logs | ForEach-Object {
            if ($_ -match "ERROR|Exception|Traceback") {
                Write-Host "  $_" -ForegroundColor Red
            } elseif ($_ -match "WARNING|WARN") {
                Write-Host "  $_" -ForegroundColor Yellow
            } elseif ($_ -match "BYTE LEVEL|ALPN|negotiate") {
                Write-Host "  $_" -ForegroundColor Cyan
            } elseif ($_ -match "Received|Sending|File|Connection|Stream") {
                Write-Host "  $_" -ForegroundColor Green
            } elseif ($_ -match "INFO.*Server|Starting|listening") {
                Write-Host "  $_" -ForegroundColor Magenta
            } else {
                Write-Host "  $_"
            }
        }
    } else {
        Write-Host "  [WARN] Could not fetch logs" -ForegroundColor Yellow
        Write-Host "  Server may not be running or log file doesn't exist"
    }
    
    Write-Host ""
    Write-Host "To follow logs in real-time, run:"
    Write-Host "  .\view_server_logs.ps1 -Follow" -ForegroundColor Yellow
}

