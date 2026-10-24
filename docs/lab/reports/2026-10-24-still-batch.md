# Stock Still Batch - 2026-10-24

This batch covered high-resolution 50 MP, HDR off/auto, and mode navigation on
the bottle/napkin scene. All captures are local; no image is committed.

## Run

| Item | Value |
|---|---|
| Device | OnePlus 13 `PJZ110`, Android 16 |
| Stock camera | `com.oplus.camera` 6.020.171 |
| Orientation | landscape, rotation 3 |
| Scene | two bottles, napkins, blurred kitchen background |
| Safety | airplane mode + DND during each capture, restored afterwards |
| Runner | marker + `find -newer` + stable on-disk size before pull |

## High-resolution 50 MP

The More grid exposes `more_item` entries with content descriptions. The
`ВЫСОКОЕ РАЗРЕШЕНИЕ` item opened a dedicated mode:

- mode title: `ВЫСОКОЕ РАЗРЕШЕНИЕclick to close`;
- preview talkback: `ВЫСОКОЕ РАЗРЕШЕНИЕ Rear Camera preview`;
- hint: best for well-lit landscapes and stationary subjects;
- Live Photo / HDR controls are not shown in this mode.

| Property | Result |
|---|---|
| File | `IMG20261024020025.jpg` |
| Dimensions | **8192x6144** |
| Bytes | 14,346,849 |
| Orientation | 1 |
| XMP / gain map / MotionPhoto | absent |
| Shutter-to-file (runner poll) | ~4.4 s |
| Pulled bytes match device | yes |

This confirms that stock has a real 50 MP JPEG path. Our app still rejects
50 MP JPEG because its 8192x6144 reader returned 1920x1440; the stock path is a
vendor/offline mode, not the normal Camera2 JPEG path. Revisit our 50 MP
strategy with this evidence; do not assume the stock result is a simple reader
size change.

## HDR off/auto

The submenu's HDR tile cycles **Auto -> Off -> Auto**; no separate manual On was
observed.

**Correction (2026-10-24):** the first HDR rows below were pulled before the
stock app appended the gain map/motion. The 2.5 MB values are unfinished
JPEGs, not proof that HDR off lacks a gain map. Re-measured with the pause
protocol from
[2026-10-24-livephoto-finalization.md](2026-10-24-livephoto-finalization.md):

| Case | Final bytes | Primary | Gain map | GainMapMax | MotionPhoto |
|---|---|---|---|---|---|
| HDR off, 1x | 4,113,066 | 3,680,112 | 432,954 | 1.74475 | no |
| HDR auto, 1x | 5,192,175 | 4,771,123 | 421,052 | 1.74009 | no |

Both cases contain a real 2048x1536 gain map. The gain-map maximum is nearly
identical (~1.74 stops); HDR auto uses a larger primary JPEG and slightly
different tone/compression. The earlier Live-Photo-off still with
`GainMapMax=1.94892` confirms that the gain map is scene/decision dependent, not
a fixed HDR-off/on property. The gain map is a normal Ultra HDR still feature,
not a Live Photo feature.

A linear reconstruction (`HDR = SDR_linear * 2^(gain * GainMapMax)`) gives:

| Case | SDR clipped >=250 | HDR max | HDR >1 | HDR >2 |
|---|---|---|---|---|
| HDR off | 0.514% | 3.35x | 2.29% | 0.60% |
| HDR auto | 0.385% | 3.34x | 2.01% | 0.44% |

The gain map recovers roughly 1.74 stops in clipped highlights. HDR auto
slightly reduces clipping and the >2x area, but the gain-map range is almost
unchanged. This is evidence for building our own gain map from RAW/brackets,
not for copying stock's tone curve.

The older rows (2.5 MB, no gain map) are retained only as examples of the
premature-pull bug: file size was stable, `is_pending=0`, and mtime unchanged,
yet the gain map had not been appended. Do not use them for HDR conclusions.

## Zoom stops

Captured in the one-lamp scene with Live Photo left on. The 3x/6x/0.6x files
were finalized with the pause protocol; the 2x file is a plain 1.8 MB SDR still
with no motion/gain map.

| Stop | File bytes | Dimensions | EXIF focal | 35 mm eq | Aperture | Lens | Motion / gain |
|---|---|---|---|---|---|---|---|
| 2x | 1,811,332 | 4096x3072 | 6.06 mm | 48 mm | f/1.6 | main | no / no |
| 3x | 11,848,558 | 4096x3072 | 6.06 mm | 73 mm | f/1.6 | main | yes / yes |
| 6x | 11,219,337 | 4096x3072 | 6.06 mm | 146 mm | f/1.6 | main | yes / yes |
| 0.6x | 11,704,584 | 4096x3072 | 2.30 mm | 15 mm | f/2.0 | ultra-wide | yes / yes |

