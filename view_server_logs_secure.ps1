# View Server Logs - Prompts for password securely

param(
    [switch]$Follow,
    [int]$Lines = 50
)

$ErrorActionPreference = "Continue"

$server = "162.221.207.169"
$logFile = "/home/dbertrand/quic_tracker_server.log"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server Log Viewer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server: $server"
Write-Host "Log File: $logFile"
Write-Host ""

# Prompt for password securely
$securePass = Read-Host "Enter SSH password" -AsSecureString
$pass = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
    [Runtime.InteropServices.Marshal]::SecureStringToBSTR($securePass)
)

Write-Host ""
Write-Host "Connecting to server..." -ForegroundColor Yellow

if ($Follow) {
    Write-Host "Following log file (live updates)..." -ForegroundColor Yellow
    Write-Host "Press Ctrl+C to stop"
    Write-Host ""
    
    # Show last N lines first
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
    }
    
    Write-Host ""
    Write-Host "--- Following new log entries (every 2 seconds) ---" -ForegroundColor Yellow
    Write-Host ""
    
    # Follow logs
    $lastCount = if ($logs) { $logs.Count } else { 0 }
    while ($true) {
        Start-Sleep -Seconds 2
        $newLogs = echo y | plink -ssh -batch dbertrand@$server -pw $pass "tail -n +$($lastCount + 1) $logFile 2>/dev/null" 2>&1
        if ($newLogs) {
            $newLogs | ForEach-Object {
                if ($_ -and $_.Trim() -ne "") {
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
                    $lastCount++
                }
            }
        }
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
    Write-Host "  .\view_server_logs_secure.ps1 -Follow" -ForegroundColor Yellow
}

# Clear password from memory
$pass = $null
$securePass = $null

