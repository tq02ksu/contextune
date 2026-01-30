package com.contexture.plugin.audio

import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Tests for RustAudioEngine JNA wrapper
 */
class RustAudioEngineTest {
    
    private lateinit var engine: AudioEngine
    
    @BeforeEach
    fun setUp() {
        engine = AudioEngine()
        engine.initialize()
    }
    
    @AfterEach
    fun tearDown() {
        engine.shutdown()
    }
    
    @Test
    fun `test engine initialization`() {
        // Engine should be initialized in setUp
        // No exception means success
    }
    
    @Test
    fun `test double initialization throws exception`() {
        assertThrows<IllegalStateException> {
            engine.initialize()
        }
    }
    
    @Test
    fun `test volume control`() {
        // Set volume to 0.5
        engine.setVolume(0.5)
        val volume = engine.getVolume()
        assertTrue(volume >= 0.45 && volume <= 0.55, "Volume should be approximately 0.5, got $volume")
        
        // Set volume to 0.0
        engine.setVolume(0.0)
        assertEquals(0.0, engine.getVolume(), 0.01)
        
        // Set volume to 1.0
        engine.setVolume(1.0)
        assertEquals(1.0, engine.getVolume(), 0.01)
    }
    
    @Test
    fun `test volume validation`() {
        assertThrows<IllegalArgumentException> {
            engine.setVolume(-0.1)
        }
        
        assertThrows<IllegalArgumentException> {
            engine.setVolume(1.1)
        }
    }
    
    @Test
    fun `test volume ramping`() {
        engine.setVolumeRamped(0.8, 100)
        // Give some time for ramping
        Thread.sleep(150)
        val volume = engine.getVolume()
        assertTrue(volume >= 0.75 && volume <= 0.85, "Volume should be approximately 0.8, got $volume")
    }
    
    @Test
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
    
    @Test
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
    
    @Test
    fun `test initial playback state`() {
        assertFalse(engine.isPlaying())
        assertEquals(0.0, engine.getPosition(), 0.01)
    }
    
    @Test
    fun `test seek validation`() {
        assertThrows<IllegalArgumentException> {
            engine.seek(-1.0)
        }
        
        // Seeking without loaded file should not throw
        // (Rust engine handles this gracefully)
        engine.seek(0.0)
        engine.seek(10.5)
    }
    
    @Test
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
    
    @Test
    fun `test operations without initialization throw exception`() {
        val uninitializedEngine = AudioEngine()
        
        assertThrows<IllegalStateException> {
            uninitializedEngine.play()
        }
        
        assertThrows<IllegalStateException> {
            uninitializedEngine.setVolume(0.5)
        }
        
        assertThrows<IllegalStateException> {
            uninitializedEngine.getVolume()
        }
    }
    
    @Test
    fun `test error handling for invalid file`() {
        assertThrows<AudioEngineException> {
            engine.loadFile("/nonexistent/file.mp3")
        }
    }
    
    @Test
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
    
    @Test
    fun `test get duration without loaded file`() {
        val duration = engine.getDuration()
        assertEquals(0.0, duration, 0.01)
    }
    
    @Test
    fun `test FFI result codes`() {
        assertTrue(FFIResult.isSuccess(FFIResult.SUCCESS))
        assertFalse(FFIResult.isSuccess(FFIResult.NULL_POINTER))
        assertFalse(FFIResult.isSuccess(FFIResult.INVALID_ARGUMENT))
        assertFalse(FFIResult.isSuccess(FFIResult.OUT_OF_MEMORY))
        assertFalse(FFIResult.isSuccess(FFIResult.INTERNAL_ERROR))
    }
    
    @Test
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
    
    @Test
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
