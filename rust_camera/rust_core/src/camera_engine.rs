//! Camera2 NDK session for the OpOpenCam hybrid shell.
//!
//! The Compose layer owns the UI and passes a `Surface` to this module. Camera2
//! writes the preview directly into that surface (zero-copy), while a second
//! `AImageReader` receives the still output selected by [`CaptureMode`]:
//!
//! * 12 MP / 50 MP JPEG
//! * 12 MP / 50 MP RAW (converted to DNG)
//! * three-frame YUV bracket, exposure-fused and encoded as JPEG
//!
//! Mode and lens changes restart the session with the same preview surface. This
//! keeps the stream configuration valid and avoids the HAL's simultaneous-output
//! limit; a seamless logical-zoom path is a later milestone.

use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use log::{info, warn};
use ndk_sys as sys;

use crate::capture_modes::CaptureMode;
use crate::dng_writer::write_dng_16bit;
use crate::hdr::{encode_jpeg, exposure_fusion_aligned, rotate90_cw, yuv420_to_rgb, YuvFrame};
use crate::sensors::{
    VENDOR_TAG_ENABLE_IDEAL_RAW, VENDOR_TAG_OPLUS_PACKAGE_NAME, VENDOR_TAG_ULTRA_RESOLUTION_MODE,
};

const PREVIEW_WIDTH: i32 = 1920;
const PREVIEW_HEIGHT: i32 = 1080;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(15);

const TEMPLATE_PREVIEW: u32 = 1;
const TEMPLATE_STILL_CAPTURE: u32 = 2;

// Camera2 metadata tags.
const TAG_AE_EXPOSURE_COMPENSATION: u32 = 65537;
const TAG_AE_MODE: u32 = 65539;
const TAG_AF_MODE: u32 = 65543;
const TAG_AWB_MODE: u32 = 65547;
const TAG_ZOOM_RATIO: u32 = 65583;
const TAG_JPEG_QUALITY: u32 = 458756;
const TAG_JPEG_ORIENTATION: u32 = 458755;

const AE_MODE_ON: u8 = 1;
const AF_MODE_CONTINUOUS_PICTURE: u8 = 4;
const AWB_MODE_AUTO: u8 = 1;
const JPEG_QUALITY: u8 = 95;
/// Portrait rear shooting. The HAL writes this into EXIF for JPEG outputs.
const JPEG_ORIENTATION: i32 = 90;
const RAW_BLACK_LEVEL: u32 = 64;
const RAW_WHITE_LEVEL_10BIT: u32 = 1023;

/// One acquired still buffer. JPEG/RAW use plane 0; YUV uses planes 0..3.
#[derive(Clone, Debug)]
struct StillFrame {
    width: i32,
    height: i32,
    planes: Vec<Vec<u8>>,
    row_strides: Vec<i32>,
    pixel_strides: Vec<i32>,
}

struct StillContext {
    frames: Mutex<Vec<StillFrame>>,
    cv: Condvar,
}

unsafe extern "C" fn on_still_available(context: *mut c_void, reader: *mut sys::AImageReader) {
    if context.is_null() || reader.is_null() {
        return;
    }
    let ctx = &*(context as *const StillContext);

    let mut image: *mut sys::AImage = ptr::null_mut();
    let status = sys::AImageReader_acquireNextImage(reader, &mut image);
    if status.0 != 0 || image.is_null() {
        warn!(
            "Still reader: acquireNextImage failed (status {})",
            status.0
        );
        return;
    }

    let mut width = 0;
    let mut height = 0;
    let mut format = 0;
    sys::AImage_getWidth(image, &mut width);
    sys::AImage_getHeight(image, &mut height);
    sys::AImage_getFormat(image, &mut format);

    let plane_count = if format == 35 { 3 } else { 1 };
    let mut planes = Vec::with_capacity(plane_count);
    let mut row_strides = Vec::with_capacity(plane_count);
    let mut pixel_strides = Vec::with_capacity(plane_count);
    let mut copied_any = false;

    for plane in 0..plane_count {
        let plane_index = plane as i32;
        let mut data: *mut u8 = ptr::null_mut();
        let mut len: i32 = 0;
        let mut row_stride: i32 = 0;
        let mut pixel_stride: i32 = 0;
        let data_status = sys::AImage_getPlaneData(image, plane_index, &mut data, &mut len);
        sys::AImage_getPlaneRowStride(image, plane_index, &mut row_stride);
        sys::AImage_getPlanePixelStride(image, plane_index, &mut pixel_stride);

        if data_status.0 == 0 && !data.is_null() && len > 0 {
            planes.push(std::slice::from_raw_parts(data, len as usize).to_vec());
            copied_any = true;
        } else {
            planes.push(Vec::new());
        }
        row_strides.push(row_stride);
        pixel_strides.push(pixel_stride);
    }

    if copied_any {
        let frame = StillFrame {
            width,
            height,
            planes,
            row_strides,
            pixel_strides,
        };
        info!(
            "Still reader: {}x{} format {} ({} planes)",
            width, height, format, plane_count
        );
        if let Ok(mut frames) = ctx.frames.lock() {
            frames.push(frame);
            ctx.cv.notify_all();
        }
    } else {
        warn!("Still reader: no plane data for format {}", format);
    }

    sys::AImage_delete(image);
}

