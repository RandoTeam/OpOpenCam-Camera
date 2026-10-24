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
│              PREVIEW                 │  one SurfaceView, full-bleed
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
| Capturing | shutter shrinks, haptic, duplicate taps ignored |
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
- The top bar shows the brand, the current format/lens summary and, only while
  busy or on error, a status message. Normal operation does not show debug text.
- A subtle rule-of-thirds grid sits over the preview.
- The bottom capture row is [gallery thumb] [86 dp shutter] [flip camera].
  The front/rear chips were removed from the zoom row.
- All controls use safe-area insets; the capture deck respects the gesture bar.
- The top-bar format badge opens a bottom sheet with the working formats only
  (12 MP JPEG, RAW 12, RAW 50, HDR 3x). 50 MP JPEG is explained as unavailable;
  there is no dead selectable row.
- Zoom pills use the orange selection accent. Pinch-to-zoom and the slider
  provide continuous logical zoom; releasing the slider after a physical-lens
  preset returns to logical auto zoom at the requested stop.
- Video mode shows a live REC timer.
- Lens presets are honest: 0.6x selects the physical ultra-wide (camera 3), 3x
  selects the physical telephoto (camera 4), and 6x is a 2x crop on that
  telephoto. The app does **not** copy the stock low-light trick of keeping
  3x/6x on the main sensor. If a physical lens cannot start the selected mode,
  the app falls back to the logical camera and says so.

Still to do in the next visual pass:

- add a mic state to the Video chrome;
- show an explicit saving/progress state for RAW 50 and HDR;
- validate portrait/landscape layout, touch targets and haptics on the device.
