#!/usr/bin/env python3
"""Ensure VERSION, Cargo.toml and app/build.gradle agree."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    version = read(ROOT / "VERSION").strip()
    cargo = read(ROOT / "rust_core" / "Cargo.toml")
    gradle = read(ROOT / "android_app" / "app" / "build.gradle")

    cargo_match = re.search(r'^version\s*=\s*"([^"]+)"', cargo, re.MULTILINE)
    gradle_match = re.search(r'versionName\s+"([^"]+)"', gradle)
    code_match = re.search(r"versionCode\s+(\d+)", gradle)

    errors = []
    if not cargo_match:
        errors.append("Cargo.toml: version not found")
    elif cargo_match.group(1) != version:
        errors.append(f"Cargo.toml version {cargo_match.group(1)!r} != VERSION {version!r}")

    if not gradle_match:
        errors.append("app/build.gradle: versionName not found")
    elif gradle_match.group(1) != version:
        errors.append(f"app/build.gradle versionName {gradle_match.group(1)!r} != VERSION {version!r}")

    if not code_match:
        errors.append("app/build.gradle: versionCode not found")

    if errors:
        print("Version check failed:")
        for error in errors:
            print(f"  - {error}")
        return 1

    print(f"Versions match: {version} (versionCode {code_match.group(1)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
