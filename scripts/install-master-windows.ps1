# bo2mp: install (or update) the online server -- iw4l-master, the relay and
# lobby list -- on a Windows VPS (Windows Server 2016 or later, or Windows
# 10/11). The VPS needs no game files, no GPU and nothing built: the
# iw4l-master.exe your own PC builds (play\iw4l-master.exe, made by
# run-bo2.cmd) is copied over beside this script.
#
#   1. Copy this folder (the "vps-kit": this script and iw4l-master.exe) to the
#      VPS -- over Remote Desktop, copy and paste works.
#   2. On the VPS: right-click install.cmd > Run as administrator (or, in an
#      administrator PowerShell: powershell -ExecutionPolicy Bypass -File
#      install-master-windows.ps1).
#
# What it does: copies iw4l-master.exe to C:\iw4l; makes the certificates
# once (C:\iw4l\certs, kept on every update so players' ca.pem stays
# valid); opens UDP 4433 in Windows Firewall; registers a scheduled task that
# starts the server with Windows (as SYSTEM, before anyone logs in) and
# restarts it if it stops; starts it; prints what to put in online.cfg and
# the ca.pem to hand to players. Run it again with a newer iw4l-master.exe to
# update. -Uninstall removes the task and the firewall rule (certificates
# kept). It records nothing on disk (docs/MASTER.md).
param(
    [string]$Exe = (Join-Path $PSScriptRoot 'iw4l-master.exe'),
    [string]$Dir = 'C:\iw4l',
    [int]$Port = 4433,
    [switch]$Uninstall
)
$ErrorActionPreference = 'Stop'
$Task = 'iw4l-master'
$Rule = 'iw4l-master (UDP)'

function Say($t) { Write-Host "`n== $t" -ForegroundColor Cyan }

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) { throw 'Run as administrator (right-click install.cmd > Run as administrator).' }

if ($Uninstall) {
    Say 'Removing the server'
    Stop-ScheduledTask -TaskName $Task -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $Task -Confirm:$false -ErrorAction SilentlyContinue
    Get-Process iw4l-master -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-NetFirewallRule -DisplayName $Rule -ErrorAction SilentlyContinue
    Write-Host "Removed. $Dir (with the certificates) is kept; delete it by hand if you want."
    return
}

if (-not (Test-Path $Exe)) { throw "iw4l-master.exe not found at ${Exe}: copy play\iw4l-master.exe from your PC beside this script." }

Say 'Installing'
New-Item -ItemType Directory -Force $Dir, (Join-Path $Dir 'certs') | Out-Null
Stop-ScheduledTask -TaskName $Task -ErrorAction SilentlyContinue
Get-Process iw4l-master -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 500
$target = Join-Path $Dir 'iw4l-master.exe'
if ((Resolve-Path $Exe).Path -ne $target) { Copy-Item $Exe $target -Force }

Say 'Certificates'
$certs = Join-Path $Dir 'certs'
& $target gen-certs --out $certs
if ($LASTEXITCODE -ne 0) { throw 'gen-certs failed' }
# Only administrators and SYSTEM read the server's key.
icacls (Join-Path $certs 'server.key') /inheritance:r /grant:r 'SYSTEM:R' 'Administrators:F' | Out-Null

Say "Firewall (UDP $Port)"
Remove-NetFirewallRule -DisplayName $Rule -ErrorAction SilentlyContinue
New-NetFirewallRule -DisplayName $Rule -Direction Inbound -Protocol UDP -LocalPort $Port -Action Allow -Profile Any | Out-Null
Write-Host "Windows Firewall: UDP $Port open. If your VPS provider has its own firewall or security group, allow UDP $Port there too."

Say 'Starts with Windows'
$serveArgs = "serve --bind 0.0.0.0:$Port --cert `"$certs\server.pem`" --key `"$certs\server.key`""
$action = New-ScheduledTaskAction -Execute $target -Argument $serveArgs -WorkingDirectory $Dir
$trigger = New-ScheduledTaskTrigger -AtStartup
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable `
    -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
$principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
Register-ScheduledTask -TaskName $Task -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null
Start-ScheduledTask -TaskName $Task
Start-Sleep -Seconds 2
if (-not (Get-Process iw4l-master -ErrorAction SilentlyContinue)) {
    throw "The server did not start. Try it by hand to see why: & '$target' $serveArgs"
}
Write-Host 'iw4l-master is running (scheduled task "iw4l-master", starts with Windows).'

$ip = try { (Invoke-RestMethod -Uri 'https://api.ipify.org' -TimeoutSec 5) } catch {
    (Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } | Select-Object -First 1).IPAddress
}
$ca = Get-Content (Join-Path $certs 'ca.pem') -Raw
Copy-Item (Join-Path $certs 'ca.pem') (Join-Path $PSScriptRoot 'ca.pem') -Force -ErrorAction SilentlyContinue

Write-Host @"

==================================================================
 Done. The online server is running on this VPS (UDP $Port).

 1. On every player's PC, in online.cfg next to run-bo2.cmd:

      server=${ip}:$Port
      ca=online\certs\ca.pem
      name=iw4l-prod

 2. Give every player the ca.pem below (also saved beside this script
    and at $certs\ca.pem); they save it as online\certs\ca.pem in
    their project folder:

$ca
 Update: copy a newer iw4l-master.exe here and run install.cmd again.
 Stop: Stop-ScheduledTask iw4l-master. Remove: install.cmd -Uninstall.
==================================================================
"@ -ForegroundColor Green
