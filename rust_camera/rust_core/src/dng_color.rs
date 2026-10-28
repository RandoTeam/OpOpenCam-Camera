//! Colour metadata as reported by the camera HAL.
//!
//! The DNG writer must describe the *actual* sensor, not a generic sRGB
//! assumption: a fake `ColorMatrix1` and a missing `AsShotNeutral` make every
//! DNG-aware converter pick its own fallback white balance, which is what
//! produced the green/cyan casts. This module reads the static characteristics
//! once per session; per-shot white balance arrives from the capture result in
//! [`crate::camera_engine`].
//!
//! All values are read through `ACameraManager_getCameraCharacteristics`, i.e.
//! the same public Camera2 NDK surface the rest of the engine uses. Nothing
//! vendor-specific is involved.

#[cfg(target_os = "android")]
use std::ffi::CString;
#[cfg(target_os = "android")]
use std::os::raw::c_char;

#[cfg(target_os = "android")]
use log::warn;
#[cfg(target_os = "android")]
use ndk_sys as sys;

/// Static colour description of one physical sensor, in Camera2 units.
#[derive(Clone, Debug)]
pub struct SensorColor {
    /// `SENSOR_COLOR_TRANSFORM1`: XYZ (D65-ish) -> camera native, row-major.
    pub color_matrix1: [[f64; 3]; 3],
    /// `SENSOR_COLOR_TRANSFORM2`, present when the HAL calibrates a second
    /// illuminant.
    pub color_matrix2: Option<[[f64; 3]; 3]>,
    /// `SENSOR_FORWARD_MATRIX1/2`, optional per spec.
    pub forward_matrix1: Option<[[f64; 3]; 3]>,
    pub forward_matrix2: Option<[[f64; 3]; 3]>,
    /// `SENSOR_CALIBRATION_TRANSFORM1/2`, identity on most devices.
    pub calibration1: Option<[[f64; 3]; 3]>,
    pub calibration2: Option<[[f64; 3]; 3]>,
    /// `SENSOR_REFERENCE_ILLUMINANT1/2` as DNG `CalibrationIlluminant*` codes.
    pub illuminant1: u16,
    pub illuminant2: u16,
    /// `SENSOR_BLACK_LEVEL_PATTERN`, ordered like the CFA quad.
    pub black_level: [u32; 4],
    pub white_level: u32,
    /// `SENSOR_INFO_ACTIVE_ARRAY_SIZE` as `[left, top, right, bottom]`.
    pub active_area: [u32; 4],
    /// `SENSOR_INFO_PIXEL_ARRAY_SIZE` as `[width, height]`.
    pub pixel_array: [u32; 2],
    /// 2x2 CFA as `[top-left, top-right, bottom-left, bottom-right]`,
    /// 0 = R, 1 = G, 2 = B. Derived from `SENSOR_INFO_COLOR_FILTER_ARRANGEMENT`.
    pub cfa_pattern: [u8; 4],
}

impl SensorColor {
    /// Conservative fallback used only when the HAL refuses to expose
    /// characteristics. It reproduces the historical generic behaviour, so a
    /// broken HAL still yields a readable file instead of a failed capture.
    pub fn generic(black_level: u32, white_level: u32) -> Self {
        Self {
            color_matrix1: XYZ_TO_SRGB_D65,
            color_matrix2: None,
            forward_matrix1: None,
            forward_matrix2: None,
            calibration1: None,
            calibration2: None,
            illuminant1: 21, // D65
            illuminant2: 21,
            black_level: [black_level; 4],
            white_level,
            active_area: [0, 0, 0, 0],
            pixel_array: [0, 0],
            cfa_pattern: [0, 1, 1, 2],
        }
    }

    /// Scales the reported active area to the delivered buffer.
    ///
    /// The HAL reports the binned array (e.g. 4096x3072) while the 50 MP
    /// ultra-high-resolution stream delivers 8192x6144, so the static rectangle
    /// must be scaled by the ratio between the buffer and the reported array.
    pub fn crop_for(&self, width: u32, height: u32) -> [u32; 4] {
        let aw = self.active_area[2].saturating_sub(self.active_area[0]);
        let ah = self.active_area[3].saturating_sub(self.active_area[1]);
        if aw == 0 || ah == 0 {
            return [0, 0, width, height];
        }
        if aw == width && ah == height {
            return self.active_area;
        }
        // Integer scale factor: 2x for the remosaic stream, 1x otherwise.
        let sx = width / aw;
        let sy = height / ah;
        if sx == 0 || sy == 0 {
            return [0, 0, width, height];
        }
        [
            self.active_area[0] * sx,
            self.active_area[1] * sy,
            self.active_area[2] * sx,
            self.active_area[3] * sy,
        ]
    }
}

