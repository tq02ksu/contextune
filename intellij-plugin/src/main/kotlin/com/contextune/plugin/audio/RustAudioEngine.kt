package com.contextune.plugin.audio

import com.contextune.plugin.utils.NativeLibraryLoader
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

/**
 * JNA wrapper for the Rust audio engine FFI
 */
interface RustAudioEngineLib : Library {
    
    companion object {
        private var _instance: RustAudioEngineLib? = null
        private var _loadAttempted = false
        private var _loadException: Exception? = null
        
        /**
         * Get the library instance, or null if loading failed
         */
        fun getInstance(): RustAudioEngineLib? {
            if (!_loadAttempted) {
                synchronized(this) {
                    if (!_loadAttempted) {
                        try {
                            // Ensure native library is loaded
                            NativeLibraryLoader.loadNativeLibrary()
                            _instance = Native.load("contextune_core", RustAudioEngineLib::class.java) as RustAudioEngineLib
                        } catch (e: Exception) {
                            _loadException = e
                            _instance = null
                        } finally {
                            _loadAttempted = true
                        }
                    }
                }
            }
            return _instance
        }
        
        /**
         * Get the exception that occurred during loading, if any
         */
        fun getLoadException(): Exception? = _loadException
        
        /**
         * Check if the library was loaded successfully
         */
        fun isLoaded(): Boolean = _instance != null
    }
    
    // Core engine functions
    fun audio_engine_create(): AudioEngineHandle
    fun audio_engine_destroy(handle: AudioEngineHandle): Int
    
    // File operations
    fun audio_engine_load_file(handle: AudioEngineHandle, filePath: String): Int
    
    // Playback control
    fun audio_engine_play(handle: AudioEngineHandle): Int
    fun audio_engine_pause(handle: AudioEngineHandle): Int
    fun audio_engine_stop(handle: AudioEngineHandle): Int
    fun audio_engine_seek(handle: AudioEngineHandle, position: Double): Int
    
    // Volume control
    fun audio_engine_set_volume(handle: AudioEngineHandle, volume: Double): Int
    fun audio_engine_set_volume_ramped(handle: AudioEngineHandle, volume: Double, rampDurationMs: Int): Int
    fun audio_engine_get_volume(handle: AudioEngineHandle, volume: DoubleArray): Int
    fun audio_engine_mute(handle: AudioEngineHandle): Int
    fun audio_engine_unmute(handle: AudioEngineHandle): Int
    fun audio_engine_is_muted(handle: AudioEngineHandle, isMuted: ByteArray): Int
    
    // State queries
    fun audio_engine_get_position(handle: AudioEngineHandle, position: DoubleArray): Int
    fun audio_engine_get_duration(handle: AudioEngineHandle, duration: DoubleArray): Int
    fun audio_engine_is_playing(handle: AudioEngineHandle, isPlaying: ByteArray): Int
    
    // Callback registration
    fun audio_engine_set_callback(handle: AudioEngineHandle, callback: AudioCallback, userData: Pointer?): Int
    fun audio_engine_clear_callback(handle: AudioEngineHandle): Int
}

/**
 * Audio engine handle (opaque pointer)
 */
@Structure.FieldOrder("inner")
class AudioEngineHandle : Structure() {
    @JvmField var inner: Pointer? = null
    
    fun isNull(): Boolean = inner == null || Pointer.nativeValue(inner) == 0L
    
    companion object {
        fun createNull(): AudioEngineHandle {
            val handle = AudioEngineHandle()
            handle.inner = null
            return handle
        }
    }
}

/**
 * FFI result codes
 */
object FFIResult {
    const val SUCCESS = 0
    const val NULL_POINTER = -1
    const val INVALID_ARGUMENT = -2
    const val OUT_OF_MEMORY = -3
    const val INTERNAL_ERROR = -4
    
    fun isSuccess(code: Int): Boolean = code == SUCCESS
    
