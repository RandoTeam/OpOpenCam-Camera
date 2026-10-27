package com.opopencam.camera

import android.Manifest
import android.graphics.Matrix
import android.graphics.SurfaceTexture
import android.content.pm.PackageManager
import android.content.Context
import android.content.res.Configuration
import android.os.Environment
import android.os.SystemClock
import android.view.Surface
import android.view.TextureView
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

private val Accent = Color(0xFFFF6A00)

/** Idle time before the camera chrome is replaced by the standby screen. */
private const val STANDBY_TIMEOUT_MS = 30_000L

/** Maximum number of queued stills; the native engine captures them serially. */
private const val MAX_CAPTURE_QUEUE = 3

/**
 * One still waiting for the serial worker.
 *
 * The JPEG orientation is sampled when the shutter is tapped, not when the
 * frame is actually read. A shot can wait behind two HDR merges, and the phone
 * may be turned in that time; the saved file must match the pose of the tap.
 */
private data class CaptureJob(
    val mode: Int,
    val cameraId: Int,
    val jpegOrientation: Int,
    /** Live Photo: keep a short clip and mux it into the still (Motion Photo v1). */
    val live: Boolean = false,
    /** Tap time, used to measure shutter-to-file latency. */
    val tappedAt: Long = android.os.SystemClock.elapsedRealtime(),
)

/** Post-capture clip length for Live Photo v1 (pre-roll ring comes later). */
private const val LIVE_PHOTO_CLIP_MS = 2_500L

/** A still waiting for its Motion Photo clip. */
private data class LiveJob(
    val jpeg: File,
    val uri: android.net.Uri?,
    val cameraId: Int,
    val mode: Int,
    /** Preview surface at tap time; the live worker must not re-read a later one. */
    val surface: Surface?,
    /**
     * Stills still waiting when this job was queued. Must be a snapshot: the
     * global counter still includes the shot that produced this job, so reading
     * it when the worker dequeues drops every clip, including a single shot.
     */
    val waiting: Int,
)

private const val MODE_JPEG12 = 0
private const val MODE_JPEG50 = 1
private const val MODE_RAW12 = 2
private const val MODE_RAW50 = 3
private const val MODE_HDR3 = 4

private fun modeLabel(mode: Int): String = when (mode) {
    MODE_JPEG12 -> "12 MP"
    MODE_JPEG50 -> "50 MP"
    MODE_RAW12 -> "RAW 12"
    MODE_RAW50 -> "RAW 50"
    MODE_HDR3 -> "HDR 3×"
    else -> "12 MP"
}

private fun cameraLabel(cameraId: Int): String = when (cameraId) {
    0 -> "auto lens"
    1 -> "front"
    2 -> "main"
    3 -> "ultra-wide"
    4 -> "3\u00d7 tele"
    else -> "camera $cameraId"
}

/**
 * Full-screen camera UI.
 *
 * The Camera2 session is started only after `surfaceChanged` reports a valid
 * buffer size. Changing capture mode or lens restarts the native session while
 * reusing the same `Surface`; this keeps the HAL stream configuration valid.
 */
