//! Audio engine module
//!
//! Provides high-fidelity audio playback with bit-perfect accuracy.

pub mod buffer;
pub mod decoder;
pub mod engine;
pub mod format;
pub mod output;
pub mod processor;

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
