package com.contextune.plugin

import com.contextune.plugin.services.ErrorNotificationService
import com.contextune.plugin.services.PlaybackService
import com.contextune.plugin.state.PlayerState
import com.contextune.plugin.utils.NativeLibraryLoader
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
        logger.info("Contextune Music Player plugin starting...")
        
        var nativeLibraryLoaded = false
        
        try {
            // Phase 1: Try to load native library (non-fatal if fails)
            nativeLibraryLoaded = loadNativeLibrary()
            
            // Phase 2: Initialize services (may have limited functionality without native lib)
            initializeServices(nativeLibraryLoaded)
            
            // Phase 3: Restore previous state (only if native lib loaded)
            if (nativeLibraryLoaded) {
                restoreState()
            }
            
            // Phase 4: Register shutdown hook
            registerShutdownHook()
            
            if (nativeLibraryLoaded) {
                logger.info("Contextune Music Player plugin started successfully")
            } else {
                logger.warn("Contextune Music Player plugin started with limited functionality (native library not loaded)")
                showNativeLibraryWarning(project)
            }
            
        } catch (e: Exception) {
            handleInitializationError(e, project)
        }
    }
    
    /**
     * Load native library
     * @return true if loaded successfully, false otherwise
     */
    private fun loadNativeLibrary(): Boolean {
        return try {
            NativeLibraryLoader.loadNativeLibrary()
            logger.info("Native library loaded successfully")
            true
        } catch (e: Exception) {
            logger.error("Failed to load native library - plugin will run with limited functionality", e)
            false
        }
    }
    
    /**
     * Show warning about native library not being loaded
     */
    private fun showNativeLibraryWarning(project: Project) {
        try {
            val errorService = service<ErrorNotificationService>()
            errorService.showWarning(
                "Music Player: Native Library Not Loaded",
                "The native audio library could not be loaded. The UI is available but audio playback will not work. " +
                        "Please check the plugin installation and ensure the native library is present.",
                project
            )
        } catch (e: Exception) {
            logger.error("Failed to show native library warning", e)
        }
    }
    
    /**
     * Initialize application services
     * @param nativeLibraryLoaded whether the native library was loaded successfully
     */
    private fun initializeServices(nativeLibraryLoaded: Boolean) {
        try {
            if (nativeLibraryLoaded) {
                val playbackService = service<PlaybackService>()
                playbackService.initialize()
                logger.info("Playback service initialized")
            } else {
                logger.info("Skipping playback service initialization (native library not loaded)")
            }
            
            // Other services will be initialized on-demand
            
        } catch (e: Exception) {
            logger.error("Failed to initialize services", e)
            // Don't throw - allow plugin to continue with limited functionality
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
            "Failed to initialize Contextune Music Player: ${e.message ?: "Unknown error"}. " +
                    "Some features may not be available.",
            project
        )
    }
}

/**
 * Custom exception for plugin initialization errors
 */
class PluginInitializationException(message: String, cause: Throwable? = null) : Exception(message, cause)
