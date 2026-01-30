package com.contextune.plugin.actions

import com.contextune.plugin.services.PlaybackService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to toggle mute
 */
class MuteAction : AnAction() {
    
    private val playbackService = ApplicationManager.getApplication().getService(PlaybackService::class.java)
    
    override fun actionPerformed(e: AnActionEvent) {
        try {
            if (playbackService.isMuted()) {
                playbackService.unmute()
            } else {
                playbackService.mute()
            }
        } catch (ex: Exception) {
            // Service not initialized or other error
        }
    }
    
    override fun update(e: AnActionEvent) {
        // Update action text based on current state
        try {
            e.presentation.text = if (playbackService.isMuted()) "Unmute" else "Mute"
        } catch (ex: Exception) {
            e.presentation.text = "Mute/Unmute"
        }
    }
}
