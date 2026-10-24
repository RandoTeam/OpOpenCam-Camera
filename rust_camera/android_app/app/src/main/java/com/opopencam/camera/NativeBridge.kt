package com.opopencam.camera

import android.view.Surface

/**
 * Thin JNI surface for the Rust camera engine.
 *
 * All blocking calls (especially [nativeCapturePhoto]) must be invoked off the
 * main thread. The Rust side serialises access to the Camera2 session.
 */
object NativeBridge {
    init {
        System.loadLibrary("opopencam")
    }

    /**
     * Starts (or restarts) the Camera2 session.
     *
     * @param cameraId 0 = logical multi-camera, 2 = main, 3 = ultra-wide, 4 = tele.
     * @param mode 0 = 12 MP JPEG, 1 = 50 MP JPEG, 2 = RAW 12, 3 = RAW 50, 4 = HDR 3×.
     */
    external fun nativeStartPreview(surface: Surface, cameraId: Int, mode: Int): Boolean

    external fun nativeStopPreview()
    external fun nativeCapturePhoto(path: String): Boolean
    external fun nativeSetZoom(ratio: Float): Boolean

    /** Rotation the viewer must apply to the next JPEG/DNG (0/90/180/270). */
    external fun nativeSetJpegOrientation(degrees: Int): Boolean
    external fun nativeIsRunning(): Boolean

    /**
     * Starts a recording session with the existing preview surface and a
     * MediaCodec input surface. Blocks while the Camera2 session is rebuilt;
     * call from a background thread.
     */
    external fun nativeStartVideo(
        previewSurface: Surface,
        videoSurface: Surface,
        cameraId: Int,
        width: Int,
        height: Int,
        fps: Int,
    ): Boolean

    external fun nativeStopVideo()
    external fun nativeIsVideoRunning(): Boolean

    /** Actual mode after HAL fallback; -1 when no session is active. */
    external fun nativeGetActiveMode(): Int

    /**
     * Assembles a Google Motion Photo from an existing JPEG and MP4.
     * Used by the Live Photo milestone.
     */
    external fun nativeAssembleMotionPhoto(
        jpegPath: String,
        mp4Path: String,
        outPath: String,
        timestampUs: Long,
    ): Boolean
}
