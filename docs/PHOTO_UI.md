# Photo-First UI

The photo screen is the product. Video is a separate mode; it must never share the
photo format chips. This document is the design contract for the Compose rewrite.

## Information architecture

| Mode | Status | Contents |
|---|---|---|
| **Photo** | default, shipping | 12 MP, RAW 12, RAW 50, HDR 3×; zoom 0.6–6×; front/rear; gallery |
| **Video** | separate, existing | 4K 4:3/16:9; record; zoom; audio/HDR later |
| **Portrait** | hidden until real | bokeh, portrait zoom, flash, lens pack |
| **Pro** | inside Photo menu | manual ISO/shutter/WB/focus/EV |
| **Night** | inside Photo menu | multi-frame night |

Pro and Night are not top-level tabs until their engine paths exist. Do not render
dead icons or dead tabs.

## Photo screen

```text
┌──────────────────────────────────────┐
│ 9:41                         ▂▄▆    │
│                                      │
│   ⚡    EV    ◐    ◎    ✿      ⋯    │  top bar, capability-gated
│   12 MP                              │  format badge → FormatSheet
│                                      │
│              PREVIEW                 │  fitted to 4:3/16:9, never stretched
│                                      │
│         0.6   1   2   3   6          │  zoom rail
│                                      │
│        Photo    Video   Portrait*    │  mode switch
│                                      │
│   [thumb]         ( ◯ )         [⇄] │  capture deck
└──────────────────────────────────────┘
```

- **Shutter** is the primary control, horizontally centered, 76 dp.
- **Zoom rail**: pinch + pills; 2×/6× are digital ratios, not optical lenses.
- **Mode switch**: Photo / Video; Portrait only when implemented.
- **Gallery thumb** and front/rear flank the shutter.
- **Format badge** opens a sheet; do not put 12/RAW/R50/HDR in the bottom row.
- **Top icons** appear only when the engine API exists. Until then, no fake flash,
  EV, filter, motion or macro icons.
- Use safe-drawing insets, 48 dp targets and gradient scrims. The shutter is white
  in Photo and red in Video; orange is selection accent only.

## Stock camera layout reference (2026-10-26)

Eight stock screenshots were captured on the device: photo, video, Portrait and
Master, each in portrait and landscape. The decisive finding: **stock does not
reflow between portrait and landscape**. It keeps one layout in phone
coordinates and rotates the whole canvas; the apparent movement is the phone
turning, not the controls changing sides.

Proof: rotate the landscape photo screenshot 90° clockwise and every control maps
onto the portrait screenshot - toolbar, preview, mode strip, shutter row, zoom
row and quick toggles.

### Rotation and glyph rules

- Design the screen in portrait coordinates (`1080x2376`) and rotate the whole
  canvas by the display rotation. Positions stay fixed relative to the phone.
- Counter-rotate only glyphs that must stay readable: settings icons, zoom
  numbers, shutter/flip/gallery, quick toggles.
- Do **not** counter-rotate `PHOTO/VIDEO/PORTRAIT/MASTER`; the labels run along
  the rail and are vertical in landscape by design.
- Fit the preview to the sensor aspect. A stretched full-screen preview is a
  bug, not a style.

### Preview geometry (stock, device pixels)

| Mode | Portrait | Landscape |
|---|---|---|
| Photo / Portrait / Master | `1080x1440` (4:3), `y=299..1739` | `1440x1080` (4:3), `x=300..1739` |
| Video | `1080x1920` (16:9), `y=299..2219` | `1920x1080` (16:9), `x=300..2219` |

### Portrait-coordinate anchors (stock)

| Element | Position |
|---|---|
| Top toolbar | `y≈60..180`, horizontal |
| Preview | `x=0..1080`, `y=299..1739` (4:3) or `y=299..2219` (16:9 video) |
| Quick toggles | right edge inside the preview, vertical |
| Zoom row | inside the preview bottom, `y≈1515`, horizontal `0.6/1/2/3/6` |
| Mode strip | `y≈1810`, horizontal |
| Shutter row | `y≈2050`: gallery thumb left, shutter centre, flip right |

Landscape mapping (clockwise): `(x', y') = (y, 1080 - x)`. The top toolbar becomes
the left rail, the mode strip becomes the right vertical rail, the zoom row
becomes the inner-right vertical rail, and the shutter row becomes the right edge
(thumb bottom, shutter centre, flip top).