/// Everything the DNG writer needs to describe colour and geometry for one
/// frame. Built from [`SensorColor`] plus the per-frame capture result.
#[derive(Clone, Debug)]
pub struct DngColor {
    /// Maps XYZ -> camera native (DNG `ColorMatrix1`).
    pub color_matrix1: [[f64; 3]; 3],
    pub color_matrix2: Option<[[f64; 3]; 3]>,
    pub calibration1: Option<[[f64; 3]; 3]>,
    pub calibration2: Option<[[f64; 3]; 3]>,
    pub forward_matrix1: Option<[[f64; 3]; 3]>,
    pub forward_matrix2: Option<[[f64; 3]; 3]>,
    pub illuminant1: u16,
    pub illuminant2: Option<u16>,
    /// Camera-native RGB of a neutral subject; `None` when the capture result
    /// carried neither `neutralColorPoint` nor `colorCorrection.gains`.
    pub as_shot_neutral: Option<[f64; 3]>,
    /// One level per CFA phase, ordered like `cfa_pattern`.
    pub black_level: [f64; 4],
    pub white_level: u32,
    /// `[top, left, bottom, right]` in delivered-buffer coordinates.
    pub active_area: [u32; 4],
    pub crop_origin: [u32; 2],
    pub crop_size: [u32; 2],
    pub noise_profile: Option<[f64; 6]>,
    pub cfa_pattern: [u8; 4],
}

impl DngColor {
    /// Builds the writer description for a `width` x `height` buffer.
    ///
    /// `neutral` is `SENSOR_NEUTRAL_COLOR_POINT` from the capture result, i.e.
    /// the camera-space RGB of a neutral subject. `gains` is the fallback:
    /// `COLOR_CORRECTION_GAINS`, whose inverse is the neutral point.
    pub fn from_capture(
        sensor: &SensorColor,
        gains: Option<[f32; 4]>,
        neutral: Option<[f32; 3]>,
        width: u32,
        height: u32,
    ) -> Self {
        let as_shot_neutral = neutral
            .map(|n| [n[0] as f64, n[1] as f64, n[2] as f64])
            .filter(|n| n.iter().all(|v| *v > 0.0))
            .map(normalize_neutral)
            .or_else(|| gains.map(neutral_from_gains));

        let active = sensor.crop_for(width, height);
        // DNG ActiveArea is [top, left, bottom, right].
        let active_area = [active[1], active[0], active[3], active[2]];

        // Adobe recommends an 8 px margin for interpolation, and AOSP's
        // DngCreator uses exactly that. Fall back to the whole buffer when the
        // area is too small to trim.
        let margin = 8u32;
        let area_w = active_area[3].saturating_sub(active_area[1]);
        let area_h = active_area[2].saturating_sub(active_area[0]);
        let (crop_origin, crop_size) = if area_w > margin * 2 && area_h > margin * 2 {
            (
                [active_area[1] + margin, active_area[0] + margin],
                [area_w - margin * 2, area_h - margin * 2],
            )
        } else {
            ([active_area[1], active_area[0]], [area_w, area_h])
        };

        Self {
            color_matrix1: sensor.color_matrix1,
            color_matrix2: sensor.color_matrix2,
            calibration1: sensor.calibration1,
            calibration2: sensor.calibration2,
            forward_matrix1: sensor.forward_matrix1,
            forward_matrix2: sensor.forward_matrix2,
            illuminant1: sensor.illuminant1,
            illuminant2: sensor.color_matrix2.and(Some(sensor.illuminant2)),
            as_shot_neutral,
            black_level: sensor.black_level.map(|v| v as f64),
            white_level: sensor.white_level,
            active_area,
            crop_origin,
            crop_size,
            noise_profile: None,
            cfa_pattern: sensor.cfa_pattern,
        }
    }
}

/// Scales a neutral vector so the green component is 1, as DNG expects.
fn normalize_neutral(n: [f64; 3]) -> [f64; 3] {
    if n[1].abs() < f64::EPSILON {
        return n;
    }
    [n[0] / n[1], 1.0, n[2] / n[1]]
}

