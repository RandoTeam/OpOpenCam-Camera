# Gallery and Gain Map - 2026-10-24

First controlled gallery test on a stock still that contains an Ultra HDR gain
map. The goal was to separate a file rewrite from display-time rendering.

## Test

| Item | Value |
|---|---|
| File | `IMG20261024012832.jpg` |
| MediaStore ID | 303618 |
| Gallery | `com.coloros.gallery3d/com.oppo.gallery3d.app.ViewGallery` |
| Launch | WARM, 188 ms |
| Network | airplane mode on during the test, restored afterwards |
| Captures | screenshots at 0.2, 0.6, 1.2, 2.5, 5 and 10 s |

## Result

| Check | Result |
|---|---|
| SHA-256 before | `50aa77a0b430c2b43291a149205fd5b41c0568c8096645864975c1a87907dfca` |
| SHA-256 after 10 s | identical |
| Size / mtime | 5,499,799 bytes / unchanged |
| Screenshots | byte-identical, pixel diff 0 |
| Visible processing | none in the SDR screenshots |

The gallery displayed the image immediately and did **not** rewrite it during
the 10 s window. For this file, the "processing then better" effect was not a
background file mutation. It may be a full-resolution decode, a display-time
gain-map/HDR render, or behaviour specific to another mode/file. SDR screencaps
cannot prove an HDR display change.

## Gain map

The still contains a separate 2048x1536 grayscale JPEG. Its own XMP says:

```text
hdrgm:Version = 1.0
hdrgm:GainMapMin = 0
hdrgm:GainMapMax = 1.94892
hdrgm:Gamma = 1
hdrgm:OffsetSDR = 0
hdrgm:OffsetHDR = 0
hdrgm:HDRCapacityMin = 0
hdrgm:HDRCapacityMax = 1.94892
hdrgm:BaseRenditionIsHDR = False
```

Gain-map pixel stats (8-bit):

| Stat | Still | Live Photo sample |
|---|---|---|
| Mean | 36.67 | 37.54 |
| Std | 33.24 | 32.59 |
| p1 / p50 / p99 | 7 / 28 / 205 | 7 / 28 / 204 |
| Correlation with primary luma | 0.825 | 0.815 |

This is a **real per-pixel gain map**, not a flat placeholder. It encodes up to
about **1.95 stops** of highlight gain and correlates with image luminance:
bright areas need more gain. It preserves highlight/dynamic-range information;
it does not add detail to the SDR primary. The SDR JPEG declares the gain map;
an HDR-aware viewer reconstructs the brighter rendition.

## Display-time HDR check

The display advertises HDR10/HLG/Dolby Vision types and a 2000-nit maximum, and
the current color mode is Display P3. With the same gain-map still open in the
stock gallery:

| Check | Result |
|---|---|
| `HdrBrightnessModifier.mMode` | `NO_HDR` |
| `mIsHdrLayerPresent` | `false` |
| `mHdrLayerSize` | `-1.0` |
| Visible layer metadata | `hdr metadata types=0` on the inspected layers |
| Gallery SurfaceView dataspace | `V0_SRGB` |

In this test the stock gallery did **not** present an HDR layer for the gain-map
still. The display is capable, but display-time HDR rendering was not observed.
That means the earlier "the photo improves" observation is not yet proven to be
HDR: it may be a full-resolution decode or an SDR render. A controlled
HDR-aware measurement (or a different gallery mode) is still required. The gain
map itself is real, as the reconstruction above shows.

## What this means

- The stock gallery may be showing the HDR gain map at display time without
  touching the file. This is consistent with the original observation, but needs
  an HDR-aware measurement (external camera or display metadata) to prove.
- The gain map is portable in principle: we can generate our own from RAW/HDR
  brackets or a 10-bit pipeline, with our own tone mapping, instead of adopting
  the vendor's watercolor/AI look.
- A good target is a natural SDR base plus a controlled gain map that recovers
  highlights without the stock neural detail treatment.

## Next

1. Repeat with a Live Photo and a 50 MP file; the gallery may treat them
   differently.
2. Use a controlled high-contrast scene; compare HDR off/auto and reconstruct
   the gain map to measure the actual dynamic-range gain.
3. Measure the display with an external camera or HDR-aware capture; SDR
   screenshots are insufficient.
4. Keep the gallery screenshot local; it contains personal thumbnails.
