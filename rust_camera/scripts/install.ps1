# Install the debug APK on the connected device.
$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Apk = Join-Path $Root "android_app\app\build\outputs\apk\debug\app-debug.apk"

if (-not (Test-Path $Apk)) {
    throw "APK not found: $Apk. Run scripts/build.ps1 first."
}

$devices = adb devices | Select-String -Pattern "\sdevice$"
if (-not $devices) {
    throw "No adb device connected."
}

adb install -r -g $Apk
if ($LASTEXITCODE -ne 0) { throw "adb install failed" }
Write-Host "Installed $Apk"
