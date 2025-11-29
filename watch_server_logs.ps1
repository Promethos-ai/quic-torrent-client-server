# Watch Server Logs - Opens a window to follow server logs in real-time

param(
    [int]$Lines = 100
)

$server = "162.221.207.169"
$logFile = "/home/dbertrand/quic_tracker_server.log"
$pass = $env:SSH_PASSWORD

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server Log Viewer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server: $server"
Write-Host "Log File: $logFile"
Write-Host ""

if ($pass) {
    Write-Host "Opening live log viewer window..." -ForegroundColor Yellow
    Write-Host ""
    
    # Create a script that will run in a new window
    $watchScript = @"
`$server = "$server"
`$logFile = "$logFile"
`$pass = "$pass"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "LIVE SERVER LOGS - Press Ctrl+C to stop" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Server: `$server"
Write-Host "Log File: `$logFile"
Write-Host ""
Write-Host "Fetching last $Lines lines, then following..."
Write-Host ""

# First show last N lines
`$logs = echo y | plink -ssh -batch dbertrand@`$server -pw `$pass "tail -$Lines `$logFile 2>/dev/null" 2>&1
if (`$logs) {
    `$logs | ForEach-Object {
        if (`$_ -match "ERROR|Exception|Traceback") {
            Write-Host "  `$_" -ForegroundColor Red
        } elseif (`$_ -match "WARNING|WARN") {
            Write-Host "  `$_" -ForegroundColor Yellow
        } elseif (`$_ -match "BYTE LEVEL|ALPN|negotiate") {
            Write-Host "  `$_" -ForegroundColor Cyan
        } elseif (`$_ -match "Received|Sending|File|Connection|Stream") {
            Write-Host "  `$_" -ForegroundColor Green
        } elseif (`$_ -match "INFO.*Server|Starting|listening") {
            Write-Host "  `$_" -ForegroundColor Magenta
        } else {
            Write-Host "  `$_"
        }
    }
}

Write-Host ""
Write-Host "--- Following new log entries (every 2 seconds) ---" -ForegroundColor Yellow
Write-Host ""

# Follow logs
`$lastCount = `$logs.Count
while (`$true) {
    Start-Sleep -Seconds 2
    `$newLogs = echo y | plink -ssh -batch dbertrand@`$server -pw `$pass "tail -n +`$(`$lastCount + 1) `$logFile 2>/dev/null" 2>&1
    if (`$newLogs) {
        `$newLogs | ForEach-Object {
            if (`$_ -and `$_.Trim() -ne "") {
                if (`$_ -match "ERROR|Exception|Traceback") {
                    Write-Host "  `$_" -ForegroundColor Red
                } elseif (`$_ -match "WARNING|WARN") {
                    Write-Host "  `$_" -ForegroundColor Yellow
                } elseif (`$_ -match "BYTE LEVEL|ALPN|negotiate") {
                    Write-Host "  `$_" -ForegroundColor Cyan
                } elseif (`$_ -match "Received|Sending|File|Connection|Stream") {
                    Write-Host "  `$_" -ForegroundColor Green
                } elseif (`$_ -match "INFO.*Server|Starting|listening") {
                    Write-Host "  `$_" -ForegroundColor Magenta
                } else {
                    Write-Host "  `$_"
                }
                `$lastCount++
            }
        }
    }
}
"@
    
    # Save script to temp file
    $tempScript = [System.IO.Path]::GetTempFileName() + ".ps1"
    $watchScript | Out-File -FilePath $tempScript -Encoding UTF8
    
    # Open in new window
    Start-Process powershell -ArgumentList "-NoExit", "-File", $tempScript -WindowStyle Normal
    
    Write-Host "  [OK] Log viewer window opened" -ForegroundColor Green
    Write-Host ""
    Write-Host "The window will show:"
    Write-Host "  - Last $Lines lines of server log"
    Write-Host "  - New log entries as they appear (every 2 seconds)"
    Write-Host "  - Color-coded output (errors=red, warnings=yellow, etc.)"
    Write-Host ""
    Write-Host "Press Ctrl+C in that window to stop following logs."
    
} else {
    Write-Host "SSH_PASSWORD not set." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To view live server logs:"
    Write-Host ""
    Write-Host "1. Set SSH password:" -ForegroundColor Cyan
    Write-Host "   `$env:SSH_PASSWORD = 'your_password'" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "2. Run this script:" -ForegroundColor Cyan
    Write-Host "   .\watch_server_logs.ps1" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "3. Or SSH directly to server:" -ForegroundColor Cyan
    Write-Host "   ssh dbertrand@$server" -ForegroundColor Yellow
    Write-Host "   tail -f $logFile" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "The log file location is:" -ForegroundColor Cyan
    Write-Host "   $logFile" -ForegroundColor Yellow
}

