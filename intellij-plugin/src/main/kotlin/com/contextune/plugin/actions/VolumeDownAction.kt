package com.contextune.plugin.actions

import com.contextune.plugin.services.PlaybackService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to decrease volume
 */
class VolumeDownAction : AnAction() {
    
    private val playbackService = ApplicationManager.getApplication().getService(PlaybackService::class.java)
    
    override fun actionPerformed(e: AnActionEvent) {
        try {
            val currentVolume = playbackService.getVolume()
            val newVolume = (currentVolume - 0.1).coerceAtLeast(0.0)
            playbackService.setVolumeRamped(newVolume, 50)
        } catch (ex: Exception) {
            // Service not initialized or other error
        }
    }
}
