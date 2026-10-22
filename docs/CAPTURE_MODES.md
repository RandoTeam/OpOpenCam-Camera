# Capture Modes

The Rust engine selects one still output per session. A mode change restarts the
Camera2 session on the same `SurfaceView` surface; this keeps the HAL stream
configuration valid and avoids the simultaneous-output limit.

## Modes

| ID | UI | Reader | Output | Status |
|---|---|---|---|---|
| 0 | `12` | JPEG 4096×3072 | JPEG | verified on device |
| 1 | `50` | JPEG 8192×6144 | JPEG | **unavailable**: HAL returns 1920×1440 |
| 2 | `RAW` | RAW_SENSOR 4096×3072 | DNG | verified: 25 MB |
| 3 | `R50` | RAW_SENSOR 8192×6144 | DNG | verified: 96 MB; requires `com.oneplus.camera` |
| 4 | `HDR` | YUV_420_888 4096×3072 | JPEG | verified: 3 frames + NCC + Mertens |

Each mode is a separate `AImageReader`; only preview + one still output are active
at a time.

## 50 MP

The device advertises `8192×6144` in `availableRawSizes` for all rear cameras.
The standard stream list does not show a 50 MP JPEG stream. With the
`com.oneplus.camera` package identity and the vendor session parameters
(`ultra.resolution.mode = 1`, `EnableIdealRAW = 1`, package name), the engine gets:

- **RAW 50 → DNG 8192×6144** — verified (96 MB, 16-bit, DNG 1.4);
- **JPEG 50 → 1920×1440** — the HAL ignores the 8192×6144 JPEG reader and returns
  a small JPEG. The engine now parses the JPEG SOF and rejects the mismatch
  instead of saving a fake 50 MP file.

Use `R50` for maximum resolution. The old patched clone with the same package must
be uninstalled before installing OpOpenCam.

## RAW and DNG

- Reader format: `RAW_SENSOR` (32), 16-bit words.
- DNG writer: `dng_writer::write_dng_16bit` (Adobe DNG 1.4, RGGB, TIFF little-endian).
- Default black/white levels: 64 / 1023 (10-bit). The 50 MP DNG was verified as
  8192×6144, 16-bit, DNG 1.4, 96 MB.
- Per-sensor colour matrices and exact black/white levels still need calibration
  from the captured `/odm` data.
- 50 MP RAW is ~96 MB; watch memory, storage and thermal behaviour.

## HDR 3×

1. Three still requests are submitted with `AE_EXPOSURE_COMPENSATION = -6 / 0 / +6`
   (1/3 EV units, i.e. −2/0/+2 EV).
2. The YUV reader delivers three 4096×3072 frames.
3. Each frame is converted to RGB with BT.601 coefficients.
4. Frames are aligned to the middle bracket with normalised cross-correlation on
   luma (coarse-to-fine, translation only) and shifted.
5. `hdr::exposure_fusion_aligned` computes Mertens-style weights:
   contrast × saturation × well-exposedness.
6. The fused image is rotated 90° to portrait and encoded as JPEG (quality 92).

Current limitation: translation-only alignment. Rotation, parallax and moving
subjects can still ghost; feature/optical-flow alignment is the next quality step.
HAL `EnableSHDR`/`EnableQHDR` vendor tags will be tested as an additional path.

## Lens mapping

| UI | Camera ID | Role |
|---|---|---|
| `0.6` | 3 | ultra-wide |
| `1x` | 0 | logical SAT (main at 1×) |
| `3x` | 4 | tele |
| API | 2 | physical main (direct) |

Changing lens restarts the session on the same surface. Physical IDs 3 / 0 / 4
were verified live; the session start now takes ~50 ms because the engine retries
`setRepeatingRequest` instead of waiting for an `onReady` callback this HAL never
invokes. The app package is `com.oneplus.camera` (the JNI class package remains
`com.opopencam.camera`); uninstall the old patched clone before installing.

## Fallback semantics

`nativeStartPreview` returns `true` when a session starts. `nativeGetActiveMode`
returns the mode actually active:

- `-1` — no session;
- `0..4` — active mode after any HAL fallback.

The Compose layer uses `nativeGetActiveMode()` to choose the file extension and to
show `HAL fallback: …`.

## Live validation checklist

- [x] 12 MP JPEG: 4096×3072 verified.
- [ ] 50 MP JPEG: **unavailable** — HAL returns 1920×1440; rejected by SOF check.
- [x] RAW 12: DNG 4096×3072, 25 MB verified.
- [x] RAW 50: DNG 8192×6144, 96 MB verified (requires `com.oneplus.camera`).
- [ ] DNG colour/black-level calibration in Lightroom/Photoshop.
- [x] HDR 3×: 3 YUV frames → fused JPEG on logical/ultra-wide/tele.
- [ ] HDR quality on a textured static scene and a moving subject.
- [x] Lens 0.6/1×/3×: cameras 3/0/4 verified; restart ~50 ms.
- [x] Mode/lens restart keeps preview alive.
- [ ] Thermal test: 10 HDR and 10 RAW 50 captures.
