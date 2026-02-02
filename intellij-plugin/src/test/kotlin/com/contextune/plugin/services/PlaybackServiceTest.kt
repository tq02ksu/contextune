package com.contextune.plugin.services

import com.contextune.plugin.audio.AudioEngineException
import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for PlaybackService
 */
class PlaybackServiceTest : BasePlatformTestCase() {
    
    private lateinit var service: PlaybackService
    
    override fun setUp() {
        super.setUp()
        service = PlaybackService()
        service.initialize()
    }
    
    override fun tearDown() {
        service.shutdown()
        super.tearDown()
    }
    
    fun `test service initialization`() {
        // Service should be initialized in setUp
        // No exception means success
    }
    
    fun `test volume control`() {
        service.setVolume(0.5)
        val volume = service.getVolume()
        assertTrue("Volume should be approximately 0.5", volume in 0.45..0.55)
    }
    
    fun `test volume validation`() {
        try {
            service.setVolume(-0.1)
            fail("Expected IllegalArgumentException")
        } catch (e: IllegalArgumentException) {
            // Expected
        }
        
        try {
            service.setVolume(1.1)
            fail("Expected IllegalArgumentException")
        } catch (e: IllegalArgumentException) {
            // Expected
        }
    }
    
    fun `test volume ramping`() {
        service.setVolumeRamped(0.8, 100)
        Thread.sleep(150)
        val volume = service.getVolume()
        assertTrue("Volume should be approximately 0.8", volume in 0.75..0.85)
    }
    
    fun `test mute and unmute`() {
        service.setVolume(0.75)
        assertFalse(service.isMuted())
        
        service.mute()
        assertTrue(service.isMuted())
        assertEquals(0.0, service.getVolume(), 0.01)
        
        service.unmute()
        assertFalse(service.isMuted())
        assertEquals(0.75, service.getVolume(), 0.01)
    }
    
    fun `test initial playback state`() {
        assertFalse(service.isPlaying())
        assertEquals(0.0, service.getPosition(), 0.01)
        assertNull(service.getCurrentFile())
    }
    
    fun `test load invalid file`() {
        try {
            service.loadFile("/nonexistent/file.mp3")
            fail("Expected AudioEngineException")
        } catch (e: AudioEngineException) {
            // Expected
        }
    }
    
    fun `test playback controls without loaded file`() {
        // These should not crash
        try {
            service.play()
            service.pause()
            service.stop()
        } catch (e: AudioEngineException) {
            // Expected - no file loaded
        }
    }
    
    fun `test seek without loaded file`() {
        // Should not crash
        try {
            service.seek(10.0)
        } catch (e: AudioEngineException) {
            // Expected - no file loaded
        }
    }
    
    fun `test operations without initialization throw exception`() {
        val uninitializedService = PlaybackService()
        
        try {
            uninitializedService.play()
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
        
        try {
            uninitializedService.setVolume(0.5)
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
        
        try {
            uninitializedService.getVolume()
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
    }
    
    fun `test get duration without loaded file`() {
        val duration = service.getDuration()
        assertEquals(0.0, duration, 0.01)
    }
    
    fun `test shutdown and reinitialize`() {
        service.setVolume(0.5)
        service.shutdown()
        
        // Should be able to reinitialize
        service.initialize()
        
        // Volume should be reset
        val volume = service.getVolume()
        assertTrue(volume >= 0.0 && volume <= 1.0)
    }
    
    fun `test multiple shutdown calls are safe`() {
        service.shutdown()
        service.shutdown()
        service.shutdown()
        // Should not crash
    }
    
    fun `test multiple initialize calls are safe`() {
        service.initialize()
        service.initialize()
        service.initialize()
        // Should not crash (idempotent)
    }
}