/// Reads the JPEG SOF dimensions.
///
/// `AImage_getWidth/Height` report the reader configuration, not the encoded
/// JPEG. The HAL can silently return a smaller JPEG (observed at "50 MP":
/// 8192x6144 reader, 1920x1440 payload), so validate the actual SOF.
fn jpeg_dimensions(bytes: &[u8]) -> Option<(u16, u16)> {
    let mut i = 2usize;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];
        if marker == 0xD8 || marker == 0xD9 {
            i += 2;
            continue;
        }
        if marker == 0xDA {
            break;
        }
        let length = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if length < 2 || i + 2 + length > bytes.len() {
            break;
        }
        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF)
            && i + 9 <= bytes.len()
        {
            let height = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]);
            let width = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]);
            return Some((width, height));
        }
        i += 2 + length;
    }
    None
}

/// A Camera2 NDK session with a direct preview surface and a mode-specific still reader.
pub struct CameraEngine {
    manager: *mut sys::ACameraManager,
    device: *mut sys::ACameraDevice,
    session: *mut sys::ACameraCaptureSession,
    preview_window: *mut sys::ANativeWindow,
    preview_output: *mut sys::ACaptureSessionOutput,
    still_output: *mut sys::ACaptureSessionOutput,
    still_reader: *mut sys::AImageReader,
    output_container: *mut sys::ACaptureSessionOutputContainer,
    preview_request: *mut sys::ACaptureRequest,
    preview_target: *mut sys::ACameraOutputTarget,
    still_target: *mut sys::ACameraOutputTarget,
    still_ctx: Arc<StillContext>,
    still_ctx_ptr: *mut c_void,
    capture_mode: CaptureMode,
    still_width: i32,
    still_height: i32,
    camera_id: String,
    running: bool,
}

// All access goes through the JNI bridge mutex; raw NDK pointers are only used
// while that mutex is held.
unsafe impl Send for CameraEngine {}

impl CameraEngine {
    /// Creates a session for `camera_id` and `mode`.
    ///
    /// The preview surface is written by the HAL directly. The still reader is
    /// allocated according to the mode. Takes ownership of `preview_window` even
    /// on failure; the window is released in [`shutdown`](Self::shutdown).
    pub fn new(
        preview_window: *mut sys::ANativeWindow,
        camera_id: &str,
        mode: CaptureMode,
    ) -> Result<Self, String> {
        let still_ctx = Arc::new(StillContext {
            frames: Mutex::new(Vec::new()),
            cv: Condvar::new(),
        });

        let (still_width, still_height) = mode.still_size();
        let mut engine = Self {
            manager: ptr::null_mut(),
            device: ptr::null_mut(),
            session: ptr::null_mut(),
            preview_window,
            preview_output: ptr::null_mut(),
            still_output: ptr::null_mut(),
            still_reader: ptr::null_mut(),
            output_container: ptr::null_mut(),
            preview_request: ptr::null_mut(),
            preview_target: ptr::null_mut(),
            still_target: ptr::null_mut(),
            still_ctx_ptr: Arc::as_ptr(&still_ctx) as *mut c_void,
            still_ctx,
            capture_mode: mode,
            still_width,
            still_height,
            camera_id: camera_id.to_string(),
            running: false,
        };

        if let Err(e) = unsafe { engine.open(camera_id) } {
            engine.shutdown();
            return Err(e);
        }
        Ok(engine)
    }

