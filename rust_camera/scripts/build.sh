#!/usr/bin/env bash
# Build the Rust engine and the Compose APK on Linux/macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/rust_core"
APP="$ROOT/android_app"

if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
  echo "ANDROID_NDK_HOME is required" >&2
  exit 1
fi

case "$(uname -s)" in
  Linux*) HOST_TAG=linux-x86_64 ;;
  Darwin*) HOST_TAG=darwin-x86_64 ;;
  *) echo "Unsupported host: $(uname -s)" >&2; exit 1 ;;
esac

TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"
LINKER="$TOOLCHAIN/aarch64-linux-android35-clang"
if [[ ! -x "$LINKER" ]]; then
  LINKER="$TOOLCHAIN/aarch64-linux-android26-clang"
fi
if [[ ! -x "$LINKER" ]]; then
  echo "aarch64 clang wrapper not found under $TOOLCHAIN" >&2
  exit 1
fi

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$LINKER"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_AR="$TOOLCHAIN/llvm-ar"

echo "==> Rust release build"
(cd "$CORE" && cargo build --release --target aarch64-linux-android)

mkdir -p "$APP/app/src/main/jniLibs/arm64-v8a"
cp "$CORE/target/aarch64-linux-android/release/libopopencam.so" \
   "$APP/app/src/main/jniLibs/arm64-v8a/libopopencam.so"
echo "==> Copied libopopencam.so"

echo "==> Compose APK"
(cd "$APP" && chmod +x gradlew && ./gradlew :app:assembleDebug)

echo ""
echo "APK: $APP/app/build/outputs/apk/debug/app-debug.apk"
