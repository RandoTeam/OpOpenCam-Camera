package com.opopencam.camera

import android.Manifest
import android.app.Activity
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaCodecList
import android.media.MediaFormat
import android.os.Bundle
import android.os.SystemClock
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Debug-only camera matrix runner. Start with:
 *
 * adb shell am start -n com.oneplus.camera/com.opopencam.camera.SelfTestActivity
 *
 * It exercises modes, logical zoom and lenses without UI taps and writes a JSONL
 * report under files/selftest/. It is intentionally not part of the product UI.
 */
class SelfTestActivity : Activity(), SurfaceHolder.Callback {
    private lateinit var surfaceView: SurfaceView
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var started = false
    private var surface: Surface? = null
    private val reportLock = Any()
    private lateinit var reportFile: File
    private var caseIndex = 0
    private var failures = 0

    private data class Case(
        val label: String,
        val camera: Int,
        val mode: Int,
        val zoom: Float,
    )

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if ((applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) == 0) {
            finish()
            return
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        reportFile = File(getExternalFilesDir(null), "selftest/report.jsonl")
        reportFile.parentFile?.mkdirs()
        reportFile.writeText("")
        surfaceView = SurfaceView(this).apply {
            holder.setFixedSize(1440, 1080)
            holder.addCallback(this@SelfTestActivity)
        }
        setContentView(surfaceView)
        if (checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(arrayOf(Manifest.permission.CAMERA), 1)
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED) maybeStart()
        else {
            record(JSONObject().put("error", "camera permission denied"))
            finish()
        }
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        surface = holder.surface
        maybeStart()
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        surface = holder.surface
        maybeStart()
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        surface = null
        NativeBridge.nativeStopPreview()
    }

    private fun maybeStart() {
        val s = surface ?: return
        if (started || checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) return
        started = true
        scope.launch { runPlan(s) }
    }

    private suspend fun runPlan(surface: Surface) {
        val cases = listOf(
            Case("jpeg12_logical_1x", 0, 0, 1f),
            Case("jpeg12_logical_06x", 0, 0, 0.6f),
            Case("jpeg12_logical_2x", 0, 0, 2f),
            Case("jpeg12_logical_3x", 0, 0, 3f),
            Case("jpeg12_logical_6x", 0, 0, 6f),
            Case("jpeg12_ultrawide_1x", 3, 0, 1f),
            Case("jpeg12_tele_1x", 4, 0, 1f),
            Case("jpeg12_front_1x", 1, 0, 1f),
            Case("raw12_logical_1x", 0, 2, 1f),
            Case("raw12_logical_3x", 0, 2, 3f),
            Case("raw25_probe", 0, 5, 1f),
            Case("raw25_probe_physical_main", 2, 5, 1f),
            Case("raw50_logical_1x", 0, 3, 1f),
            Case("hdr_logical_1x", 0, 4, 1f),
        )
        for (case in cases) {
            runCase(surface, case)
        }
        recordDolbyCodecs()
        runVideoProbe(surface, "video_8k30_logical", 0, 7680, 4320, 30)
        runVideoProbe(surface, "video_8k60_logical", 0, 7680, 4320, 60)
        runVideoProbe(surface, "video_6k30_main", 2, 6016, 4512, 30)
        runVideoProbe(surface, "video_6k60_main", 2, 6016, 4512, 60)
        val summary = JSONObject()
            .put("cases", cases.size)
            .put("failures", failures)
            .put("finished_ms", System.currentTimeMillis())
        append(summary)
        Log.i(TAG, "DONE $summary")
        NativeBridge.nativeStopPreview()
        finish()
    }

    private suspend fun runCase(surface: Surface, case: Case) {
        caseIndex++
        val json = JSONObject()
            .put("index", caseIndex)
            .put("label", case.label)
            .put("camera", case.camera)
            .put("mode_requested", case.mode)
            .put("zoom_requested", case.zoom)
        val startedAt = SystemClock.elapsedRealtime()
        try {
            NativeBridge.nativeStopPreview()
            delay(120)
            val ok = NativeBridge.nativeStartPreview(surface, case.camera, case.mode)
            val active = NativeBridge.nativeGetActiveMode()
            json.put("start_ok", ok).put("mode_active", active)
            if (!ok || active != case.mode) {
                failures++
                json.put("error", "session did not activate requested mode")
                json.put("elapsed_ms", SystemClock.elapsedRealtime() - startedAt)
                append(json)
                return
            }
            val zoomOk = NativeBridge.nativeSetZoom(case.zoom)
            json.put("zoom_ok", zoomOk)
            delay(350)

            val ext = if (case.mode == 2 || case.mode == 3 || case.mode == 5) "dng" else "jpg"
            val dir = File(getExternalFilesDir(null), "selftest/captures").apply { mkdirs() }
            val file = File(dir, "%02d_%s.%s".format(caseIndex, case.label, ext))
            file.delete()
            val before = SystemClock.elapsedRealtime()
            val okCapture = NativeBridge.nativeCapturePhoto(file.absolutePath)
            val elapsed = SystemClock.elapsedRealtime() - before
            json.put("capture_ok", okCapture).put("capture_ms", elapsed)
                .put("bytes", file.length())
            if (!okCapture || !file.isFile || file.length() == 0L) {
                failures++
                json.put("error", "capture failed")
            } else {
                val dims = if (ext == "dng") dngSize(file) else jpegSize(file)
                json.put("width", dims?.first ?: 0)
                json.put("height", dims?.second ?: 0)
                if (dims == null) {
                    failures++
                    json.put("error", "could not parse output dimensions")
                }
                if (case.label == "jpeg12_logical_1x") {
                    val uri = MediaStoreSaver.publishImage(this, file)
                    json.put("media_store_uri", uri?.toString() ?: "")
                }
            }
            json.put("elapsed_ms", SystemClock.elapsedRealtime() - startedAt)
        } catch (t: Throwable) {
            failures++
            json.put("error", t.message ?: t.javaClass.simpleName)
            json.put("elapsed_ms", SystemClock.elapsedRealtime() - startedAt)
        }
        append(json)
    }

