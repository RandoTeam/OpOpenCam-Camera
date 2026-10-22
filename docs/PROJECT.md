# Project

## Vision

A **rootless, professional camera** for the OnePlus 13 that we control end to end:
a Rust Camera2 engine, a fast native Compose UI, and a detail-preserving image
pipeline that does not turn photographs into “watercolor”.

## Problem

- The stock OPlus camera is a system app with proprietary APS/neural processing;
  it is not modifiable or redistributable.
- GCam/LMC ports produce pleasant stills but are closed, slow in burst, and cannot
  switch lenses smoothly; their Motion Photos have no audio or low-resolution video.
- Third-party camera apps are restricted by the vendor HAL and rarely expose 50 MP
  RAW, 10-bit streams or smooth logical zoom.

## Goals

| ID | Goal |
|---|---|
| G1 | Direct Camera2 NDK control of preview, stills and video |
| G2 | Deep, natural HDR: unclipped highlights, detailed shadows, preserved texture |
| G3 | 50 MP RAW and a calibrated DNG workflow |
| G4 | Smooth lens switching and zoom |
| G5 | Live Photo: 6 s (3 s before + 3 s after), 4:3, stereo audio |
| G6 | 4K 4:3 30 fps and 4K 16:9 60 fps video |
| G7 | A fast, beautiful Compose UI with real animations |
| G8 | Open, auditable, legally clean public source |
| G9 | Full clean parity with stock/module features where technically possible |
| G10 | Pro mode: manual ISO/shutter/WB/focus, overlays, RAW |
| G11 | 1080p 120, 720p 60, front video, stereo audio, EIS/OIS |
| G12 | Portrait, macro, night, burst, panorama, document scanning |
| G13 | Original filters/LUT engine, watermark editor, basic gallery/editor |
| G14 | 10-bit P010/HDR10 and Android Ultra HDR where supported |

## Non-goals

- Rooting, unlocking the bootloader, flashing or modifying system partitions.
- Redistributing stock APKs, vendor libraries, LUTs, EEPROM or calibration data.
- Rebuilding Google Camera or shipping Google’s proprietary HDR+ pipeline.
- Copying the GPL-3.0 `com.tlsu.opluscamerapro` code or its vendor-hook approach.
- Cloning vendor-only features that cannot be implemented cleanly: 50 MP JPEG,
  4K 120, front 4K, Dolby Vision, Hasselblad/GR/LUMO/XPAN branding, AI super-res
  and vendor AI gallery models.
- Supporting every Android device in the first release; the target is OnePlus 13.

See [FEATURE_PARITY.md](FEATURE_PARITY.md) for the full mapping and the honest
“not portable” list.

## Target and constraints

- OnePlus 13 `PJZ110`, Snapdragon 8 Elite, ColorOS 16 / Android 16.
- Camera2 NDK + public Android APIs; no root.
- App package `com.oneplus.camera` for Chi HAL interoperability; JNI class package
  remains `com.opopencam.camera`.
- Public build contains only original code and generic calibration.

## Success criteria

**Near term (alpha):** 12 MP JPEG, RAW 12/50 DNG, HDR 3×, lens switching, CI and
release automation. *Mostly done — see [STATUS.md](STATUS.md).*

**Mid term (beta):** calibrated DNG, HDR quality on textured/moving scenes,
Live Photo 6 s with stereo audio, 4K 4:3/16:9 video, 1080p 120, Pro mode, smooth
zoom, gallery integration.

**Long term (stable):** portrait/macro/night/burst/panorama, user LUTs and original
filters, watermark editor, 10-bit P010/HDR10, Ultra HDR, gallery editor, polished
UX, reproducible releases. Vendor-only features remain documented as not portable.

## Principles

1. **Evidence over claims.** “Done” requires device logs/files, not just code.
2. **Small increments.** One feature, one branch, one PR, one changelog entry.
3. **No fake success.** If the HAL returns 1920×1440 for a 50 MP JPEG, say so.
4. **Preserve detail.** No aggressive neural denoise or over-sharpening.
5. **Public/private separation.** Vendor research stays private.
6. **Legal hygiene.** MIT for our code; no proprietary assets in public.
7. **Clean-room parity.** Reproduce behaviour from specs and tests, never copy
   GPL/vendor code; a clean alternative is better than a cloned trademark.
8. **User safety.** Re-enable the stock camera after tests; never wipe data.
9. **Benefit over checkbox.** Do not ship a feature that produces worse photos just
   to match a list; document the trade-off.

## Roles

- **Product owner:** defines priorities and accepts releases.
- **Engineer/agent:** implements, tests, documents and reports evidence.
- **CI:** enforces formatting, tests, Clippy, build and version consistency.
- **Device:** the OnePlus 13; the final source of truth for camera behavior.
