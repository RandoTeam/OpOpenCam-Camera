package com.opopencam.camera

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.media.MediaMuxer
import android.media.MediaRecorder
import android.view.Surface
import java.io.File
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Foreground video recorder.
 *
 * Kotlin owns the codecs and muxer; Rust only attaches [inputSurface] to the
 * Camera2 session. [start] and [stop] block and must be called from an IO
 * dispatcher. The current first slice is 4096x3072 @30 HEVC + AAC stereo.
 */
class VideoRecorder(private val context: Context) {
    data class Config(
        val width: Int = 4096,
        val height: Int = 3072,
        val fps: Int = 30,
        val videoBitrate: Int = 80_000_000,
        val audioBitrate: Int = 192_000,
    )

    private data class Pending(
        val video: Boolean,
        val data: ByteArray,
        val ptsUs: Long,
        val flags: Int,
    )

    private val stopRequested = AtomicBoolean(false)
    private val muxerLock = Any()
    private val pending = ArrayDeque<Pending>()
    private val pendingLimit = 120

    private var videoCodec: MediaCodec? = null
    private var audioCodec: MediaCodec? = null
    private var audioRecord: AudioRecord? = null
    private var inputSurface: Surface? = null
    private var muxer: MediaMuxer? = null
    private var videoFormat: MediaFormat? = null
    private var audioFormat: MediaFormat? = null
    private var videoTrack = -1
    private var audioTrack = -1
    private var muxerStarted = false
    private var videoEpochUs = -1L
    @Volatile private var videoLastPtsUs = -1L
    @Volatile private var videoEnded = false
    private var videoThread: Thread? = null
    private var audioThread: Thread? = null
    private var partFile: File? = null
    private var finalFile: File? = null
    private var lastError: String? = null

    fun lastError(): String? = lastError

    /** Starts codecs and a Camera2 recording session. Blocking. */
    fun start(
        previewSurface: Surface,
        cameraId: Int,
        outputFile: File,
        config: Config = Config(),
    ) {
        check(context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED) {
            "RECORD_AUDIO permission is required"
        }
        outputFile.parentFile?.mkdirs()
        finalFile = outputFile
        partFile = File(outputFile.parentFile, outputFile.name + ".part")
        partFile?.delete()
        stopRequested.set(false)
        lastError = null

        try {
            muxer = MediaMuxer(
                partFile!!.absolutePath,
                MediaMuxer.OutputFormat.MUXER_OUTPUT_MPEG_4,
            ).apply { setOrientationHint(90) }

            val video = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_VIDEO_HEVC)
            val vf = MediaFormat.createVideoFormat(
                MediaFormat.MIMETYPE_VIDEO_HEVC,
                config.width,
                config.height,
            ).apply {
                setInteger(
                    MediaFormat.KEY_COLOR_FORMAT,
                    MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface,
                )
                setInteger(MediaFormat.KEY_BIT_RATE, config.videoBitrate)
                setInteger(MediaFormat.KEY_FRAME_RATE, config.fps)
                setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1)
            }
            video.configure(vf, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            inputSurface = video.createInputSurface()
            video.start()
            videoCodec = video

            val audio = MediaCodec.createEncoderByType(MediaFormat.MIMETYPE_AUDIO_AAC)
            val af = MediaFormat.createAudioFormat(MediaFormat.MIMETYPE_AUDIO_AAC, 48_000, 2).apply {
                setInteger(MediaFormat.KEY_AAC_PROFILE, MediaCodecInfo.CodecProfileLevel.AACObjectLC)
                setInteger(MediaFormat.KEY_BIT_RATE, config.audioBitrate)
                setInteger(MediaFormat.KEY_MAX_INPUT_SIZE, 16_384)
            }
            audio.configure(af, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            audio.start()
            audioCodec = audio

            val minBytes = AudioRecord.getMinBufferSize(
                48_000,
                AudioFormat.CHANNEL_IN_STEREO,
                AudioFormat.ENCODING_PCM_16BIT,
            )
            check(minBytes > 0) { "AudioRecord buffer size is invalid" }
            val record = AudioRecord(
                MediaRecorder.AudioSource.CAMCORDER,
                48_000,
                AudioFormat.CHANNEL_IN_STEREO,
                AudioFormat.ENCODING_PCM_16BIT,
                minBytes * 2,
            )
            check(record.state == AudioRecord.STATE_INITIALIZED) {
                "AudioRecord failed to initialize"
            }
            audioRecord = record
            record.startRecording()
            val audioStartedNs = System.nanoTime()

            videoThread = Thread({ videoLoop() }, "OpOpenCam-video").apply { start() }

            val ok = NativeBridge.nativeStartVideo(
                previewSurface,
                inputSurface!!,
                cameraId,
                config.width,
                config.height,
                config.fps,
            )
            check(ok) { "Camera2 rejected the video session" }

            // AudioRecord starts before the Camera2 session is rebuilt. Drop the
            // pre-roll so both tracks start at the first recorded video frame.
            val cameraStartedNs = System.nanoTime()
            val skipBytes = (((cameraStartedNs - audioStartedNs).coerceAtLeast(0L) *
                48_000L * 4L) / 1_000_000_000L).toInt()
            audioThread = Thread({ audioLoop(minBytes * 2, skipBytes) }, "OpOpenCam-audio").apply {
                start()
            }
        } catch (t: Throwable) {
            lastError = t.message ?: t.javaClass.simpleName
            cleanup(failed = true)
            throw t
        }
    }