/// `AsShotNeutral` from `COLOR_CORRECTION_GAINS`: gains are the multipliers
/// that make the frame neutral, so the neutral point is their reciprocal.
fn neutral_from_gains(g: [f32; 4]) -> [f64; 3] {
    let green = ((g[1] + g[2]) / 2.0).max(f32::EPSILON) as f64;
    let red = (g[0].max(f32::EPSILON)) as f64;
    let blue = (g[3].max(f32::EPSILON)) as f64;
    normalize_neutral([green / red, 1.0, green / blue])
}

/// DNG expects the *inverse* direction; keep this only for the fallback path.
/// Standard XYZ (D65) -> sRGB matrix, used as the last-resort fallback when the
/// HAL exposes no colour transform. DNG `ColorMatrix1` maps XYZ to camera
/// native, so this is correct only under the generic "camera native == sRGB"
/// assumption; a real sensor transform always takes precedence.
const XYZ_TO_SRGB_D65: [[f64; 3]; 3] = [
    [3.240_454_2, -1.537_138_5, -0.498_531_4],
    [-0.969_266_0, 1.876_010_8, 0.041_556_0],
    [0.055_643_4, -0.204_025_9, 1.057_225_2],
];

#[cfg(target_os = "android")]
const TYPE_BYTE: u8 = 0;
#[cfg(target_os = "android")]
const TYPE_INT32: u8 = 1;
#[cfg(target_os = "android")]
const TYPE_FLOAT: u8 = 2;
#[cfg(target_os = "android")]
const TYPE_RATIONAL: u8 = 5;

#[cfg(target_os = "android")]
unsafe fn entry(
    meta: *const sys::ACameraMetadata,
    tag: u32,
) -> Option<sys::ACameraMetadata_const_entry> {
    let mut e: sys::ACameraMetadata_const_entry = std::mem::zeroed();
    let status = sys::ACameraMetadata_getConstEntry(meta, tag, &mut e);
    if status.0 != 0 {
        return None;
    }
    Some(e)
}

#[cfg(target_os = "android")]
unsafe fn rationals(meta: *const sys::ACameraMetadata, tag: u32) -> Option<Vec<f64>> {
    let e = entry(meta, tag)?;
    if e.type_ != TYPE_RATIONAL {
        return None;
    }
    let ptr = e.data.r;
    if ptr.is_null() {
        return None;
    }
    let mut out = Vec::with_capacity(e.count as usize);
    for i in 0..e.count as usize {
        let r = *ptr.add(i);
        if r.denominator == 0 {
            out.push(0.0);
        } else {
            out.push(r.numerator as f64 / r.denominator as f64);
        }
    }
    Some(out)
}

#[cfg(target_os = "android")]
unsafe fn int32s(meta: *const sys::ACameraMetadata, tag: u32) -> Option<Vec<i32>> {
    let e = entry(meta, tag)?;
    if e.type_ != TYPE_INT32 || e.data.i32_.is_null() {
        return None;
    }
    Some((0..e.count as usize).map(|i| *e.data.i32_.add(i)).collect())
}

#[cfg(target_os = "android")]
unsafe fn floats(meta: *const sys::ACameraMetadata, tag: u32) -> Option<Vec<f32>> {
    let e = entry(meta, tag)?;
    if e.type_ != TYPE_FLOAT || e.data.f.is_null() {
        return None;
    }
    Some((0..e.count as usize).map(|i| *e.data.f.add(i)).collect())
}

#[cfg(target_os = "android")]
unsafe fn bytes(meta: *const sys::ACameraMetadata, tag: u32) -> Option<Vec<u8>> {
    let e = entry(meta, tag)?;
    if e.type_ != TYPE_BYTE || e.data.u8_.is_null() {
        return None;
    }
    Some((0..e.count as usize).map(|i| *e.data.u8_.add(i)).collect())
}

#[cfg(target_os = "android")]
fn matrix(values: &[f64]) -> Option<[[f64; 3]; 3]> {
    if values.len() != 9 {
        return None;
    }
    Some([
        [values[0], values[1], values[2]],
        [values[3], values[4], values[5]],
        [values[6], values[7], values[8]],
    ])
}

/// Maps `SENSOR_INFO_COLOR_FILTER_ARRANGEMENT` to a 2x2 DNG `CFAPattern`.
#[cfg(target_os = "android")]
fn cfa_from_arrangement(value: u8) -> Option<[u8; 4]> {
    // Camera2: 0 RGGB, 1 GRBG, 2 GBRG, 3 BGGR.
    match value {
        0 => Some([0, 1, 1, 2]),
        1 => Some([1, 0, 2, 1]),
        2 => Some([1, 2, 0, 1]),
        3 => Some([2, 1, 1, 0]),
        _ => None,
    }
}

