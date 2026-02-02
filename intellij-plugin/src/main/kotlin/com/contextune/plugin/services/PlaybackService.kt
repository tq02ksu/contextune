package com.contextune.plugin.services

import com.contextune.plugin.audio.AudioEngine
import com.contextune.plugin.audio.AudioEngineException
import com.contextune.plugin.audio.AudioEvent
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger

/**
 * Service for managing audio playback
 */
@Service
class PlaybackService {
    
    private val logger = Logger.getInstance(PlaybackService::class.java)
    private var audioEngine: AudioEngine? = null
    private var initialized = false
    private var currentFilePath: String? = null
    
    fun initialize() {
        if (initialized) {
            return
        }
        
        try {
            // Lazy create audio engine only when initialize is called
            if (audioEngine == null) {
                audioEngine = AudioEngine()
            }
            
            audioEngine?.initialize()
            
            // Set up callback for audio events
            audioEngine?.setCallback { event ->
                handleAudioEvent(event)
            }
            
            logger.info("PlaybackService initialized successfully")
            initialized = true
        } catch (e: Throwable) {
            logger.error("Failed to initialize audio engine", e)
            throw e
        }
    }
    
    fun shutdown() {
        if (!initialized) {
            return
        }
        
        try {
            audioEngine?.clearCallback()
            audioEngine?.shutdown()
            logger.info("PlaybackService shutdown successfully")
        } catch (e: Exception) {
            logger.error("Error during shutdown", e)
        } finally {
            initialized = false
            currentFilePath = null
        }
    }
    
    /**
     * Load an audio file for playback
     */
    fun loadFile(filePath: String) {
        ensureInitialized()
        try {
            audioEngine?.loadFile(filePath)
            currentFilePath = filePath
            logger.info("Loaded file: $filePath")
        } catch (e: AudioEngineException) {
            logger.error("Failed to load file: $filePath", e)
            throw e
        }
    }
    
    /**
     * Start or resume playback
     */
    fun play() {
        ensureInitialized()
        try {
            audioEngine?.play()
            logger.debug("Playback started")
        } catch (e: AudioEngineException) {
            logger.error("Failed to start playback", e)
            throw e
        }
    }
    
    /**
     * Pause playback
     */
    fun pause() {
        ensureInitialized()
        try {
            audioEngine?.pause()
            logger.debug("Playback paused")
        } catch (e: AudioEngineException) {
            logger.error("Failed to pause playback", e)
            throw e
        }
    }
    
    /**
     * Stop playback
     */
    fun stop() {
        ensureInitialized()
        try {
            audioEngine?.stop()
            logger.debug("Playback stopped")
        } catch (e: AudioEngineException) {
            logger.error("Failed to stop playback", e)
            throw e
        }
    }
    
    /**
     * Seek to a specific position in seconds
     */
    fun seek(positionSeconds: Double) {
        ensureInitialized()
        try {
            audioEngine?.seek(positionSeconds)
            logger.debug("Seeked to position: $positionSeconds seconds")
        } catch (e: AudioEngineException) {
            logger.error("Failed to seek", e)
            throw e
        }
    }
    
    /**
     * Set volume (0.0 to 1.0)
     */
    fun setVolume(volume: Double) {
        ensureInitialized()
        require(volume in 0.0..1.0) { "Volume must be between 0.0 and 1.0" }
        try {
            audioEngine?.setVolume(volume)
            logger.debug("Volume set to: $volume")
        } catch (e: AudioEngineException) {
            logger.error("Failed to set volume", e)
            throw e
        }
    }
    
    /**
     * Set volume with smooth ramping
     */
    fun setVolumeRamped(volume: Double, rampDurationMs: Int = 100) {
        ensureInitialized()
        require(volume in 0.0..1.0) { "Volume must be between 0.0 and 1.0" }
        try {
            audioEngine?.setVolumeRamped(volume, rampDurationMs)
            logger.debug("Volume ramped to: $volume over ${rampDurationMs}ms")
        } catch (e: AudioEngineException) {
            logger.error("Failed to set volume ramped", e)
            throw e
        }
    }
    
    /**
     * Get current volume
     */
    fun getVolume(): Double {
        ensureInitialized()
        return try {
            audioEngine?.getVolume() ?: 0.0
        } catch (e: AudioEngineException) {
            logger.error("Failed to get volume", e)
            0.0
        }
    }
    
    /**
     * Mute audio
     */
    fun mute() {
        ensureInitialized()
        try {
            audioEngine?.mute()
            logger.debug("Audio muted")
        } catch (e: AudioEngineException) {
            logger.error("Failed to mute", e)
            throw e
        }
    }
    
    /**
     * Unmute audio
     */
    fun unmute() {
        ensureInitialized()
        try {
            audioEngine?.unmute()
            logger.debug("Audio unmuted")
        } catch (e: AudioEngineException) {
            logger.error("Failed to unmute", e)
            throw e
        }
    }
    
    /**
     * Check if audio is muted
     */
    fun isMuted(): Boolean {
        ensureInitialized()
        return try {
            audioEngine?.isMuted() ?: false
        } catch (e: AudioEngineException) {
            logger.error("Failed to check mute status", e)
            false
        }
    }
    
    /**
     * Get current playback position in seconds
     */
    fun getPosition(): Double {
        ensureInitialized()
        return try {
            audioEngine?.getPosition() ?: 0.0
        } catch (e: AudioEngineException) {
            logger.error("Failed to get position", e)
            0.0
        }
    }
    
    /**
     * Get track duration in seconds
     */
    fun getDuration(): Double {
        ensureInitialized()
        return try {
            audioEngine?.getDuration() ?: 0.0
        } catch (e: AudioEngineException) {
            logger.error("Failed to get duration", e)
            0.0
        }
    }
    
    /**
     * Check if audio is currently playing
     */
    fun isPlaying(): Boolean {
        ensureInitialized()
        return try {
            audioEngine?.isPlaying() ?: false
        } catch (e: AudioEngineException) {
            logger.error("Failed to check playing status", e)
            false
        }
    }
    
    /**
     * Get the currently loaded file path
     */
    fun getCurrentFile(): String? = currentFilePath
    
    /**
     * Handle audio events from the engine
     */
    private fun handleAudioEvent(event: AudioEvent) {
        val eventType = event.getEventType()
        logger.debug("Audio event received: $eventType")
        
        when (eventType) {
            com.contextune.plugin.audio.AudioEventType.STATE_CHANGED -> {
                val state = event.getState()
                logger.info("Playback state changed to: $state")
                // TODO: Notify UI listeners
            }
            com.contextune.plugin.audio.AudioEventType.POSITION_CHANGED -> {
                // Position updates can be frequent, so only log at trace level
                logger.trace("Position changed to: ${event.position}")
                // TODO: Update UI progress bar
            }
            com.contextune.plugin.audio.AudioEventType.TRACK_ENDED -> {
                logger.info("Track ended")
                // TODO: Trigger next track or stop
            }
            com.contextune.plugin.audio.AudioEventType.ERROR -> {
                val errorMsg = event.getErrorMessage() ?: "Unknown error"
                logger.error("Audio engine error: $errorMsg")
                // TODO: Show error notification to user
            }
            com.contextune.plugin.audio.AudioEventType.BUFFER_UNDERRUN -> {
                logger.warn("Buffer underrun detected")
                // TODO: Show buffering indicator
            }
            null -> {
                logger.warn("Unknown audio event type: ${event.eventType}")
            }
        }
    }
    
    private fun ensureInitialized() {
        if (!initialized) {
            throw IllegalStateException("PlaybackService not initialized. Call initialize() first.")
        }
    }
}
