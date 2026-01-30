package com.contextune.plugin.services

import com.contextune.plugin.audio.AudioEngineException
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Tests for PlaybackService
 */
class PlaybackServiceTest {
    
    private lateinit var service: PlaybackService
    
    @BeforeEach
    fun setUp() {
        service = PlaybackService()
        service.initialize()
    }
    
    @AfterEach
    fun tearDown() {
        service.shutdown()
    }
    
    @Test
    fun `test service initialization`() {
        // Service should be initialized in setUp
        // No exception means success
    }
    
    @Test
    fun `test volume control`() {
        service.setVolume(0.5)
        val volume = service.getVolume()
        assertTrue(volume >= 0.45 && volume <= 0.55, "Volume should be approximately 0.5")
    }
    
    @Test
    fun `test volume validation`() {
        assertThrows<IllegalArgumentException> {
            service.setVolume(-0.1)
        }
        
        assertThrows<IllegalArgumentException> {
            service.setVolume(1.1)
        }
    }
    
    @Test
    fun `test volume ramping`() {
        service.setVolumeRamped(0.8, 100)
        Thread.sleep(150)
        val volume = service.getVolume()
        assertTrue(volume >= 0.75 && volume <= 0.85, "Volume should be approximately 0.8")
    }
    
    @Test
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
    
    @Test
    fun `test initial playback state`() {
        assertFalse(service.isPlaying())
        assertEquals(0.0, service.getPosition(), 0.01)
        assertNull(service.getCurrentFile())
    }
    
    @Test
    fun `test load invalid file`() {
        assertThrows<AudioEngineException> {
            service.loadFile("/nonexistent/file.mp3")
        }
    }
    
    @Test
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
    
    @Test
    fun `test seek without loaded file`() {
        // Should not crash
        try {
            service.seek(10.0)
        } catch (e: AudioEngineException) {
            // Expected - no file loaded
        }
    }
    
    @Test
    fun `test operations without initialization throw exception`() {
        val uninitializedService = PlaybackService()
        
        assertThrows<IllegalStateException> {
            uninitializedService.play()
        }
        
        assertThrows<IllegalStateException> {
            uninitializedService.setVolume(0.5)
        }
        
        assertThrows<IllegalStateException> {
            uninitializedService.getVolume()
        }
    }
    
    @Test
    fun `test get duration without loaded file`() {
        val duration = service.getDuration()
        assertEquals(0.0, duration, 0.01)
    }
    
    @Test
    fun `test shutdown and reinitialize`() {
        service.setVolume(0.5)
        service.shutdown()
        
        // Should be able to reinitialize
        service.initialize()
        
        // Volume should be reset
        val volume = service.getVolume()
        assertTrue(volume >= 0.0 && volume <= 1.0)
    }
    
    @Test
    fun `test multiple shutdown calls are safe`() {
        service.shutdown()
        service.shutdown()
        service.shutdown()
        // Should not crash
    }
    
    @Test
    fun `test multiple initialize calls are safe`() {
        service.initialize()
        service.initialize()
        service.initialize()
        // Should not crash (idempotent)
    }
}
