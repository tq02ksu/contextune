//! Playlist management module
//!
//! Handles playlist CRUD operations and playback queue

pub mod manager;
pub mod queue;
pub mod smart;

pub use manager::{Playlist, PlaylistManager, Track};