At this light level, 3x and 6x are **digital crops of the main sensor**
(6.06 mm, f/1.6, 23 mm lens model), not the 3x tele sensor. The 0.6x stop uses
the ultra-wide (2.30 mm, f/2.0, 15 mm eq). The stock app therefore prefers the
main sensor at 3x/6x in low light; a brighter scene is needed to confirm when it
switches to the tele. All stops still write 4096x3072.

## Master mode

`МАСТЕР` is reachable from Photo with a downward single-finger swipe on the mode
strip (Photo -> Master); an upward swipe from Photo goes to `ЕЩЕ` (More). The
right rail has `raw_control` (`Форматjpg` / `Форматraw`), global EV, Live Photo
and the submenu.

| Format | File | Bytes | Dimensions | Notes |
|---|---|---|---|---|
| RAW | `IMG20261024043149.dng` | 30,516,142 | 4096x3072 | 10-bit stored in 16-bit, white 1023, black 64, CFA RGGB, 23 mm f/1.6, 1/50 s, ISO 500, -0.33 EV |
| JPEG | `IMG20261024043221.jpg` | 3,077,529 | 4096x3072 | no gain map, no MotionPhoto |

Master RAW is **12 MP**, not 50 MP. It is a normal DNG with a 4096x3072 main
raw in a SubIFD, an 8-bit JPEG preview and full DNG color metadata. The JPEG is
a plain 3.08 MB render. This is the natural/neutral stock path to compare with
our RAW 12 rendering.

## Mode navigation

The landscape mode strip is a vertical carousel. A single-finger swipe on the
`headline_view` area moves between Photo, Portrait, Video and More; tapping the
small labels alone did not switch. The More grid is accessible through
`more_item` content descriptions:

| Item | Content description | Bounds |
|---|---|---|
| Night | `НОЧЬ` | `[1800,81][2058,339]` |
| High resolution | `ВЫСОКОЕ РАЗРЕШЕНИЕ` | `[1800,411][2058,669]` |
| Panorama | `ПАНОРАМНАЯ` | `[1800,741][2058,999]` |
| Movies | `ФИЛЬМЫ` | `[1512,81][1770,339]` |
| Slow motion | `ЗАМЕДЛЕННАЯ СЪЁМКА` | `[1512,411][1770,669]` |
| Time-lapse | `УСКОРЕННАЯ` | `[1512,741][1770,999]` |
| Long exposure | `ДЛИННАЯ ВЫДЕРЖКА` | `[1224,81][1482,339]` |
| Dual-view video | `ДВУХОКОННЫЙ РЕЖИМ ВИДЕО` | `[1224,411][1482,669]` |
| Sticker | `СТИКЕР` | `[1224,741][1482,999]` |
| AI ID photo | `ФОТО ДЛЯ УДОСТОВЕРЕНИЯ С ИИ` | `[936,81][1194,339]` |
| Text scan | `СКАНИРОВАНИЕ ТЕКСТА` | `[936,411][1194,669]` |
| Breeno scan | `СКАНИРОВАНИЕ BREENO` | `[936,741][1194,999]` |
| Tilt-shift | `СДВИГ/УКЛОН` | `[648,81][906,339]` |
| XPAN | `XPAN` | `[648,411][906,669]` |

No Pro entry was visible in this grid. Pro mode discovery is still open; it may
need the mode-edit list or a different top-level carousel position.

## Automation notes

- High-resolution mode is exited by tapping its mode title (`click to close`);
  this returns safely to Photo.
- The HDR tile is inside the submenu; close the submenu with the UI close
  button, not system Back.
- The 50 MP capture is slower and larger; wait for size stability, not just
  MediaStore `is_pending`.
- All captured files were left in the user's gallery; no user media was deleted.
- Airplane mode/DND were restored after each capture.

## Next

1. ~~Resolve Live Photo muxing reproducibility~~ **Resolved:** deferred append;
   pause the preview before pulling.
2. Use the corrected HDR off/auto data; reconstruct the gain map and compare
   highlight recovery on a controlled high-contrast scene.
3. Measure 50 MP detail/noise against our RAW 50 and stock 12 MP in the same
   scene (current files are from different sessions).
4. Confirm the tele switch at 3x/6x in brighter light; front camera needs a
   neutral target or user consent.
5. Gallery display test for the gain map.