@Composable
fun CameraScreen() {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    // Single portrait canvas: the host rotates it with the display, so the
    // layout is never reflowed into a separate landscape arrangement.
    val landscape = false
    val haptics = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()

    var hasCamera by remember {
        mutableStateOf(
            context.checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
        )
    }
    var status by remember { mutableStateOf("Starting camera…") }
    var capturing by remember { mutableStateOf(false) }
    // `queuedCaptures` counts every accepted shot that has not finished yet,
    // including the one the worker is processing. Showing the raw count while
    // a shot is in flight would say "1 queued" with nothing waiting, so the
    // status line subtracts the in-flight job.
    var queuedCaptures by remember { mutableIntStateOf(0) }
    var processingStatus by remember { mutableStateOf("") }
    val captureQueue = remember { Channel<CaptureJob>(capacity = MAX_CAPTURE_QUEUE) }
    // Live clips are muxed by a separate worker so a 2.5 s recording never keeps
    // the shutter, the still queue or the preview waiting.
    val liveQueue = remember { Channel<LiveJob>(capacity = 4) }
    var zoom by remember { mutableStateOf(1f) }
    var sensorZoom by remember { mutableStateOf(1f) }
    var cameraId by remember { mutableStateOf(0) }
    var captureMode by remember { mutableStateOf(MODE_JPEG12) }
    var activeMode by remember { mutableStateOf(-1) }
    var currentSurface by remember { mutableStateOf<Surface?>(null) }
    var videoMode by remember { mutableStateOf(false) }
    var video4k16 by remember { mutableStateOf(false) }
    var recording by remember { mutableStateOf(false) }
    var recordingSeconds by remember { mutableStateOf(0) }
    var videoBusy by remember { mutableStateOf(false) }
    var switching by remember { mutableStateOf(false) }
    var showGallery by remember { mutableStateOf(false) }
    var showFormatSheet by remember { mutableStateOf(false) }
    var showSettings by remember { mutableStateOf(false) }
    val prefs = remember { context.getSharedPreferences("opopencam", Context.MODE_PRIVATE) }
    var standbyEnabled by remember { mutableStateOf(prefs.getBoolean("standby_enabled", true)) }
    var livePhotoEnabled by remember {
        mutableStateOf(prefs.getBoolean("live_photo_enabled", false))
    }
    var standby by remember { mutableStateOf(false) }
    var lastInteraction by remember { mutableStateOf(0L) }
    var resumed by remember {
        mutableStateOf(lifecycleOwner.lifecycle.currentState.isAtLeast(Lifecycle.State.RESUMED))
    }
    var galleryItems by remember { mutableStateOf<List<GalleryItem>>(emptyList()) }
    var recorder by remember { mutableStateOf<VideoRecorder?>(null) }
    var audioGranted by remember {
        mutableStateOf(
            context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
                PackageManager.PERMISSION_GRANTED
        )
    }

    val cameraIdState = rememberUpdatedState(cameraId)
    val modeState = rememberUpdatedState(captureMode)

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_RESUME -> {
                    resumed = true
                    lastInteraction++
                }
                Lifecycle.Event.ON_PAUSE,
                Lifecycle.Event.ON_STOP -> {
                    resumed = false
                    standby = false
                    showFormatSheet = false
                }
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    // UI-only standby: hide the chrome, keep the SurfaceView and session alive.
    // Like the stock screen, a black idle screen also saves panel burn-in,
    // battery and heat.
    LaunchedEffect(
        lastInteraction,
        capturing,
        queuedCaptures,
        recording,
        videoBusy,
        switching,
        showGallery,
        showFormatSheet,
        resumed,
        standbyEnabled,
    ) {
        if (!standbyEnabled || !resumed || capturing || queuedCaptures > 0 || recording || videoBusy || switching || showGallery || showFormatSheet) {
            standby = false
            return@LaunchedEffect
        }
        delay(STANDBY_TIMEOUT_MS)
        standby = true
    }

    // Coalesce slider movement into ~50 Hz native zoom requests; never call JNI
    // from the main thread. `sensorZoom` is the ratio sent to the active sensor;
    // `zoom` is the equivalent camera-app stop shown in the UI.
    LaunchedEffect(sensorZoom) {
        delay(20)
        withContext(Dispatchers.IO) { NativeBridge.nativeSetZoom(sensorZoom) }
    }
    LaunchedEffect(recording) {
        if (!recording) {
            recordingSeconds = 0
            return@LaunchedEffect
        }
        val started = System.currentTimeMillis()
        while (true) {
            recordingSeconds = ((System.currentTimeMillis() - started) / 1000).toInt()
            delay(500)
        }
    }

    LaunchedEffect(Unit) {
        galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
    }

    val permissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        hasCamera = granted
        if (!granted) status = "Camera permission denied"
    }

    val audioPermissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        audioGranted = granted
        status = if (granted) "Microphone ready" else "Microphone permission denied"
    }

    LaunchedEffect(Unit) {
        if (!hasCamera) permissionLauncher.launch(Manifest.permission.CAMERA)
    }

    LaunchedEffect(Unit) {
        galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
    }

    val textureView = remember { TextureView(context) }


    fun describeStatus(requestedCamera: Int, requestedMode: Int, ok: Boolean) {
        status = when {
            !ok -> "Camera start failed — see logcat"
            activeMode >= 0 && activeMode != requestedMode -> "HAL fallback: ${modeLabel(activeMode)}"
            else -> "${cameraLabel(requestedCamera)} · ${modeLabel(requestedMode)}"
        }
    }

    fun restart(
        requestedCamera: Int,
        requestedMode: Int,
        requestedSensorZoom: Float = 1f,
        requestedDisplayZoom: Float = 1f,
    ) {
        if (queuedCaptures > 0) {
            status = "Wait for processing"
            return
        }
        if (recording || videoBusy) {
            status = "Stop recording first"
            return
        }
        if (switching) return
        cameraId = requestedCamera
        captureMode = requestedMode
        zoom = requestedDisplayZoom
        sensorZoom = requestedSensorZoom
        val surface = currentSurface
        if (surface == null) {
            status = "Waiting for preview surface\u2026"
            return
        }
        switching = true
        status = "Switching camera\u2026"
        scope.launch {
            val ok = withContext(Dispatchers.IO) {
                NativeBridge.nativeStopPreview()
                NativeBridge.nativeStartPreview(surface, requestedCamera, requestedMode)
                    .also { started ->
                        if (started) NativeBridge.nativeSetZoom(requestedSensorZoom)
                    }
            }
            if (currentSurface !== surface) {
                switching = false
                return@launch
            }
            if (!ok && requestedCamera != 0) {
                // The selected physical lens may not support this mode. Fall
                // back to the logical camera instead of leaving a black preview.
                val fallback = withContext(Dispatchers.IO) {
                    NativeBridge.nativeStopPreview()
                    NativeBridge.nativeStartPreview(surface, 0, requestedMode)
                        .also { started ->
                            if (started) NativeBridge.nativeSetZoom(requestedDisplayZoom)
                        }
                }
                if (fallback) {
                    cameraId = 0
                    sensorZoom = requestedDisplayZoom
                    status = "Lens unavailable \u2014 using auto lens"
                } else {
                    status = "Camera start failed \u2014 see logcat"
                }
            }
            activeMode = NativeBridge.nativeGetActiveMode()
            if (ok) describeStatus(requestedCamera, requestedMode, true)
            switching = false
        }
    }

    fun setZoomPreset(value: Float) {
        if (queuedCaptures > 0) {
            status = "Wait for processing \u00b7 ${queuedCaptures} pending"
            return
        }
        if (videoBusy || switching) return
        haptics.performHapticFeedback(HapticFeedbackType.LongPress)
        // Do not copy the stock low-light trick of keeping 3x/6x on the main
        // sensor. These presets select the real telephoto for 3x/6x and the
        // ultra-wide for 0.6x; the sensor ratio is only the crop on that lens.
        val (lens, sensorRatio) = when (value) {
            0.6f -> 3 to 1f
            1f -> 0 to 1f
            2f -> 0 to 2f
            3f -> 4 to 1f
            6f -> 4 to 2f
            else -> 0 to value
        }
        if (lens != cameraId) {
            restart(lens, captureMode, sensorRatio, value)
        } else {
            cameraId = lens
            zoom = value
            sensorZoom = sensorRatio
            status = "%.1f\u00d7 \u00b7 %s".format(value, cameraLabel(lens))
        }
    }

    fun beginVideo() {
        if (queuedCaptures > 0) {
            status = "Wait for processing"
            return
        }
        if (recording || videoBusy) return
        if (!audioGranted) {
            audioPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
            return
        }
        val surface = currentSurface
        if (surface == null) {
            status = "Waiting for preview surface…"
            return
        }
        val dir = context.getExternalFilesDir(Environment.DIRECTORY_MOVIES) ?: context.filesDir
        dir.mkdirs()
        val file = File(dir, "VID_${System.currentTimeMillis()}.mp4")
        val config = if (video4k16) {
            VideoRecorder.Config(width = 3840, height = 2160, fps = 60)
        } else {
            VideoRecorder.Config()
        }
        videoBusy = true
        status = "Starting video…"
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    val rec = VideoRecorder(context)
                    rec.start(surface, cameraId, file, config)
                    rec
                }
            }
            result.onSuccess {
                recorder = it
                recording = true
                videoBusy = false
                status = "Recording ${file.name}"
            }.onFailure {
                recording = false
                videoBusy = false
                status = "Video failed: ${it.message ?: it.javaClass.simpleName}"
            }
        }
    }

    fun endVideo() {
        if (!recording || videoBusy) return
        val rec = recorder
        videoBusy = true
        status = "Saving video…"
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching { rec?.stop() ?: error("video recorder is not active") }
            }
            recorder = null
            recording = false
            videoBusy = false
            val surface = currentSurface
            if (surface != null) {
                withContext(Dispatchers.IO) {
                    NativeBridge.nativeStartPreview(surface, cameraId, captureMode)
                }
            }
            val savedFile = result.getOrNull()
            val published = if (savedFile != null) {
                withContext(Dispatchers.IO) { MediaStoreSaver.publishVideo(context, savedFile) }
            } else {
                null
            }
            result.onSuccess {
                val suffix = if (published != null) " · Gallery" else " · not indexed"
                galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
                status = "Saved ${it.name}$suffix"
                Toast.makeText(context, "Saved ${it.name}$suffix", Toast.LENGTH_SHORT).show()
            }.onFailure {
                status = "Video stop failed: ${it.message ?: it.javaClass.simpleName}"
            }
        }
    }

    /**
     * Records a [LIVE_PHOTO_CLIP_MS] clip and muxes it into the still as a Google
     * Motion Photo v1 (JPEG + appended MP4 + XMP). Blocking: runs on the capture
     * worker, never on the main thread.
     *
     * Returns (short note, file to publish) or null to fall back to the plain
     * still. The camera session is returned to still preview in every path.
     */
    fun finishLivePhoto(
        jpegFile: File,
        job: CaptureJob,
        previewSurface: Surface? = currentSurface,
    ): Pair<String, File>? {
        val surface = previewSurface ?: return null
        val mp4 = File(jpegFile.parentFile, "MOTION_${System.currentTimeMillis()}.mp4")
        val out = File(jpegFile.parentFile, jpegFile.nameWithoutExtension + "MP.jpg")
        val recorder = VideoRecorder(context)
        return try {
            recorder.start(
                surface,
                job.cameraId,
                mp4,
                VideoRecorder.Config(
                    width = 1920,
                    height = 1440,
                    fps = 30,
                    videoBitrate = 20_000_000,
                ),
            )
            Thread.sleep(LIVE_PHOTO_CLIP_MS)
            recorder.stop()
            val assembled = NativeBridge.nativeAssembleMotionPhoto(
                jpegFile.absolutePath,
                mp4.absolutePath,
                out.absolutePath,
                0L,
            )
            if (assembled && out.isFile && out.length() > jpegFile.length()) {
                "live ${out.length() / 1024} KB" to out
            } else {
                null
            }
        } catch (t: Throwable) {
            android.util.Log.w("OpOpenCamLive", "live photo failed", t)
            null
        } finally {
            android.util.Log.i(
                "OpOpenCamLive",
                "clip=${if (mp4.isFile) mp4.length() else -1} out=${if (out.isFile) out.length() else -1} jpeg=${jpegFile.length()}",
            )
            mp4.delete()
            // Video recording restarts the session; put the still preview back.
            runCatching { NativeBridge.nativeStartPreview(surface, job.cameraId, job.mode) }
        }
    }

    fun processCapture(job: CaptureJob): String {
        val dir = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
            ?: context.filesDir
        dir.mkdirs()
        val effectiveMode = NativeBridge.nativeGetActiveMode()
            .let { if (it >= 0) it else job.mode }
        val extension = if (effectiveMode == MODE_RAW12 || effectiveMode == MODE_RAW50) {
            "dng"
        } else {
            "jpg"
        }
        val file = File(dir, "IMG_${System.currentTimeMillis()}.$extension")
        // Orientation was sampled at tap time; see CaptureJob.
        NativeBridge.nativeSetJpegOrientation(job.jpegOrientation)
        val ok = NativeBridge.nativeCapturePhoto(file.absolutePath)
        return when {
            ok -> {
                val uri = MediaStoreSaver.publishImage(context, file)
                // The still is published immediately; the clip is assembled later
                // by the separate worker, so the queue slot is released at once.
                val waiting = (queuedCaptures - 1).coerceAtLeast(0)
                val queued = job.live && extension == "jpg" &&
                    liveQueue.trySend(
                        LiveJob(file, uri, job.cameraId, job.mode, currentSurface, waiting)
                    ).isSuccess
                when {
                    queued -> "Saved ${file.name} \u00b7 live queued"
                    uri != null -> "Saved ${file.name} \u00b7 Gallery"
                    else -> "Saved ${file.name} \u00b7 not indexed"
                }
            }
            effectiveMode == MODE_JPEG50 -> "50 MP JPEG unavailable \u2014 use RAW 50"
            else -> "Capture failed - see logcat"
        }
    }

    /** Waiting shots, i.e. everything except the one currently being processed. */
    fun waitingCaptures(): Int = (queuedCaptures - if (capturing) 1 else 0).coerceAtLeast(0)

    fun queuedSuffix(waiting: Int): String =
        if (waiting > 0) " \u00b7 $waiting queued" else ""

    fun pendingSuffix(waiting: Int): String =
        if (waiting > 0) " \u00b7 $waiting pending" else ""

    /** Visible feedback instead of a silent no-op while the queue drains. */
    fun busyMessage(): String =
        if (capturing && processingStatus.isNotEmpty()) {
            processingStatus + queuedSuffix(waitingCaptures())
        } else {
            "Wait for processing \u00b7 ${queuedCaptures} pending"
        }

    fun capturePhoto() {
        if (switching || videoBusy) return
        if (queuedCaptures >= MAX_CAPTURE_QUEUE) {
            status = "Processing queue full \u00b7 ${queuedCaptures} pending"
            return
        }
        // Sampling the display rotation here keeps the saved still upright even
        // if the phone is turned while earlier shots are still being processed.
        val displayRotation = (context.display?.rotation ?: Surface.ROTATION_0) * 90
        val sensorOrientation = if (cameraId == 1) 270 else 90
        val jpegOrientation = ((sensorOrientation - displayRotation) % 360 + 360) % 360
        val job = CaptureJob(captureMode, cameraId, jpegOrientation, live = livePhotoEnabled)
        if (captureQueue.trySend(job).isSuccess) {
            queuedCaptures++
            // Never replace a "Processing HDR..." line with the queue text; the
            // per-shot progress is the more useful state while a shot is busy.
            status = if (capturing && processingStatus.isNotEmpty()) {
                processingStatus + queuedSuffix(waitingCaptures())
            } else {
                "Queued ${modeLabel(job.mode)}" + pendingSuffix(waitingCaptures())
            }
        } else {
            status = "Processing queue full"
        }
    }

    fun onShutter() {
        haptics.performHapticFeedback(HapticFeedbackType.LongPress)
        if (videoMode) {
            if (recording) endVideo() else beginVideo()
        } else {
            capturePhoto()
        }
    }

    // Single serial worker: HDR fusion and 50 MP RAW are heavy and the native
    // engine is not re-entrant, so exactly one shot is captured at a time. It
    // runs on the composition scope (main) and only the blocking JNI call moves
    // to Dispatchers.IO, which keeps `queuedCaptures` free of data races.
    LaunchedEffect(captureQueue) {
        for (job in captureQueue) {
            capturing = true
            processingStatus = when (job.mode) {
                MODE_HDR3 -> "Processing HDR\u2026"
                MODE_RAW50 -> "Saving RAW 50\u2026"
                else -> "Capturing\u2026"
            }
            // This job is now in flight, so the remaining count is the queue.
            status = processingStatus + queuedSuffix(waitingCaptures())
            var result = "Capture failed"
            try {
                result = try {
                    withContext(Dispatchers.IO) { processCapture(job) }
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (t: Throwable) {
                    "Capture failed: ${t.message ?: t.javaClass.simpleName}"
                }
                try {
                    galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
                } catch (e: kotlinx.coroutines.CancellationException) {
                    throw e
                } catch (t: Throwable) {
                    result += " \u00b7 gallery refresh failed"
                }
                android.util.Log.i(
                    "OpOpenCamPerf",
                    "still mode=${job.mode} live=${job.live} " +
                        "in ${SystemClock.elapsedRealtime() - job.tappedAt} ms",
                )
            } finally {
                // Always release the queue slot, even if gallery refresh or a
                // future post-processing step throws. Otherwise the shutter and
                // every mode control stay blocked until the app is killed.
                queuedCaptures = (queuedCaptures - 1).coerceAtLeast(0)
                capturing = false
                processingStatus = ""
                // Keep the per-shot result; only append the count of shots that
                // are still waiting behind it.
                status = result + queuedSuffix(queuedCaptures)
            }
            Toast.makeText(context, result, Toast.LENGTH_SHORT).show()
        }
    }

    // Live Photo assembler: one clip at a time, entirely off the shutter path.
    LaunchedEffect(liveQueue) {
        for (live in liveQueue) {
            // Recording restarts the camera session, so skip the clip while more
            // stills are waiting: a burst must never fail because of Live Photo.
            // The plain still has already been published and stays valid.
            if (live.waiting > 0) {
                android.util.Log.i(
                    "OpOpenCamLive",
                    "skipped: ${live.waiting} stills were waiting at tap time",
                )
                continue
            }
            val started = SystemClock.elapsedRealtime()
            val assembled = withContext(Dispatchers.IO) {
                finishLivePhoto(
                    live.jpeg,
                    CaptureJob(live.mode, live.cameraId, 0),
                    live.surface,
                )
            } ?: continue
            val uri = withContext(Dispatchers.IO) {
                MediaStoreSaver.publishImage(context, assembled.second)
            }
            if (uri != null) {
                withContext(Dispatchers.IO) {
                    live.uri?.let { runCatching { context.contentResolver.delete(it, null, null) } }
                    live.jpeg.delete()
                }
                galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
            }
            android.util.Log.i(
                "OpOpenCamLive",
                "assembled in ${SystemClock.elapsedRealtime() - started} ms (${assembled.first})",
            )
        }
    }

    DisposableEffect(textureView, hasCamera) {
        if (!hasCamera) {
            return@DisposableEffect onDispose { }
        }

        var previewSurface: Surface? = null
        var started = false
        val startIfReady = {
            if (!started && textureView.isAvailable) {
                val surfaceTexture = textureView.surfaceTexture
                if (surfaceTexture != null) {
                    surfaceTexture.setDefaultBufferSize(1440, 1080)
                    val surface = Surface(surfaceTexture)
                    previewSurface = surface
                    currentSurface = surface
                    val requestedCamera = cameraIdState.value
                    val requestedMode = modeState.value
                    val ok = NativeBridge.nativeStartPreview(surface, requestedCamera, requestedMode)
                    activeMode = NativeBridge.nativeGetActiveMode()
                    describeStatus(requestedCamera, requestedMode, ok)
                    started = ok
                    if (ok) {
                        textureView.post {
                            configurePreviewTransform(textureView)
                        }
                    } else {
                        currentSurface = null
                        previewSurface = null
                        surface.release()
                    }
                }
            }
        }

        val listener = object : TextureView.SurfaceTextureListener {
            override fun onSurfaceTextureAvailable(st: SurfaceTexture, width: Int, height: Int) {
                startIfReady()
            }

            override fun onSurfaceTextureSizeChanged(st: SurfaceTexture, width: Int, height: Int) {
                configurePreviewTransform(textureView)
            }

            override fun onSurfaceTextureDestroyed(st: SurfaceTexture): Boolean {
                started = false
                if (recording) {
                    recorder?.abort()
                    recorder = null
                    recording = false
                    videoBusy = false
                }
                currentSurface = null
                NativeBridge.nativeStopPreview()
                previewSurface?.release()
                previewSurface = null
                return false
            }

            override fun onSurfaceTextureUpdated(st: SurfaceTexture) = Unit
        }

        textureView.surfaceTextureListener = listener
        if (textureView.isAvailable) startIfReady()

        // The stock camera takes the camera while it is in front. Stop our preview
        // when we leave and retry the open when we return, instead of keeping a
        // stale device that fails on the next capture.
        var retries = 0
        val retryStart = object : Runnable {
            override fun run() {
                if (started || !textureView.isAvailable) return
                startIfReady()
                if (!started && retries++ < 10) textureView.postDelayed(this, 400)
            }
        }
        val lifecycleObserver = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_RESUME -> {
                    retries = 0
                    textureView.postDelayed(retryStart, 300)
                }
                Lifecycle.Event.ON_STOP -> {
                    started = false
                    currentSurface = null
                    NativeBridge.nativeStopPreview()
                    previewSurface?.release()
                    previewSurface = null
                }
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(lifecycleObserver)

        onDispose {
            lifecycleOwner.lifecycle.removeObserver(lifecycleObserver)
            textureView.surfaceTextureListener = null
            if (recording) {
                recorder?.abort()
                recorder = null
                recording = false
                videoBusy = false
            }
            currentSurface = null
            NativeBridge.nativeStopPreview()
            previewSurface?.release()
            previewSurface = null
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            // Observe pointer downs in the initial pass without consuming them:
            // every control keeps working and any touch restarts the idle timer.
            .pointerInput(Unit) {
                awaitPointerEventScope {
                    while (true) {
                        val event = awaitPointerEvent(PointerEventPass.Initial)
                        if (event.type == PointerEventType.Press) {
                            lastInteraction++
                        }
                    }
                }
            }
    ) {
        Box(
            modifier = (if (landscape) {
                // Stock landscape: a 1440x1080 (4:3) preview centred between the
                // control rail and the settings rail, never full-bleed.
                Modifier
                    .aspectRatio(4f / 3f)
                    .fillMaxHeight()
                    .align(Alignment.Center)
            } else {
                // Stock anchor: the photo preview starts at canvas y=300px and
                // is 1080x1440 (3:4). 300px = 100dp on this 480dpi canvas.
                // Stock preview: photo 1080x1440 (3:4), video 1080x1920 (9:16),
                // both starting at canvas y=300px.
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(if (videoMode) 9f / 16f else 3f / 4f)
                    .align(Alignment.TopCenter)
                    .offset(y = canvasPx(300f))
            }).pointerInput(cameraId) {
                    detectTransformGestures { _, _, zoomChange, _ ->
                        if (queuedCaptures == 0 && !videoBusy && !switching && !recording) {
                            // Keep the pinch within the active lens; use the
                            // pills to cross lens boundaries. This avoids a
                            // silent logical-camera switch during a gesture.
                            val (displayFactor, minSensor, maxSensor) = when (cameraId) {
                                3 -> Triple(0.6f, 1f, 1.667f)
                                4 -> Triple(3f, 1f, 2f)
                                1 -> Triple(1f, 1f, 2f)
                                else -> Triple(1f, 0.6f, 6f)
                            }
                            val newSensor = (sensorZoom * zoomChange).coerceIn(minSensor, maxSensor)
                            sensorZoom = newSensor
                            zoom = displayFactor * newSensor
                        }
                    }
                }
        ) {
            AndroidView(
                factory = { textureView },
                modifier = Modifier
                    .fillMaxSize()
                    .onSizeChanged {
                        configurePreviewTransform(textureView)
                    }
            )
            Canvas(Modifier.matchParentSize()) {
                val gridColor = Color.White.copy(alpha = 0.07f)
                val stroke = 1.dp.toPx()
                for (i in 1..2) {
                    val x = size.width * i / 3f
                    val y = size.height * i / 3f
                    drawLine(gridColor, Offset(x, 0f), Offset(x, size.height), stroke)
                    drawLine(gridColor, Offset(0f, y), Offset(size.width, y), stroke)
                }
            }
        }

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(
                    Brush.verticalGradient(
                        listOf(Color.Black.copy(alpha = 0.55f), Color.Transparent)
                    )
                )
                .statusBarsPadding()
                .padding(horizontal = 20.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
            Column(Modifier.upright()) {
                Text(
                    text = "OpOpenCam",
                    color = Color.White,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 15.sp
                )
                Text(
                    text = if (videoMode) {
                        "VIDEO"
                    } else {
                        "${modeLabel(captureMode)} \u00b7 ${cameraLabel(cameraId)} \u00b7 " +
                            "${"%.1f".format(zoom)}\u00d7" + if (livePhotoEnabled) " \u00b7 LIVE" else ""
                    },
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 10.sp
                )
                SettingsGlyph { showSettings = true }
            }
            }
            if (capturing || queuedCaptures > 0 || switching || videoBusy || recording ||
                status.contains("failed", ignoreCase = true) ||
                status.contains("denied", ignoreCase = true) ||
                status.contains("unavailable", ignoreCase = true)
            ) {
                Text(
                    text = status,
                    color = if (recording) Color(0xFFFF5252) else Color.White.copy(alpha = 0.72f),
                    fontSize = 10.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.width(150.dp).upright()
                )
            } else if (!videoMode) {
                FormatBadge(modeLabel(captureMode)) {
                    if (queuedCaptures == 0 && !switching && !videoBusy) {
                        showFormatSheet = true
                        lastInteraction++
                    } else if (queuedCaptures > 0) {
                        status = busyMessage()
                    }
                }
            }
        }

        // Replaced by the stock-anchored chrome below (UI-303). Kept compiled out
        // for one release so the old flow layout cannot render by accident; the
        // block is removed in the cleanup commit.
        if (false) {
        Column(
            modifier = if (landscape) {
                Modifier
                    .align(Alignment.CenterEnd)
                    .width(280.dp)
                    .fillMaxHeight()
                    .background(
                        Brush.horizontalGradient(
                            listOf(
                                Color.Transparent,
                                Color.Black.copy(alpha = 0.72f),
                                Color.Black.copy(alpha = 0.94f)
                            )
                        )
                    )
                    .navigationBarsPadding()
                    .padding(start = 10.dp, end = 12.dp, top = 84.dp, bottom = 12.dp)
            } else {
                Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .background(
                        Brush.verticalGradient(
                            listOf(
                                Color.Black.copy(alpha = 0.20f),
                                Color.Black.copy(alpha = 0.78f),
                                Color.Black.copy(alpha = 0.96f)
                            )
                        )
                    )
                    .navigationBarsPadding()
                    .padding(start = 16.dp, end = 16.dp, top = 34.dp, bottom = 14.dp)
            },
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            ModeSegment(videoMode) { isVideo ->
                if (queuedCaptures == 0 && !recording && !videoBusy) {
                    videoMode = isVideo
                    status = if (isVideo) {
                        if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                    } else {
                        "Photo mode"
                    }
                } else if (queuedCaptures > 0) {
                    status = busyMessage()
                }
            }

            Spacer(Modifier.height(10.dp))

            if (videoMode) {
                SettingChip(
                    if (video4k16) "4K 16:9 \u00b7 60" else "4K 4:3 \u00b7 30",
                    true,
                ) {
                    if (queuedCaptures == 0 && !recording && !videoBusy) {
                        video4k16 = !video4k16
                        status = if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                    }
                }
                Spacer(Modifier.height(10.dp))
            }

            Row(
                modifier = Modifier.horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(7.dp)
            ) {
                SettingChip("0.6", zoom == 0.6f) { setZoomPreset(0.6f) }
                SettingChip("1\u00d7", zoom == 1f) { setZoomPreset(1f) }
                SettingChip("2\u00d7", zoom == 2f) { setZoomPreset(2f) }
                SettingChip("3\u00d7", zoom == 3f) { setZoomPreset(3f) }
                SettingChip("6\u00d7", zoom == 6f) { setZoomPreset(6f) }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                LatestMediaThumbnail(context, galleryItems.firstOrNull()) {
                    showGallery = true
                    lastInteraction++
                }

                ShutterButton(
                    capturing = capturing,
                    recording = recording,
                    videoBusy = videoBusy,
                    videoMode = videoMode,
                    enabled = if (videoMode) !videoBusy else queuedCaptures < MAX_CAPTURE_QUEUE && !switching,
                    onClick = { onShutter() }
                )

                FlipCameraButton(
                    enabled = queuedCaptures == 0 && !recording && !videoBusy && !switching
                ) {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    if (cameraId == 1) restart(0, captureMode) else restart(1, MODE_JPEG12)
                }
            }

            if (videoMode) {
                Spacer(Modifier.height(8.dp))
                Text(
                    text = run {
                        val spec = if (video4k16) "4K 16:9 \u00b7 60 fps" else "4K 4:3 \u00b7 30 fps"
                        if (recording) "REC ${formatDuration(recordingSeconds)} \u00b7 $spec \u00b7 tap to stop"
                        else "VIDEO \u00b7 $spec \u00b7 ${cameraLabel(cameraId)} \u00b7 tap to record"
                    },
                    color = if (recording) Color(0xFFFF5252) else Color.White.copy(alpha = 0.55f),
                    fontSize = 10.sp
                )
            }
        }

        }

        // Stock live-photo cell: canvas x=375..531, glyph centre (453,162).
        Box(
            modifier = Modifier
                .align(Alignment.TopStart)
                .offset(x = canvasPx(375f))
                .size(canvasPx(156f), canvasPx(240f))
                .clickable {
                    if (queuedCaptures == 0 && !recording && !videoBusy) {
                        livePhotoEnabled = !livePhotoEnabled
                        prefs.edit().putBoolean("live_photo_enabled", livePhotoEnabled).apply()
                        haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                        status = if (livePhotoEnabled) "Live Photo on" else "Live Photo off"
                        lastInteraction++
                    } else if (queuedCaptures > 0) {
                        status = busyMessage()
                    }
                },
            contentAlignment = Alignment.Center,
        ) {
            LivePhotoGlyph(
                enabled = livePhotoEnabled,
                modifier = Modifier.size(canvasPx(60f)),
            )
        }

        // ---- Stock-anchored chrome (canvas 1080x2376 px, 1dp = 3px) ----
        StockModeRail(
            videoMode = videoMode,
            enabled = queuedCaptures == 0 && !recording && !videoBusy,
            modifier = Modifier
                .align(Alignment.TopCenter)
                // Pill top 1789: centre 1830, matching the stock text line.
                .offset(y = canvasPx(1789f)),
        ) { isVideo ->
            if (queuedCaptures == 0 && !recording && !videoBusy) {
                videoMode = isVideo
                status = if (isVideo) {
                    if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                } else {
                    "Photo mode"
                }
            } else if (queuedCaptures > 0) {
                status = busyMessage()
            }
        }

        StockZoomRow(
            zoom = zoom,
            enabled = queuedCaptures == 0 && !videoBusy && !switching && !recording,
            modifier = Modifier
                .align(Alignment.TopCenter)
                // Slot height 108 px (36 dp); top 1597 puts the centres on 1651.
                .offset(y = canvasPx(1597f)),
        ) { setZoomPreset(it) }

        // Capture deck: stock centres (180,2061), (540,2061), (900,2061).
        Box(Modifier.fillMaxSize()) {
            Box(
                Modifier
                    .align(Alignment.TopStart)
                    .offset(x = canvasPx(108f), y = canvasPx(1989f))
            ) {
                LatestMediaThumbnail(context, galleryItems.firstOrNull()) {
                    showGallery = true
                    lastInteraction++
                }
            }
            Box(
                Modifier
                    .align(Alignment.TopCenter)
                    .offset(y = canvasPx(1944f))
            ) {
                ShutterButton(
                    capturing = capturing,
                    recording = recording,
                    videoBusy = videoBusy,
                    videoMode = videoMode,
                    enabled = if (videoMode) !videoBusy else queuedCaptures < MAX_CAPTURE_QUEUE && !switching,
                    onClick = { onShutter() }
                )
            }
            Box(
                Modifier
                    .align(Alignment.TopEnd)
                    .offset(x = -canvasPx(108f), y = canvasPx(1989f))
            ) {
                FlipCameraButton(
                    enabled = queuedCaptures == 0 && !recording && !videoBusy && !switching
                ) {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    if (cameraId == 1) restart(0, captureMode) else restart(1, MODE_JPEG12)
                }
            }
        }

        if (landscape) {
            // Stock-like landscape rails: capture controls on the left,
            // zoom stops on the right, mode switch at the bottom.
            Column(
                modifier = Modifier
                    .align(Alignment.CenterStart)
                    .padding(start = 14.dp)
                    .width(96.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(14.dp)
            ) {
                LatestMediaThumbnail(context, galleryItems.firstOrNull()) {
                    showGallery = true
                    lastInteraction++
                }
                ShutterButton(
                    capturing = capturing,
                    recording = recording,
                    videoBusy = videoBusy,
                    videoMode = videoMode,
                    enabled = if (videoMode) !videoBusy else queuedCaptures < MAX_CAPTURE_QUEUE && !switching,
                    onClick = { onShutter() }
                )
                FlipCameraButton(
                    enabled = queuedCaptures == 0 && !recording && !videoBusy && !switching
                ) {
                    haptics.performHapticFeedback(HapticFeedbackType.LongPress)
                    if (cameraId == 1) restart(0, captureMode) else restart(1, MODE_JPEG12)
                }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.CenterEnd)
                    .padding(end = 12.dp, top = 64.dp, bottom = 18.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                SettingChip("0.6", zoom == 0.6f, compact = true) { setZoomPreset(0.6f) }
                SettingChip("1\u00d7", zoom == 1f, compact = true) { setZoomPreset(1f) }
                SettingChip("2\u00d7", zoom == 2f, compact = true) { setZoomPreset(2f) }
                SettingChip("3\u00d7", zoom == 3f, compact = true) { setZoomPreset(3f) }
                SettingChip("6\u00d7", zoom == 6f, compact = true) { setZoomPreset(6f) }
            }

            Column(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .navigationBarsPadding()
                    .padding(bottom = 12.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                ModeSegment(videoMode) { isVideo ->
                    // Same guard as the portrait fallback: a PHOTO/VIDEO switch
                    // during processing would restart the session and overwrite
                    // the per-shot status, so the queue must be empty first.
                    if (queuedCaptures == 0 && !recording && !videoBusy) {
                        videoMode = isVideo
                        status = if (isVideo) {
                            if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                        } else {
                            "Photo mode"
                        }
                    } else if (queuedCaptures > 0) {
                        status = busyMessage()
                    }
                }
                if (videoMode) {
                    Spacer(Modifier.height(6.dp))
                    Text(
                        text = if (recording) {
                            val spec = if (video4k16) "4K 16:9 \u00b7 60 fps" else "4K 4:3 \u00b7 30 fps"
                            "REC ${formatDuration(recordingSeconds)} \u00b7 $spec"
                        } else {
                            if (video4k16) "4K 16:9 \u00b7 60 fps" else "4K 4:3 \u00b7 30 fps"
                        },
                        color = if (recording) Color(0xFFFF5252) else Color.White.copy(alpha = 0.55f),
                        fontSize = 10.sp
                    )
                }
            }
        }

        if (showSettings) {
            AlertDialog(
                onDismissRequest = { showSettings = false },
                containerColor = Color(0xFF1C1C1C),
                titleContentColor = Color.White,
                textContentColor = Color.White,
                title = { Text("Настройки") },
                text = {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text("Экран ожидания", fontSize = 15.sp)
                            Text(
                                "Чёрный экран «Tap for preview» после паузы",
                                fontSize = 11.sp,
                                color = Color.White.copy(alpha = 0.6f)
                            )
                        }
                        Switch(
                            checked = standbyEnabled,
                            onCheckedChange = {
                                standbyEnabled = it
                                prefs.edit().putBoolean("standby_enabled", it).apply()
                            }
                        )
                    }
                },
                confirmButton = {
                    TextButton(onClick = { showSettings = false }) { Text("Готово") }
                }
            )
        }

        if (showFormatSheet && !showGallery) {
            FormatSheet(
                current = captureMode,
                onSelect = { mode ->
                    showFormatSheet = false
                    // Keep the current lens/zoom if it supports the new format;
                    // restart() falls back to the logical camera if it does not.
                    restart(cameraId, mode, sensorZoom, zoom)
                },
                onDismiss = { showFormatSheet = false }
            )
        }

        if (showGallery) {
            GalleryOverlay(context) { showGallery = false }
        }

        if (standby && !showGallery && !showFormatSheet && !capturing && queuedCaptures == 0 && !recording && !videoBusy && !switching && resumed) {
            StandbyOverlay {
                standby = false
                lastInteraction++
            }
        }
    }
}