    /** Stops the camera, finalizes the MP4 and returns the saved file. Blocking. */
    fun stop(): File {
        stopRequested.set(true)
        try {
            audioRecord?.stop()
        } catch (_: Throwable) {
        }
        NativeBridge.nativeStopVideo()

        videoThread?.join(2_000)
        audioThread?.join(2_000)
        try {
            videoCodec?.signalEndOfInputStream()
        } catch (_: Throwable) {
        }

        // Video EOS must be known before the audio tail is drained so audio
        // samples beyond the last video timestamp can be dropped.
        drainVideo(eos = true)
        videoEnded = true
        drainAudio(eos = true)
        videoThread?.join(1_000)
        audioThread?.join(1_000)

        val file = try {
            finishFile()
        } catch (t: Throwable) {
            cleanup(failed = true)
            throw t
        }
        cleanup(failed = false)
        return file
    }

    /** Best-effort abort used when the surface/activity is destroyed. */
    fun abort() {
        NativeBridge.nativeStopVideo()
        stopRequested.set(true)
        try {
            audioRecord?.stop()
        } catch (_: Throwable) {
        }
        cleanup(failed = true)
    }

    private fun videoLoop() {
        val info = MediaCodec.BufferInfo()
        val codec = videoCodec ?: return
        while (!Thread.currentThread().isInterrupted) {
            val index = codec.dequeueOutputBuffer(info, 10_000)
            when {
                index == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                    synchronized(muxerLock) {
                        videoFormat = codec.outputFormat
                        tryStartMuxerLocked()
                    }
                }
                index >= 0 -> {
                    val buffer = codec.getOutputBuffer(index)
                    if (buffer != null && info.size > 0 &&
                        (info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) == 0
                    ) {
                        buffer.position(info.offset)
                        val data = ByteArray(info.size)
                        buffer.get(data)
                        val pts = if (videoEpochUs < 0) {
                            videoEpochUs = info.presentationTimeUs
                            0L
                        } else {
                            (info.presentationTimeUs - videoEpochUs).coerceAtLeast(0L)
                        }
                        videoLastPtsUs = pts
                        synchronized(muxerLock) {
                            queueOrWriteLocked(true, data, pts, info.flags)
                        }
                    }
                    codec.releaseOutputBuffer(index, false)
                    if ((info.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0) {
                        videoEnded = true
                        return
                    }
                }
                index == MediaCodec.INFO_TRY_AGAIN_LATER -> {
                    if (stopRequested.get()) return
                }
            }
        }
    }

    private fun audioLoop(bufferBytes: Int, initialSkipBytes: Int) {
        val codec = audioCodec ?: return
        val record = audioRecord ?: return
        val buffer = ByteArray(bufferBytes)
        var skipBytes = initialSkipBytes
        var ptsUs = 0L
        while (!stopRequested.get()) {
            var read = record.read(buffer, 0, buffer.size)
            if (read > 0 && skipBytes > 0) {
                if (read <= skipBytes) {
                    skipBytes -= read
                    continue
                }
                val remaining = read - skipBytes
                System.arraycopy(buffer, skipBytes, buffer, 0, remaining)
                skipBytes = 0
                read = remaining
            }
            if (read > 0) {
                val index = codec.dequeueInputBuffer(10_000)
                if (index >= 0) {
                    val input = codec.getInputBuffer(index)
                    if (input != null) {
                        input.clear()
                        input.put(buffer, 0, read)
                        codec.queueInputBuffer(index, 0, read, ptsUs, 0)
                        // 16-bit stereo: 4 bytes per frame, 48 kHz.
                        ptsUs += (read / 4L) * 1_000_000L / 48_000L
                    }
                }
                drainAudio(eos = false)
            } else if (read < 0) {
                break
            }
        }
        val index = codec.dequeueInputBuffer(10_000)
        if (index >= 0) {
            codec.queueInputBuffer(
                index,
                0,
                0,
                ptsUs,
                MediaCodec.BUFFER_FLAG_END_OF_STREAM,
            )
        }
        // Drain on the stop path after the last video PTS is known.
    }

