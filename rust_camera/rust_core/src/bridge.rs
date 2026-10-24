//! JNI surface for the Kotlin/Compose shell.
//!
//! Package: `com.opopencam.camera`
//! Class:   `NativeBridge` (Kotlin `object`)
//!
//! The camera session is process-global: only one client may own the preview
//! surface. Every entry point serialises through `ENGINE`.

use std::path::Path;
use std::sync::Mutex;

use jni::objects::{JObject, JString};
use jni::sys::{jboolean, jfloat, jint, jlong, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use log::{error, info};

use crate::camera_engine::{init_logging, CameraEngine};
use crate::capture_modes::CaptureMode;
use crate::motion_photo_xmp::assemble_motion_photo_to_file;
use crate::raw_render::render_dng_file;
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

/// Starts a recording session with preview plus a MediaCodec input surface.
///
/// Kotlin owns the MediaCodec/MediaMuxer/AudioRecord pipeline; Rust only adds
/// the encoder surface to Camera2. The caller must keep both surfaces alive
/// until `nativeStopVideo` returns.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeStartVideo(
    env: JNIEnv,
    _this: JObject,
    preview_surface: JObject,
    video_surface: JObject,
    camera_id: jint,
    width: jint,
    height: jint,
    fps: jint,
) -> jboolean {
    init_logging();
    if preview_surface.is_null() || video_surface.is_null() {
        error!("nativeStartVideo: surface is null");
        return JNI_FALSE;
    }

    let sensor = OnePlus13Sensor::from_id(camera_id);
    let mut guard = engine_lock();
    *guard = None;

    let preview_window = surface_to_window(&env, &preview_surface);
    if preview_window.is_null() {
        error!("nativeStartVideo: preview ANativeWindow is null");
        return JNI_FALSE;
    }
    let video_window = surface_to_window(&env, &video_surface);
    if video_window.is_null() {
        error!("nativeStartVideo: video ANativeWindow is null");
        unsafe { ndk_sys::ANativeWindow_release(preview_window) };
        return JNI_FALSE;
    }

    match CameraEngine::new_video(
        preview_window,
        video_window,
        sensor.camera_id_str(),
        width,
        height,
        fps,
    ) {
        Ok(engine) => {
            info!(
                "nativeStartVideo: camera {} ({}) {}x{}@{} ready",
                engine.camera_id(),
                sensor.sensor_name(),
                width,
                height,
                fps
            );
            *guard = Some(engine);
            JNI_TRUE
        }
        Err(e) => {
            error!(
                "nativeStartVideo: camera {} failed: {}",
                sensor.camera_id_str(),
                e
            );
            JNI_FALSE
        }
    }
}

/// Stops the active video session. The caller must then restore preview with
/// `nativeStartPreview` if the UI remains open.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeStopVideo(
    _env: JNIEnv,
    _this: JObject,
) {
    let mut guard = engine_lock();
    *guard = None;
    info!("nativeStopVideo: stopped");
}

/// Returns whether the active session is a video session.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeIsVideoRunning(
    _env: JNIEnv,
    _this: JObject,
) -> jboolean {
    let guard = engine_lock();
    if guard.as_ref().map(|e| e.is_video()).unwrap_or(false) {
        JNI_TRUE
    } else {
        JNI_FALSE
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

/// Updates the JPEG/DNG orientation for the next still capture.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeSetJpegOrientation(
    _env: JNIEnv,
    _this: JObject,
    degrees: jint,
) -> jboolean {
    let mut guard = engine_lock();
    match guard.as_mut() {
        Some(engine) => match engine.set_orientation(degrees) {
            Ok(()) => JNI_TRUE,
            Err(e) => {
                error!("nativeSetJpegOrientation failed: {}", e);
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

/// Renders a 16-bit Bayer DNG to a neutral JPEG at `output_path`.
///
/// The raw renderer is CPU-heavy (several seconds for a 50 MP DNG) and works on
/// plain files, so this entry point deliberately does **not** take `ENGINE`: a
/// gallery render must not block preview, capture or the camera session. Always
/// call it from a background thread. Returns false on invalid paths, render or
/// write errors and logs the reason.
#[no_mangle]
pub extern "system" fn Java_com_opopencam_camera_NativeBridge_nativeRenderDngToJpeg(
    mut env: JNIEnv,
    _this: JObject,
    input_path: JString,
    output_path: JString,
) -> jboolean {
    init_logging();

    let read_path = |env: &mut JNIEnv, value: &JString, label: &str| -> Option<String> {
        if value.is_null() {
            error!("nativeRenderDngToJpeg: {label} path is null");
            return None;
        }
        let path: String = match env.get_string(value) {
            Ok(value) => value.into(),
            Err(e) => {
                error!("nativeRenderDngToJpeg: invalid {label} path: {e}");
                return None;
            }
        };
        if path.trim().is_empty() {
            error!("nativeRenderDngToJpeg: {label} path is empty");
            return None;
        }
        Some(path)
    };

    let Some(input_path) = read_path(&mut env, &input_path, "input") else {
        return JNI_FALSE;
    };
    let Some(output_path) = read_path(&mut env, &output_path, "output") else {
        return JNI_FALSE;
    };

    let input = Path::new(&input_path);
    let output = Path::new(&output_path);
    if !input.is_file() {
        error!("nativeRenderDngToJpeg: input is not a file: {input_path}");
        return JNI_FALSE;
    }
    if output == input {
        error!("nativeRenderDngToJpeg: input and output are the same path: {input_path}");
        return JNI_FALSE;
    }
    if output.is_dir() {
        error!("nativeRenderDngToJpeg: output is a directory: {output_path}");
        return JNI_FALSE;
    }
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            error!(
                "nativeRenderDngToJpeg: output directory does not exist: {}",
                parent.display()
            );
            return JNI_FALSE;
        }
    }

    let started = std::time::Instant::now();
    match render_dng_file(input, output) {
        Ok(summary) => {
            info!(
                "nativeRenderDngToJpeg: {}x{} -> {}x{} in {:.1}s, {} bytes -> {}",
                summary.input_width,
                summary.input_height,
                summary.output_width,
                summary.output_height,
                started.elapsed().as_secs_f32(),
                summary.jpeg_bytes,
                output_path
            );
            JNI_TRUE
        }
        Err(e) => {
            error!("nativeRenderDngToJpeg: {input_path} -> {output_path}: {e}");
            JNI_FALSE
        }
    }
}