/** Converts a stock canvas pixel (1080x2376 reference) to dp on the 480 dpi host. */
private fun canvasPx(px: Float) = (px / 3f).dp

/**
 * Stock zoom row: bare labels, not pills. Slot centres at canvas x
 * 324/432/540/648/756 and centre line y=1651 (the caller offsets the row).
 */
@Composable
private fun StockZoomRow(
    zoom: Float,
    enabled: Boolean,
    modifier: Modifier = Modifier,
    onSelect: (Float) -> Unit,
) {
    val stops = listOf(0.6f to "0.6", 1f to "1\u00d7", 2f to "2\u00d7", 3f to "3\u00d7", 6f to "6\u00d7")
    Row(modifier = modifier, horizontalArrangement = Arrangement.Center) {
        stops.forEach { (value, label) ->
            val selected = zoom == value
            Box(
                modifier = Modifier
                    .size(36.dp) // stock hit slot: 108 px
                    .clickable(enabled = enabled) { onSelect(value) },
                contentAlignment = Alignment.Center,
            ) {
                if (selected) {
                    Box(
                        Modifier
                            .size(32.dp) // stock scrim circle: 96 px
                            .background(Color.Black.copy(alpha = 0.55f), CircleShape)
                    )
                }
                Text(
                    text = label,
                    color = if (selected) Accent else Color.White,
                    fontSize = 11.sp,
                    modifier = Modifier.upright(),
                )
            }
        }
    }
}

