package com.contexture.plugin

import com.contexture.plugin.services.PlaybackService
import com.contexture.plugin.utils.NativeLibraryLoader
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity

/**
 * Plugin entry point and lifecycle manager
 */
class MusicPlayerPlugin : StartupActivity {
    
    override fun runActivity(project: Project) {
        // Load native library on plugin startup
        try {
            NativeLibraryLoader.loadNativeLibrary()
            
            // Initialize services
            val playbackService = service<PlaybackService>()
            playbackService.initialize()
            
        } catch (e: Exception) {
            // Log error but don't crash the IDE
            System.err.println("Failed to initialize Contexture Music Player: ${e.message}")
            e.printStackTrace()
        }
    }
}
