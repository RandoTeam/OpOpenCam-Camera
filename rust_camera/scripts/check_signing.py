#!/usr/bin/env python3
"""Fail the build if the release APK is signed with an unexpected certificate.

The expected digest lives in rust_camera/EXPECTED_CERT_SHA256. Rotating the key
is a deliberate act: update the file, the GitHub secrets and docs/RELEASING.md
in the same change. This guard makes an accidental fallback to the runner debug
key (or a leftover personal key) impossible to ship unnoticed.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = ROOT / "EXPECTED_CERT_SHA256"
APK = ROOT / "android_app" / "app" / "build" / "outputs" / "apk" / "release" / "app-release.apk"


def find_apksigner() -> str:
    import os

    sdk = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT")
    if not sdk:
        raise SystemExit("ANDROID_HOME is not set")
    candidates = sorted(Path(sdk, "build-tools").glob("*/apksigner"))
    if not candidates:
        raise SystemExit("apksigner not found in the Android SDK")
    return str(candidates[-1])


def main() -> int:
    if not APK.is_file():
        print(f"APK not found: {APK}")
        return 1
    expected = EXPECTED.read_text(encoding="utf-8").strip().lower()
    out = subprocess.run(
        [find_apksigner(), "verify", "--print-certs", str(APK)],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    print(out.strip())
    match = re.search(r"certificate SHA-256 digest: ([0-9a-f]+)", out)
    if not match:
        print("Could not read the signing certificate")
        return 1
    actual = match.group(1).lower()
    if actual != expected:
        print(f"Signing certificate mismatch: {actual} != {expected}")
        return 1
    print(f"Signing certificate matches the expected key ({actual})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
