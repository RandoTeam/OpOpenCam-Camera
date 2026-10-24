package com.opopencam.camera

import android.Manifest
import android.graphics.Matrix
import android.graphics.RectF
import android.graphics.SurfaceTexture
import android.content.pm.PackageManager
import android.content.res.Configuration
import android.hardware.SensorManager
import android.os.Environment
import android.view.Surface
import android.view.OrientationEventListener
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
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
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

private val Accent = Color(0xFFFF6A00)

/** Idle time before the camera chrome is replaced by the standby screen. */
private const val STANDBY_TIMEOUT_MS = 30_000L

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
    val configuration = LocalConfiguration.current
    val landscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE
    val displayRotationDegrees = remember {
        (context.display?.rotation ?: Surface.ROTATION_0) * 90
    }
    var deviceRotation by remember { mutableIntStateOf(displayRotationDegrees) }
    val deviceRotationState = rememberUpdatedState(deviceRotation)
    val uiRotation = ((deviceRotation - displayRotationDegrees) % 360 + 360) % 360
    val haptics = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()

    var hasCamera by remember {
        mutableStateOf(
            context.checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
        )
    }
    var status by remember { mutableStateOf("Starting camera…") }
    var capturing by remember { mutableStateOf(false) }
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

    // Physical rotation is independent of the activity lock. The camera UI is
    // landscape-first, but standby and the preview must still know which way
    // the phone is being held.
    DisposableEffect(context) {
        val listener = object : OrientationEventListener(context, SensorManager.SENSOR_DELAY_NORMAL) {
            override fun onOrientationChanged(orientation: Int) {
                if (orientation == ORIENTATION_UNKNOWN) return
                val rounded = ((orientation + 45) / 90 * 90) % 360
                if (rounded != deviceRotation) {
                    deviceRotation = rounded
                }
            }
        }
        listener.enable()
        onDispose { listener.disable() }
    }

    // UI-only standby: hide the chrome, keep the SurfaceView and session alive.
    // Like the stock screen, a black idle screen also saves panel burn-in,
    // battery and heat.
    LaunchedEffect(
        lastInteraction,
        capturing,
        recording,
        videoBusy,
        switching,
        showGallery,
        showFormatSheet,
        resumed,
    ) {
        if (!resumed || capturing || recording || videoBusy || switching || showGallery || showFormatSheet) {
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

    LaunchedEffect(deviceRotation) {
        if (textureView.isAvailable) {
            configurePreviewTransform(textureView, 1440, 1080, deviceRotationState.value)
        }
    }


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
        if (capturing) {
            status = "Wait for the current photo"
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
        if (capturing || videoBusy || switching) return
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
        if (capturing || recording || videoBusy) return
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

    fun capturePhoto() {
        if (capturing || switching) return
        capturing = true
        status = if (captureMode == MODE_HDR3) {
            "Processing HDR\u2026"
        } else if (captureMode == MODE_RAW50) {
            "Saving RAW 50\u2026"
        } else {
            "Capturing\u2026"
        }
        scope.launch {
            // nativeCapturePhoto blocks until the buffer is written; HDR can
            // take several seconds.
            val result = withContext(Dispatchers.IO) {
                val dir = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
                    ?: context.filesDir
                dir.mkdirs()
                val effectiveMode = NativeBridge.nativeGetActiveMode()
                    .let { if (it >= 0) it else captureMode }
                val extension = if (effectiveMode == MODE_RAW12 || effectiveMode == MODE_RAW50) {
                    "dng"
                } else {
                    "jpg"
                }
                val file = File(dir, "IMG_${System.currentTimeMillis()}.$extension")
                val displayRotation = context.display?.rotation ?: Surface.ROTATION_0
                val sensorOrientation = if (cameraId == 1) 270 else 90
                val jpegOrientation = ((sensorOrientation - displayRotation * 90) % 360 + 360) % 360
                NativeBridge.nativeSetJpegOrientation(jpegOrientation)
                val ok = NativeBridge.nativeCapturePhoto(file.absolutePath)
                when {
                    ok -> {
                        val uri = MediaStoreSaver.publishImage(context, file)
                        if (uri != null) {
                            "Saved ${file.name} \u00b7 Gallery"
                        } else {
                            "Saved ${file.name} \u00b7 not indexed"
                        }
                    }
                    effectiveMode == MODE_JPEG50 -> "50 MP JPEG unavailable \u2014 use RAW 50"
                    else -> "Capture failed - see logcat"
                }
            }
            galleryItems = withContext(Dispatchers.IO) { listGalleryItems(context) }
            status = result
            capturing = false
            Toast.makeText(context, result, Toast.LENGTH_SHORT).show()
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
                    started = true
                    textureView.post {
                        configurePreviewTransform(textureView, 1440, 1080, deviceRotationState.value)
                    }
                }
            }
        }

        val listener = object : TextureView.SurfaceTextureListener {
            override fun onSurfaceTextureAvailable(st: SurfaceTexture, width: Int, height: Int) {
                startIfReady()
            }

            override fun onSurfaceTextureSizeChanged(st: SurfaceTexture, width: Int, height: Int) {
                configurePreviewTransform(textureView, 1440, 1080, deviceRotationState.value)
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

        onDispose {
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
                Modifier.fillMaxSize()
            } else {
                Modifier
                    .fillMaxWidth()
                    .aspectRatio(3f / 4f)
                    .align(Alignment.Center)
            }).pointerInput(cameraId) {
                    detectTransformGestures { _, _, zoomChange, _ ->
                        if (!capturing && !videoBusy && !switching && !recording) {
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
                        configurePreviewTransform(textureView, 1440, 1080, deviceRotationState.value)
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
            Column {
                Text(
                    text = "OpOpenCam",
                    color = Color.White,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 15.sp
                )
                Text(
                    text = if (videoMode) "VIDEO" else "${modeLabel(captureMode)} \u00b7 ${cameraLabel(cameraId)} \u00b7 ${"%.1f".format(zoom)}\u00d7",
                    color = Color.White.copy(alpha = 0.55f),
                    fontSize = 10.sp
                )
            }
            if (capturing || switching || videoBusy || recording ||
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
                    modifier = Modifier.width(150.dp)
                )
            } else if (!videoMode) {
                FormatBadge(modeLabel(captureMode)) {
                    if (!capturing && !switching && !videoBusy) {
                        showFormatSheet = true
                        lastInteraction++
                    }
                }
            }
        }

        if (!landscape) {
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
                if (!recording && !videoBusy) {
                    videoMode = isVideo
                    status = if (isVideo) {
                        if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                    } else {
                        "Photo mode"
                    }
                }
            }

            Spacer(Modifier.height(10.dp))

            if (videoMode) {
                SettingChip(
                    if (video4k16) "4K 16:9 \u00b7 60" else "4K 4:3 \u00b7 30",
                    true,
                ) {
                    if (!recording && !videoBusy) {
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
                    enabled = if (videoMode) !videoBusy else !capturing && !switching,
                    onClick = { onShutter() }
                )

                FlipCameraButton(
                    enabled = !recording && !videoBusy && !capturing && !switching
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
                    enabled = if (videoMode) !videoBusy else !capturing && !switching,
                    onClick = { onShutter() }
                )
                FlipCameraButton(
                    enabled = !recording && !videoBusy && !capturing && !switching
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
                    if (!recording && !videoBusy) {
                        videoMode = isVideo
                        status = if (isVideo) {
                            if (video4k16) "Video 4K 16:9 \u00b7 60 fps" else "Video 4K 4:3 \u00b7 30 fps"
                        } else {
                            "Photo mode"
                        }
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

        if (capturing && !showGallery && !showFormatSheet) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.20f)),
                contentAlignment = Alignment.Center
            ) {
                Column(
                    modifier = Modifier
                        .clip(RoundedCornerShape(18.dp))
                        .background(Color.Black.copy(alpha = 0.78f))
                        .padding(horizontal = 24.dp, vertical = 18.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    CircularProgressIndicator(
                        color = Accent,
                        strokeWidth = 3.dp,
                        modifier = Modifier.size(38.dp)
                    )
                    Spacer(Modifier.height(10.dp))
                    Text(
                        text = status,
                        color = Color.White,
                        fontSize = 12.sp
                    )
                }
            }
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

        if (standby && !showGallery && !showFormatSheet && !capturing && !recording && !videoBusy && !switching && resumed) {
            StandbyOverlay(rotationDegrees = uiRotation) {
                standby = false
                lastInteraction++
            }
        }
    }
}

private fun configurePreviewTransform(
    view: TextureView,
    previewWidth: Int,
    previewHeight: Int,
    deviceRotationDegrees: Int
) {
    val viewWidth = view.width
    val viewHeight = view.height
    if (viewWidth == 0 || viewHeight == 0) return

    // Rear sensor is mounted at 90 degrees. Use the physical device rotation,
    // not the locked activity rotation, so a portrait-held phone does not show
    // a sideways preview.
    val rotationDegrees = (90 - deviceRotationDegrees + 360) % 360
    val landscape = viewWidth > viewHeight

    val matrix = Matrix()
    matrix.postTranslate(-previewWidth / 2f, -previewHeight / 2f)
    matrix.postRotate(rotationDegrees.toFloat())
    val rotatedWidth = if (rotationDegrees % 180 == 0) previewWidth else previewHeight
    val rotatedHeight = if (rotationDegrees % 180 == 0) previewHeight else previewWidth
    val scale = if (landscape) {
        // Fill the landscape screen like a camera viewfinder; the still keeps
        // the full 4:3 sensor framing.
        maxOf(viewWidth.toFloat() / rotatedWidth, viewHeight.toFloat() / rotatedHeight)
    } else {
        // Portrait: show the complete 3:4 framing without cropping.
        minOf(viewWidth.toFloat() / rotatedWidth, viewHeight.toFloat() / rotatedHeight)
    }
    matrix.postScale(scale, scale)
    matrix.postTranslate(viewWidth / 2f, viewHeight / 2f)
    view.setTransform(matrix)
}

/**
 * Full-screen idle screen inspired by the stock camera's "tap to preview".
 *
 * UI-only: the preview SurfaceView stays alive underneath, so waking does not
 * touch or restart the camera session.
 */
@Composable
private fun StandbyOverlay(rotationDegrees: Int, onWake: () -> Unit) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .clickable(onClick = onWake),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.rotate(rotationDegrees.toFloat())
        ) {
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
            .padding(horizontal = 12.dp, vertical = 6.dp),
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
            fontSize = 11.sp
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
            fontSize = if (compact) 11.sp else 12.sp
        )
    }
}