    unsafe fn open(&mut self, camera_id: &str) -> Result<(), String> {
        self.manager = sys::ACameraManager_create();
        if self.manager.is_null() {
            return Err("ACameraManager_create returned null".into());
        }
        info!("Camera2 NDK: manager created");

        let c_id = CString::new(camera_id).map_err(|e| e.to_string())?;
        let mut device_callbacks = sys::ACameraDevice_StateCallbacks {
            context: ptr::null_mut(),
            onDisconnected: None,
            onError: None,
        };
        let status = sys::ACameraManager_openCamera(
            self.manager,
            c_id.as_ptr() as *const c_char,
            &mut device_callbacks,
            &mut self.device,
        );
        if status.0 != 0 || self.device.is_null() {
            return Err(format!(
                "openCamera({}) failed with status {}",
                camera_id, status.0
            ));
        }
        info!("Camera2 NDK: camera {} opened", camera_id);

        let before_w = sys::ANativeWindow_getWidth(self.preview_window);
        let before_h = sys::ANativeWindow_getHeight(self.preview_window);
        info!(
            "Camera2 NDK: preview window before geometry: {}x{}",
            before_w, before_h
        );
        let geom_status = sys::ANativeWindow_setBuffersGeometry(
            self.preview_window,
            PREVIEW_WIDTH,
            PREVIEW_HEIGHT,
            0,
        );
        let after_w = sys::ANativeWindow_getWidth(self.preview_window);
        let after_h = sys::ANativeWindow_getHeight(self.preview_window);
        info!(
            "Camera2 NDK: preview window after geometry: {}x{} (status {})",
            after_w, after_h, geom_status
        );

        let preview_status =
            sys::ACaptureSessionOutput_create(self.preview_window, &mut self.preview_output);
        if preview_status.0 != 0 || self.preview_output.is_null() {
            return Err(format!(
                "preview output create failed with status {}",
                preview_status.0
            ));
        }

        let max_images = if self.capture_mode.is_hdr() { 4 } else { 2 };
        let reader_status = sys::AImageReader_new(
            self.still_width,
            self.still_height,
            self.capture_mode.image_format(),
            max_images,
            &mut self.still_reader,
        );
        if reader_status.0 != 0 || self.still_reader.is_null() {
            return Err(format!(
                "still AImageReader create failed for {} ({}x{}, format {}, status {})",
                self.capture_mode.label(),
                self.still_width,
                self.still_height,
                self.capture_mode.image_format(),
                reader_status.0
            ));
        }

        let mut still_window: *mut sys::ANativeWindow = ptr::null_mut();
        sys::AImageReader_getWindow(self.still_reader, &mut still_window);
        if still_window.is_null() {
            return Err("still AImageReader returned a null window".into());
        }

        let mut listener = sys::AImageReader_ImageListener {
            context: self.still_ctx_ptr,
            onImageAvailable: Some(on_still_available),
        };
        sys::AImageReader_setImageListener(self.still_reader, &mut listener);

        let still_status = sys::ACaptureSessionOutput_create(still_window, &mut self.still_output);
        if still_status.0 != 0 || self.still_output.is_null() {
            return Err(format!(
                "still output create failed with status {}",
                still_status.0
            ));
        }

        let container_status =
            sys::ACaptureSessionOutputContainer_create(&mut self.output_container);
        if container_status.0 != 0 || self.output_container.is_null() {
            return Err(format!(
                "output container create failed with status {}",
                container_status.0
            ));
        }
        sys::ACaptureSessionOutputContainer_add(self.output_container, self.preview_output);
        sys::ACaptureSessionOutputContainer_add(self.output_container, self.still_output);

        let session_callbacks = sys::ACameraCaptureSession_stateCallbacks {
            context: ptr::null_mut(),
            onClosed: None,
            onReady: None,
            onActive: None,
        };

        // 50 MP needs the OPlus/Qualcomm ultra-resolution session parameters.
        // The standard stream list only advertises 4096x3072 JPEG; without the
        // vendor mode the HAL silently returns a 1920x1440 JPEG.
        let session_params = if self.capture_mode.is_50mp() {
            let mut params: *mut sys::ACaptureRequest = ptr::null_mut();
            let create = sys::ACameraDevice_createCaptureRequest(
                self.device,
                sys::ACameraDevice_request_template(TEMPLATE_PREVIEW),
                &mut params,
            );
            if create.0 == 0 && !params.is_null() {
                let one_i32: i32 = 1;
                sys::ACaptureRequest_setEntry_i32(
                    params,
                    VENDOR_TAG_ULTRA_RESOLUTION_MODE,
                    1,
                    &one_i32,
                );
                let one_u8: u8 = 1;
                sys::ACaptureRequest_setEntry_u8(params, VENDOR_TAG_ENABLE_IDEAL_RAW, 1, &one_u8);
                let package = b"com.opopencam.camera\0";
                sys::ACaptureRequest_setEntry_u8(
                    params,
                    VENDOR_TAG_OPLUS_PACKAGE_NAME,
                    package.len() as u32,
                    package.as_ptr(),
                );
                info!("Camera2 NDK: enabling 50 MP vendor session parameters");
            }
            params
        } else {
            ptr::null_mut()
        };

        let session_status = if session_params.is_null() {
            sys::ACameraDevice_createCaptureSession(
                self.device,
                self.output_container,
                &session_callbacks,
                &mut self.session,
            )
        } else {
            sys::ACameraDevice_createCaptureSessionWithSessionParameters(
                self.device,
                self.output_container,
                session_params,
                &session_callbacks,
                &mut self.session,
            )
        };
        if !session_params.is_null() {
            sys::ACaptureRequest_free(session_params);
        }
        if session_status.0 != 0 || self.session.is_null() {
            return Err(format!(
                "createCaptureSession failed with status {}",
                session_status.0
            ));
        }

        let request_status = sys::ACameraDevice_createCaptureRequest(
            self.device,
            sys::ACameraDevice_request_template(TEMPLATE_PREVIEW),
            &mut self.preview_request,
        );
        if request_status.0 != 0 || self.preview_request.is_null() {
            return Err(format!(
                "preview request create failed with status {}",
                request_status.0
            ));
        }

        let target_status =
            sys::ACameraOutputTarget_create(self.preview_window, &mut self.preview_target);
        if target_status.0 != 0 || self.preview_target.is_null() {
            return Err(format!(
                "preview target create failed with status {}",
                target_status.0
            ));
        }
        sys::ACaptureRequest_addTarget(self.preview_request, self.preview_target);

        let mut still_window_for_target: *mut sys::ANativeWindow = ptr::null_mut();
        sys::AImageReader_getWindow(self.still_reader, &mut still_window_for_target);
        let still_target_status =
            sys::ACameraOutputTarget_create(still_window_for_target, &mut self.still_target);
        if still_target_status.0 != 0 || self.still_target.is_null() {
            return Err(format!(
                "still target create failed with status {}",
                still_target_status.0
            ));
        }

        Self::apply_common_entries(self.preview_request);
        let mut seq = 0;
        let mut repeat_status = sys::ACameraCaptureSession_setRepeatingRequest(
            self.session,
            ptr::null_mut(),
            1,
            &mut self.preview_request,
            &mut seq,
        );
        // Some HALs report "session not ready" briefly after creation and never
        // invoke onReady. Retry quickly instead of blocking the UI for 2 seconds.
        for _ in 0..19 {
            if repeat_status.0 == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
            repeat_status = sys::ACameraCaptureSession_setRepeatingRequest(
                self.session,
                ptr::null_mut(),
                1,
                &mut self.preview_request,
                &mut seq,
            );
        }
        if repeat_status.0 != 0 {
            return Err(format!(
                "setRepeatingRequest failed with status {}",
                repeat_status.0
            ));
        }

        self.running = true;
        info!(
            "Camera2 NDK: preview active; mode={} still={}x{} format={}",
            self.capture_mode.label(),
            self.still_width,
            self.still_height,
            self.capture_mode.image_format()
        );
        Ok(())
    }

