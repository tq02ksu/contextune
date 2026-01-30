//! FFI (Foreign Function Interface) module
//!
//! Provides C-compatible interface for Java/Kotlin integration

pub mod c_api;
pub mod jni;
pub mod playlist_api;
pub mod types;

// Re-export commonly used types and functions
pub use c_api::*;
pub use playlist_api::*;
pub use types::{
    AudioEngineHandle, FFIAudioCallback, FFIAudioEvent, FFIAudioEventType, FFIPlaybackState,
    FFIResult,
};