/**
 * Stock mode rail: canvas y 1740..1920, text centre y=1830, 112 px between text
 * bounds, selected content centred on x=540. Labels rotate with the canvas; only
 * PHOTO and VIDEO get the pill and caret.
 */
@Composable
private fun StockModeRail(
    videoMode: Boolean,
    enabled: Boolean,
    modifier: Modifier = Modifier,
    onSelect: (Boolean) -> Unit,
) {
    data class Item(
        val label: String,
        val selected: Boolean,
        val clickable: Boolean,
        val onClick: () -> Unit,
    )

    val items = listOf(
        Item("МАСТЕР", false, false) {},
        Item("ВИДЕО", videoMode, enabled) { onSelect(true) },
        Item("ФОТО", !videoMode, enabled) { onSelect(false) },
        Item("ПОРТРЕТ", false, false) {},
        Item("ЕЩЕ", false, false) {},
    )
    // Stock rule: 112 px between text bounds and the selected item centred on
    // canvas x=540. Centring the whole row instead moved PHOTO to ~706.
    Layout(
        modifier = modifier
            .fillMaxWidth()
            .clipToBounds(),
        content = {
            items.forEach { item ->
                ModeRailItem(item.label, item.selected, item.clickable, item.onClick)
            }
        },
    ) { measurables, constraints ->
        val gap = (112f / 3f * density).roundToInt()
        val placeables = measurables.map { it.measure(Constraints()) }
        val selectedIndex = items.indexOfFirst { it.selected }.coerceAtLeast(0)
        var before = 0
        for (i in 0 until selectedIndex) before += placeables[i].width + gap
        val selectedCentre = before + placeables[selectedIndex].width / 2
        val shift = (540f / 3f * density).roundToInt() - selectedCentre
        val height = placeables.maxOfOrNull { it.height } ?: 0
        layout(constraints.maxWidth, height) {
            var x = shift
            placeables.forEach { placeable ->
                placeable.place(x, 0)
                x += placeable.width + gap
            }
        }
    }
}