### Mode-specific notes

- **Photo (01/02):** flash, EV, macro, motion, filter, menu; `0.6-6x`; orange
  shutter.
- **Video (03/04):** 16:9 preview; HDR/4K60; red shutter; zoom, mode strip and
  shutter overlay the lower/right preview.
- **Portrait (05/06):** `Готово`; `f/7.1` + `1/2/3x`; same skeleton.
- **Master (07/08):** JPG, EV, Auto/Pro, layers; `0.6-6x`.

### UI-302 acceptance

- One rotated canvas, no per-orientation reflow.
- Stock positions above; no arbitrary migration to other edges.
- Only the listed glyphs counter-rotate; mode labels stay along the rail.
- Preview letterboxed to 4:3/16:9; no full-screen stretch.
- Validate 0/90/180/270 against the stock reference on the device.

The reference screenshots stay in `C:\Temp\opencode\stock-ref` and are **not**
committed because they contain personal content.

## Format sheet

| Row | Behavior |
|---|---|
| 12 MP JPEG | default |
| RAW 12 | DNG ~25 MB |
| RAW 50 | DNG ~96 MB; unavailable on front |
| HDR 3× | 3-frame merge; hold still |
| 50 MP JPEG | not selectable; footnote |
| 25 MP | not selectable; footnote (RAW25 probe pending) |
| 5K/6K/8K | not shown in Photo; Video roadmap rows only when real |

The main screen advertises only working formats. Unavailable formats live in the
sheet footnote with a reason.

## States

| State | Behavior |
|---|---|
| Permission | dedicated screen; Allow / Open settings |
| Starting | black preview, shutter disabled, "Starting camera…" |
| Switching | keep last preview; small spinner; disable shutter/mode switch |
| Capturing | shutter returns immediately; taps queue in the background worker; show the pending count |
| Saving RAW/HDR | progress + size; keep screen awake |
| Saved | thumbnail updates; snackbar "Saved · RAW 50" |
| Error | snackbar, delete partial, no fake gallery entry |
| HAL fallback | amber banner; format badge resets to actual mode |
| Low storage | preflight RAW 50/HDR; explain before disabling shutter |
| Microphone | requested only in Video; never silent recording |

## Video screen

```text
┌──────────────────────────────────────┐
│   4K · 30 ▾       MIC ●      SDR     │
│              PREVIEW                 │
│          ● REC   00:07               │
│         0.6   1   2   3   6          │
│        Photo    Video   Portrait*    │
│   [thumb]        ( ● )          [⇄]  │
└──────────────────────────────────────┘
```

- Resolution/fps sheet; only existing configs selectable.
- 1080p120, 720p60, 5K/6K/8K, HDR/Dolby are roadmap rows until verified.
- Lock mode/resolution/lens while recording; stop is a square.
- Show `SDR` honestly until HDR metadata is real.

## Implementation phases

1. **Session safety:** extract SurfaceView host/session controller; no main-thread
   JNI; generation guard.
2. **Photo shell:** Photo default; mode switch; format sheet; zoom rail; gallery
   thumb; front/rear; remove 50 MP chip.
3. **Truth layer:** capability matrix, saving/error/fallback/low-storage states,
   MediaStore/thumbnail.
4. **Zoom/lens:** 0.6/1/2/3/6, pinch, hand-off, per-lens tests.
5. **Focus/EV/flash:** only after native APIs.
6. **Video UI:** separate chrome; preserve 4K path.
7. **Later modes:** Portrait, Pro, Night, filters, Live Photo, macro.

## Acceptance

- Cold launch is Photo; no video controls.
- One SurfaceView; mode/format/lens changes do not recreate it.
- 12 MP/RAW 12/RAW 50/HDR work; front 8 MP works; gallery thumb updates.
- No 50 MP/25 MP/4K120/front-4K/HDR claim without a real file.
- Photo/Video mode switch is blocked while capturing/recording.
- Test every format at 0.6/1/2/3/6 and on each available lens.

## Standby / idle screen (implemented)

After 30 s without interaction, the camera hides its controls behind a black
"Tap for preview" overlay. The preview session stays alive; tapping wakes the
UI and resets the timer. The overlay is suppressed while capturing, recording,
switching or when the gallery is open. This is UI-only for now; stopping the
preview stream later can save additional battery/heat.