    private suspend fun runVideoProbe(
        surface: Surface,
        label: String,
        camera: Int,
        width: Int,
        height: Int,
        fps: Int,
    ) {
        val json = JSONObject().put("label", label).put("kind", "video_probe")
            .put("camera", camera).put("width", width).put("height", height).put("fps", fps)
        var codec: MediaCodec? = null
        var input: Surface? = null
        try {
            val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_HEVC, width, height)
            format.setInteger(
                MediaFormat.KEY_COLOR_FORMAT,
                MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface,
            )
            format.setInteger(MediaFormat.KEY_BIT_RATE, 80_000_000)
            format.setInteger(MediaFormat.KEY_FRAME_RATE, fps)
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1)
            codec = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_VIDEO_HEVC)
            codec.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            input = codec.createInputSurface()
            codec.start()
            NativeBridge.nativeStopPreview()
            delay(120)
            val ok = NativeBridge.nativeStartVideo(surface, input, camera, width, height, fps)
            json.put("session_ok", ok)
            if (ok) {
                delay(250)
                json.put("session_active", NativeBridge.nativeIsVideoRunning())
                NativeBridge.nativeStopVideo()
            }
            NativeBridge.nativeStopPreview()
        } catch (t: Throwable) {
            failures++
            json.put("error", t.message ?: t.javaClass.simpleName)
        } finally {
            try { codec?.stop() } catch (_: Throwable) {}
            try { codec?.release() } catch (_: Throwable) {}
            try { input?.release() } catch (_: Throwable) {}
        }
        append(json)
    }

    private fun recordDolbyCodecs() {
        val codecs = MediaCodecList(MediaCodecList.ALL_CODECS).codecInfos
            .filter { info -> info.supportedTypes.any { it.equals("video/dolby-vision", true) } }
            .map { it.name }
        append(JSONObject().put("label", "codec_probe").put("kind", "dolby")
            .put("video/dolby-vision", codecs.joinToString(",")))
    }

    private fun append(json: JSONObject) {
        val line = json.toString()
        Log.i(TAG, line)
        synchronized(reportLock) {
            FileOutputStream(reportFile, true).use { it.write((line + "\n").toByteArray()) }
        }
    }

    private fun record(json: JSONObject) = append(json)

    private fun jpegSize(file: File): Pair<Int, Int>? {
        val bytes = file.readBytes()
        if (bytes.size < 4 || bytes[0] != 0xFF.toByte() || bytes[1] != 0xD8.toByte()) return null
        var i = 2
        while (i + 9 < bytes.size) {
            if (bytes[i] != 0xFF.toByte()) { i++; continue }
            val marker = bytes[i + 1].toInt() and 0xFF
            if (marker == 0xD9 || marker == 0xDA) return null
            val len = ((bytes[i + 2].toInt() and 0xFF) shl 8) or (bytes[i + 3].toInt() and 0xFF)
            if (marker in 0xC0..0xCF && marker != 0xC4 && marker != 0xC8 && marker != 0xCC) {
                val h = ((bytes[i + 5].toInt() and 0xFF) shl 8) or (bytes[i + 6].toInt() and 0xFF)
                val w = ((bytes[i + 7].toInt() and 0xFF) shl 8) or (bytes[i + 8].toInt() and 0xFF)
                return w to h
            }
            if (len < 2) return null
            i += 2 + len
        }
        return null
    }

    private fun dngSize(file: File): Pair<Int, Int>? {
        val bytes = file.readBytes()
        if (bytes.size < 8) return null
        val order = if (bytes[0] == 'I'.code.toByte() && bytes[1] == 'I'.code.toByte()) {
            ByteOrder.LITTLE_ENDIAN
        } else if (bytes[0] == 'M'.code.toByte() && bytes[1] == 'M'.code.toByte()) {
            ByteOrder.BIG_ENDIAN
        } else return null
        val buf = ByteBuffer.wrap(bytes).order(order)
        if ((buf.getShort(2).toInt() and 0xFFFF) != 42) return null
        val ifd = buf.getInt(4)
        if (ifd < 8 || ifd + 2 > bytes.size) return null
        val count = buf.getShort(ifd).toInt() and 0xFFFF
        var width = 0
        var height = 0
        for (n in 0 until count) {
            val p = ifd + 2 + n * 12
            if (p + 12 > bytes.size) return null
            val tag = buf.getShort(p).toInt() and 0xFFFF
            val type = buf.getShort(p + 2).toInt() and 0xFFFF
            val value = buf.getInt(p + 8)
            if (tag == 0x0100 && type == 4) width = value
            if (tag == 0x0101 && type == 4) height = value
        }
        return if (width > 0 && height > 0) width to height else null
    }

    override fun onDestroy() {
        NativeBridge.nativeStopPreview()
        scope.launch { }
        super.onDestroy()
    }

    companion object {
        private const val TAG = "OpOpenCamSelfTest"
    }
}
