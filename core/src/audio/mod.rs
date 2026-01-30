//! Audio engine module
//!
//! Provides high-fidelity audio playback with bit-perfect accuracy.

pub mod buffer;
pub mod checksum;
pub mod decoder;
pub mod engine;
pub mod format;
pub mod output;
pub mod processor;
pub mod ring_buffer;

pub use engine::{AudioEngine, AudioEngineInterface, PlaybackState, AudioEvent, AudioCallback, AudioDeviceInfo};
pub use format::{AudioFormat, SampleFormat, ChannelLayout, Channel, FormatError};
pub use buffer::AudioBuffer;
pub use decoder::{AudioDecoder, DecodedPacket, AudioFormatInfo, AudioStreamReaderWithRingBuffer};
pub use ring_buffer::{AudioRingBuffer, RingBufferProducer, RingBufferConsumer, RingBufferConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_module_exports() {
        // Test that we can create instances of exported types
        let format = AudioFormat::default();
        let buffer = AudioBuffer::new(format, 1024);
        let engine = AudioEngine::new();
        
        assert!(!buffer.is_empty());
        assert!(engine.is_ok());
    }
}