## Photo UI v2 (build-verified, device validation pending)

The first visual pass is implemented in `CameraScreen.kt`:

- PHOTO/VIDEO is now a stock-like segmented switch; the active segment uses the
  orange selection accent.
- The top bar shows the brand, the current format/lens/zoom summary and, only
  while busy or on error, a status message. Normal operation does not show debug
  text; the redundant photo status line was removed.
- A subtle rule-of-thirds grid sits over the preview.
- The bottom capture row is [gallery thumb] [86 dp shutter] [flip camera].
  The front/rear chips were removed from the zoom row.
- All controls use safe-area insets; the capture deck respects the gesture bar.
- The top-bar format badge opens a bottom sheet with the working formats only
  (12 MP JPEG, RAW 12, RAW 50, HDR 3x). 50 MP JPEG is explained as unavailable;
  there is no dead selectable row. Selecting a format keeps the current
  lens/zoom when that lens supports it, otherwise the app falls back to auto.
- Zoom pills use the orange selection accent. Pinch-to-zoom stays within the
  active lens (0.6-1x on the ultra-wide, 3-6x on the telephoto); use the pills
  to cross a lens boundary. The slider provides continuous logical zoom;
  releasing it after a physical-lens preset returns to logical auto zoom at the
  requested stop.
- Video mode shows a live REC timer.
- A centered progress card with the current state is shown while a still is
  being captured/saved; RAW 50 and HDR no longer look frozen.
- Lens presets are honest: 0.6x selects the physical ultra-wide (camera 3), 3x
  selects the physical telephoto (camera 4), and 6x is a 2x crop on that
  telephoto. The app does **not** copy the stock low-light trick of keeping
  3x/6x on the main sensor. If a physical lens cannot start the selected mode,
  the app falls back to the logical camera and says so.

Still to do in the next visual pass:

- add a mic state to the Video chrome;
- validate portrait/landscape layout, touch targets and haptics on the device;
- device-test the physical-lens presets and the fallback path.

### Landscape-first device pass (2026-10-24, superseded)

> **Superseded 2026-10-26.** The stock camera supports both orientations and uses
> one portrait-coordinate canvas rotated with the device. The `sensorLandscape`
> layout below was an intermediate pass; do not extend it. UI-302 replaces it
> with the single-canvas model above.

The first device pass assumed the stock camera was landscape-only and that a
portrait-locked app produced sideways previews and stills. The app used
`sensorLandscape` and a `TextureView` preview:

- preview rotation follows the display; the landscape preview was max-scaled to
  fill (later found to be a crop, not the stock letterbox);
- JPEG/DNG orientation is computed from the sensor and display rotation and
  applied through a new JNI setter;
- landscape controls: gallery/shutter/flip on the left, zoom stops on the
  right, PHOTO/VIDEO at the bottom, format badge at the top right.

Device-validated:

- 3x opens the physical telephoto (camera 4), 6x applies a 2x crop on it;
- 0.6x opens the physical ultra-wide (camera 3), 1x returns to logical 0;
- 3x, 6x and 0.6x JPEGs are upright (orientation 3 in landscape at rotation 270);
- RAW 50 DNG is 8192x6144, orientation 3, and renders upright;
- HDR 3x output is 4096x3072 and upright after orientation-aware pixel rotation;
- the format sheet, saving progress card and gallery thumbnail work.

Pending: front camera, video orientation, pinch on a physical two-finger
gesture, portrait crop, and a final visual pass on the thumbnail/preview
letterboxing.

## Orientation detection - step 1 (2026-10-24)

`OrientationEventListener` reports the physical device rotation separately from
the locked landscape activity. The standby overlay uses that rotation.

The preview uses the display rotation, not the raw sensor angle: the preview
stream arrives 90° rotated relative to the JPEG stream, and using the physical
angle made a phone lying flat report portrait and flip the preview. The preview
counter-rotation is `(360 - displayRotation * 90) % 360`; the JPEG orientation
remains `(sensorOrientation - displayRotation * 90) % 360`.

This step did not implement the stock layout. UI-302 replaces the fixed
`sensorLandscape` layout with one portrait-coordinate canvas rotated by the
display rotation; only readable glyphs are counter-rotated, and the mode labels
stay along the rail. Device validation requires turning the phone through
0/90/180/270 while the app is open.
