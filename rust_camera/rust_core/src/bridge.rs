//! JNI surface for the Kotlin/Compose shell.
//!
//! Package: `com.opopencam.camera`
//! Class:   `NativeBridge` (Kotlin `object`)
//!
//! The camera session is process-global: only one client may own the preview
//! surface. Every entry point serialises through `ENGINE`.

use std::sync::Mutex;

use jni::objects::{JObject, JString};
use jni::sys::{jboolean, jfloat, jint, jlong, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use log::{error, info};

use crate::camera_engine::{init_logging, CameraEngine};
use crate::capture_modes::CaptureMode;
use crate::motion_photo_xmp::assemble_motion_photo_to_file;
use crate::sensors::OnePlus13Sensor;

static ENGINE: Mutex<Option<CameraEngine>> = Mutex::new(None);

fn engine_lock() -> std::sync::MutexGuard<'static, Option<CameraEngine>> {
    ENGINE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn surface_to_window(env: &JNIEnv, surface: &JObject) -> *mut ndk_sys::ANativeWindow {
    let raw_env = env.get_native_interface() as *mut _;
    let raw_surface = surface.as_raw();
    unsafe { ndk_sys::ANativeWindow_fromSurface(raw_env, raw_surface) }
}

/// Starts a Camera2 NDK session for `camera_id` and capture `mode`.
///
/// `mode` is a [`CaptureMode`] integer (0 = 12 MP JPEG, 1 = 50 MP JPEG,
/// 2 = RAW 12, 3 = RAW 50, 4 = HDR). If a high-resolution/RAW/HDR mode cannot
/// be configured, the engine retries with 12 MP JPEG so the preview still comes
/// up. Takes ownership of the native window; the previous session is dropped.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeStartPreview(
    env: JNIEnv,
    _this: JObject,
    surface: JObject,
    camera_id: jint,
    mode: jint,
) -> jboolean {
    init_logging();
    if surface.is_null() {
        error!("nativeStartPreview: surface is null");
        return JNI_FALSE;
    }

    let requested_mode = CaptureMode::from_i32(mode);
    let sensor = OnePlus13Sensor::from_id(camera_id);
    let mut guard = engine_lock();
    *guard = None;

    let window = surface_to_window(&env, &surface);
    if window.is_null() {
        error!("nativeStartPreview: ANativeWindow_fromSurface returned null");
        return JNI_FALSE;
    }

    match CameraEngine::new(window, sensor.camera_id_str(), requested_mode) {
        Ok(engine) => {
            info!(
                "nativeStartPreview: camera {} ({}) mode {} ready",
                engine.camera_id(),
                sensor.sensor_name(),
                engine.capture_mode().label()
            );
            *guard = Some(engine);
            JNI_TRUE
        }
        Err(first_error) => {
            error!(
                "nativeStartPreview: mode {} on camera {} failed: {}",
                requested_mode.label(),
                sensor.camera_id_str(),
                first_error
            );
            if requested_mode == CaptureMode::Jpeg12 {
                return JNI_FALSE;
            }

            // Fallback keeps the app usable if the HAL rejects 50 MP/RAW/HDR.
            let fallback_window = surface_to_window(&env, &surface);
            if fallback_window.is_null() {
                error!("nativeStartPreview: fallback window is null");
                return JNI_FALSE;
            }
            match CameraEngine::new(fallback_window, sensor.camera_id_str(), CaptureMode::Jpeg12) {
                Ok(engine) => {
                    info!("nativeStartPreview: fell back to 12 MP JPEG");
                    *guard = Some(engine);
                    JNI_TRUE
                }
                Err(second_error) => {
                    error!("nativeStartPreview: fallback failed: {}", second_error);
                    JNI_FALSE
                }
            }
        }
    }
}

/// Stops the active session and releases the preview window.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeStopPreview(
    _env: JNIEnv,
    _this: JObject,
) {
    let mut guard = engine_lock();
    *guard = None;
    info!("nativeStopPreview: stopped");
}

