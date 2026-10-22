# ADR 0001 — Hybrid Rust engine + Compose UI

**Status:** accepted

## Context

The original Rust track used a `NativeActivity` with a hand-drawn framebuffer UI.
The camera core was a prototype: preview worked, capture was a placeholder, and the
UI was slow to polish. A full Rust UI would delay a usable camera.

## Decision

- **Rust** owns Camera2 NDK, still/HDR/RAW processing, DNG and Motion Photo.
- **Kotlin/Compose** owns the UI, permissions, lifecycle and animations.
- A small JNI bridge exposes start/stop, capture, zoom and mode.

## Alternatives

1. Full Rust `NativeActivity` + egui/wgpu: more work, less native UI, harder Android
   integration.
2. Kotlin Camera2 only: simpler, but the image pipeline would be Kotlin/Java rather
   than the intended Rust engine.

## Consequences

- Fast native engine and a modern UI.
- Two languages and a JNI boundary.
- Preview is zero-copy: the HAL writes directly into `SurfaceView`.
- UI polish is no longer blocked by the camera core.
