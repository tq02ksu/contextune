package com.contextune.plugin.ui

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory

/**
 * Factory for creating the music player tool window
 */
class MusicPlayerToolWindowFactory : ToolWindowFactory {
    
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val musicPlayerPanel = MusicPlayerPanel(project)
        val contentFactory = ContentFactory.getInstance()
        val content = contentFactory.createContent(musicPlayerPanel, "", false)
        
        // Store panel reference for cleanup
        content.putUserData(MUSIC_PLAYER_PANEL_KEY, musicPlayerPanel)
        
        // Add dispose handler
        content.setDisposer {
            musicPlayerPanel.dispose()
        }
        
        toolWindow.contentManager.addContent(content)
    }
    
    companion object {
        private val MUSIC_PLAYER_PANEL_KEY = com.intellij.openapi.util.Key.create<MusicPlayerPanel>("MUSIC_PLAYER_PANEL")
    }
}
