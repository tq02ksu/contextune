package com.contextune.plugin.ui

import com.contextune.plugin.services.PlaylistService
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBList
import com.intellij.ui.components.JBPanel
import com.intellij.ui.components.JBScrollPane
import com.intellij.util.ui.JBUI
import java.awt.BorderLayout
import java.awt.FlowLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.*

/**
 * Playlist view component for displaying and managing playlists
 */
class PlaylistView(private val project: Project) : JBPanel<PlaylistView>(BorderLayout()) {
    
    private val playlistService: PlaylistService?
    private val errorService = ApplicationManager.getApplication().getService(com.contextune.plugin.services.ErrorNotificationService::class.java)
    
    // UI Components
    private lateinit var playlistListPanel: JPanel
    private lateinit var trackListPanel: JPanel
    private lateinit var playlistList: JBList<PlaylistItem>
    private lateinit var trackList: JBList<TrackItem>
    private lateinit var playlistListModel: DefaultListModel<PlaylistItem>
    private lateinit var trackListModel: DefaultListModel<TrackItem>
    
    // Control buttons
    private lateinit var newPlaylistButton: JButton
    private lateinit var deletePlaylistButton: JButton
    private lateinit var addTracksButton: JButton
    private lateinit var removeTracksButton: JButton
    
    // Current state
    private var currentPlaylistId: String? = null
    private var selectedPlaylistItem: PlaylistItem? = null
    
    init {
        // Try to get playlist service
        playlistService = try {
            ApplicationManager.getApplication().getService(PlaylistService::class.java)
        } catch (e: Exception) {
            errorService?.logError("Failed to get PlaylistService", e)
            null
        }
        
        initializeComponents()
        setupUI()
        setupListeners()
        loadPlaylists()
    }
    
    /**
     * Initialize UI components
     */
    private fun initializeComponents() {
        // List models
        playlistListModel = DefaultListModel()
        trackListModel = DefaultListModel()
        
        // Lists
        playlistList = JBList(playlistListModel)
        trackList = JBList(trackListModel)
        
        // Buttons
        newPlaylistButton = JButton("New Playlist")
        deletePlaylistButton = JButton("Delete")
        addTracksButton = JButton("Add Tracks")
        removeTracksButton = JButton("Remove")
        
        // Initially disable buttons that require selection
        deletePlaylistButton.isEnabled = false
        addTracksButton.isEnabled = false
        removeTracksButton.isEnabled = false
    }
    
    /**
     * Setup the main UI layout
     */
    private fun setupUI() {
        border = JBUI.Borders.empty(5)
        
        // Create split pane for playlists and tracks
        val splitPane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
        splitPane.leftComponent = createPlaylistPanel()
        splitPane.rightComponent = createTrackPanel()
        splitPane.dividerLocation = 250
        splitPane.resizeWeight = 0.3
        
        add(splitPane, BorderLayout.CENTER)
    }
    
    /**
     * Create the playlist list panel (left side)
     */
    private fun createPlaylistPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        panel.border = JBUI.Borders.empty(5)
        
        // Header
        val headerPanel = JBPanel<JBPanel<*>>(BorderLayout())
        val titleLabel = JBLabel("Playlists")
        titleLabel.font = titleLabel.font.deriveFont(14f).deriveFont(java.awt.Font.BOLD)
        headerPanel.add(titleLabel, BorderLayout.WEST)
        
        // Playlist controls
        val controlPanel = JBPanel<JBPanel<*>>(FlowLayout(FlowLayout.RIGHT, 5, 0))
        newPlaylistButton.toolTipText = "Create a new playlist"
        deletePlaylistButton.toolTipText = "Delete selected playlist"
        controlPanel.add(newPlaylistButton)
        controlPanel.add(deletePlaylistButton)
        headerPanel.add(controlPanel, BorderLayout.EAST)
        
        // Playlist list
        playlistList.selectionMode = ListSelectionModel.SINGLE_SELECTION
        playlistList.cellRenderer = PlaylistCellRenderer()
        val scrollPane = JBScrollPane(playlistList)
        scrollPane.preferredSize = java.awt.Dimension(240, 300)
        
