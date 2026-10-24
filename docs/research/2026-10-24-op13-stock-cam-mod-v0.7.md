# OP13 Stock Cam Mod (GCam Killer) v0.7 - local analysis

## Provenance and handling

| Item | Value |
|---|---|
| Local archive | `OP13-Stock-Cam-Mod-GCam-Killer-v0.7.zip` |
| Size | 103,258,957 bytes |
| SHA-256 | `A4F9F353B9DB7346E755931A52C62BD87F013CCD3BEAA46DBA14FC21CA38BA4D` |
| Public repo | **not committed**; the archive and extracted files are under `.local-research/`, which is ignored |
| Device | not installed; no root/Magisk changes were made |

The archive is a Magisk/KernelSU module. Its `module.prop` calls it "OnePlus 12
StockCam Mod v0.7 (GCam Killer) for Android 16" and says it disables aggressive
sharpening and noise reduction in Photo, Portrait and Video. The module id is
`stock_cam_mod_gcam_killer_op13`.

## Contents

| Path | Bytes | What it is |
|---|---|---|
| `service.sh` | 2,370 | boot-time bind-mount and camera-service restart |
| `action.sh` | 1,906 | volume-key menu to enable/disable the mod |
| `module.prop` | 304 | Magisk metadata |
| `META-INF/.../update-binary` | 612 | standard Magisk installer stub |
| `files/odm/lib64/libarcsoft_turbo_hdr_raw.so` | 136,876,696 | replacement ArcSoft Turbo HDR RAW library |
| `files/odm/lib64/camera/com.qti.tuned.dodgemain.bin` | 32,756,615 | main-sensor Qualcomm Chromatix tuning |
| `...dodgetele.bin` | 8,019,504 | 3x tele tuning |
| `...dodgetele2.bin` | 23,819,206 | tele2 tuning |
| `...dodgeultrawide.bin` | 10,781,665 | ultra-wide tuning |
| `...dodgefront.bin` | 6,361,651 | front tuning |

## What the installer actually does

`service.sh`:

1. increments `bootloop_counter` and disables the module after three failed
   boots;
2. kills `com.oplus.camera`, then bind-mounts the replacement library over
   `/odm/lib64/libarcsoft_turbo_hdr_raw.so`;
3. bind-mounts four camera tuning files;
4. runs `restorecon` on the mounted paths;
5. restarts `vendor.oplus.hardware.cammidasservice-V1-service`,
   `vendor.qti.camera.provider-service_64` and `cameraserver`;
6. `action.sh` toggles `mod_disabled` and re-runs `service.sh`.

### Critical packaging mismatch

The module contains **OP13** tuning files named:

```text
com.qti.tuned.dodgemain.bin
com.qti.tuned.dodgetele.bin
com.qti.tuned.dodgetele2.bin
com.qti.tuned.dodgeultrawide.bin
com.qti.tuned.dodgefront.bin
```

but `service.sh` mounts **OnePlus 12** names:

```text
com.qti.tuned.qtech_lyt808_main.bin
com.qti.tuned.sunny_ov64b40_tele.bin
com.qti.tuned.qtech_imx581_wide.bin
com.qti.tuned.qtech_imx615_front.bin
```

On an OP13 those four target paths are not the files shipped in the module.
The library mount can still apply; the tuning-bin mounts are likely to fail.
The script also does not mount `dodgetele2` or `dodgeultrawide` at all.
`module.prop` saying "OnePlus 12" is consistent with a repackaged OP12 script
with OP13 payloads, or a simple naming mistake.

**Conclusion:** do not assume the published v0.7 produces the advertised
tuning change on an OP13 until the mount paths are corrected and verified.
A corrected local script would need the five `dodge*` names and a check that
each target exists before `mount --bind`.

## Library analysis

`libarcsoft_turbo_hdr_raw.so` is an AArch64 ELF shared object:

- 136.9 MB; `.rodata` alone is about 108 MB; `.text` about 6.6 MB;
- SONAME `libarcsoft_turbo_hdr_raw.so`;
- depends on `libarc.ion.so`, `libarcsoft_qnnhtp.so`, `libcdsprpc.so`,
  `libtrace.so`, `libmpbase.so`, `liblog.so`, `libc.so`, `libm.so`, `libdl.so`;
- exports the ArcSoft Turbo HDR API (`ARC_Turbo_HDR_Init`, `Process`,
  `PreProcess`, `PostProcess`, `SetParam`, bokeh/DCIR entry points);
- build strings identify ArcSoft TurboHDR 4.0 and 2025 components, including
  `AIDenoise`, `RAWAISR`, `RAW_HDR`, `MFSR`, `SegModel`, `Hand_Detection`,
  `SingleCameraFusion`, and QNN/HTP/DSP paths.

This is **not** a "neural processing removed" library. It still contains AI
denoise, segmentation, hand detection and super-resolution components. The mod
appears to reduce or bypass some processing through tuning and runtime
controls, not to strip the neural stack.

Observed runtime property/config names include:

