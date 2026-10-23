# Photo UI Inventory

The product UI must be a deliberate, professional camera UI, not a pile of
chips. Before redesigning, we need a complete map of the stock photo surface:
every control, state, gesture, animation and haptic.

This is a behavioural inventory. We record layout, labels and timing; we do not
copy branded artwork, LUTs or assets into the repository.

## 1. Method

1. Launch the stock camera with a controlled scene.
2. Dump the view tree: `adb shell uiautomator dump /sdcard/ui.xml`.
3. Pull the XML and parse it as UTF-8. The shell console may garble Cyrillic;
   the file itself is valid.
4. Take a screenshot for each stable state.
5. Record a 60 fps screen capture for transitions.
6. Use `dumpsys gfxinfo com.oplus.camera framestats` and
   `dumpsys SurfaceFlinger --latency` for frame timing.
7. Use `getevent -lt` to timestamp the touch and correlate it with the frame
   where the UI reacts.
8. For each control, record: resource ID, class, label, bounds, states, action,
   animation, haptic, persistence.
9. Tap by parsed bounds, never by a remembered coordinate. Coordinates are
   allowed only for gestures on the preview (pinch, drag).

No personal content in screenshots. Use the lab chart or a neutral surface.

## 2. Screen map

```text
Stock camera
├── Photo (default)
│   ├── top quick bar
│   ├── preview / focus / exposure
│   ├── right-side tools
│   ├── zoom bar and pills
│   ├── transient mode chips
│   ├── bottom deck: thumbnail, shutter, front/rear
│   └── mode tabs: Photo, Portrait, More
├── More grid
│   ├── Night
│   ├── Panorama
│   ├── Movies
│   ├── Slow motion
│   ├── Time-lapse
│   ├── Long exposure
│   ├── Dual-view video
│   ├── Sticker
│   ├── XPAN
│   ├── Tilt-shift
│   └── scan/AI entries
├── Portrait
├── Pro
├── Settings / submenus
├── Gallery (stock) / editor
└── Permission / onboarding / error states
```

The exact list is confirmed by the device, not by this diagram. Labels that are
clipped in a capture are marked "unconfirmed".

## 3. Initial inventory (2026-10-23)

### Photo top bar

| Resource ID | Type | Label (ru) | States / action | Notes |
|---|---|---|---|---|
| `flash_mode` | ImageView | `ВспышкаВыкл.` | off / auto / on | tap cycles; icon changes |
| `global_ev` | ImageView | `Global EVВыкл` | off / slider | global exposure compensation |
| `live_photo` | ImageView | `Живое фотоoff` | off / on | see [LIVE_PHOTO.md](LIVE_PHOTO.md) |
| `motion_capture` | ImageView | `Спортивный режимoff` | off / on | action/sports mode, not Live Photo |
| `macro_close_up` | ImageView | `Макросъемкаoff` | off / on | macro toggle |
| `submenu` | ImageView | `SubSet` | opens submenu | contents to inventory |

### Right-side tools

| Resource ID | Label | States / action | Notes |
|---|---|---|---|
| `camera_menu_right_enter_button_soft_light` | `soft lightoff` | off / on | soft light / fill light |
| `camera_menu_right_enter_button_filter` | `filteroff` | off / picker | filters likely LUT-based |
| `camera_menu_right_enter_button` | `face beautyoff` | off / picker | beauty/portrait controls |

### Preview and zoom

| Resource ID | Type | Notes |
|---|---|---|
| `zoom_seek_bar` | View | continuous pinch/drag zoom |
| `zoom_seek_bar_container` | RelativeLayout | bottom overlay container |
| `preview_talkback` | FrameLayout | content description includes current mode and `Rear Camera preview` |
| `subject_view` | View | focus/subject overlay |

Zoom stops observed in the product context: 0.6, 1, 2, 3, 6. Higher stops
(10/20/30/60/120) need confirmation from the live UI.

### Transient chips