        panel.add(headerPanel, BorderLayout.NORTH)
        panel.add(scrollPane, BorderLayout.CENTER)
        
        return panel
    }
    
    /**
     * Create the track list panel (right side)
     */
    private fun createTrackPanel(): JPanel {
        val panel = JBPanel<JBPanel<*>>(BorderLayout())
        panel.border = JBUI.Borders.empty(5)
        
        // Header
        val headerPanel = JBPanel<JBPanel<*>>(BorderLayout())
        val titleLabel = JBLabel("Tracks")
        titleLabel.font = titleLabel.font.deriveFont(14f).deriveFont(java.awt.Font.BOLD)
        headerPanel.add(titleLabel, BorderLayout.WEST)
        
        // Track controls
        val controlPanel = JBPanel<JBPanel<*>>(FlowLayout(FlowLayout.RIGHT, 5, 0))
        addTracksButton.toolTipText = "Add tracks to playlist"
        removeTracksButton.toolTipText = "Remove selected tracks"
        controlPanel.add(addTracksButton)
        controlPanel.add(removeTracksButton)
        headerPanel.add(controlPanel, BorderLayout.EAST)
        
        // Track list
        trackList.selectionMode = ListSelectionModel.MULTIPLE_INTERVAL_SELECTION
        trackList.cellRenderer = TrackCellRenderer()
        val scrollPane = JBScrollPane(trackList)
        
        panel.add(headerPanel, BorderLayout.NORTH)
        panel.add(scrollPane, BorderLayout.CENTER)
        
        return panel
    }
    
    /**
     * Setup event listeners
     */
    private fun setupListeners() {
        // Playlist selection
        playlistList.addListSelectionListener { e ->
            if (!e.valueIsAdjusting) {
                val selectedIndex = playlistList.selectedIndex
                if (selectedIndex >= 0) {
                    selectedPlaylistItem = playlistListModel.getElementAt(selectedIndex)
                    currentPlaylistId = selectedPlaylistItem?.id
                    loadTracksForPlaylist(currentPlaylistId)
                    deletePlaylistButton.isEnabled = true
                    addTracksButton.isEnabled = true
                } else {
                    selectedPlaylistItem = null
                    currentPlaylistId = null
                    trackListModel.clear()
                    deletePlaylistButton.isEnabled = false
                    addTracksButton.isEnabled = false
                }
            }
        }
        
        // Track selection
        trackList.addListSelectionListener { e ->
            if (!e.valueIsAdjusting) {
                removeTracksButton.isEnabled = trackList.selectedIndices.isNotEmpty()
            }
        }
        
        // Double-click to play track
        trackList.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                if (e.clickCount == 2) {
                    val selectedIndex = trackList.selectedIndex
                    if (selectedIndex >= 0) {
                        val trackItem = trackListModel.getElementAt(selectedIndex)
                        playTrack(trackItem)
                    }
                }
            }
            
            override fun mousePressed(e: MouseEvent) {
                if (e.isPopupTrigger) {
                    showTrackContextMenu(e)
                }
            }
            
            override fun mouseReleased(e: MouseEvent) {
                if (e.isPopupTrigger) {
                    showTrackContextMenu(e)
                }
            }
        })
        
        // Playlist context menu
        playlistList.addMouseListener(object : MouseAdapter() {
            override fun mousePressed(e: MouseEvent) {
                if (e.isPopupTrigger) {
                    showPlaylistContextMenu(e)
                }
            }
            
            override fun mouseReleased(e: MouseEvent) {
                if (e.isPopupTrigger) {
                    showPlaylistContextMenu(e)
                }
            }
        })
        
        // Setup drag and drop for track reordering
        setupDragAndDrop()
        
        // Button listeners
        newPlaylistButton.addActionListener { createNewPlaylist() }
        deletePlaylistButton.addActionListener { deleteSelectedPlaylist() }
        addTracksButton.addActionListener { addTracksToPlaylist() }
        removeTracksButton.addActionListener { removeSelectedTracks() }
    }
    
    /**
     * Setup drag and drop for track reordering
     */
    private fun setupDragAndDrop() {
        val dragDropHandler = PlaylistDragDropHandler(
            trackList,
            trackListModel
        ) { fromIndex, toIndex ->
            // Handle track reordering
            onTrackReordered(fromIndex, toIndex)
        }
    }
    
    /**
     * Handle track reordering
     */
    private fun onTrackReordered(fromIndex: Int, toIndex: Int) {
        try {
            // TODO: Update playlist order in the service
            showStatus("Moved track from position ${fromIndex + 1} to ${toIndex + 1}")
            
        } catch (e: Exception) {
            errorService?.logError("Failed to reorder tracks", e)
            showError("Reorder Error", "Failed to reorder tracks: ${e.message}")
        }
    }
    
    /**
     * Show context menu for tracks
     */
    private fun showTrackContextMenu(e: MouseEvent) {
        val selectedIndices = trackList.selectedIndices
        if (selectedIndices.isEmpty()) return
        
        val popupMenu = JPopupMenu()
        
        // Play track
        if (selectedIndices.size == 1) {
            val playItem = JMenuItem("Play")
            playItem.addActionListener {
                val trackItem = trackListModel.getElementAt(selectedIndices[0])
                playTrack(trackItem)
            }
            popupMenu.add(playItem)
            popupMenu.addSeparator()
        }
        
        // Queue tracks
        val queueItem = JMenuItem("Add to Queue")
        queueItem.addActionListener {
            val tracks = selectedIndices.map { trackListModel.getElementAt(it) }
            queueTracks(tracks)
        }
        popupMenu.add(queueItem)
        
        popupMenu.addSeparator()
        
        // Remove from playlist
        val removeItem = JMenuItem("Remove from Playlist")
        removeItem.addActionListener { removeSelectedTracks() }
        popupMenu.add(removeItem)
        
        // Show track info
        if (selectedIndices.size == 1) {
            popupMenu.addSeparator()
            val infoItem = JMenuItem("Track Information")
            infoItem.addActionListener {
                val trackItem = trackListModel.getElementAt(selectedIndices[0])
                showTrackInfo(trackItem)
            }
            popupMenu.add(infoItem)
        }
        
        popupMenu.show(trackList, e.x, e.y)
    }
    
    /**
     * Show context menu for playlists
     */
    private fun showPlaylistContextMenu(e: MouseEvent) {
        val selectedIndex = playlistList.selectedIndex
        if (selectedIndex < 0) return
        
        val playlistItem = playlistListModel.getElementAt(selectedIndex)
        if (playlistItem.id.isEmpty()) return // Skip placeholder items
        
        val popupMenu = JPopupMenu()
        
        // Play playlist
        val playItem = JMenuItem("Play Playlist")
        playItem.addActionListener { playPlaylist(playlistItem) }
        popupMenu.add(playItem)
        
        // Queue playlist
        val queueItem = JMenuItem("Add to Queue")
        queueItem.addActionListener { queuePlaylist(playlistItem) }
        popupMenu.add(queueItem)
        
        popupMenu.addSeparator()
        
        // Rename playlist
        val renameItem = JMenuItem("Rename")
        renameItem.addActionListener { renamePlaylist(playlistItem) }
        popupMenu.add(renameItem)
        
        // Delete playlist
        val deleteItem = JMenuItem("Delete")
        deleteItem.addActionListener { deleteSelectedPlaylist() }
        popupMenu.add(deleteItem)
        
        popupMenu.addSeparator()
        
        // Export playlist
        val exportItem = JMenuItem("Export...")
        exportItem.addActionListener { exportPlaylist(playlistItem) }
        popupMenu.add(exportItem)
        
        popupMenu.show(playlistList, e.x, e.y)
    }
    
    /**
     * Load playlists from the service
     */
    private fun loadPlaylists() {
        if (playlistService == null) {
            showPlaceholderMessage("Playlist service not available")
            return
        }
        
        try {
            // TODO: Implement actual playlist loading when service is ready
            // For now, show placeholder playlists
            playlistListModel.clear()
            
            // Add some sample playlists for UI testing
            playlistListModel.addElement(PlaylistItem("1", "Coding Focus", "High-energy tracks for coding", 12))
            playlistListModel.addElement(PlaylistItem("2", "Relaxing Classical", "Calm classical pieces", 8))
            playlistListModel.addElement(PlaylistItem("3", "Favorites", "My favorite tracks", 25))
            
        } catch (e: Exception) {
            errorService?.logError("Failed to load playlists", e)
            showPlaceholderMessage("Error loading playlists: ${e.message}")
        }
    }
    
    /**
     * Load tracks for the selected playlist
     */
    private fun loadTracksForPlaylist(playlistId: String?) {
        trackListModel.clear()
        
        if (playlistId == null || playlistService == null) {
            return
        }
        
        try {
            // TODO: Implement actual track loading when service is ready
            // For now, show placeholder tracks based on playlist
            when (playlistId) {
                "1" -> {
                    trackListModel.addElement(TrackItem("t1", "Vivaldi - Four Seasons: Summer", "Antonio Vivaldi", "Classical Masterpieces", "4:32"))
                    trackListModel.addElement(TrackItem("t2", "Two Steps From Hell - Heart of Courage", "Two Steps From Hell", "Invincible", "3:45"))
                    trackListModel.addElement(TrackItem("t3", "Hans Zimmer - Time", "Hans Zimmer", "Inception Soundtrack", "4:35"))
                }
                "2" -> {
                    trackListModel.addElement(TrackItem("t4", "Moonlight Sonata", "Ludwig van Beethoven", "Piano Sonatas", "14:52"))
                    trackListModel.addElement(TrackItem("t5", "Clair de Lune", "Claude Debussy", "Suite Bergamasque", "5:23"))
                    trackListModel.addElement(TrackItem("t6", "Gymnopédie No. 1", "Erik Satie", "Gymnopédies", "3:58"))
                }
                "3" -> {
                    trackListModel.addElement(TrackItem("t7", "Bohemian Rhapsody", "Queen", "A Night at the Opera", "5:55"))
                    trackListModel.addElement(TrackItem("t8", "Stairway to Heaven", "Led Zeppelin", "Led Zeppelin IV", "8:02"))
                    trackListModel.addElement(TrackItem("t9", "Hotel California", "Eagles", "Hotel California", "6:30"))
                }
            }
            
        } catch (e: Exception) {
            errorService?.logError("Failed to load tracks for playlist $playlistId", e)
        }
    }
    
    /**
     * Show placeholder message when no data is available
     */
    private fun showPlaceholderMessage(message: String) {
        playlistListModel.clear()
        playlistListModel.addElement(PlaylistItem("", message, "", 0))
    }
    
    /**
     * Create a new playlist
     */
    private fun createNewPlaylist() {
        val name = JOptionPane.showInputDialog(
            this,
            "Enter playlist name:",
            "New Playlist",
            JOptionPane.PLAIN_MESSAGE
        )
        
        if (name != null && name.trim().isNotEmpty()) {
            try {
                // TODO: Implement actual playlist creation when service is ready
                val newId = "new_${System.currentTimeMillis()}"
                val newPlaylist = PlaylistItem(newId, name.trim(), "Created ${java.time.LocalDateTime.now()}", 0)
                playlistListModel.addElement(newPlaylist)
                
                // Select the new playlist
                val index = playlistListModel.size() - 1
                playlistList.selectedIndex = index
                
                showStatus("Created playlist: ${name.trim()}")
                
            } catch (e: Exception) {
                errorService?.logError("Failed to create playlist", e)
                showError("Create Playlist Error", "Failed to create playlist: ${e.message}")
            }
        }
    }
    
    /**
     * Delete the selected playlist
     */
    private fun deleteSelectedPlaylist() {
        val selectedItem = selectedPlaylistItem ?: return
        
        val result = JOptionPane.showConfirmDialog(
            this,
            "Are you sure you want to delete playlist '${selectedItem.name}'?",
            "Delete Playlist",
            JOptionPane.YES_NO_OPTION,
            JOptionPane.WARNING_MESSAGE
        )
        
        if (result == JOptionPane.YES_OPTION) {
            try {
                // TODO: Implement actual playlist deletion when service is ready
                playlistListModel.removeElement(selectedItem)
                trackListModel.clear()
                currentPlaylistId = null
                selectedPlaylistItem = null
                
                showStatus("Deleted playlist: ${selectedItem.name}")
                
            } catch (e: Exception) {
                errorService?.logError("Failed to delete playlist", e)
                showError("Delete Playlist Error", "Failed to delete playlist: ${e.message}")
            }
        }
    }
    
    /**
     * Add tracks to the current playlist
     */
    private fun addTracksToPlaylist() {
        if (currentPlaylistId == null) return
        
        // TODO: Implement file chooser for adding tracks
        // For now, show a placeholder dialog
        JOptionPane.showMessageDialog(
            this,
            "Add tracks functionality will be implemented when the library scanner is ready.\n" +
                    "This will allow you to browse and select audio files to add to the playlist.",
            "Add Tracks",
            JOptionPane.INFORMATION_MESSAGE
        )
    }
    
    /**
     * Remove selected tracks from the playlist
     */
    private fun removeSelectedTracks() {
        val selectedIndices = trackList.selectedIndices
        if (selectedIndices.isEmpty()) return
        
        val selectedTracks = selectedIndices.map { trackListModel.getElementAt(it) }
        val trackNames = selectedTracks.joinToString(", ") { it.title }
        
        val result = JOptionPane.showConfirmDialog(
            this,
            "Remove ${selectedTracks.size} track(s) from playlist?\n$trackNames",
            "Remove Tracks",
            JOptionPane.YES_NO_OPTION,
            JOptionPane.WARNING_MESSAGE
        )
        
        if (result == JOptionPane.YES_OPTION) {
            try {
                // Remove in reverse order to maintain indices
                selectedIndices.sortedDescending().forEach { index ->
                    trackListModel.removeElementAt(index)
                }
                
                showStatus("Removed ${selectedTracks.size} track(s)")
                
            } catch (e: Exception) {
                errorService?.logError("Failed to remove tracks", e)
                showError("Remove Tracks Error", "Failed to remove tracks: ${e.message}")
            }
        }
    }
    
    /**
     * Play the selected track
     */
    private fun playTrack(trackItem: TrackItem) {
        try {
            // TODO: Integrate with playback service to play the track
            showStatus("Playing: ${trackItem.title}")
            
            // For now, just show a message
            JOptionPane.showMessageDialog(
                this,
                "Playing: ${trackItem.title}\nby ${trackItem.artist}",
                "Now Playing",
                JOptionPane.INFORMATION_MESSAGE
            )
            
        } catch (e: Exception) {
            errorService?.logError("Failed to play track", e)
            showError("Playback Error", "Failed to play track: ${e.message}")
        }
    }
    
    /**
     * Queue tracks for playback
     */
    private fun queueTracks(tracks: List<TrackItem>) {
        try {
            // TODO: Integrate with playback service to queue tracks
            val trackNames = tracks.joinToString(", ") { it.title }
            showStatus("Queued ${tracks.size} track(s): $trackNames")
            
        } catch (e: Exception) {
            errorService?.logError("Failed to queue tracks", e)
            showError("Queue Error", "Failed to queue tracks: ${e.message}")
        }
    }
    
    /**
     * Show track information dialog
     */
    private fun showTrackInfo(trackItem: TrackItem) {
        val infoText = """
            Title: ${trackItem.title}
            Artist: ${trackItem.artist}
            Album: ${trackItem.album}
            Duration: ${trackItem.duration}
            ID: ${trackItem.id}
        """.trimIndent()
        
        JOptionPane.showMessageDialog(
            this,
            infoText,
            "Track Information",
            JOptionPane.INFORMATION_MESSAGE
        )
    }
    
    /**
     * Play entire playlist
     */
    private fun playPlaylist(playlistItem: PlaylistItem) {
        try {
            // TODO: Integrate with playback service to play playlist
            showStatus("Playing playlist: ${playlistItem.name}")
            
        } catch (e: Exception) {
            errorService?.logError("Failed to play playlist", e)
            showError("Playback Error", "Failed to play playlist: ${e.message}")
        }
    }
    
    /**
     * Queue entire playlist
     */
    private fun queuePlaylist(playlistItem: PlaylistItem) {
        try {
            // TODO: Integrate with playback service to queue playlist
            showStatus("Queued playlist: ${playlistItem.name}")
            
        } catch (e: Exception) {
            errorService?.logError("Failed to queue playlist", e)
            showError("Queue Error", "Failed to queue playlist: ${e.message}")
        }
    }
    
    /**
     * Rename playlist
     */
    private fun renamePlaylist(playlistItem: PlaylistItem) {
        val newName = JOptionPane.showInputDialog(
            this,
            "Enter new name for playlist:",
            "Rename Playlist",
            JOptionPane.PLAIN_MESSAGE,
            null,
            null,
            playlistItem.name
        ) as String?
        
        if (newName != null && newName.trim().isNotEmpty() && newName.trim() != playlistItem.name) {
            try {
                // TODO: Update playlist name in service
                // For now, update the model
                val index = playlistListModel.indexOf(playlistItem)
                if (index >= 0) {
                    val updatedItem = playlistItem.copy(name = newName.trim())
                    playlistListModel.setElementAt(updatedItem, index)
                    selectedPlaylistItem = updatedItem
                }
                
                showStatus("Renamed playlist to: ${newName.trim()}")
                
            } catch (e: Exception) {
                errorService?.logError("Failed to rename playlist", e)
                showError("Rename Error", "Failed to rename playlist: ${e.message}")
            }
        }
    }
    
    /**
     * Export playlist to file
     */
    private fun exportPlaylist(playlistItem: PlaylistItem) {
        try {
            // TODO: Implement playlist export functionality
            JOptionPane.showMessageDialog(
                this,
                "Export functionality will be implemented in Phase 4.6.\n" +
                        "This will allow you to export playlists in M3U format.",
                "Export Playlist",
                JOptionPane.INFORMATION_MESSAGE
            )
            
        } catch (e: Exception) {
            errorService?.logError("Failed to export playlist", e)
            showError("Export Error", "Failed to export playlist: ${e.message}")
        }
    }
    
    /**
     * Show status message
     */
    private fun showStatus(message: String) {
        // TODO: Integrate with main player status display
        println("PlaylistView Status: $message")
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
     * Get the currently selected playlist ID
     */
    fun getCurrentPlaylistId(): String? = currentPlaylistId
    
    /**
     * Get the currently selected tracks
     */
    fun getSelectedTracks(): List<TrackItem> {
        return trackList.selectedIndices.map { trackListModel.getElementAt(it) }
    }
    
    /**
     * Refresh the playlist view
     */
    fun refresh() {
        loadPlaylists()
        if (currentPlaylistId != null) {
            loadTracksForPlaylist(currentPlaylistId)
        }
    }
}

