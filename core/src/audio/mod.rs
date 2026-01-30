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

pub use buffer::AudioBuffer;
pub use decoder::{AudioDecoder, AudioFormatInfo, AudioStreamReaderWithRingBuffer, DecodedPacket};
pub use engine::{
    AudioCallback, AudioDeviceInfo, AudioEngine, AudioEngineInterface, AudioEvent, PlaybackState,
};
pub use format::{AudioFormat, Channel, ChannelLayout, FormatError, SampleFormat};
pub use ring_buffer::{AudioRingBuffer, RingBufferConfig, RingBufferConsumer, RingBufferProducer};

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
