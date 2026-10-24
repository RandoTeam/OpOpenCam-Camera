# Stock Gallery Post-Processing

The user observed that after capture, opening the stock gallery shows the photo
"processing" for a couple of seconds and then becoming more detailed, with a
deeper dynamic range. We must determine whether this is:

1. a background rewrite of the original file;
2. a display-time rendering (gain map, HDR/ProXDR, GPU shader, AI upscale);
3. an on-device NPU enhancement cached as a new rendition;
4. a cloud enhancement;
5. merely the difference between a fast thumbnail and the full-resolution
   decode.

This matters because if the stock gallery is doing the heavy lifting, we may be
able to publish compatible files to MediaStore and let it render them. But we
must not build the product on an undisclosed, permission-protected dependency
without evidence.

## 1. Known starting points

- Stock gallery: `com.coloros.gallery3d`, main activity
  `.app.MainActivity`.
- It exposes `PhotoEditorActivity` with `android.intent.action.EDIT` and
  `GalleryActionTransitActivity`; these are candidate integration points.
- AI service: `com.oplus.aiunit`.
- Vendor model inventory includes detail/GAN models, segmentation models and a
  Live Photo cover enhancer. Some may be used by the gallery rather than the
  camera.
- The camera writes to `DCIM/Camera`; our app writes to `DCIM/OpOpenCam`.

## 1a. First result (2026-10-24)

A stock still with an Ultra HDR gain map (`IMG20261024012832.jpg`, 5,499,799
bytes) was opened in the stock gallery. The SHA-256, size and mtime were
unchanged after 10 s, and screenshots at 0.2-10 s were byte-identical. For this
file, the gallery did **not** rewrite the original during viewing.

The embedded 2048x1536 gain map declares `GainMapMax=1.94892` (~1.95 stops) and
has real per-pixel variation (mean 36.7, std 33.2, p99 205; correlation 0.825
with primary luma). It is a genuine HDR gain map, not a flat placeholder.

A display-time check found that the panel supports HDR (types 1-4, 2000 nits),
but while the gallery showed this file `HdrBrightnessModifier` reported
`NO_HDR`, `mIsHdrLayerPresent=false`, and the visible layers had
`hdr metadata types=0`. In this test the gallery did **not** present an HDR
layer for the gain-map still. The earlier visual improvement is therefore not
proven to be HDR; it may be a full-resolution decode or an SDR render. An
HDR-aware measurement or a different gallery mode is still required.
Full report: [reports/2026-10-24-gallery-gainmap.md](reports/2026-10-24-gallery-gainmap.md).

## 2. Hypotheses

| # | Hypothesis | Confirming evidence | Rejecting evidence |
|---|---|---|---|
| G1 | The original JPEG is rewritten in the background | file hash/mtime/size change; MediaStore `generation_modified` changes; temp file appears | hash/mtime unchanged |
| G2 | A sidecar rendition is written | new `.jpg`/`.xmp`/cache file appears | no new files |
| G3 | The gallery renders a gain map / HDR/ProXDR variant | file has gain-map metadata; screenshot brightness/contrast changes with display HDR; no file rewrite | no gain map; no display change |
| G4 | An NPU model upscales/denoises at display time | QNN/HTP activity; detail appears only at 100%; cached texture | no model activity; detail already in file |
| G5 | A cloud service enhances the photo | network traffic; airplane mode disables it; account dependency | works offline |
| G6 | The "processing" is only thumbnail -> full decode | improvement is instant once full decode completes; no file/model activity | improvement continues after decode |

## 3. Test assets

Use three controlled files:

1. **Stock JPEG** from the stock camera, 12 MP, 4:3, HDR off.
2. **Stock HDR JPEG**, same scene.
3. **Our JPEG**, same scene and framing, written to `DCIM/OpOpenCam`.
4. Optional: our 50 MP DNG and a stock 50 MP/RAW file.

Keep the originals; never edit them. Work on copies when testing metadata.

Scene: the lab chart from [STOCK_CAMERA.md](STOCK_CAMERA.md), with fine text,
a Siemens star, colour patches, a gray ramp and a high-contrast edge. A face is
not required; if portrait enhancement is tested, use a printed target, not a
real person.

## 4. Experiment protocol

### G-A: file mutation

1. Record the file hash, size, mtime, EXIF/XMP and MediaStore row.
2. Open the stock gallery from the camera thumbnail.
3. Screen-record 10 s at 60 fps; take screenshots at 0.25 s intervals.
4. Wait 30 s; record hash/size/mtime/MediaStore again.
5. List the directory and app cache for new files (do not delete anything).
6. Compare.

If the hash changes, preserve both versions and diff the JPEG markers,
quantization tables and pixels. If it does not, go to G-B.

### G-B: display-time rendering