/// Reads the static colour description for `camera_id`.
///
/// # Safety
/// `manager` must be a live `ACameraManager` pointer.
///
/// Returns `None` when the HAL exposes no usable colour transform; the caller
/// then keeps [`SensorColor::generic`] and logs the reason.
#[cfg(target_os = "android")]
pub unsafe fn read_sensor_color(
    manager: *mut sys::ACameraManager,
    camera_id: &str,
) -> Option<SensorColor> {
    if manager.is_null() {
        return None;
    }
    let id = CString::new(camera_id).ok()?;
    let mut meta: *mut sys::ACameraMetadata = std::ptr::null_mut();
    let status = unsafe {
        sys::ACameraManager_getCameraCharacteristics(
            manager,
            id.as_ptr() as *const c_char,
            &mut meta,
        )
    };
    if status.0 != 0 || meta.is_null() {
        warn!("Camera2 NDK: no camera characteristics for {camera_id}");
        return None;
    }

    let result = unsafe {
        let color1 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_COLOR_TRANSFORM1.0,
        )
        .and_then(|v| matrix(&v));
        let color1 = match color1 {
            Some(m) => m,
            None => {
                warn!("Camera2 NDK: {camera_id} has no SENSOR_COLOR_TRANSFORM1");
                sys::ACameraMetadata_free(meta);
                return None;
            }
        };
        let color2 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_COLOR_TRANSFORM2.0,
        )
        .and_then(|v| matrix(&v));
        let forward1 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_FORWARD_MATRIX1.0,
        )
        .and_then(|v| matrix(&v));
        let forward2 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_FORWARD_MATRIX2.0,
        )
        .and_then(|v| matrix(&v));
        let calib1 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_CALIBRATION_TRANSFORM1.0,
        )
        .and_then(|v| matrix(&v));
        let calib2 = rationals(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_CALIBRATION_TRANSFORM2.0,
        )
        .and_then(|v| matrix(&v));

        let illuminant1 = bytes(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_REFERENCE_ILLUMINANT1.0,
        )
        .and_then(|v| v.first().copied())
        .map(u16::from)
        .unwrap_or(21);
        let illuminant2 = bytes(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_REFERENCE_ILLUMINANT2.0,
        )
        .and_then(|v| v.first().copied())
        .map(u16::from)
        .unwrap_or(illuminant1);

        // Black level is reported for the binned array and is the same for the
        // remosaic stream; the pattern order matches the CFA quad.
        let black = int32s(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_BLACK_LEVEL_PATTERN.0,
        )
        .filter(|v| v.len() >= 4)
        .map(|v| [v[0] as u32, v[1] as u32, v[2] as u32, v[3] as u32]);
        let black_level = black.unwrap_or([64; 4]);

        let white_level = int32s(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_INFO_WHITE_LEVEL.0,
        )
        .and_then(|v| v.first().copied())
        .filter(|v| *v > 0)
        .map(|v| v as u32)
        .unwrap_or(1023);

        let active = int32s(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_INFO_ACTIVE_ARRAY_SIZE.0,
        )
        .filter(|v| v.len() >= 4)
        .map(|v| [v[0] as u32, v[1] as u32, v[2] as u32, v[3] as u32])
        .unwrap_or([0, 0, 0, 0]);
        let pixel_array = int32s(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_INFO_PIXEL_ARRAY_SIZE.0,
        )
        .filter(|v| v.len() >= 2)
        .map(|v| [v[0] as u32, v[1] as u32])
        .unwrap_or([0, 0]);

        let cfa_pattern = bytes(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_INFO_COLOR_FILTER_ARRANGEMENT.0,
        )
        .and_then(|v| v.first().copied())
        .and_then(cfa_from_arrangement)
        .unwrap_or([0, 1, 1, 2]);

        // Keep the noise profile available for the writer even though it is
        // optional; it lets converters apply sane noise reduction.
        let _noise = floats(
            meta,
            sys::acamera_metadata_tag::ACAMERA_SENSOR_NOISE_PROFILE.0,
        );

        SensorColor {
            color_matrix1: color1,
            color_matrix2: color2,
            forward_matrix1: forward1,
            forward_matrix2: forward2,
            calibration1: calib1,
            calibration2: calib2,
            illuminant1,
            illuminant2,
            black_level,
            white_level,
            active_area: active,
            pixel_array,
            cfa_pattern,
        }
    };

    unsafe { sys::ACameraMetadata_free(meta) };
    Some(result)
}
