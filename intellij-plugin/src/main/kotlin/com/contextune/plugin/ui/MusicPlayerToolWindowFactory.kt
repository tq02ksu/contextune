package com.contextune.plugin.ui

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.components.JBTabbedPane
import java.awt.BorderLayout
import javax.swing.JPanel

/**
 * Factory for creating the music player tool window
 */
class MusicPlayerToolWindowFactory : ToolWindowFactory {
    
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        // Create main container panel
        val mainPanel = JPanel(BorderLayout())
        
        // Create tabbed pane
        val tabbedPane = JBTabbedPane()
        
        // Create player panel
        val musicPlayerPanel = MusicPlayerPanel(project)
        tabbedPane.addTab("Player", musicPlayerPanel)
        
        // Create playlist panel
        val playlistView = PlaylistView(project)
        tabbedPane.addTab("Playlists", playlistView)
        
        // Add tabbed pane to main panel
        mainPanel.add(tabbedPane, BorderLayout.CENTER)
        
        // Create content
        val contentFactory = ContentFactory.getInstance()
        val content = contentFactory.createContent(mainPanel, "", false)
        
        // Store panel references for cleanup
        content.putUserData(MUSIC_PLAYER_PANEL_KEY, musicPlayerPanel)
        content.putUserData(PLAYLIST_VIEW_KEY, playlistView)
        
        // Add dispose handler
        content.setDisposer {
            musicPlayerPanel.dispose()
            // PlaylistView doesn't need special disposal yet
        }
        
        toolWindow.contentManager.addContent(content)
    }
    
    companion object {
        private val MUSIC_PLAYER_PANEL_KEY = com.intellij.openapi.util.Key.create<MusicPlayerPanel>("MUSIC_PLAYER_PANEL")
        private val PLAYLIST_VIEW_KEY = com.intellij.openapi.util.Key.create<PlaylistView>("PLAYLIST_VIEW")
    }
}