    fun toException(code: Int, operation: String): AudioEngineException {
        val message = when (code) {
            NULL_POINTER -> "Null pointer error during $operation"
            INVALID_ARGUMENT -> "Invalid argument during $operation"
            OUT_OF_MEMORY -> "Out of memory during $operation"
            INTERNAL_ERROR -> "Internal error during $operation"
            else -> "Unknown error ($code) during $operation"
        }
        return AudioEngineException(message, code)
    }
}

/**
 * Audio event types
 */
enum class AudioEventType(val value: Int) {
    STATE_CHANGED(0),
    POSITION_CHANGED(1),
    TRACK_ENDED(2),
    ERROR(3),
    BUFFER_UNDERRUN(4);
    
    companion object {
        fun fromValue(value: Int): AudioEventType? {
            return values().find { it.value == value }
        }
    }
}

/**
 * Playback states
 */
enum class PlaybackState(val value: Int) {
    STOPPED(0),
    PLAYING(1),
    PAUSED(2),
    BUFFERING(3),
    ERROR(4);
    
    companion object {
        fun fromValue(value: Int): PlaybackState? {
            return values().find { it.value == value }
        }
    }
}

/**
 * Audio event structure
 */
@Structure.FieldOrder("eventType", "state", "position", "errorMessage")
class AudioEvent : Structure() {
    @JvmField var eventType: Int = 0
    @JvmField var state: Int = 0
    @JvmField var position: Long = 0
    @JvmField var errorMessage: Pointer? = null
    
    fun getEventType(): AudioEventType? = AudioEventType.fromValue(eventType)
    fun getState(): PlaybackState? = PlaybackState.fromValue(state)
    fun getErrorMessage(): String? {
        return errorMessage?.getString(0, "UTF-8")
    }
}

/**
 * Audio callback interface
 */
interface AudioCallback : Callback {
    fun invoke(event: AudioEvent, userData: Pointer?)
}

/**
 * Custom exception for audio engine errors
 */
class AudioEngineException(message: String, val errorCode: Int) : Exception(message)

/**
 * High-level Kotlin wrapper for the Rust audio engine
 */
class AudioEngine {
    
    private var handle: AudioEngineHandle? = null
    private val lib: RustAudioEngineLib
    private var callback: AudioCallback? = null
    
    init {
        val instance = RustAudioEngineLib.getInstance()
        if (instance == null) {
            val exception = RustAudioEngineLib.getLoadException()
            throw AudioEngineException(
                "Failed to load native audio library: ${exception?.message ?: "Unknown error"}", 
                FFIResult.INTERNAL_ERROR
            )
        }
        lib = instance
    }
    
    /**
     * Initialize the audio engine
     */
    fun initialize() {
        if (handle != null) {
            throw IllegalStateException("Audio engine already initialized")
        }
        
        handle = lib.audio_engine_create()
        if (handle?.isNull() == true) {
            throw AudioEngineException("Failed to create audio engine", FFIResult.INTERNAL_ERROR)
        }
    }
    
    /**
     * Shutdown the audio engine
     */
    fun shutdown() {
        handle?.let {
            if (!it.isNull()) {
                val result = lib.audio_engine_destroy(it)
                if (!FFIResult.isSuccess(result)) {
                    System.err.println("Warning: Failed to destroy audio engine: $result")
                }
            }
        }
        handle = null
        callback = null
    }
    
