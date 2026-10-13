@echo off
rem bo2mp: install or update the online server on this Windows VPS (run as administrator).
rem install.cmd -Uninstall removes it (certificates kept).
title Online server - install
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-master-windows.ps1" %*
pause