    unsafe fn apply_common_entries(request: *mut sys::ACaptureRequest) {
        let ae = AE_MODE_ON;
        let af = AF_MODE_CONTINUOUS_PICTURE;
        let awb = AWB_MODE_AUTO;
        sys::ACaptureRequest_setEntry_u8(request, TAG_AE_MODE, 1, &ae);
        sys::ACaptureRequest_setEntry_u8(request, TAG_AF_MODE, 1, &af);
        sys::ACaptureRequest_setEntry_u8(request, TAG_AWB_MODE, 1, &awb);
    }

    unsafe fn apply_jpeg_entries(request: *mut sys::ACaptureRequest) {
        let quality = JPEG_QUALITY;
        sys::ACaptureRequest_setEntry_u8(request, TAG_JPEG_QUALITY, 1, &quality);
        let orientation = JPEG_ORIENTATION;
        sys::ACaptureRequest_setEntry_i32(request, TAG_JPEG_ORIENTATION, 1, &orientation);
    }

    unsafe fn create_still_request(&self) -> Result<*mut sys::ACaptureRequest, String> {
        let mut request: *mut sys::ACaptureRequest = ptr::null_mut();
        let status = sys::ACameraDevice_createCaptureRequest(
            self.device,
            sys::ACameraDevice_request_template(TEMPLATE_STILL_CAPTURE),
            &mut request,
        );
        if status.0 != 0 || request.is_null() {
            return Err(format!(
                "still request create failed with status {}",
                status.0
            ));
        }
        let add_status = sys::ACaptureRequest_addTarget(request, self.still_target);
        if add_status.0 != 0 {
            sys::ACaptureRequest_free(request);
            return Err(format!(
                "add still target failed with status {}",
                add_status.0
            ));
        }
        Self::apply_common_entries(request);
        Ok(request)
    }

