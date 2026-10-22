# Glossary

| Term | Meaning |
|---|---|
| **Camera2** | Android camera API (`android.hardware.camera2` / NDK `camera2ndk`) |
| **HAL** | Vendor camera hardware abstraction layer (Qualcomm CamX/Chi on this device) |
| **Logical camera** | Camera ID 0; routes to physical cameras (3/2/4) |
| **Physical camera** | Direct sensor ID: 2 main, 3 ultra-wide, 4 tele |
| **ZSL** | Zero-shutter-lag: frames buffered before the shutter |
| **HDR** | High dynamic range; here: bracketed capture + fusion |
| **Bracketing** | Multiple frames at different exposures (−2/0/+2 EV) |
| **Mertens fusion** | Exposure fusion using contrast/saturation/well-exposedness weights |
| **NCC** | Normalised cross-correlation; exposure-invariant alignment metric |
| **DNG** | Adobe Digital Negative; TIFF-based RAW container |
| **CFA** | Color filter array (RGGB on this sensor) |
| **P010** | 10-bit YUV 4:2:0 format |
| **Motion Photo** | JPEG + appended MP4 + XMP metadata in one file |
| **Live Photo** | User-facing name for Motion Photo |
| **HEVC** | H.265 video codec |
| **AAC** | Audio codec used for MP4 audio |
| **EIS/OIS** | Electronic/optical image stabilization |
| **APS** | OPlus image-processing daemon (not used by the public build) |
| **GCam/LMC** | Google Camera port; used only as a comparison reference |
| **JNI** | Java Native Interface between Compose and Rust |
| **ADR** | Architecture Decision Record in `docs/DECISIONS/` |
| **HAL fallback** | Retrying 12 MP JPEG when a requested mode is rejected |
| **SOF** | JPEG Start Of Frame marker; contains real image dimensions |
| **Package identity** | Installed applicationId; `com.oneplus.camera` unlocks Chi HAL features |
| **25 MP Turbo RAW** | Vendor mode advertised by the module; 5888×4416; not in Camera2 stream configs |
| **ProXDR** | OPlus HDR display/photo pipeline; vendor-specific |
| **Ultra HDR** | Android JPEG + gainmap HDR format; cross-device |
| **Gainmap** | Auxiliary HDR luminance map appended to an SDR image |
| **HEIF/HEIC** | HEVC-based still image container; 10-bit capable |
| **HDR10/HLG** | Open HDR video transfer/metadata standards; alternative to Dolby Vision |
| **Dolby Vision** | Licensed HDR format; requires Dolby/vendor support |
| **LUT** | Look-up table for colour grading; `.cube` 33³ is the user format |
| **Depth map** | Per-pixel distance image used for portrait/bokeh |
| **Focus stacking** | Merge multiple focus distances into one sharp macro image |
| **Native 4K** | Real 3840×2160+ stream; not an upscale |
| **Vendor-only** | Requires stock app/system signature/proprietary algorithm; not cleanly portable |
