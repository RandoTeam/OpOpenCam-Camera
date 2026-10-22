# Quality Contract

The goal is a professional, natural image — not a phone-style “AI” render.

## Principles

1. **Preserve texture.** Luma denoise is gentle; chroma denoise may be stronger.
2. **Protect highlights.** Soft-knee roll-off and exposure fusion; clipped whites are
   a failure, not a style.
3. **Lift shadows without noise.** Shadow detail is recovered with noise-aware weights,
   not a global brightness boost.
4. **No hallucinated detail.** No neural upscaling, sky replacement or synthetic bokeh
   in the base pipeline.
5. **Natural color.** Neutral white balance, accurate skin tones, restrained saturation;
   Hasselblad/Leica LUTs are optional looks, not the default.
6. **10-bit where possible.** P010 capture and HEIF/AV1 output are first-class targets.
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

## Anti-goals

- “Watercolor” denoise that removes pores and fabric texture.
- Over-sharpening halos.
- HDR that looks flat or has a glowing edge.
- Motion Photo with no audio or a low-resolution video track.
