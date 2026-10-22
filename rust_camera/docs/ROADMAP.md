# Roadmap

Status reflects the alpha branch. Each milestone has a measurable definition of done.

> **0.3.0-alpha.1:** device validation completed on 2026-10-22.
> M2: RAW 12/50 verified, 50 MP JPEG unavailable (HAL returns 1920×1440).
> M3: cameras 3/0/4 verified, restart ~50 ms. M4: HDR 3× verified on
> logical/ultra-wide/tele; quality tuning and motion-scene testing remain.

## M0 — Baseline (done)

- Hybrid Rust + Compose project builds.
- Preview from Camera2 NDK into `SurfaceView`.
- Real JPEG 4096×3072 with EXIF.
- Device capability matrix and reference Motion Photo analysis captured.

## M1 — Reliable still capture (done, alpha)

**Done when:**

- a clean scene produces an upright 4096×3072 JPEG;
- the app survives 30 consecutive captures;
- orientation is correct in gallery apps;
- no ANR/crash on background/foreground.

## M2 — 50 MP, RAW and DNG (RAW verified; JPEG 50 unavailable)

**Scope:**

- add an 8192×6144 JPEG reader or the `ultra.resolution.mode` vendor tag;
- add a RAW10/RAW12 `AImageReader` and feed `dng_writer`;
- expose a 12 MP / 50 MP toggle;
- measure latency, memory and thermal cost.

**Done when:**

- RAW 50 DNG (8192×6144) opens in Lightroom/Photoshop with correct color matrices;
- 50 MP RAW capture completes without OOM (verified: 96 MB DNG);
- 50 MP JPEG unavailability is detected and documented (HAL returns 1920×1440);
- the app falls back cleanly when the HAL rejects a mode.

## M3 — Lenses and zoom (verified)

**Scope:**

- open physical IDs 2 (main), 3 (ultra-wide), 4 (tele);
- use logical camera 0 + `CONTROL_ZOOM_RATIO` for the slider;
- record `activePhysicalId` and switch latency;
- add a crossfade only if the HAL produces a visible jump.

**Done when:**

- 0.6× / 1× / 3× map to the expected physical camera;
- preview resumes under 300 ms;
- no app restart is required.

## M4 — Exposure bracketing and HDR fusion (verified; quality tuning pending)

**Scope:**

- 3/5 frame capture at −3/0/+3 EV with manual sensor settings;
- alignment (gyro + feature) and ghost handling;
- Mertens fusion and/or radiance map + tone mapping;
- optional HAL `EnableSHDR` / `EnableQHDR` vendor tags;
- side-by-side comparison with LMC and stock.

**Done when:**

- highlights are not clipped and shadows retain detail;
- no visible ghosting on static scenes;
- capture + merge under 2 s on device (measured ~2 s in the alpha);
- the result is judged better than a single frame by a documented comparison.
- Verified on logical, ultra-wide and tele; textured-scene/motion comparison still
  to be captured.

## M5 — Live Photo

**Spec:** 4096×3072 4:3 HEVC 30 fps, 6.0 s (3 s before + 3 s after),
AAC 48 kHz stereo 256 kbps, JPEG 4096×3072, Google Motion Photo XMP v1.

**Scope:**

- encoded-sample ring buffer (not raw frames);
- audio ring buffer;
- muxer and XMP assembly using `motion_photo_xmp`;
- optional GainMap.

**Done when:**

- `ffprobe` reports 6.0 s, one HEVC video track and one stereo AAC track;
- Google Photos opens it as a Motion Photo;
- A/V sync error < 50 ms;
- file size and thermal behaviour are documented.

## M6 — Video and polish

- 4K 4:3 30 fps and 4K 16:9 60 fps recording;
- 10-bit P010 / dynamic-range profile where the encoder accepts it;
- 48 kHz stereo audio;
- long-recording thermal test;
- Compose UI polish, animations, gallery, settings;
- beta releases.

## Cross-cutting

- CI: fmt, clippy (`-D warnings`), host tests, Android build;
- version consistency check;
- no personal media in the repository;
- every release has a changelog and a test report.
