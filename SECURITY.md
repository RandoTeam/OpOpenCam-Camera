# Security Policy

## Reporting a vulnerability

Please use GitHub's **private security advisory** for this repository
(Security → Advisories → Report a vulnerability). Do not open a public issue for a
security problem.

## Scope

Supported: `0.3.x-alpha` and later. Alpha builds are debug-signed and intended for
testing, not as a daily camera.

## Secrets and personal data

This repository must never contain:

- keystores (`*.jks`, `*.keystore`), private keys or signing passwords;
- GitHub tokens, API keys or cloud credentials;
- device serial numbers, account names or personal photos;
- raw device dumps or vendor binaries.

The public repository is audited for these before release. If you find a leak,
report it privately so it can be removed from history.

## Build integrity

Build from source with the documented scripts. Release APKs are published with a
SHA-256 checksum; verify it before installing:

```bash
sha256sum -c app-release.apk.sha256
```

## Network and telemetry

The app currently declares no `INTERNET` permission and contains no analytics,
telemetry or crash-upload SDK. Photos are written to app-specific external storage.
