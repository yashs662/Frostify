# Debug launcher (REMOVABLE — delete the `debug/` dir + `automation`
# feature to rip out). Kills any stale opal.exe (the lock that breaks
# `cargo run` mid-session), builds with the automation feature, and runs
# against a JSON config/script.
#
#   .\debug\run.ps1                 # uses debug/home.json
#   .\debug\run.ps1 debug/liked.json
param([string]$Config = "debug/home.json")

Get-Process opal -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

cargo build --features automation
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

& ".\target\debug\opal.exe" --config $Config
