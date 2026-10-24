# Stock 12 MP vs Our 12 MP - 2026-10-24

## Purpose

Same-scene comparison of the stock 12 MP JPEG and our Camera2 12 MP JPEG after
the landscape/orientation pass. The goal is to separate texture/processing
differences from the stock Ultra HDR gain map.

## Files

| Item | Stock | Ours |
|---|---|---|
| File | `IMG20261024131224.jpg` | `IMG_1792835036174.jpg` |
| Bytes | 4,203,889 | 1,846,394 |
| Dimensions | 4096x3072 | 4096x3072 |
| EXIF orientation | 1 | 3 |
| Lens | 23 mm eq, f/1.6 | 23 mm eq, f/1.6 |
| Gain map | 2048x1536 JPEG, 436,006 bytes | none |
| GainMapMax | 2.35462 (~2.35 stops) | n/a |
| MotionPhoto | none | none |

Both files were taken at 1x in the same indoor scene. The stock file is larger
mainly because it carries the gain map and a larger primary JPEG. The captures
are not pixel-aligned.

## Crop metrics

A 2000x1800 crop around the bottles:

| Metric | Stock | Ours |
|---|---|---|
| Mean luma | 132.3 | 128.9 |
| Std luma | 53.1 | 57.2 |
| Laplacian variance | 157.6 | 154.9 |
| Mean gradient magnitude | 31.3 | 28.6 |

The local-detail metrics are close. In this crop the stock JPEG is slightly
more contrasty, but it does **not** show an obvious watercolor/neural advantage
over our SDR JPEG. The main measurable difference is the embedded gain map.

## Visual check

A side-by-side 100% crop shows:

- nearly the same bottle edges, droplets and label texture;
- stock slightly warmer/contrastier;
- our file slightly larger in shadow contrast (std), likely due to tone;
- no stock-only detail that would justify its extra processing in this scene.

This is a single scene and not a replacement for a controlled tripod/colour
target test. It does show that our 12 MP SDR base is viable and that the stock
advantage here is primarily the Ultra HDR gain map.

## Implication

The next quality feature is **our own gain map**, not more sharpening or AI:

1. keep the natural SDR base;
2. generate a gain map from our HDR bracket or RAW;
3. attach a standard Ultra HDR XMP container;
4. verify in an HDR-aware viewer and compare against stock's 2.35-stop map.

The existing neutral RAW renderer is the other half of the quality path: it
gives a natural base without vendor ANR/ASF processing, and can later carry the
same gain-map container.

## Caveats

- One scene, not pixel-aligned, no colour target.
- Stock gain-map range is scene-dependent (2.35 stops here; 1.74-1.95 in other
  scenes).
- The stock SDR primary was not reconstructed through the gain map in this
  comparison; the display test earlier did not observe an HDR layer.
