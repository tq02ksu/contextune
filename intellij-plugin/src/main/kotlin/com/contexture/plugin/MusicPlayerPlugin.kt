package com.contexture.plugin

import com.contexture.plugin.services.ErrorNotificationService
import com.contexture.plugin.services.PlaybackService
import com.contexture.plugin.state.PlayerState
import com.contexture.plugin.utils.NativeLibraryLoader
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

/**
 * Plugin entry point and lifecycle manager
 */
class MusicPlayerPlugin : StartupActivity {
    
    private val logger = Logger.getInstance(MusicPlayerPlugin::class.java)
    
    override fun runActivity(project: Project) {
        logger.info("Contexture Music Player plugin starting...")
        
        try {
            // Phase 1: Load native library
            loadNativeLibrary()
            
            // Phase 2: Initialize services
            initializeServices()
            
            // Phase 3: Restore previous state
            restoreState()
            
            // Phase 4: Register shutdown hook
            registerShutdownHook()
            
            logger.info("Contexture Music Player plugin started successfully")
            
        } catch (e: Exception) {
            handleInitializationError(e, project)
        }
    }
    
    /**
     * Load native library
     */
    private fun loadNativeLibrary() {
        try {
            NativeLibraryLoader.loadNativeLibrary()
            logger.info("Native library loaded successfully")
        } catch (e: Exception) {
            logger.error("Failed to load native library", e)
            throw PluginInitializationException("Failed to load native audio library", e)
        }
    }
    
    /**
     * Initialize application services
     */
    private fun initializeServices() {
        try {
            val playbackService = service<PlaybackService>()
            playbackService.initialize()
            logger.info("Playback service initialized")
            
            // Other services will be initialized on-demand
            
        } catch (e: Exception) {
            logger.error("Failed to initialize services", e)
            throw PluginInitializationException("Failed to initialize audio services", e)
        }
    }
    
    /**
     * Restore previous state from persistent storage
     */
    private fun restoreState() {
        try {
            val state = PlayerState.getInstance()
            val playbackService = service<PlaybackService>()
            
            // Restore volume
            if (state.volume in 0.0..1.0) {
                playbackService.setVolume(state.volume)
                logger.info("Restored volume: ${state.volume}")
            }
            
            // Restore mute state
            if (state.isMuted) {
                playbackService.mute()
                logger.info("Restored mute state")
            }
            
            // Restore last file (optional - don't auto-play)
            if (state.lastFilePath.isNotEmpty()) {
                try {
                    playbackService.loadFile(state.lastFilePath)
                    logger.info("Restored last file: ${state.lastFilePath}")
                    
                    // Optionally restore position (but don't auto-play)
                    if (state.lastPosition > 0) {
                        playbackService.seek(state.lastPosition)
                        logger.info("Restored position: ${state.lastPosition}")
                    }
                } catch (e: Exception) {
                    logger.warn("Could not restore last file: ${e.message}")
                    // Clear invalid file path
                    state.lastFilePath = ""
                }
            }
            
        } catch (e: Exception) {
            logger.warn("Failed to restore state: ${e.message}", e)
            // Non-fatal - continue without restored state
        }
    }
    
    /**
     * Register shutdown hook to save state
     */
    private fun registerShutdownHook() {
        ApplicationManager.getApplication().invokeLater {
            // Add shutdown listener
            Runtime.getRuntime().addShutdownHook(Thread {
                try {
                    saveState()
                    cleanupResources()
                } catch (e: Exception) {
                    logger.error("Error during shutdown", e)
                }
            })
        }
    }
    
    /**
     * Save current state to persistent storage
     */
    private fun saveState() {
        try {
            val state = PlayerState.getInstance()
            val playbackService = service<PlaybackService>()
            
            // Save volume
            state.volume = playbackService.getVolume()
            
            // Save mute state
            state.isMuted = playbackService.isMuted()
            
            // Save current file and position
            state.lastFilePath = playbackService.getCurrentFile() ?: ""
            state.lastPosition = playbackService.getPosition()
            
            logger.info("State saved successfully")
            
        } catch (e: Exception) {
            logger.error("Failed to save state", e)
        }
    }
    
    /**
     * Cleanup resources on shutdown
     */
    private fun cleanupResources() {
        try {
            val playbackService = service<PlaybackService>()
            playbackService.shutdown()
            logger.info("Resources cleaned up successfully")
        } catch (e: Exception) {
            logger.error("Failed to cleanup resources", e)
        }
    }
    
    /**
     * Handle initialization errors
     */
    private fun handleInitializationError(e: Exception, project: Project) {
        logger.error("Plugin initialization failed", e)
        
        val errorService = service<ErrorNotificationService>()
        errorService.showError(
            "Music Player Initialization Failed",
            "Failed to initialize Contexture Music Player: ${e.message ?: "Unknown error"}. " +
                    "Some features may not be available.",
            project
        )
    }
}

/**
 * Custom exception for plugin initialization errors
 */
class PluginInitializationException(message: String, cause: Throwable? = null) : Exception(message, cause)
