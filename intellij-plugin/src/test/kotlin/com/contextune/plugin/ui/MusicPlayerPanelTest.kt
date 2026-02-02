package com.contextune.plugin.ui

import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for MusicPlayerPanel UI components
 */
class MusicPlayerPanelTest : BasePlatformTestCase() {

    fun `test panel creation`() {
        val panel = MusicPlayerPanel(project)
        assertNotNull(panel)
    }

    fun `test panel has playback controls`() {
        val panel = MusicPlayerPanel(project)

        // Check that panel contains components
        val components = panel.components
        assertTrue("Panel should have components", components.isNotEmpty())
    }

    fun `test panel cleanup`() {
        val panel = MusicPlayerPanel(project)

        // Should not throw exception
        panel.dispose()
    }

    fun `test track metadata update`() {
        val panel = MusicPlayerPanel(project)

        // Update metadata
        panel.updateTrackMetadata("Test Title", "Test Artist", "Test Album")

        // Should not throw exception
    }

    fun `test format time`() {
        val panel = MusicPlayerPanel(project)

        // Access private method through reflection for testing
        val method = panel.javaClass.getDeclaredMethod("formatTime", Double::class.java)
        method.isAccessible = true

        assertEquals("0:00", method.invoke(panel, 0.0))
        assertEquals("1:30", method.invoke(panel, 90.0))
        assertEquals("3:45", method.invoke(panel, 225.0))
        assertEquals("10:05", method.invoke(panel, 605.0))
    }
}