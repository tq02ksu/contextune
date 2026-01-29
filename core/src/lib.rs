//! Contexture Core Library
//!
//! A high-fidelity music player core with AI-powered features.
//! This library provides professional-grade audio playback with bit-perfect accuracy.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod audio;
pub mod cue;
pub mod playlist;
pub mod library;
pub mod ai;
pub mod streaming;
pub mod ffi;
pub mod state;
pub mod error;

pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = "Contexture";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "Contexture");
    }
}