@Composable
private fun ModeRailItem(label: String, selected: Boolean, enabled: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .clickable(enabled = enabled, onClick = onClick)
            // The stock pill is a background layer 216x81 px centred on the label.
            // Drawing it behind keeps the 112 px gap between text bounds exact.
            .drawBehind {
                if (selected) {
                    val w = 72.dp.toPx()
                    val h = 27.dp.toPx()
                    drawRoundRect(
                        color = Color(0xFF1E1E1E),
                        topLeft = Offset((size.width - w) / 2f, (size.height - h) / 2f),
                        size = Size(w, h),
                        cornerRadius = CornerRadius(h / 2f),
                    )
                }
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            color = if (selected) Accent else Color.White,
            fontSize = 15.sp,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
        )
        if (selected) {
            // Stock caret sits 21 px right of the label.
            Spacer(Modifier.width(canvasPx(21f)))
            Canvas(Modifier.size(canvasPx(16f), canvasPx(11f))) {
                val path = Path().apply {
                    moveTo(size.width / 2f, 0f)
                    lineTo(size.width, size.height)
                    lineTo(0f, size.height)
                    close()
                }
                drawPath(path, Accent)
            }
        }
    }
}

/**
 * Stock Live Photo glyph: a dotted ring, with a filled centre when enabled and a
 * diagonal slash when off (measured 60x60 px at canvas centre 453,162).
 */
