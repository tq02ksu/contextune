package com.contextune.plugin.audio

import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for RustAudioEngine JNA wrapper
 */
class RustAudioEngineTest : BasePlatformTestCase() {
    
    private lateinit var engine: AudioEngine
    
    override fun setUp() {
        super.setUp()
        engine = AudioEngine()
        engine.initialize()
    }
    
    override fun tearDown() {
        engine.shutdown()
        super.tearDown()
    }
    
    fun `test engine initialization`() {
        // Engine should be initialized in setUp
        // No exception means success
    }
    
    fun `test double initialization throws exception`() {
        try {
            engine.initialize()
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
    }
    
    fun `test volume control`() {
        // Set volume to 0.5
        engine.setVolume(0.5)
        val volume = engine.getVolume()
        assertTrue("Volume should be approximately 0.5, got $volume", volume in 0.45..0.55)
        
        // Set volume to 0.0
        engine.setVolume(0.0)
        assertEquals(0.0, engine.getVolume(), 0.01)
        
        // Set volume to 1.0
        engine.setVolume(1.0)
        assertEquals(1.0, engine.getVolume(), 0.01)
    }
    
    fun `test volume validation`() {
        try {
            engine.setVolume(-0.1)
            fail("Expected IllegalArgumentException")
        } catch (e: IllegalArgumentException) {
            // Expected
        }
        
        try {
            engine.setVolume(1.1)
            fail("Expected IllegalArgumentException")
        } catch (e: IllegalArgumentException) {
            // Expected
        }
    }
    
    fun `test volume ramping`() {
        engine.setVolumeRamped(0.8, 100)
        // Give some time for ramping
        Thread.sleep(150)
        val volume = engine.getVolume()
        assertTrue("Volume should be approximately 0.8, got $volume", volume in 0.75..0.85)
    }
    
    fun `test mute and unmute`() {
        // Set initial volume
        engine.setVolume(0.75)
        assertFalse(engine.isMuted())
        
        // Mute
        engine.mute()
        assertTrue(engine.isMuted())
        assertEquals(0.0, engine.getVolume(), 0.01)
        
        // Unmute
        engine.unmute()
        assertFalse(engine.isMuted())
        assertEquals(0.75, engine.getVolume(), 0.01)
    }
    
    fun `test mute is idempotent`() {
        engine.setVolume(0.5)
        
        engine.mute()
        engine.mute()
        engine.mute()
        
        assertTrue(engine.isMuted())
        assertEquals(0.0, engine.getVolume(), 0.01)
        
        engine.unmute()
        engine.unmute()
        engine.unmute()
        
        assertFalse(engine.isMuted())
        assertEquals(0.5, engine.getVolume(), 0.01)
    }
    
    fun `test initial playback state`() {
        assertFalse(engine.isPlaying())
        assertEquals(0.0, engine.getPosition(), 0.01)
    }
    
    fun `test seek validation`() {
        try {
            engine.seek(-1.0)
            fail("Expected IllegalArgumentException")
        } catch (e: IllegalArgumentException) {
            // Expected
        }
        
        // Seeking without loaded file should not throw
        // (Rust engine handles this gracefully)
        engine.seek(0.0)
        engine.seek(10.5)
    }
    
    fun `test callback registration`() {
        var callbackInvoked = false
        
        engine.setCallback { event ->
            callbackInvoked = true
            assertNotNull(event)
        }
        
        // Trigger an event by seeking
        engine.seek(1.0)
        Thread.sleep(50)
        
        // Clear callback
        engine.clearCallback()
    }
    
    fun `test operations without initialization throw exception`() {
        val uninitializedEngine = AudioEngine()
        
        try {
            uninitializedEngine.play()
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
        
        try {
            uninitializedEngine.setVolume(0.5)
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
        
        try {
            uninitializedEngine.getVolume()
            fail("Expected IllegalStateException")
        } catch (e: IllegalStateException) {
            // Expected
        }
    }
    
    fun `test error handling for invalid file`() {
        try {
            engine.loadFile("/nonexistent/file.mp3")
            fail("Expected AudioEngineException")
        } catch (e: AudioEngineException) {
            // Expected
        }
    }
    
    fun `test playback controls without loaded file`() {
        // These should not crash, but may return errors from Rust
        // The engine handles this gracefully
        try {
            engine.play()
            engine.pause()
            engine.stop()
        } catch (e: AudioEngineException) {
            // Expected - no file loaded
        }
    }
    
    fun `test get duration without loaded file`() {
        val duration = engine.getDuration()
        assertEquals(0.0, duration, 0.01)
    }
    
    fun `test FFI result codes`() {
        assertTrue(FFIResult.isSuccess(FFIResult.SUCCESS))
        assertFalse(FFIResult.isSuccess(FFIResult.NULL_POINTER))
        assertFalse(FFIResult.isSuccess(FFIResult.INVALID_ARGUMENT))
        assertFalse(FFIResult.isSuccess(FFIResult.OUT_OF_MEMORY))
        assertFalse(FFIResult.isSuccess(FFIResult.INTERNAL_ERROR))
    }
    
    fun `test audio event type enum`() {
        assertEquals(0, AudioEventType.STATE_CHANGED.value)
        assertEquals(1, AudioEventType.POSITION_CHANGED.value)
        assertEquals(2, AudioEventType.TRACK_ENDED.value)
        assertEquals(3, AudioEventType.ERROR.value)
        assertEquals(4, AudioEventType.BUFFER_UNDERRUN.value)
        
        assertEquals(AudioEventType.STATE_CHANGED, AudioEventType.fromValue(0))
        assertEquals(AudioEventType.POSITION_CHANGED, AudioEventType.fromValue(1))
        assertEquals(AudioEventType.TRACK_ENDED, AudioEventType.fromValue(2))
        assertEquals(AudioEventType.ERROR, AudioEventType.fromValue(3))
        assertEquals(AudioEventType.BUFFER_UNDERRUN, AudioEventType.fromValue(4))
    }
    
    fun `test playback state enum`() {
        assertEquals(0, PlaybackState.STOPPED.value)
        assertEquals(1, PlaybackState.PLAYING.value)
        assertEquals(2, PlaybackState.PAUSED.value)
        assertEquals(3, PlaybackState.BUFFERING.value)
        assertEquals(4, PlaybackState.ERROR.value)
        
        assertEquals(PlaybackState.STOPPED, PlaybackState.fromValue(0))
        assertEquals(PlaybackState.PLAYING, PlaybackState.fromValue(1))
        assertEquals(PlaybackState.PAUSED, PlaybackState.fromValue(2))
        assertEquals(PlaybackState.BUFFERING, PlaybackState.fromValue(3))
        assertEquals(PlaybackState.ERROR, PlaybackState.fromValue(4))
    }
}
