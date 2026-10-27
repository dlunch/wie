package net.dlunch.wie.audio

import android.media.AudioAttributes
import android.media.MediaDataSource
import android.media.MediaPlayer
import android.os.Handler
import android.os.HandlerThread
import android.util.Log
import androidx.annotation.Keep
import java.util.concurrent.FutureTask

@Keep
class NativeAudio {
    private val thread = HandlerThread("wie-midi").apply { start() }
    private val handler = Handler(thread.looper)
    private val players = mutableMapOf<Long, MediaPlayer>()
    private var paused = false

    private fun run(action: () -> Unit) {
        val task = FutureTask { action() }
        handler.post(task)
        task.get()
    }

    fun play(handle: Long, bytes: ByteArray, repeat: Boolean, volume: Float) = run {
        players.remove(handle)?.release()
        val player = MediaPlayer()
        try {
            player.setAudioAttributes(AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build())
            player.setDataSource(object : MediaDataSource() {
                override fun getSize(): Long = bytes.size.toLong()
                override fun readAt(position: Long, buffer: ByteArray, offset: Int, size: Int): Int {
                    if (size == 0) return 0
                    if (position >= bytes.size) return -1
                    val count = minOf(size, bytes.size - position.toInt())
                    bytes.copyInto(buffer, offset, position.toInt(), position.toInt() + count)
                    return count
                }
                override fun close() {}
            })
            player.isLooping = repeat
            player.setVolume(volume, volume)
            player.setOnCompletionListener { finished ->
                players.remove(handle)
                finished.release()
            }
            player.setOnErrorListener { failed, what, extra ->
                Log.e("wie-audio", "MIDI playback failed: $what/$extra")
                players.remove(handle)
                failed.release()
                true
            }
            player.prepare()
            players[handle] = player
            if (!paused) player.start()
        } catch (error: Exception) {
            players.remove(handle)
            player.release()
            throw error
        }
    }

    fun stop(handle: Long) = run { players.remove(handle)?.release(); Unit }

    fun volume(value: Float) = run { players.values.forEach { it.setVolume(value, value) } }

    fun pause(value: Boolean) = run {
        if (paused != value) {
            paused = value
            players.values.forEach { if (value) it.pause() else it.start() }
        }
    }

    fun close() {
        run { players.values.forEach { it.release() }; players.clear() }
        thread.quitSafely()
        thread.join()
    }
}