/**
 * Data class representing a playlist item
 */
data class PlaylistItem(
    val id: String,
    val name: String,
    val description: String,
    val trackCount: Int
)

/**
 * Data class representing a track item
 */
data class TrackItem(
    val id: String,
    val title: String,
    val artist: String,
    val album: String,
    val duration: String
)

/**
 * Custom cell renderer for playlist list
 */
class PlaylistCellRenderer : DefaultListCellRenderer() {
    override fun getListCellRendererComponent(
        list: JList<*>?,
        value: Any?,
        index: Int,
        isSelected: Boolean,
        cellHasFocus: Boolean
    ): java.awt.Component {
        super.getListCellRendererComponent(list, value, index, isSelected, cellHasFocus)
        
        if (value is PlaylistItem) {
            if (value.id.isEmpty()) {
                // Placeholder message
                text = value.name
                font = font.deriveFont(java.awt.Font.ITALIC)
            } else {
                text = "<html><b>${value.name}</b><br><small>${value.trackCount} tracks</small></html>"
            }
        }
        
        return this
    }
}

/**
 * Custom cell renderer for track list
 */
class TrackCellRenderer : DefaultListCellRenderer() {
    override fun getListCellRendererComponent(
        list: JList<*>?,
        value: Any?,
        index: Int,
        isSelected: Boolean,
        cellHasFocus: Boolean
    ): java.awt.Component {
        super.getListCellRendererComponent(list, value, index, isSelected, cellHasFocus)
        
        if (value is TrackItem) {
            text = "<html><b>${value.title}</b><br>" +
                    "<small>${value.artist} - ${value.album} (${value.duration})</small></html>"
        }
        
        return this
    }
}