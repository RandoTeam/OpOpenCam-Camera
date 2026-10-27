# Release key management

The signing key is project infrastructure. Losing it forces every installed beta
to be uninstalled once, and a lost key cannot be recovered from GitHub Secrets
(they are write-only). The 0.5.3 key was lost when its temporary folder was
cleaned; this procedure exists so that cannot happen again.

## Single source of truth

- Private keystore: `%USERPROFILE%\.opopencam\opopencam-release.jks`
  (RSA 4096, alias `opopencam`, validity 10000 days).
- Passwords: `%USERPROFILE%\.opopencam\password.txt` (same ACL as the keystore).
- Never place the keystore in a temporary directory, the repository, a build
  output, or a chat/ticket. Temporary folders are cleaned by the system.

## Backups (3-2-1)

1. Working copy: `%USERPROFILE%\.opopencam\`.
2. Local backup: `%USERPROFILE%\Documents\opopencam-keys\` (keystore + password).
3. Off-machine backup: password manager or encrypted archive outside this PC.

A backup is only valid after it has been verified; see below.

## Verification

```bash
python3 rust_camera/scripts/check_signing.py
```

The script reads `rust_camera/EXPECTED_CERT_SHA256`, runs `apksigner` on the
release APK and fails if the certificate differs. The release workflow runs the
same check, so a fallback to a runner debug key or an accidental key change
cannot be published.

Expected certificate: `CN=OpOpenCam Release, O=RandoTeam, C=RU`.

## Rotation procedure

1. Create the new keystore outside the repository (RSA 4096, alias `opopencam`).
2. Back it up twice and verify both copies with `keytool -list`.
3. Update the four GitHub secrets:
   `OPOPENCAM_KEYSTORE_BASE64`, `OPOPENCAM_KEYSTORE_PASSWORD`,
   `OPOPENCAM_KEY_ALIAS`, `OPOPENCAM_KEY_PASSWORD`.
4. Update `rust_camera/EXPECTED_CERT_SHA256` and the fingerprint in
   `docs/RELEASING.md`.
5. Update the local `rust_camera/android_app/keystore.properties`.
6. Build locally and with CI, compare both fingerprints, and note in the release
   that installation over the previous key needs one uninstall.
