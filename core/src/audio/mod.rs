//! Audio engine module
//!
//! Provides high-fidelity audio playback with bit-perfect accuracy.

pub mod engine;
pub mod decoder;
pub mod output;
pub mod buffer;
pub mod processor;
pub mod format;

pub use engine::AudioEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_module() {
        // Module structure test
        assert!(true);
    }
}
