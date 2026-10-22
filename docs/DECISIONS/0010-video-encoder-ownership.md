# ADR 0010 — Video ownership: Rust Camera2, Kotlin MediaCodec

**Status:** accepted; 4K 4:3 verified

## Context

The app needs 4K video now and encoded-sample Live Photo later. The Camera2 NDK
session is owned by Rust; `MediaRecorder` is capped by the vendor profile
(HEVC ~73.5 Mbps, no HFR) and does not expose encoded samples, so it cannot be
reused for Live Photo.

## Decision

- Rust owns `ACameraDevice` / `ACameraCaptureSession`, preview and the optional
  MediaCodec input surface.
- Kotlin owns `MediaCodec` (HEVC video, AAC audio), `MediaMuxer` and `AudioRecord`.
- JNI starts a recording session with preview + video surface; stopping drops the
  session and the UI restores preview with `nativeStartPreview`.
- The future Live Photo ring lives on the Kotlin side, where encoded samples are
  already available.

## Alternatives

1. **MediaRecorder:** simplest, but bitrate/profile caps and no encoded-sample
   access; rejected for Live Photo.
2. **MediaCodec inside Rust:** more JNI/codec surface and harder Kotlin UI
   integration; rejected for the first slice.
3. **Vendor recorder:** root/vendor-only; rejected.

## Consequences

- 4K 4:3 4096×3072@30 HEVC + AAC stereo is working on device.
- Recording requires a short Camera2 session rebuild (~preview freeze).
- Audio/video timestamps must be normalized; the current audio tail is
  ~0.15–0.2 s longer and needs trimming before beta.
- 4K 16:9 60, 1080p120 and 10-bit/HDR fit the same architecture; high-speed
  sessions will need a constrained-session variant.
