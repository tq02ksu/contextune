package com.contexture.plugin.actions

import com.contexture.plugin.services.PlaybackService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to toggle play/pause
 */
class PlayPauseAction : AnAction() {
    
    private val playbackService = ApplicationManager.getApplication().getService(PlaybackService::class.java)
    
    override fun actionPerformed(e: AnActionEvent) {
        try {
            if (playbackService.isPlaying()) {
                playbackService.pause()
            } else {
                playbackService.play()
            }
        } catch (ex: Exception) {
            // Service not initialized or other error
            // UI will handle error display
        }
    }
    
    override fun update(e: AnActionEvent) {
        // Update action text based on current state
        try {
            e.presentation.text = if (playbackService.isPlaying()) "Pause" else "Play"
        } catch (ex: Exception) {
            e.presentation.text = "Play/Pause"
        }
    }
}
