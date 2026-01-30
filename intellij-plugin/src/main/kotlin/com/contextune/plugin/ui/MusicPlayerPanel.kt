package com.contextune.plugin.ui

import com.contextune.plugin.services.PlaybackService
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBPanel
import com.intellij.util.ui.JBUI
import java.awt.BorderLayout
import java.awt.FlowLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.*

/**
 * Main music player UI panel
 */
class MusicPlayerPanel(private val project: Project) : JBPanel<MusicPlayerPanel>(BorderLayout()) {
    
    private val playbackService: PlaybackService?
    private val errorService = ApplicationManager.getApplication().getService(com.contextune.plugin.services.ErrorNotificationService::class.java)
    private val playerState = com.contextune.plugin.state.PlayerState.getInstance()
    
    init {
        // Try to get playback service, but don't fail if it's not available
        playbackService = try {
            ApplicationManager.getApplication().getService(PlaybackService::class.java)
        } catch (e: Exception) {
            errorService?.logError("Failed to get PlaybackService", e)
            null
        }
        
        // Check if playback service is available
        if (playbackService == null) {
            // Show a simple message when service is not available
            setupUnavailableUI()
        } else {
            // Setup normal UI
            setupUI()
        }
    }
    
    /**
     * Setup UI when playback service is not available
     */
    private fun setupUnavailableUI() {
        val messagePanel = JPanel(GridBagLayout())
        val gbc = GridBagConstraints()
        gbc.gridx = 0
        gbc.gridy = 0
        gbc.insets = JBUI.insets(10)
        
        val messageLabel = JBLabel("<html><center>" +
                "<h2>Music Player Unavailable</h2>" +
                "<p>The native audio library could not be loaded.</p>" +
                "<p>Audio playback functionality is not available.</p>" +
                "<p>Please check the plugin installation.</p>" +
                "</center></html>", SwingConstants.CENTER)
        
        messagePanel.add(messageLabel, gbc)
        add(messagePanel, BorderLayout.CENTER)
    }
    
    // UI Components - Track Info
    private val trackTitleLabel = JBLabel("No track loaded", SwingConstants.CENTER)
    private val trackArtistLabel = JBLabel("", SwingConstants.CENTER)
    private val trackAlbumLabel = JBLabel("", SwingConstants.CENTER)
    private val statusLabel = JBLabel("Stopped", SwingConstants.CENTER)
    
    // UI Components - Progress
    private val progressBar = JSlider(0, 1000, 0)
    private val positionLabel = JBLabel("0:00")
    private val durationLabel = JBLabel("0:00")
    
    // Playback control buttons
    private val playPauseButton = JButton("Play")
    private val stopButton = JButton("Stop")
    private val previousButton = JButton("Previous")
    private val nextButton = JButton("Next")
    
    // Volume control
    private val volumeSlider = JSlider(0, 100, 75)
    private val volumeLabel = JBLabel("Volume: 75%")
    private val muteButton = JButton("Mute")
    
    private var isPlaying = false
    private var isMuted = false
    private var currentDuration = 0.0
    
    // Update timer
    private val updateTimer = Timer(100) { updatePlaybackPosition() }
    
    /**
     * Setup normal UI when playback service is available
     */
    private fun setupUI() {
        border = JBUI.Borders.empty(10)
        initializeUI()
        setupListeners()
        initializePlaybackService()
        startUpdateTimer()
    }
    
    /**
     * Check if playback service is available and show error if not
     */
    private fun checkServiceAvailable(): Boolean {
        if (playbackService == null) {
            errorService?.showWarning(
                "Playback Service Unavailable",
                "The audio playback service is not available. Please restart the IDE or reinstall the plugin.",
                project
            )
            return false
        }
        return true
    }
    
    private fun initializeUI() {
        // Main layout with sections
        add(createTopPanel(), BorderLayout.NORTH)
        add(createProgressPanel(), BorderLayout.CENTER)
        add(createBottomPanel(), BorderLayout.SOUTH)
    }
    
    /**
     * Top panel with track info and status
     */
    private fun createTopPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(GridBagLayout())
        panel.border = JBUI.Borders.empty(0, 0, 10, 0)
        