@Composable
private fun LivePhotoGlyph(enabled: Boolean, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        val ink = if (enabled) Accent else Color.White
        val stroke = 2.dp.toPx()
        val radius = size.minDimension / 2f - stroke
        drawCircle(
            color = ink,
            radius = radius,
            style = Stroke(
                width = stroke,
                pathEffect = PathEffect.dashPathEffect(floatArrayOf(6f, 5f)),
            ),
        )
        if (enabled) {
            drawCircle(color = ink, radius = radius * 0.32f)
        } else {
            drawLine(
                color = ink,
                start = Offset(size.width * 0.18f, size.height * 0.82f),
                end = Offset(size.width * 0.82f, size.height * 0.18f),
                strokeWidth = stroke,
            )
        }
    }
}

@Composable
private fun SettingsGlyph(onClick: () -> Unit) {
    Canvas(
        Modifier
            .size(28.dp)
            .clickable(onClick = onClick)
            .padding(6.dp)
    ) {
        val ink = Color.White.copy(alpha = 0.8f)
        val stroke = 1.6.dp.toPx()
        for (i in 0..2) {
            val y = size.height * (0.2f + 0.3f * i)
            drawLine(color = ink, start = Offset(0f, y), end = Offset(size.width, y), strokeWidth = stroke)
        }
    }
}

