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

| Case | File bytes | Dimensions | Gain map | MotionPhoto |
|---|---|---|---|---|
| HDR off, first capture | 2,504,234 | 4096x3072 | no | no |
| HDR off, after 5 s wait | 2,528,323 | 4096x3072 | no | no |
| HDR auto | 2,474,285 | 4096x3072 | no | no |

All three files are plain SDR JPEGs. The earlier 5.5 MB still (Live Photo off,
HDR auto) did contain a gain map, so gain-map presence is **scene/decision
dependent**, not a fixed property of HDR auto. A controlled high-contrast scene
is required before drawing conclusions about HDR.

The UI showed `Живое фотоon` during these captures, but they contain no motion
clip. This is the Live Photo reproducibility issue documented in
[2026-10-24-livephoto-smoke.md](2026-10-24-livephoto-smoke.md); do not treat
these files as Live Photo samples.

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

1. Resolve Live Photo muxing reproducibility.
2. Capture a controlled high-contrast HDR scene and compare HDR off/auto with
   the gain map.
3. Measure 50 MP detail/noise against our RAW 50 and stock 12 MP.
4. Zoom stops and front camera (front needs a neutral target or user consent).
5. Gallery display test for the gain map.