        val gbc = GridBagConstraints()
        gbc.gridx = 0
        gbc.gridy = 0
        gbc.weightx = 1.0
        gbc.fill = GridBagConstraints.HORIZONTAL
        gbc.insets = JBUI.insets(3)
        
        // Track title
        trackTitleLabel.font = trackTitleLabel.font.deriveFont(16f).deriveFont(java.awt.Font.BOLD)
        panel.add(trackTitleLabel, gbc)
        
        // Artist
        gbc.gridy = 1
        trackArtistLabel.font = trackArtistLabel.font.deriveFont(13f)
        panel.add(trackArtistLabel, gbc)
        
        // Album
        gbc.gridy = 2
        trackAlbumLabel.font = trackAlbumLabel.font.deriveFont(12f)
        panel.add(trackAlbumLabel, gbc)
        
        // Status
        gbc.gridy = 3
        statusLabel.font = statusLabel.font.deriveFont(11f)
        panel.add(statusLabel, gbc)
        
        return panel
    }
    
    /**
     * Center panel with progress bar and time display
     */
    private fun createProgressPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        panel.border = JBUI.Borders.empty(10, 0, 10, 0)
        
        // Time labels panel
        val timePanel = JBPanel<JBPanel<*>>(BorderLayout())
        positionLabel.font = positionLabel.font.deriveFont(11f)
        durationLabel.font = durationLabel.font.deriveFont(11f)
        timePanel.add(positionLabel, BorderLayout.WEST)
        timePanel.add(durationLabel, BorderLayout.EAST)
        
        // Progress bar
        progressBar.paintTicks = false
        progressBar.paintLabels = false
        progressBar.toolTipText = "Click to seek"
        progressBar.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                seekToPosition(e)
            }
        })
        
        // Control buttons panel
        val controlPanel = createControlPanel()
        
        // Assemble
        panel.add(timePanel, BorderLayout.NORTH)
        panel.add(progressBar, BorderLayout.CENTER)
        panel.add(controlPanel, BorderLayout.SOUTH)
        
        return panel
    }
    
    /**
     * Control panel with playback buttons
     */
    private fun createControlPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(FlowLayout(FlowLayout.CENTER, 10, 10))
        
        // Configure buttons
        previousButton.toolTipText = "Previous Track (Ctrl+Alt+B)"
        playPauseButton.toolTipText = "Play/Pause (Ctrl+Alt+P)"
        stopButton.toolTipText = "Stop (Ctrl+Alt+S)"
        nextButton.toolTipText = "Next Track (Ctrl+Alt+N)"
        
        // Add buttons in order
        panel.add(previousButton)
        panel.add(playPauseButton)
        panel.add(stopButton)
        panel.add(nextButton)
        
        return panel
    }
    
    /**
     * Bottom panel with volume control
     */
    private fun createBottomPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        panel.border = JBUI.Borders.empty(10, 0, 0, 0)
        
        // Volume label
        val labelPanel = JBPanel<JBPanel<*>>(FlowLayout(FlowLayout.LEFT))
        labelPanel.add(volumeLabel)
        panel.add(labelPanel, BorderLayout.NORTH)
        
        // Volume slider and mute button
        val controlPanel = JBPanel<JBPanel<*>>(BorderLayout(5, 0))
        
        volumeSlider.majorTickSpacing = 25
        volumeSlider.minorTickSpacing = 5
        volumeSlider.paintTicks = true
        volumeSlider.paintLabels = true
        volumeSlider.toolTipText = "Volume (Ctrl+Alt+↑/↓)"
        
        muteButton.toolTipText = "Mute/Unmute (Ctrl+Alt+M)"
        
        controlPanel.add(volumeSlider, BorderLayout.CENTER)
        controlPanel.add(muteButton, BorderLayout.EAST)
        
        panel.add(controlPanel, BorderLayout.CENTER)
        
        return panel
    }
    
    /**
     * Set up event listeners for UI components
     */
    private fun setupListeners() {
        // Play/Pause button
        playPauseButton.addActionListener {
            togglePlayPause()
        }
        
        // Stop button
        stopButton.addActionListener {
            stop()
        }
        
        // Previous button
        previousButton.addActionListener {
            previous()
        }
        
        // Next button
        nextButton.addActionListener {
            next()
        }
        
        // Volume slider
        volumeSlider.addChangeListener {
            if (!volumeSlider.valueIsAdjusting) {
                val volume = volumeSlider.value / 100.0
                setVolume(volume)
            }
        }
        
        // Mute button
        muteButton.addActionListener {
            toggleMute()
        }
    }
    
    /**
     * Initialize playback service
     */
    private fun initializePlaybackService() {
        if (playbackService == null) {
            return
        }
        
        try {
            playbackService?.initialize()
            
            // Restore volume from state
            val savedVolume = playerState.volume
            if (savedVolume in 0.0..1.0) {
                volumeSlider.value = (savedVolume * 100).toInt()
                playbackService?.setVolume(savedVolume)
            } else {
                // Use slider default
                val initialVolume = volumeSlider.value / 100.0
                playbackService?.setVolume(initialVolume)
            }
            
            // Restore mute state
            if (playerState.isMuted) {
                playbackService?.mute()
                isMuted = true
                muteButton.text = "Unmute"
                volumeLabel.text = "Volume: Muted"
            }
            
            updateStatus("Ready")
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}")
            errorService?.showError(
                "Initialization Error",
                "Failed to initialize audio engine: ${e.message}",
                project
            )
        }
    }
    
    /**
     * Start update timer for position updates
     */
    private fun startUpdateTimer() {
        updateTimer.start()
    }
    
    /**
     * Update playback position display
     */
    private fun updatePlaybackPosition() {
        if (!isPlaying || playbackService == null) return
        
        try {
            val position = playbackService?.getPosition() ?: return
            val duration = playbackService?.getDuration() ?: return
            
            if (duration > 0) {
                currentDuration = duration
                val progress = ((position / duration) * 1000).toInt().coerceIn(0, 1000)
                
                SwingUtilities.invokeLater {
                    progressBar.value = progress
                    positionLabel.text = formatTime(position)
                    durationLabel.text = formatTime(duration)
                }
            }
        } catch (e: Exception) {
            // Ignore errors during position updates
        }
    }
    
    /**
     * Seek to position based on mouse click
     */
    private fun seekToPosition(e: MouseEvent) {
        if (!checkServiceAvailable() || currentDuration <= 0) return
        
        val percent = e.x.toDouble() / progressBar.width
        val seekPosition = percent * currentDuration
        
        try {
            playbackService?.seek(seekPosition)
            updateStatus("Seeked to ${formatTime(seekPosition)}")
        } catch (ex: Exception) {
            showError("Seek Error", ex.message ?: "Unknown error")
        }
    }
    
    /**
     * Format time in seconds to MM:SS
     */
    private fun formatTime(seconds: Double): String {
        val totalSeconds = seconds.toInt()
        val minutes = totalSeconds / 60
        val secs = totalSeconds % 60
        return String.format("%d:%02d", minutes, secs)
    }
    
    /**
     * Toggle play/pause
     */
    private fun togglePlayPause() {
        if (!checkServiceAvailable()) return
        
        try {
            if (isPlaying) {
                playbackService?.pause()
                isPlaying = false
                playPauseButton.text = "Play"
                updateStatus("Paused")
            } else {
                playbackService?.play()
                isPlaying = true
                playPauseButton.text = "Pause"
                updateStatus("Playing")
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}")
            showError("Playback Error", e.message ?: "Unknown error")
        }
    }
    
    /**
     * Stop playback
     */
    private fun stop() {
        if (!checkServiceAvailable()) return
        
        try {
            playbackService?.stop()
            isPlaying = false
            playPauseButton.text = "Play"
            updateStatus("Stopped")
            
            // Reset progress
            SwingUtilities.invokeLater {
                progressBar.value = 0
                positionLabel.text = "0:00"
            }
        } catch (e: Exception) {
            updateStatus("Error: ${e.message}")
            showError("Stop Error", e.message ?: "Unknown error")
        }
    }
    
    /**
     * Play previous track
     */
    private fun previous() {
        // TODO: Implement playlist navigation in Phase 4
        updateStatus("Previous track - Not implemented yet")
    }
    
    /**
     * Play next track
     */
    private fun next() {
        // TODO: Implement playlist navigation in Phase 4
        updateStatus("Next track - Not implemented yet")
    }
    
    /**
     * Set volume
     */
    private fun setVolume(volume: Double) {
        if (!checkServiceAvailable()) return
        
        try {
            playbackService?.setVolumeRamped(volume, 50) // 50ms ramp
            val percentage = (volume * 100).toInt()
            volumeLabel.text = "Volume: $percentage%"
            
            // Update mute button if volume is 0
            if (volume == 0.0 && !isMuted) {
                muteButton.text = "Unmute"
            } else if (volume > 0.0 && !isMuted) {
                muteButton.text = "Mute"
            }
        } catch (e: Exception) {
            showError("Volume Error", e.message ?: "Unknown error")
        }
    }
    
    /**
     * Toggle mute
     */
    private fun toggleMute() {
        if (!checkServiceAvailable()) return
        
        try {
            if (isMuted) {
                playbackService?.unmute()
                isMuted = false
                muteButton.text = "Mute"
                
                // Update slider to current volume
                val volume = playbackService?.getVolume() ?: 0.75
                volumeSlider.value = (volume * 100).toInt()
                volumeLabel.text = "Volume: ${(volume * 100).toInt()}%"
            } else {
                playbackService?.mute()
                isMuted = true
                muteButton.text = "Unmute"
                volumeLabel.text = "Volume: Muted"
            }
        } catch (e: Exception) {
            showError("Mute Error", e.message ?: "Unknown error")
        }
    }
    
    /**
     * Load a file for playback
     */
    fun loadFile(filePath: String) {
        if (!checkServiceAvailable()) return
        
        try {
            playbackService?.loadFile(filePath)
            
            // Extract filename and set as title
            val fileName = filePath.substringAfterLast('/')
            trackTitleLabel.text = fileName
            
            // Try to parse metadata from filename (basic implementation)
            // Format: Artist - Title.ext or just Title.ext
            val nameWithoutExt = fileName.substringBeforeLast('.')
            if (nameWithoutExt.contains(" - ")) {
                val parts = nameWithoutExt.split(" - ", limit = 2)
                trackArtistLabel.text = parts[0]
                trackTitleLabel.text = parts[1]
            } else {
                trackTitleLabel.text = nameWithoutExt
                trackArtistLabel.text = ""
            }
            
            trackAlbumLabel.text = "" // Will be populated with real metadata in Phase 5
            
            // Get duration
            currentDuration = playbackService?.getDuration() ?: 0.0
            durationLabel.text = formatTime(currentDuration)
            positionLabel.text = "0:00"
            progressBar.value = 0
            
            updateStatus("Loaded")
        } catch (e: Exception) {
            updateStatus("Error loading file")
            showError("Load Error", "Failed to load file: ${e.message}")
        }
    }
    
    /**
     * Update track metadata display
     */
    fun updateTrackMetadata(title: String, artist: String = "", album: String = "") {
        SwingUtilities.invokeLater {
            trackTitleLabel.text = title.ifEmpty { "Unknown Title" }
            trackArtistLabel.text = artist
            trackAlbumLabel.text = album
        }
    }
    
    /**
     * Update status label
     */
    private fun updateStatus(status: String) {
        SwingUtilities.invokeLater {
            statusLabel.text = status
        }
    }
    
    /**
     * Show error dialog
     */
    private fun showError(title: String, message: String) {
        SwingUtilities.invokeLater {
            JOptionPane.showMessageDialog(
                this,
                message,
                title,
                JOptionPane.ERROR_MESSAGE
            )
        }
    }
    
    /**
     * Cleanup when panel is disposed
     */
    fun dispose() {
        updateTimer.stop()
        
        if (playbackService == null) return
        
        // Save current state
        try {
            playerState.volume = playbackService?.getVolume() ?: 0.75
            playerState.isMuted = playbackService?.isMuted() ?: false
            playerState.lastFilePath = playbackService?.getCurrentFile() ?: ""
            playerState.lastPosition = playbackService?.getPosition() ?: 0.0
        } catch (e: Exception) {
            errorService?.logError("Failed to save player state", e)
        }
        
        // Shutdown service
        try {
            playbackService?.shutdown()
        } catch (e: Exception) {
            errorService?.logError("Error during shutdown", e)
        }
    }
}
