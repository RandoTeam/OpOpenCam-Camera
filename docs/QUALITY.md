# Quality Contract

The goal is a professional, natural image — not a phone-style “AI” render.

## Principles

1. **Preserve texture.** Luma denoise is gentle; chroma denoise may be stronger.
2. **Protect highlights.** Soft-knee roll-off and exposure fusion; clipped whites are
   a failure, not a style.
3. **Lift shadows without noise.** Shadow detail is recovered with noise-aware weights,
   not a global brightness boost.
4. **No hallucinated detail.** No neural upscaling, sky replacement or synthetic bokeh
   in the base pipeline. Portrait bokeh is an explicit, labelled mode with depth or
   segmentation — never silently applied.
5. **Natural color.** Neutral white balance, accurate skin tones, restrained saturation.
   Branded/vendor LUTs are not bundled; user-supplied LUTs are optional looks.
6. **10-bit where possible.** P010 capture and HEIF/HDR output are first-class targets,
   but only when the actual bit depth/metadata is verified.
7. **Measure, don’t guess.** Every quality claim is compared on the same scene against
   LMC 8.3 and the stock camera.

## Pipeline target

```
RAW / YUV frames (3–9, exposure-bracketed or ZSL)
        │
        ├─ alignment (gyro + feature)
        ├─ ghost / motion masks
        ├─ merge (Mertens fusion or radiance map)
        ├─ chroma denoise + gentle luma denoise
        ├─ highlight roll-off + filmic tone curve
        ├─ 3D LUT (optional)
        └─ JPEG / HEIF 10-bit
```

## Metrics for comparisons

- highlight clipping percentage;
- shadow noise (SNR in dark patches);
- detail retention (texture/edge acutance, no halo metric);
- color error on a ColorChecker;
- capture + processing latency;
- file size and thermal cost.

## Parity quality gates

- **Video:** stable frame rate, correct duration, A/V sync, no dropped audio; 4K 4:3
  must be real 4096×3072, not an upscale.
- **Live Photo:** 6.0 s ±50 ms, one HEVC video + one stereo AAC track, Motion Photo
  XMP opens; a silent/low-res clip is a failure.
- **10-bit/HDR:** verified bit depth and HDR10/HLG/gainmap metadata; “HDR” in the
  UI without a real HDR file is a failure.
- **Pro:** manual values actually applied (read back from capture result); no fake
  ISO/shutter labels.
- **Portrait/macro:** depth/stack present; edges natural; no halo cutouts.
- **Filters/watermark:** original assets only; no Hasselblad/GR/LUMO/XPAN marks or
  branded LUTs. LUTs are user-supplied.
- **Gallery/editor:** non-destructive export, correct orientation/colour, no quality
  loss beyond the chosen JPEG settings.
- **25 MP:** closed by evidence — session accepted 5888×4416, JPEG was 1920×1440.
  No upscale and no 25 MP label.

## Anti-goals

- “Watercolor” denoise that removes pores and fabric texture.
- Over-sharpening halos.
- HDR that looks flat or has a glowing edge.
- Motion Photo with no audio or a low-resolution video track.
- Fake 25/50 MP files produced by resizing.
- Branded lookalikes without a licence.
- Shipping a feature just to match a checklist when it degrades the photo.