1. Keep the file unchanged.
2. Screenshot at t=0.25, 0.5, 1, 2, 5 and 10 s after opening.
3. Crop the same region and measure edge energy, local contrast, noise and
   colour.
4. Toggle display brightness/HDR mode if the gallery exposes it.
5. Compare the screenshot with a neutral decoder (Pillow/ffmpeg) of the same
   file at the same crop.
6. If the gallery image is sharper/brighter than the neutral decode, it is a
   display-time enhancement.

### G-C: network and account

1. Repeat G-A/G-B in airplane mode.
2. Repeat signed out / with a local-only album if possible.
3. Record `dumpsys netstats` for the gallery/AI UIDs before and after.
4. Do not use personal accounts or cloud albums.

### G-D: process and engine

During G-A/G-B capture:

- `ps -AT` for `com.coloros.gallery3d`, `com.oplus.aiunit`, media providers;
- logcat tags: `Gallery`, `AIUnit`, `QNN`, `HTP`, `adsprpc`, `fastrpc`,
  `HDR`, `XDR`, `ProXDR`, `UltraHDR`, `Denoise`, `SuperRes`, `Enhance`;
- Perfetto `nnapi`, `gfx`, `sched`, `freq`, `thermal`, `memory`;
- `dumpsys media.metrics` for image codecs (if any);
- thermal and battery delta.

Record inference-shaped latency and thread activity, but do not claim NPU
utilization without a counter.

### G-E: metadata compatibility

1. Copy our JPEG to `DCIM/Camera` with a test name (do not overwrite stock).
2. Open it in the stock gallery; repeat G-A/G-B.
3. Add/remove candidate metadata (gain map, XMP, vendor fields) on copies.
4. Observe whether the gallery treats our file like a stock file.
5. Do not ship vendor metadata unless its meaning and licence are understood.

### G-F: editor and share integration

1. Open our file with `ACTION_EDIT` to `PhotoEditorActivity`.
2. Save a copy; compare the result with the original (re-encode, resolution,
   metadata, quality).
3. Test `ACTION_VIEW` and share.
4. Measure latency and whether the editor requires extra permissions.
5. Decide whether an "Open in Gallery" action is acceptable as an optional
   hand-off, not as the core pipeline.

## 5. Metrics

| Metric | How |
|---|---|
| time_to_first_image | screen recording / frame timestamps |
| time_to_final_render | screenshots until stable |
| file_changed | SHA-256 before/after |
| media_store_changed | `_size`, `generation_modified`, `is_pending` |
| sidecar_created | directory diff |
| network_used | netstats, airplane-mode A/B |
| process_cpu | `/proc/<pid>/stat` deltas or Perfetto |
| npu_signal | QNN/HTP logs, inference-shaped latency |
| sharpness_delta | edge energy / MTF on fixed crop |
| contrast_delta | local contrast / histogram |
| colour_delta | patch ΔE |
| noise_delta | flat-patch sigma |
| editor_quality_loss | PSNR/SSIM + file size + metadata |

## 6. Integration options

| Option | Quality | Speed | Control | Licensing/maintenance | Verdict |
|---|---|---|---|---|---|
| A. Publish to MediaStore, let stock gallery render | stock-like if metadata matches | fast | low | depends on undocumented behaviour | useful fallback, not core |
| B. `ACTION_EDIT` hand-off to stock editor | stock editor quality | user-initiated | low | re-encode may lose quality | optional action |
| C. Bind to a gallery/AI service directly | potentially stock quality | fast | low | likely signature-protected; unstable | reject unless exported and documented |
| D. Our own GPU/NPU pipeline + own gallery | full control | tunable | high | we own it | recommended core |
| E. Hybrid: D + optional "Open in Gallery" | high | fast | high | clean | recommended product shape |

The public app must not silently depend on a private vendor service. If the
stock gallery applies a display-time gain map, we can implement an equivalent
open rendering path (gain map / tone mapping) instead of calling the vendor.

## 7. Acceptance

- We can state, with evidence, whether the stock gallery rewrites the file,
  writes a sidecar, or only changes rendering.
- We know whether network/account is required.
- We know the latency and the engine (CPU/GPU/NPU/cloud) within the rootless
  limits.
- We have a tested "Open in Gallery" path that does not degrade the original.
- The product decision (A-E) is recorded with the evidence, not assumed.

## 8. Open questions

1. Does the stock gallery use the same RRDBNet/cover enhancer for stills, or a
   different model?
2. Is the "processing" indicator tied to file writing, thumbnail generation,
   or AI inference?
3. Does the gallery apply ProXDR only on the internal display, or also export a
   gain map?
4. Can a third-party JPEG trigger the same path by carrying the right metadata?
5. Does the stock editor preserve the original when saving, or overwrite it?
