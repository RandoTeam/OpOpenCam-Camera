# Run the full local verification suite.
# Requires Rust + aarch64 target, Android NDK, JDK 21.

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
$Core = Join-Path $Root "rust_core"
$App = Join-Path $Root "android_app"

function Find-Ndk {
    if ($env:ANDROID_NDK_HOME) { return $env:ANDROID_NDK_HOME }
    $sdk = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
    foreach ($c in @((Join-Path $sdk "ndk\28.2.13676358"), (Join-Path $sdk "ndk\27.3.13750724"))) {
        if (Test-Path $c) { return $c }
    }
    throw "Android NDK not found. Set ANDROID_NDK_HOME."
}

function Invoke-Native([string]$Name, [string]$Command, [string]$WorkDir) {
    Write-Host ""
    Write-Host "==> $Name"
    Push-Location $WorkDir
    try {
        cmd /c $Command
        if ($LASTEXITCODE -ne 0) { throw "$Name failed with exit code $LASTEXITCODE" }
    } finally {
        Pop-Location
    }
}

Invoke-Native "cargo fmt --check" "cargo fmt --check" $Core
Invoke-Native "cargo test (host)" "cargo test" $Core

$Ndk = Find-Ndk
$Toolchain = Join-Path $Ndk "toolchains\llvm\prebuilt\windows-x86_64\bin"
$Linker = Join-Path $Toolchain "aarch64-linux-android35-clang.cmd"
if (-not (Test-Path $Linker)) { $Linker = Join-Path $Toolchain "aarch64-linux-android26-clang.cmd" }
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $Linker
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_AR = Join-Path $Toolchain "llvm-ar.exe"

Invoke-Native "cargo clippy (Android target)" "cargo clippy --target aarch64-linux-android -- -D warnings" $Core
Invoke-Native "version consistency" "python scripts\check_version.py" $Root

$Jdk = $env:JAVA_HOME
if (-not $Jdk) {
    $Jdk = Get-ChildItem "C:\Program Files\Eclipse Adoptium\jdk-21*" -Directory -ErrorAction SilentlyContinue |
        Select-Object -First 1 -ExpandProperty FullName
}
if ($Jdk) { $env:JAVA_HOME = $Jdk }
Invoke-Native "Gradle assembleDebug" "gradlew.bat :app:assembleDebug" $App

Write-Host ""
Write-Host "All checks passed."
