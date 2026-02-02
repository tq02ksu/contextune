package com.contextune.plugin.state

import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for PlayerState persistence
 */
class PlayerStateTest : BasePlatformTestCase() {
    
    fun `test state creation`() {
        val state = PlayerState()
        assertNotNull(state)
    }
    
    fun `test default values`() {
        val state = PlayerState()
        
        assertEquals("", state.lastFilePath)
        assertEquals(0.0, state.lastPosition)
        assertEquals(0.75, state.volume)
        assertFalse(state.isMuted)
        assertFalse(state.toolWindowVisible)
    }
    
    fun `test state modification`() {
        val state = PlayerState()
        
        state.lastFilePath = "/path/to/file.mp3"
        state.lastPosition = 123.45
        state.volume = 0.8
        state.isMuted = true
        
        assertEquals("/path/to/file.mp3", state.lastFilePath)
        assertEquals(123.45, state.lastPosition)
        assertEquals(0.8, state.volume)
        assertEquals(true, state.isMuted)
    }
    
    fun `test state serialization`() {
        val state1 = PlayerState()
        state1.lastFilePath = "/test/file.mp3"
        state1.volume = 0.5
        state1.isMuted = true
        
        val state2 = PlayerState()
        state2.loadState(state1)
        
        assertEquals(state1.lastFilePath, state2.lastFilePath)
        assertEquals(state1.volume, state2.volume)
        assertEquals(state1.isMuted, state2.isMuted)
    }
}
