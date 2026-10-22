## Summary

<!-- What changed and why? Keep it focused on rust_camera/. -->

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo test`
- [ ] `cargo clippy --target aarch64-linux-android -- -D warnings`
- [ ] `./gradlew :app:assembleDebug`
- [ ] Live device test (if camera behavior changed) — attach the summary, not personal media

## Checklist

- [ ] No personal photos, device dumps, APKs or keystores
- [ ] Version files are unchanged unless this is a release PR
- [ ] Comments/docstrings explain non-obvious camera/HAL decisions
- [ ] Linked issue / milestone

## Known limitations

<!-- Anything intentionally left for a follow-up. -->