    private fun drainAudio(eos: Boolean) {
        val codec = audioCodec ?: return
        val info = MediaCodec.BufferInfo()
        while (true) {
            val index = codec.dequeueOutputBuffer(info, if (eos) 10_000 else 0)
            when {
                index == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                    synchronized(muxerLock) {
                        audioFormat = codec.outputFormat
                        tryStartMuxerLocked()
                    }
                }
                index >= 0 -> {
                    val buffer = codec.getOutputBuffer(index)
                    if (buffer != null && info.size > 0 &&
                        (info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) == 0
                    ) {
                        buffer.position(info.offset)
                        val data = ByteArray(info.size)
                        buffer.get(data)
                        synchronized(muxerLock) {
                            queueOrWriteLocked(false, data, info.presentationTimeUs, info.flags)
                        }
                    }
                    codec.releaseOutputBuffer(index, false)
                    if ((info.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0) return
                }
                index == MediaCodec.INFO_TRY_AGAIN_LATER -> return
            }
        }
    }

    private fun drainVideo(eos: Boolean) {
        val codec = videoCodec ?: return
        val info = MediaCodec.BufferInfo()
        while (true) {
            val index = codec.dequeueOutputBuffer(info, if (eos) 10_000 else 0)
            when {
                index == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                    synchronized(muxerLock) {
                        videoFormat = codec.outputFormat
                        tryStartMuxerLocked()
                    }
                }
                index >= 0 -> {
                    val buffer = codec.getOutputBuffer(index)
                    if (buffer != null && info.size > 0 &&
                        (info.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) == 0
                    ) {
                        buffer.position(info.offset)
                        val data = ByteArray(info.size)
                        buffer.get(data)
                        val pts = if (videoEpochUs < 0) {
                            videoEpochUs = info.presentationTimeUs
                            0L
                        } else {
                            (info.presentationTimeUs - videoEpochUs).coerceAtLeast(0L)
                        }
                        videoLastPtsUs = pts
                        synchronized(muxerLock) {
                            queueOrWriteLocked(true, data, pts, info.flags)
                        }
                    }
                    codec.releaseOutputBuffer(index, false)
                    if ((info.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM) != 0) {
                        videoEnded = true
                        return
                    }
                }
                index == MediaCodec.INFO_TRY_AGAIN_LATER -> return
            }
        }
    }

    private fun queueOrWriteLocked(video: Boolean, data: ByteArray, ptsUs: Long, flags: Int) {
        if (muxerStarted) {
            writeLocked(video, data, ptsUs, flags)
            return
        }
        if (pending.size >= pendingLimit) {
            lastError = "muxer track formats did not arrive in time"
            return
        }
        pending.addLast(Pending(video, data, ptsUs, flags))
    }

    private fun tryStartMuxerLocked() {
        if (muxerStarted) return
        val vf = videoFormat ?: return
        val af = audioFormat ?: return
        val out = muxer ?: return
        videoTrack = out.addTrack(vf)
        audioTrack = out.addTrack(af)
        out.start()
        muxerStarted = true
        while (pending.isNotEmpty()) {
            val sample = pending.removeFirst()
            writeLocked(sample.video, sample.data, sample.ptsUs, sample.flags)
        }
    }

    private fun writeLocked(video: Boolean, data: ByteArray, ptsUs: Long, flags: Int) {
        val out = muxer ?: return
        if (!video && videoEnded && videoLastPtsUs >= 0 && ptsUs > videoLastPtsUs) {
            return
        }
        val track = if (video) videoTrack else audioTrack
        if (track < 0) return
        val info = MediaCodec.BufferInfo().apply {
            set(0, data.size, ptsUs, flags)
        }
        out.writeSampleData(track, ByteBuffer.wrap(data), info)
    }

    private fun finishFile(): File {
        val part = partFile ?: error("no partial file")
        val final = finalFile ?: error("no output file")
        if (!muxerStarted) error(lastError ?: "no muxer tracks were started")
        val out = muxer
        if (out != null) {
            try {
                out.stop()
            } catch (t: Throwable) {
                throw IllegalStateException("MediaMuxer stop failed: ${t.message}", t)
            }
        }
        check(part.exists() && part.length() > 0) { "recording is empty" }
        if (final.exists()) final.delete()
        check(part.renameTo(final)) { "could not rename ${part.name} to ${final.name}" }
        return final
    }

    private fun cleanup(failed: Boolean) {
        try {
            videoCodec?.stop()
        } catch (_: Throwable) {
        }
        try {
            audioCodec?.stop()
        } catch (_: Throwable) {
        }
        videoCodec?.release()
        audioCodec?.release()
        videoCodec = null
        audioCodec = null
        try {
            audioRecord?.release()
        } catch (_: Throwable) {
        }
        audioRecord = null
        try {
            muxer?.release()
        } catch (_: Throwable) {
        }
        muxer = null
        inputSurface?.release()
        inputSurface = null
        if (failed) {
            partFile?.delete()
        }
        partFile = null
        finalFile = null
        videoFormat = null
        audioFormat = null
        videoTrack = -1
        audioTrack = -1
        muxerStarted = false
        videoLastPtsUs = -1L
        videoEnded = false
        synchronized(muxerLock) { pending.clear() }
    }
}