```text
vendor.arcsoft.turbo_hdr.bypass
vendor.arcsoft.turbo_hdr_force_id
vendor.arcsoft.turbo_hdr.input_mode
vendor.arcsoft.turbo_hdr.algmodel
vendor.arcsoft.turbo_hdr.use_outer_trigger
vendor.arcsoft.turbo_hdr_deblur_type
vendor.arcsoft.turbo_hdr_low_deblur_level
vendor.arcsoft.turbo_hdr_tmc_sharpen_out
vendor.arcsoft.turbo_hdr_tmc_clarity_out
vendor.arcsoft.turbo_hdr_sharpen_evlowlow
vendor.arcsoft.turbo_hdr_gpu_hint
vendor.arcsoft.turbo_hdr_bokeh_loglevel
vendor.camera.arcsoft.turbo_hdr.bypass
```

The binary also contains explicit denoise/sharpen paths:

- `AIDenoiseOCL`, `Arc_RGBDenoise_L2`;
- `ArcSoft_DenoiseAI_1.5` and `2.1`;
- `usm_sharpen_with_mask`, `tmc_sharpen_type/in/out`;
- `denoise_intensity >= 0 && denoise_intensity <= 100`;
- `AIDenoise` models loaded through QNN/HTP and CDSP.

So the stock look is produced by a combination of:

1. Qualcomm Chromatix tuning (ANR/ASF/gamma/LTM/HDR/SAT/video);
2. ArcSoft Turbo HDR RAW fusion and AI denoise/sharpen;
3. vendor camera policy/props.

The module changes capture-side libraries/tuning. It does **not** touch the
gallery.

## Tuning bins

The `.bin` files are Qualcomm Chromatix blobs, header
`QTI Chromatix Header`, parser `V5.5.0 (2411011106)`. They contain modules such
as:

```text
anr142_ipe_v2     noise reduction
asf352_ipe_v2     adaptive spatial filter / sharpening
gamma152_ipe_v2
ltm203_ipe_v2
hdr10p10_ipe_v2
rgbhdr512_ipe_v2
sat10_sw
video13_sw_v2
```

The actionable insight is that the "aggressive" stock look is largely in
**ANR** (noise reduction) and **ASF** (sharpening) tuning, plus ArcSoft
post-processing. Replacing these bins can reduce smoothing/edge halos. But
without the stock bins from the exact firmware, we cannot quantify how much the
module changes; the payload may be from a different build.

## Gallery vs capture pipeline

The module is additional evidence that the stock camera's watercolor/neural
look is **capture-side**, not added later by the gallery:

- the module modifies camera libraries and Chromatix tuning only;
- it contains no gallery code, LUT, or display shader;
- the gallery test found no file rewrite and, in the inspected file, no HDR
  layer;
- the black-and-white gain map is a per-pixel luminance gain map, not a detail
  or monochrome overlay, and it is present in the file before the gallery opens
  it.

The gallery may still do a full-resolution decode, SDR tone mapping, or a
standard Ultra HDR/gain-map render at display time. That is separate from the
capture-side neural processing. Binding the stock gallery to our camera would
give us a viewer, not a way to remove the stock capture look. If the gallery has
an optional AI "remaster/enhance" action, that would add processing, not remove
it.

## What is useful for OpOpenCam

1. **Stay rootless and vendor-free.** We cannot use these bind-mounted vendor
   libraries in the public app. They are proprietary and tied to the OP13
   firmware.
2. **The RAW path is the correct escape hatch.** Our RAW 12/50 captures bypass
   most of the ANR/ASF/TurboHDR chain. The neutral DNG renderer should remain
   the quality baseline.
3. **Tune our own NR/sharpening explicitly.** The mod confirms the important
   knobs: noise reduction strength, spatial sharpening, clarity, deblur, and
   local tone mapping. Our renderer should expose a neutral default and a mild
   "natural detail" option, with no AI/watercolor.
4. **Do not chase the stock gallery.** Write standard JPEG/Ultra HDR files and
   let any viewer open them. Our own gallery should own the SDR/HDR rendering.
5. **Measure before/after, not claims.** A rootless comparison of stock JPEG,
   stock RAW, and our neutral render is the useful experiment; installing a
   vendor mod on the daily phone is not required.

## Risks

- Magisk/KernelSU module modifies `/odm` via bind mounts and kills camera
  services; a bad mount can bootloop or break the camera.
- The script's OP12 filenames do not match the OP13 payload names.
- The library is a 2025 vendor build with AI components; it is not a clean
  "neural off" switch.
- Vendor binaries/licensing: keep the archive local; do not add it to the
  public repository.

## Next when the device is available

1. Pull the stock `/odm/lib64/camera/com.qti.tuned.dodge*.bin` hashes/sizes and
   the stock `libarcsoft_turbo_hdr_raw.so` hash (read-only).
2. Compare the module's payload with the stock files; identify whether it is a
   different firmware build or a patched build.
3. If the user explicitly wants to test the module, correct the mount paths
   first and keep the bootloop counter; do not test on a phone with unsaved
   data.
4. Continue the rootless path: same-scene stock JPEG vs our RAW render, with
   controlled NR/sharpening settings.