private fun configurePreviewTransform(view: TextureView) {
    val viewWidth = view.width.toFloat()
    val viewHeight = view.height.toFloat()
    if (viewWidth == 0f || viewHeight == 0f) return

    // Measured against the stock preview in both poses: the preview stream already
    // arrives in screen orientation, and TextureView keeps the buffer aspect
    // ratio. Any rotation or rect-to-rect mapping here only introduces error
    // (portrait kept aspect at ratio 1.000; landscape rotated the frame 90 deg).
    // So the transform is identity and the preview is shaped by its layout box.
    view.setTransform(Matrix())
}

/**
 * Full-screen idle screen inspired by the stock camera's "tap to preview".
 *
 * UI-only: the preview SurfaceView stays alive underneath, so waking does not
 * touch or restart the camera session.
 */
@Composable
private fun StandbyOverlay(onWake: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .clickable(onClick = onWake),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Canvas(Modifier.size(64.dp)) {
                val ink = Color.White.copy(alpha = 0.85f)
                val stroke = Stroke(width = 2.dp.toPx())
                drawRoundRect(
                    color = ink,
                    topLeft = Offset(size.width * 0.08f, size.height * 0.30f),
                    size = Size(size.width * 0.84f, size.height * 0.50f),
                    cornerRadius = CornerRadius(8.dp.toPx()),
                    style = stroke,
                )
                drawRoundRect(
                    color = ink,
                    topLeft = Offset(size.width * 0.33f, size.height * 0.20f),
                    size = Size(size.width * 0.22f, size.height * 0.12f),
                    cornerRadius = CornerRadius(2.dp.toPx()),
                    style = stroke,
                )
                drawCircle(
                    color = ink,
                    radius = size.minDimension * 0.15f,
                    center = Offset(size.width * 0.5f, size.height * 0.55f),
                    style = stroke,
                )
                drawCircle(
                    color = ink,
                    radius = size.minDimension * 0.02f,
                    center = Offset(size.width * 0.82f, size.height * 0.38f),
                )
            }
            Spacer(Modifier.height(18.dp))
            Text(
                text = "Tap for preview",
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 14.sp,
            )
        }
    }
}