/// Captures using the active mode and writes the result to `path`.
///
/// Blocks until the file is written (HDR may take several seconds). Always call
/// from a background thread.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeCapturePhoto(
    mut env: JNIEnv,
    _this: JObject,
    path: JString,
) -> jboolean {
    if path.is_null() {
        error!("nativeCapturePhoto: path is null");
        return JNI_FALSE;
    }
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(e) => {
            error!("nativeCapturePhoto: invalid path: {}", e);
            return JNI_FALSE;
        }
    };
    if path_str.trim().is_empty() {
        error!("nativeCapturePhoto: path is empty");
        return JNI_FALSE;
    }

    let mut guard = engine_lock();
    match guard.as_mut() {
        Some(engine) => match engine.capture_photo(&path_str) {
            Ok(()) => JNI_TRUE,
            Err(e) => {
                error!("nativeCapturePhoto failed: {}", e);
                JNI_FALSE
            }
        },
        None => {
            error!("nativeCapturePhoto: no active session");
            JNI_FALSE
        }
    }
}

/// Updates `CONTROL_ZOOM_RATIO` on the repeating preview request.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeSetZoom(
    _env: JNIEnv,
    _this: JObject,
    ratio: jfloat,
) -> jboolean {
    let mut guard = engine_lock();
    match guard.as_mut() {
        Some(engine) => match engine.set_zoom(ratio) {
            Ok(()) => JNI_TRUE,
            Err(e) => {
                error!("nativeSetZoom failed: {}", e);
                JNI_FALSE
            }
        },
        None => JNI_FALSE,
    }
}

/// Returns the mode actually active after any HAL fallback, or -1 without a session.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeGetActiveMode(
    _env: JNIEnv,
    _this: JObject,
) -> jint {
    let guard = engine_lock();
    guard
        .as_ref()
        .map(|engine| engine.capture_mode().as_i32())
        .unwrap_or(-1)
}

/// Returns whether a Camera2 session is currently active.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeIsRunning(
    _env: JNIEnv,
    _this: JObject,
) -> jboolean {
    let guard = engine_lock();
    if guard.as_ref().map(|e| e.is_running()).unwrap_or(false) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

/// Assembles a Google Motion Photo from an existing JPEG and MP4.
///
/// Used by the Live Photo milestone; kept here so the container logic can be
/// exercised from the UI without changing the camera session.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeAssembleMotionPhoto(
    mut env: JNIEnv,
    _this: JObject,
    jpeg_path: JString,
    mp4_path: JString,
    out_path: JString,
    timestamp_us: jlong,
) -> jboolean {
    let read_string = |env: &mut JNIEnv, value: &JString| -> Option<String> {
        if value.is_null() {
            return None;
        }
        env.get_string(value).ok().map(|s| s.into())
    };

    let Some(jpeg_path) = read_string(&mut env, &jpeg_path) else {
        error!("nativeAssembleMotionPhoto: invalid JPEG path");
        return JNI_FALSE;
    };
    let Some(mp4_path) = read_string(&mut env, &mp4_path) else {
        error!("nativeAssembleMotionPhoto: invalid MP4 path");
        return JNI_FALSE;
    };
    let Some(out_path) = read_string(&mut env, &out_path) else {
        error!("nativeAssembleMotionPhoto: invalid output path");
        return JNI_FALSE;
    };

    let jpeg = match std::fs::read(&jpeg_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("nativeAssembleMotionPhoto: read {}: {}", jpeg_path, e);
            return JNI_FALSE;
        }
    };
    let mp4 = match std::fs::read(&mp4_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("nativeAssembleMotionPhoto: read {}: {}", mp4_path, e);
            return JNI_FALSE;
        }
    };

    match assemble_motion_photo_to_file(&out_path, &jpeg, &mp4, timestamp_us) {
        Ok(()) => {
            info!("nativeAssembleMotionPhoto: wrote {}", out_path);
            JNI_TRUE
        }
        Err(e) => {
            error!("nativeAssembleMotionPhoto: {}", e);
            JNI_FALSE
        }
    }
}