    /**
     * Load an audio file
     */
    fun loadFile(filePath: String) {
        val h = requireHandle()
        val result = lib.audio_engine_load_file(h, filePath)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "load file")
        }
    }
    
    /**
     * Start playback
     */
    fun play() {
        val h = requireHandle()
        val result = lib.audio_engine_play(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "play")
        }
    }
    
    /**
     * Pause playback
     */
    fun pause() {
        val h = requireHandle()
        val result = lib.audio_engine_pause(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "pause")
        }
    }
    
    /**
     * Stop playback
     */
    fun stop() {
        val h = requireHandle()
        val result = lib.audio_engine_stop(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "stop")
        }
    }
    
    /**
     * Seek to position (in seconds)
     */
    fun seek(positionSeconds: Double) {
        val h = requireHandle()
        val result = lib.audio_engine_seek(h, positionSeconds)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "seek")
        }
    }
    
    /**
     * Set volume (0.0 to 1.0)
     */
    fun setVolume(volume: Double) {
        require(volume in 0.0..1.0) { "Volume must be between 0.0 and 1.0" }
        val h = requireHandle()
        val result = lib.audio_engine_set_volume(h, volume)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "set volume")
        }
    }
    
    /**
     * Set volume with ramping (0.0 to 1.0)
     */
    fun setVolumeRamped(volume: Double, rampDurationMs: Int) {
        require(volume in 0.0..1.0) { "Volume must be between 0.0 and 1.0" }
        val h = requireHandle()
        val result = lib.audio_engine_set_volume_ramped(h, volume, rampDurationMs)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "set volume ramped")
        }
    }
    
    /**
     * Get current volume
     */
    fun getVolume(): Double {
        val h = requireHandle()
        val volume = DoubleArray(1)
        val result = lib.audio_engine_get_volume(h, volume)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "get volume")
        }
        return volume[0]
    }
    
    /**
     * Mute audio
     */
    fun mute() {
        val h = requireHandle()
        val result = lib.audio_engine_mute(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "mute")
        }
    }
    
    /**
     * Unmute audio
     */
    fun unmute() {
        val h = requireHandle()
        val result = lib.audio_engine_unmute(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "unmute")
        }
    }
    
    /**
     * Check if audio is muted
     */
    fun isMuted(): Boolean {
        val h = requireHandle()
        val isMuted = ByteArray(1)
        val result = lib.audio_engine_is_muted(h, isMuted)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "is muted")
        }
        return isMuted[0] != 0.toByte()
    }
    
    /**
     * Get current playback position (in seconds)
     */
    fun getPosition(): Double {
        val h = requireHandle()
        val position = DoubleArray(1)
        val result = lib.audio_engine_get_position(h, position)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "get position")
        }
        return position[0]
    }
    
    /**
     * Get track duration (in seconds)
     */
    fun getDuration(): Double {
        val h = requireHandle()
        val duration = DoubleArray(1)
        val result = lib.audio_engine_get_duration(h, duration)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "get duration")
        }
        return duration[0]
    }
    
    /**
     * Check if audio is currently playing
     */
    fun isPlaying(): Boolean {
        val h = requireHandle()
        val isPlaying = ByteArray(1)
        val result = lib.audio_engine_is_playing(h, isPlaying)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "is playing")
        }
        return isPlaying[0] != 0.toByte()
    }
    
    /**
     * Set audio event callback
     */
    fun setCallback(callback: (AudioEvent) -> Unit) {
        val h = requireHandle()
        
        // Create JNA callback
        val jnaCallback = object : AudioCallback {
            override fun invoke(event: AudioEvent, userData: Pointer?) {
                callback(event)
            }
        }
        
        val result = lib.audio_engine_set_callback(h, jnaCallback, null)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "set callback")
        }
        
        // Keep reference to prevent garbage collection
        this.callback = jnaCallback
    }
    
    /**
     * Clear audio event callback
     */
    fun clearCallback() {
        val h = requireHandle()
        val result = lib.audio_engine_clear_callback(h)
        if (!FFIResult.isSuccess(result)) {
            throw FFIResult.toException(result, "clear callback")
        }
        callback = null
    }
    
    private fun requireHandle(): AudioEngineHandle {
        val h = handle
        if (h == null || h.isNull()) {
            throw IllegalStateException("Audio engine not initialized")
        }
        return h
    }
}
