package com.opopencam.camera

import android.Manifest
import android.content.pm.PackageManager
import android.os.Environment
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

private val Accent = Color(0xFFFF6A00)

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
    0 -> "logical"
    2 -> "main"
    3 -> "ultra-wide"
    4 -> "tele"
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
    val scope = rememberCoroutineScope()

    var hasCamera by remember {
        mutableStateOf(
            context.checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED
        )
    }
    var status by remember { mutableStateOf("Starting camera…") }
    var capturing by remember { mutableStateOf(false) }
    var zoom by remember { mutableStateOf(1f) }
    var cameraId by remember { mutableStateOf(0) }
    var captureMode by remember { mutableStateOf(MODE_JPEG12) }
    var activeMode by remember { mutableStateOf(-1) }
    var currentSurface by remember { mutableStateOf<Surface?>(null) }
    var videoMode by remember { mutableStateOf(false) }
    var video4k16 by remember { mutableStateOf(false) }
    var recording by remember { mutableStateOf(false) }
    var videoBusy by remember { mutableStateOf(false) }
    var recorder by remember { mutableStateOf<VideoRecorder?>(null) }
    var audioGranted by remember {
        mutableStateOf(
            context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
                PackageManager.PERMISSION_GRANTED
        )
    }

    val cameraIdState = rememberUpdatedState(cameraId)
    val modeState = rememberUpdatedState(captureMode)

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

    val surfaceView = remember {
        SurfaceView(context).apply {
            holder.setFixedSize(1920, 1080)
        }
    }

    fun describeStatus(requestedCamera: Int, requestedMode: Int, ok: Boolean) {
        status = when {
            !ok -> "Camera start failed — see logcat"
            activeMode >= 0 && activeMode != requestedMode -> "HAL fallback: ${modeLabel(activeMode)}"
            else -> "${cameraLabel(requestedCamera)} · ${modeLabel(requestedMode)}"
        }
    }

    fun restart(requestedCamera: Int, requestedMode: Int) {
        if (recording || videoBusy) {
            status = "Stop recording first"
            return
        }
        cameraId = requestedCamera
        captureMode = requestedMode
        zoom = 1f
        val surface = currentSurface
        if (surface == null) {
            status = "Waiting for preview surface…"
            return
        }
        NativeBridge.nativeStopPreview()
        val ok = NativeBridge.nativeStartPreview(surface, requestedCamera, requestedMode)
        activeMode = NativeBridge.nativeGetActiveMode()
        describeStatus(requestedCamera, requestedMode, ok)
    }

    fun beginVideo() {
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
            result.onSuccess {
                status = "Saved ${it.name}"
                Toast.makeText(context, "Saved ${it.name}", Toast.LENGTH_SHORT).show()
            }.onFailure {
                status = "Video stop failed: ${it.message ?: it.javaClass.simpleName}"
            }
        }
    }

    DisposableEffect(surfaceView, hasCamera) {
        if (!hasCamera) {
            return@DisposableEffect onDispose { }
        }

        var started = false
        val startIfSized = {
            val holder = surfaceView.holder
            val frame = holder.surfaceFrame
            if (!started && holder.surface.isValid && frame.width() > 0 && frame.height() > 0) {
                started = true
                currentSurface = holder.surface
                val requestedCamera = cameraIdState.value
                val requestedMode = modeState.value
                val ok = NativeBridge.nativeStartPreview(holder.surface, requestedCamera, requestedMode)
                activeMode = NativeBridge.nativeGetActiveMode()
                describeStatus(requestedCamera, requestedMode, ok)
            }
        }

        val callback = object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) {
                // The buffer geometry is final only after surfaceChanged.
                holder.setFixedSize(1920, 1080)
                surfaceView.post { startIfSized() }
            }

            override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                surfaceView.post { startIfSized() }
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
                started = false
                if (recording) {
                    recorder?.abort()
                    recorder = null
                    recording = false
                    videoBusy = false
                }
                currentSurface = null
                NativeBridge.nativeStopPreview()
            }
        }

        surfaceView.holder.addCallback(callback)
        surfaceView.post { startIfSized() }

        onDispose {
            surfaceView.holder.removeCallback(callback)
            if (recording) {
                recorder?.abort()
                recorder = null
                recording = false
                videoBusy = false
            }
            NativeBridge.nativeStopPreview()
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        AndroidView(
            factory = { surfaceView },
            modifier = Modifier.fillMaxSize()
        )

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 52.dp, start = 20.dp, end = 20.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "OpOpenCam",
                color = Color.White,
                fontWeight = FontWeight.SemiBold,
                fontSize = 16.sp
            )
            Text(
                text = status,
                color = Color.White.copy(alpha = 0.72f),
                fontSize = 11.sp,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.width(230.dp)
            )
        }

        Column(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .background(
                    Brush.verticalGradient(
                        listOf(Color.Transparent, Color.Black.copy(alpha = 0.92f))
                    )
                )
                .padding(bottom = 34.dp, top = 36.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Row(
                modifier = Modifier.horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                SettingChip("VIDEO", videoMode) {
                    if (!recording && !videoBusy) {
                        videoMode = !videoMode
                        status = if (videoMode) "Video 4K 4:3 · 30 fps" else "Photo mode"
                    }
                }
                SettingChip("12", captureMode == MODE_JPEG12) { restart(cameraId, MODE_JPEG12) }
                SettingChip("50", captureMode == MODE_JPEG50) { restart(cameraId, MODE_JPEG50) }
                SettingChip("RAW", captureMode == MODE_RAW12) { restart(cameraId, MODE_RAW12) }
                SettingChip("R50", captureMode == MODE_RAW50) { restart(cameraId, MODE_RAW50) }
                SettingChip("HDR", captureMode == MODE_HDR3) { restart(cameraId, MODE_HDR3) }
            }

            if (videoMode) {
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    SettingChip(
                        if (video4k16) "4K 16:9 · 60" else "4K 4:3 · 30",
                        true,
                    ) {
                        if (!recording && !videoBusy) {
                            video4k16 = !video4k16
                            status = if (video4k16) "Video 4K 16:9 · 60 fps" else "Video 4K 4:3 · 30 fps"
                        }
                    }
                }
            }

            Spacer(Modifier.height(10.dp))

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SettingChip("0.6", cameraId == 3) { restart(3, captureMode) }
                SettingChip("1x", cameraId == 0) { restart(0, captureMode) }
                SettingChip("3x", cameraId == 4) { restart(4, captureMode) }
            }

            Spacer(Modifier.height(10.dp))

            Slider(
                value = zoom,
                onValueChange = { zoom = it },
                onValueChangeFinished = { applyZoom(zoom) { status = it } },
                valueRange = if (cameraId == 0) 0.6f..3f else 1f..3f,
                colors = SliderDefaults.colors(
                    thumbColor = Accent,
                    activeTrackColor = Accent,
                    inactiveTrackColor = Color.White.copy(alpha = 0.25f)
                ),
                modifier = Modifier.width(240.dp)
            )

            Spacer(Modifier.height(12.dp))

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
                    .clickable(enabled = if (videoMode) !videoBusy else !capturing) {
                        if (videoMode) {
                            if (recording) endVideo() else beginVideo()
                            return@clickable
                        }
                        if (capturing) return@clickable
                        capturing = true
                        if (captureMode == MODE_HDR3) status = "Processing HDR…"
                        scope.launch {
                            // nativeCapturePhoto blocks until the buffer is written;
                            // HDR can take several seconds.
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
                                val ok = NativeBridge.nativeCapturePhoto(file.absolutePath)
                                when {
                                    ok -> "Saved ${file.name}"
                                    effectiveMode == MODE_JPEG50 -> "50 MP JPEG unavailable — use R50"
                                    else -> "Capture failed - see logcat"
                                }
                            }
                            status = result
                            capturing = false
                            Toast.makeText(context, result, Toast.LENGTH_SHORT).show()
                        }
                    },
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

            Spacer(Modifier.height(8.dp))
            Text(
                text = if (videoMode) {
                    val spec = if (video4k16) "4K 16:9 · 60 fps" else "4K 4:3 · 30 fps"
                    if (recording) "REC · $spec · tap to stop"
                    else "VIDEO · $spec · ${cameraLabel(cameraId)} · tap to record"
                } else {
                    "${modeLabel(captureMode)} · ${cameraLabel(cameraId)} · tap to capture"
                },
                color = if (recording) Color(0xFFFF5252) else Color.White.copy(alpha = 0.55f),
                fontSize = 11.sp
            )
        }
    }
}

/** Compact pill used for capture modes and lens stops. */
@Composable
private fun SettingChip(label: String, selected: Boolean, onClick: () -> Unit) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(50))
            .background(if (selected) Color.White else Color.White.copy(alpha = 0.12f))
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 7.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (selected) Color.Black else Color.White,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
            fontSize = 12.sp
        )
    }
}

/**
 * Applies a logical zoom ratio through the Rust engine.
 *
 * The HAL may reject ratios it cannot map to a physical camera; in that case
 * the status line explains the failure instead of leaving the UI stuck.
 */
private fun applyZoom(ratio: Float, onStatus: (String) -> Unit) {
    val ok = NativeBridge.nativeSetZoom(ratio)
    onStatus(if (ok) "Zoom %.1fx".format(ratio) else "Zoom not supported by HAL")
}