    unsafe fn submit_requests(
        &self,
        requests: &mut [*mut sys::ACaptureRequest],
    ) -> Result<(), String> {
        let mut seq = 0;
        let status = sys::ACameraCaptureSession_capture(
            self.session,
            ptr::null_mut(),
            requests.len() as i32,
            requests.as_mut_ptr(),
            &mut seq,
        );
        for request in requests.iter() {
            if !request.is_null() {
                sys::ACaptureRequest_free(*request);
            }
        }
        if status.0 != 0 {
            return Err(format!("capture failed with status {}", status.0));
        }
        Ok(())
    }

    fn clear_still_frames(&self) {
        if let Ok(mut frames) = self.still_ctx.frames.lock() {
            frames.clear();
        }
    }

    fn wait_for_frames(&self, count: usize, timeout: Duration) -> Result<Vec<StillFrame>, String> {
        let deadline = Instant::now() + timeout;
        let mut frames = self.still_ctx.frames.lock().map_err(|e| e.to_string())?;
        while frames.len() < count {
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "timed out waiting for {} frame(s); got {}",
                    count,
                    frames.len()
                ));
            }
            let (guard, wait) = self
                .still_ctx
                .cv
                .wait_timeout(frames, deadline - now)
                .map_err(|e| e.to_string())?;
            frames = guard;
            if wait.timed_out() && frames.len() < count {
                return Err(format!(
                    "timed out waiting for {} frame(s); got {}",
                    count,
                    frames.len()
                ));
            }
        }
        Ok(frames.drain(..count).collect())
    }

    /// Captures according to the active [`CaptureMode`] and writes the result to `path`.
    ///
    /// JPEG/RAW modes write a single file. HDR captures three YUV frames, fuses
    /// them and writes a JPEG. This method blocks until the file is written.
    pub fn capture_photo(&mut self, path: &str) -> Result<(), String> {
        if !self.running || self.session.is_null() || self.device.is_null() {
            return Err("camera session is not running".into());
        }
        match self.capture_mode {
            CaptureMode::Jpeg12 | CaptureMode::Jpeg50 => self.capture_jpeg(path),
            CaptureMode::Raw12 | CaptureMode::Raw50 => self.capture_raw(path),
            CaptureMode::Hdr3 => self.capture_hdr(path),
        }
    }

    fn capture_jpeg(&mut self, path: &str) -> Result<(), String> {
        self.clear_still_frames();
        let request = unsafe { self.create_still_request()? };
        unsafe {
            Self::apply_jpeg_entries(request);
            let mut requests = [request];
            self.submit_requests(&mut requests)?;
        }
        let frames = self.wait_for_frames(1, CAPTURE_TIMEOUT)?;
        let jpeg = frames[0]
            .planes
            .first()
            .ok_or("JPEG frame has no plane data")?;
        let (width, height) = jpeg_dimensions(jpeg).ok_or("JPEG has no SOF dimensions")?;
        if width as i32 != self.still_width || height as i32 != self.still_height {
            return Err(format!(
                "HAL returned {}x{} JPEG, expected {}x{} (50 MP mode unavailable?)",
                width, height, self.still_width, self.still_height
            ));
        }
        std::fs::write(path, jpeg).map_err(|e| format!("write {}: {}", path, e))?;
        info!(
            "Camera2 NDK: saved JPEG {}x{} ({} bytes) to {}",
            width,
            height,
            jpeg.len(),
            path
        );
        Ok(())
    }

    fn capture_raw(&mut self, path: &str) -> Result<(), String> {
        self.clear_still_frames();
        let request = unsafe { self.create_still_request()? };
        unsafe {
            let mut requests = [request];
            self.submit_requests(&mut requests)?;
        }
        let frames = self.wait_for_frames(1, CAPTURE_TIMEOUT)?;
        let frame = &frames[0];
        let width = frame.width.max(0) as usize;
        let height = frame.height.max(0) as usize;
        if width == 0 || height == 0 {
            return Err("RAW frame has invalid dimensions".into());
        }
        let plane = frame.planes.first().ok_or("RAW frame has no plane data")?;
        let stride = frame.row_strides.first().copied().unwrap_or(0).max(0) as usize;
        if stride < width * 2 {
            return Err(format!(
                "RAW row stride {} is smaller than {} bytes",
                stride,
                width * 2
            ));
        }
        let mut pixels = vec![0u16; width * height];
        for y in 0..height {
            let start = y * stride;
            let end = start + width * 2;
            if end > plane.len() {
                return Err("RAW plane is shorter than expected".into());
            }
            for x in 0..width {
                let i = start + x * 2;
                pixels[y * width + x] = u16::from_le_bytes([plane[i], plane[i + 1]]);
            }
        }
        write_dng_16bit(
            path,
            width as u32,
            height as u32,
            &pixels,
            "OnePlus 13",
            RAW_BLACK_LEVEL,
            RAW_WHITE_LEVEL_10BIT,
        )
        .map_err(|e| format!("write DNG {}: {}", path, e))?;
        info!("Camera2 NDK: saved DNG {}x{} to {}", width, height, path);
        Ok(())
    }

    fn capture_hdr(&mut self, path: &str) -> Result<(), String> {
        self.clear_still_frames();
        let steps = self.capture_mode.bracket_ev_steps();
        let mut requests = Vec::with_capacity(steps.len());

        unsafe {
            for step in steps {
                let request = self.create_still_request()?;
                let ev = *step;
                sys::ACaptureRequest_setEntry_i32(request, TAG_AE_EXPOSURE_COMPENSATION, 1, &ev);
                requests.push(request);
            }
            self.submit_requests(&mut requests)?;
        }

        let frames = self.wait_for_frames(steps.len(), CAPTURE_TIMEOUT)?;
        let mut rgb_frames = Vec::with_capacity(frames.len());
        for frame in &frames {
            if frame.planes.len() < 3 {
                return Err("HDR frame does not contain three YUV planes".into());
            }
            let yuv = YuvFrame {
                width: frame.width.max(0) as usize,
                height: frame.height.max(0) as usize,
                y: frame.planes[0].clone(),
                u: frame.planes[1].clone(),
                v: frame.planes[2].clone(),
                y_stride: frame.row_strides[0].max(0) as usize,
                u_stride: frame.row_strides[1].max(0) as usize,
                v_stride: frame.row_strides[2].max(0) as usize,
                u_pixel_stride: frame.pixel_strides[1].max(1) as usize,
                v_pixel_stride: frame.pixel_strides[2].max(1) as usize,
            };
            rgb_frames.push(yuv420_to_rgb(&yuv).map_err(|e| e.to_string())?);
        }

        let fused = exposure_fusion_aligned(&rgb_frames).map_err(|e| e.to_string())?;
        // The sensor output is landscape; rotate to portrait for the UI.
        let portrait = rotate90_cw(&fused);
        let jpeg = encode_jpeg(&portrait, 92).map_err(|e| e.to_string())?;
        std::fs::write(path, &jpeg).map_err(|e| format!("write {}: {}", path, e))?;
        info!(
            "Camera2 NDK: HDR saved {} frames -> {} bytes to {}",
            rgb_frames.len(),
            jpeg.len(),
            path
        );
        Ok(())
    }

    /// Sets `CONTROL_ZOOM_RATIO` on the repeating preview request.
    pub fn set_zoom(&mut self, ratio: f32) -> Result<(), String> {
        if self.preview_request.is_null() || ratio <= 0.0 {
            return Err("invalid zoom request".into());
        }
        unsafe {
            let status = sys::ACaptureRequest_setEntry_float(
                self.preview_request,
                TAG_ZOOM_RATIO,
                1,
                &ratio,
            );
            if status.0 != 0 {
                return Err(format!("zoom ratio rejected with status {}", status.0));
            }
            let mut seq = 0;
            let repeat_status = sys::ACameraCaptureSession_setRepeatingRequest(
                self.session,
                ptr::null_mut(),
                1,
                &mut self.preview_request,
                &mut seq,
            );
            if repeat_status.0 != 0 {
                return Err(format!(
                    "setRepeatingRequest after zoom failed with status {}",
                    repeat_status.0
                ));
            }
        }
        info!("Camera2 NDK: zoom ratio set to {:.2}", ratio);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn camera_id(&self) -> &str {
        &self.camera_id
    }

    pub fn capture_mode(&self) -> CaptureMode {
        self.capture_mode
    }

    /// Stops repeating, closes the session/device and releases every NDK resource.
    pub fn shutdown(&mut self) {
        unsafe {
            if !self.session.is_null() {
                sys::ACameraCaptureSession_stopRepeating(self.session);
                sys::ACameraCaptureSession_close(self.session);
                self.session = ptr::null_mut();
            }
            if !self.preview_request.is_null() {
                sys::ACaptureRequest_free(self.preview_request);
                self.preview_request = ptr::null_mut();
            }
            if !self.preview_target.is_null() {
                sys::ACameraOutputTarget_free(self.preview_target);
                self.preview_target = ptr::null_mut();
            }
            if !self.still_target.is_null() {
                sys::ACameraOutputTarget_free(self.still_target);
                self.still_target = ptr::null_mut();
            }
            if !self.output_container.is_null() {
                sys::ACaptureSessionOutputContainer_free(self.output_container);
                self.output_container = ptr::null_mut();
            }
            if !self.preview_output.is_null() {
                sys::ACaptureSessionOutput_free(self.preview_output);
                self.preview_output = ptr::null_mut();
            }
            if !self.still_output.is_null() {
                sys::ACaptureSessionOutput_free(self.still_output);
                self.still_output = ptr::null_mut();
            }
            if !self.still_reader.is_null() {
                sys::AImageReader_delete(self.still_reader);
                self.still_reader = ptr::null_mut();
            }
            if !self.device.is_null() {
                sys::ACameraDevice_close(self.device);
                self.device = ptr::null_mut();
            }
            if !self.manager.is_null() {
                sys::ACameraManager_delete(self.manager);
                self.manager = ptr::null_mut();
            }
            if !self.preview_window.is_null() {
                sys::ANativeWindow_release(self.preview_window);
                self.preview_window = ptr::null_mut();
            }
        }
        self.running = false;
        info!("Camera2 NDK: session shut down");
    }
}

impl Drop for CameraEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Initializes Android logcat output for the native core.
pub fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("OpOpenCamNative"),
    );
}