private fun formatDuration(seconds: Int): String =
    "%02d:%02d".format(seconds / 60, seconds % 60)

/** Compact top-bar badge that opens the capture-format sheet. */
@Composable
private fun FormatBadge(label: String, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(50))
            .background(Color.White.copy(alpha = 0.10f))
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 6.dp)
            .upright(),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = "$label \u25be",
            color = Color.White,
            fontWeight = FontWeight.Medium,
            fontSize = 11.sp
        )
    }
}

/** Bottom sheet for working capture formats only. */
@Composable
private fun FormatSheet(current: Int, onSelect: (Int) -> Unit, onDismiss: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.55f))
            .clickable(onClick = onDismiss)
    ) {
        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                .background(Color(0xFF111111))
                .navigationBarsPadding()
                .padding(horizontal = 20.dp, vertical = 18.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "CAPTURE FORMAT",
                    color = Color.White.copy(alpha = 0.55f),
                    fontWeight = FontWeight.Medium,
                    fontSize = 11.sp
                )
                Text(
                    text = "Close",
                    color = Accent,
                    fontSize = 12.sp,
                    modifier = Modifier
                        .clickable(onClick = onDismiss)
                        .padding(8.dp)
                )
            }

            Spacer(Modifier.height(8.dp))

            FormatRow("12 MP JPEG", "Standard still", current == MODE_JPEG12) {
                onSelect(MODE_JPEG12)
            }
            FormatRow("RAW 12", "DNG, about 25 MB", current == MODE_RAW12) {
                onSelect(MODE_RAW12)
            }
            FormatRow("RAW 50", "DNG, about 96 MB; maximum detail", current == MODE_RAW50) {
                onSelect(MODE_RAW50)
            }
            FormatRow("HDR 3x", "Three-frame merge; hold still", current == MODE_HDR3) {
                onSelect(MODE_HDR3)
            }

            Spacer(Modifier.height(8.dp))
            Text(
                text = "50 MP JPEG is unavailable through Camera2 on this device; use RAW 50.",
                color = Color.White.copy(alpha = 0.40f),
                fontSize = 10.sp
            )
        }
    }
}

@Composable
private fun FormatRow(
    title: String,
    subtitle: String,
    selected: Boolean,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(if (selected) Accent.copy(alpha = 0.16f) else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(Modifier.weight(1f)) {
            Text(
                text = title,
                color = Color.White,
                fontWeight = FontWeight.Medium,
                fontSize = 14.sp
            )
            Text(
                text = subtitle,
                color = Color.White.copy(alpha = 0.50f),
                fontSize = 10.sp
            )
        }
        Text(
            text = if (selected) "\u2713" else "",
            color = Accent,
            fontWeight = FontWeight.Bold,
            fontSize = 18.sp
        )
    }
}

/** Stock-like photo/video segmented switch. */
@Composable
private fun ModeSegment(videoMode: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        modifier = Modifier
            .clip(RoundedCornerShape(50))
            .background(Color.White.copy(alpha = 0.10f))
            .padding(3.dp),
        horizontalArrangement = Arrangement.spacedBy(2.dp)
    ) {
        SegmentButton("PHOTO", !videoMode) { onChange(false) }
        SegmentButton("VIDEO", videoMode) { onChange(true) }
    }
}

@Composable
private fun SegmentButton(label: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(50))
            .background(if (selected) Accent else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = 22.dp, vertical = 7.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White.copy(alpha = 0.68f),
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
            fontSize = 11.sp,
            modifier = Modifier.upright()
        )
    }
}

@Composable
private fun ShutterButton(
    capturing: Boolean,
    recording: Boolean,
    videoBusy: Boolean,
    videoMode: Boolean,
    enabled: Boolean,
    onClick: () -> Unit
) {
    val shutterScale by animateFloatAsState(
        targetValue = if (capturing) 0.9f else 1f,
        animationSpec = spring(),
        label = "shutterScale"
    )
    Box(
        modifier = Modifier
            .size(86.dp)
            .scale(shutterScale)
            .clip(CircleShape)
            .border(4.dp, Color.White, CircleShape)
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        if (videoMode) {
            Box(
                modifier = Modifier
                    .size(if (recording) 34.dp else 58.dp)
                    .clip(if (recording) RoundedCornerShape(6.dp) else CircleShape)
                    .background(if (videoBusy) Color(0xFF888888) else Color(0xFFE53935))
            )
        } else {
            Box(
                modifier = Modifier
                    .size(if (capturing) 54.dp else 66.dp)
                    .clip(CircleShape)
                    .background(if (capturing) Color(0xFF888888) else Accent)
            )
        }
    }
}

/** Round flip-camera control used in the bottom capture row. */
@Composable
private fun FlipCameraButton(enabled: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .size(52.dp)
            .clip(CircleShape)
            .background(Color.White.copy(alpha = if (enabled) 0.12f else 0.05f))
            .clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = "\u21c4",
            color = if (enabled) Color.White else Color.White.copy(alpha = 0.30f),
            fontSize = 24.sp
        )
    }
}

/** Compact pill used for capture modes and lens stops. */
@Composable
private fun SettingChip(
    label: String,
    selected: Boolean,
    compact: Boolean = false,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(50))
            .background(if (selected) Accent else Color.White.copy(alpha = 0.10f))
            .clickable(onClick = onClick)
            .padding(
                horizontal = if (compact) 10.dp else 12.dp,
                vertical = if (compact) 4.dp else 7.dp
            ),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White.copy(alpha = 0.85f),
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
            fontSize = if (compact) 11.sp else 12.sp,
            modifier = Modifier.upright()
        )
    }
}
