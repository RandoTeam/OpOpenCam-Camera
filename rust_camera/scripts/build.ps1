# Build the Rust engine and the Compose APK.
#
# Prerequisites: Rust + aarch64-linux-android, Android SDK/NDK, JDK 21.
# Environment overrides: ANDROID_NDK_HOME, ANDROID_SDK_ROOT, JAVA_HOME.

$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent
$Core = Join-Path $Root "rust_core"
$App = Join-Path $Root "android_app"

function Find-Ndk {
    if ($env:ANDROID_NDK_HOME) { return $env:ANDROID_NDK_HOME }
    $sdk = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
    $candidates = @(
        (Join-Path $sdk "ndk\28.2.13676358"),
        (Join-Path $sdk "ndk\27.3.13750724")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return $c }
    }
    throw "Android NDK not found. Set ANDROID_NDK_HOME."
}

$Ndk = Find-Ndk
$Toolchain = Join-Path $Ndk "toolchains\llvm\prebuilt\windows-x86_64\bin"
$Linker = Join-Path $Toolchain "aarch64-linux-android35-clang.cmd"
if (-not (Test-Path $Linker)) {
    $Linker = Join-Path $Toolchain "aarch64-linux-android26-clang.cmd"
}
if (-not (Test-Path $Linker)) {
    throw "aarch64 clang wrapper not found under $Toolchain"
}

$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $Linker
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_AR = Join-Path $Toolchain "llvm-ar.exe"

Write-Host "==> Rust release build (aarch64-linux-android)"
Push-Location $Core
try {
    & cargo build --release --target aarch64-linux-android
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
} finally {
    Pop-Location
}

$So = Join-Path $Core "target\aarch64-linux-android\release\libopopencam.so"
$JniDir = Join-Path $App "app\src\main\jniLibs\arm64-v8a"
New-Item -ItemType Directory -Force $JniDir | Out-Null
Copy-Item $So (Join-Path $JniDir "libopopencam.so") -Force
Write-Host "==> Copied libopopencam.so"

$Jdk = $env:JAVA_HOME
if (-not $Jdk) {
    $Jdk = Get-ChildItem "C:\Program Files\Eclipse Adoptium\jdk-21*" -Directory -ErrorAction SilentlyContinue |
        Select-Object -First 1 -ExpandProperty FullName
}
if ($Jdk) { $env:JAVA_HOME = $Jdk }

Write-Host "==> Compose APK"
Push-Location $App
try {
    & .\gradlew.bat :app:assembleDebug
    if ($LASTEXITCODE -ne 0) { throw "Gradle build failed" }
} finally {
    Pop-Location
}

$Apk = Join-Path $App "app\build\outputs\apk\debug\app-debug.apk"
Write-Host ""
Write-Host "APK: $Apk"
