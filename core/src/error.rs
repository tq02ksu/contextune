//! Error types for Contexture

use thiserror::Error;

/// Result type alias for Contexture operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the music player
#[derive(Error, Debug)]
pub enum Error {
    /// Audio engine error
    #[error("Audio engine error: {0}")]
    AudioEngine(String),

    /// Audio device error
    #[error("Audio device error: {0}")]
    AudioDevice(String),

    /// Audio format error
    #[error("Audio format error: {0}")]
    AudioFormat(String),

    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Audio decoding error
    #[error("Audio decoding error: {0}")]
    Decoding(String),

    /// CUE parsing error
    #[error("CUE parsing error: {0}")]
    CueParsing(String),

    /// FFI error
    #[error("FFI error: {0}")]
    Ffi(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Not supported
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Playlist error
    #[error("Playlist error: {0}")]
    Playlist(String),

    /// Library error
    #[error("Library error: {0}")]
    Library(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// AI classification error
    #[error("AI classification error: {0}")]
    AiClassification(String),
}
