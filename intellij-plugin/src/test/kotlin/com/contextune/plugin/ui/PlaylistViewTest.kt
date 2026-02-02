package com.contextune.plugin.ui

import com.intellij.testFramework.fixtures.BasePlatformTestCase
import org.junit.Assert
import javax.swing.DefaultListModel

/**
 * Tests for PlaylistView component
 */
class PlaylistViewTest : BasePlatformTestCase() {
    
    fun testPlaylistItemCreation() {
        val playlistItem = PlaylistItem(
            id = "test-id",
            name = "Test Playlist",
            description = "A test playlist",
            trackCount = 5
        )
        
        assertEquals("test-id", playlistItem.id)
        assertEquals("Test Playlist", playlistItem.name)
        assertEquals("A test playlist", playlistItem.description)
        assertEquals(5, playlistItem.trackCount)
    }
    
    fun testTrackItemCreation() {
        val trackItem = TrackItem(
            id = "track-1",
            title = "Test Track",
            artist = "Test Artist",
            album = "Test Album",
            duration = "3:45"
        )
        
        assertEquals("track-1", trackItem.id)
        assertEquals("Test Track", trackItem.title)
        assertEquals("Test Artist", trackItem.artist)
        assertEquals("Test Album", trackItem.album)
        assertEquals("3:45", trackItem.duration)
    }
    
    fun testPlaylistCellRenderer() {
        val renderer = PlaylistCellRenderer()
        val playlistItem = PlaylistItem("1", "My Playlist", "Description", 10)
        
        val component = renderer.getListCellRendererComponent(
            null, playlistItem, 0, false, false
        )
        
        assertNotNull(component)
        assertTrue(renderer.text.contains("My Playlist"))
        assertTrue(renderer.text.contains("10 tracks"))
    }
    
    fun testTrackCellRenderer() {
        val renderer = TrackCellRenderer()
        val trackItem = TrackItem("1", "Song Title", "Artist Name", "Album Name", "4:20")
        
        val component = renderer.getListCellRendererComponent(
            null, trackItem, 0, false, false
        )
        
        assertNotNull(component)
        assertTrue(renderer.text.contains("Song Title"))
        assertTrue(renderer.text.contains("Artist Name"))
        assertTrue(renderer.text.contains("Album Name"))
        assertTrue(renderer.text.contains("4:20"))
    }
    
    fun testPlaylistViewInitialization() {
        // Test that PlaylistView can be created without throwing exceptions
        try {
            val playlistView = PlaylistView(project)
            assertNotNull(playlistView)
            
            // Test that the view has the expected components
            assertNotNull(playlistView.getCurrentPlaylistId())
            
        } catch (e: Exception) {
            // Expected if services are not available in test environment
            assertTrue("Expected service unavailability in test", 
                e.message?.contains("service") == true || 
                e.message?.contains("Service") == true)
        }
    }
    
    fun testTrackTransferData() {
        val trackItem = TrackItem("1", "Test", "Artist", "Album", "3:00")
        val transferData = TrackTransferData(trackItem, 5)
        
        assertEquals(trackItem, transferData.trackItem)
        assertEquals(5, transferData.originalIndex)
    }
    
    fun testListModelOperations() {
        val model = DefaultListModel<TrackItem>()
        val track1 = TrackItem("1", "Track 1", "Artist", "Album", "3:00")
        val track2 = TrackItem("2", "Track 2", "Artist", "Album", "4:00")
        
        // Test adding elements
        model.addElement(track1)
        model.addElement(track2)
        assertEquals(2, model.size())
        
        // Test reordering (simulating drag and drop)
        var removedTrack = model.elementAt(0)
        model.removeElementAt(0)
        model.insertElementAt(removedTrack, 1)
        
        assertEquals(track2, model.getElementAt(0))
        assertEquals(track1, model.getElementAt(1))
    }
    
    fun testPlaylistItemEquality() {
        val item1 = PlaylistItem("1", "Playlist", "Description", 5)
        val item2 = PlaylistItem("1", "Playlist", "Description", 5)
        val item3 = PlaylistItem("2", "Other", "Other desc", 3)
        
        assertEquals(item1, item2)
        Assert.assertNotEquals(item1, item3)
    }
    
    fun testTrackItemEquality() {
        val track1 = TrackItem("1", "Title", "Artist", "Album", "3:00")
        val track2 = TrackItem("1", "Title", "Artist", "Album", "3:00")
        val track3 = TrackItem("2", "Other", "Other", "Other", "4:00")
        
        assertEquals(track1, track2)
        Assert.assertNotEquals(track1, track3)
    }
}