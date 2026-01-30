package com.contextune.plugin.actions

import com.contextune.plugin.services.PlaybackService
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager

/**
 * Action to stop playback
 */
class StopAction : AnAction() {
    
    private val playbackService = ApplicationManager.getApplication().getService(PlaybackService::class.java)
    
    override fun actionPerformed(e: AnActionEvent) {
        try {
            playbackService.stop()
        } catch (ex: Exception) {
            // Service not initialized or other error
        }
    }
}