| Resource ID | Label | Notes |
|---|---|---|
| `macro_switch_button` | `Макросъемкаon` | transient macro suggestion |
| `se_night_switch_button` | `Ночьon` + `3s` | night suggestion/timer; confirm semantics |
| `headline_view` | `ФОТОmode, double finger slide to change mode` | mode label; double-finger slide gesture |

### Bottom deck

| Resource ID | Label | Action | Notes |
|---|---|---|---|
| `thumbnail` | `Latest Photos` | opens stock gallery | latest capture |
| `shutter_button` | `"Shutter" button` | tap = photo; long-press = burst | central control |
| `switch_camera_button` | `Front Camera` | front/rear | state depends on lens |

### Mode tabs and More grid

The bottom tabs observed: `ФОТО`, `ПОРТРЕТ`, `ЕЩЕ` (Photo, Portrait, More).
The More grid observed (some labels clipped by the first capture):

| Row | Labels |
|---|---|
| 1 | `НОЧЬ` (Night), motion/action (unconfirmed), high resolution (unconfirmed), `ПАНОРАМНАЯ` (Panorama) |
| 2 | `ФИЛЬМЫ` (Movies), slow motion (unconfirmed), `УСКОРЕННАЯ` (Time-lapse) |
| 3 | macro/long exposure (clipped), `ДЛИНН` (Long exposure, unconfirmed), dual-view video (unconfirmed), `СТИКЕР` (Sticker) |
| 4 | AI resolution (unconfirmed), text scan (unconfirmed), GreenO scan (unconfirmed) |
| 5 | `СДВИГ/УКЛОН` (Tilt-shift), `XPAN` |

A clean capture with the grid fully settled and the full labels visible is a
required follow-up. Do not finalise the feature list from clipped labels.

## 4. Per-control record

Use this schema in `ui_inventory.json`:

```json
{
  "screen": "photo",
  "id": "com.oplus.camera:id/live_photo",
  "class": "android.widget.ImageView",
  "label": "Живое фото",
  "bounds": [375, 0, 531, 240],
  "states": ["off", "on"],
  "action": "tap toggles",
  "animation": "icon cross-fade + scale",
  "haptic": "none observed",
  "persistence": "survives app restart",
  "evidence": ["ui/photo_01.xml", "ui/photo_01.png"],
  "notes": ""
}
```

## 5. Animations and haptics

For each transition:

- record at 60 fps with `screenrecord --bugreport`;
- extract frames with ffmpeg;
- measure first reaction frame, duration, easing and end state;
- record `dumpsys gfxinfo ... framestats` for jank;
- record `getevent -lt` for the input timestamp;
- listen/observe haptics; `dumpsys vibrator_manager` may show recent effects;
- note whether the preview stays live during the animation.

Priority transitions:

1. app cold start -> live preview;
2. shutter press -> shutter animation -> thumbnail update;
3. thumbnail -> gallery open -> gallery close;
4. zoom pill/slider -> lens hand-off -> preview settle;
5. mode tab switch (Photo/Portrait/More);
6. More grid open/close;
7. filter picker live preview;
8. HDR/AI processing indicator;
9. front/rear switch;
10. permission/error dialogs.

## 6. Design deliverables

1. `ui_inventory.json` - complete machine-readable map.
2. `UI_INVENTORY.md` - human-readable tables and open questions.
3. Local screenshot set for every state; not committed.
4. Animation reference clips; not committed.
5. A clean original wireframe/spec for our app, with our own visual language.
6. A parity checklist: control, behaviour, animation, state, priority.

## 7. Acceptance

- Every photo control visible on the stock app is mapped.
- Every state of each control is known (off/on/auto/disabled).
- Every transition has a measured duration or is marked "not measured".
- The More grid labels are confirmed with a clean capture.
- No stock artwork, LUT or branded asset is committed.
- The product UI spec can be implemented without reopening the stock app for
  basic layout questions.
