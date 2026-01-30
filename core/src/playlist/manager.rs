//! Playlist manager implementation
//!
//! CRUD operations for playlists

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Represents a single track in a playlist
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    /// Unique track ID
    pub id: String,
    /// File path to the audio file
    pub file_path: String,
    /// Track title
    pub title: Option<String>,
    /// Artist name
    pub artist: Option<String>,
    /// Album name
    pub album: Option<String>,
    /// Track duration in seconds
    pub duration: Option<f64>,
    /// Track number in album
    pub track_number: Option<u32>,
    /// Year of release
    pub year: Option<u32>,
    /// Genre
    pub genre: Option<String>,
}

impl Track {
    /// Create a new track with just a file path
    pub fn new(file_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            title: None,
            artist: None,
            album: None,
            duration: None,
            track_number: None,
            year: None,
            genre: None,
        }
    }

    /// Create a new track with metadata
    pub fn with_metadata(
        file_path: String,
        title: Option<String>,
        artist: Option<String>,
        album: Option<String>,
        duration: Option<f64>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            title,
            artist,
            album,
            duration,
            track_number: None,
            year: None,
            genre: None,
        }
    }
}

/// Represents a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Unique playlist ID
    pub id: String,
    /// Playlist name
    pub name: String,
    /// Playlist description
    pub description: Option<String>,
    /// List of tracks in order
    pub tracks: Vec<Track>,
    /// Creation timestamp (Unix timestamp)
    pub created_at: i64,
    /// Last modified timestamp (Unix timestamp)
    pub modified_at: i64,
}

impl Playlist {
    /// Create a new empty playlist
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            tracks: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Add a track to the end of the playlist
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.modified_at = chrono::Utc::now().timestamp();
    }

    /// Insert a track at a specific position
    pub fn insert_track(&mut self, index: usize, track: Track) -> Result<()> {
        if index > self.tracks.len() {
            return Err(Error::Playlist(format!(
                "Index {} out of bounds for playlist with {} tracks",
                index,
                self.tracks.len()
            )));
        }
        self.tracks.insert(index, track);
        self.modified_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// Remove a track by index
    pub fn remove_track(&mut self, index: usize) -> Result<Track> {
        if index >= self.tracks.len() {
            return Err(Error::Playlist(format!(
                "Index {} out of bounds for playlist with {} tracks",
                index,
                self.tracks.len()
            )));
        }
        let track = self.tracks.remove(index);
        self.modified_at = chrono::Utc::now().timestamp();
        Ok(track)
    }

    /// Move a track from one position to another
    pub fn move_track(&mut self, from_index: usize, to_index: usize) -> Result<()> {
        if from_index >= self.tracks.len() {
            return Err(Error::Playlist(format!(
                "From index {} out of bounds for playlist with {} tracks",
                from_index,
                self.tracks.len()
            )));
        }
        if to_index >= self.tracks.len() {
            return Err(Error::Playlist(format!(
                "To index {} out of bounds for playlist with {} tracks",
                to_index,
                self.tracks.len()
            )));
        }
        let track = self.tracks.remove(from_index);
        self.tracks.insert(to_index, track);
        self.modified_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Clear all tracks
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.modified_at = chrono::Utc::now().timestamp();
    }
}

/// Playlist manager for CRUD operations
pub struct PlaylistManager {
    /// Map of playlist ID to playlist
    playlists: Arc<RwLock<HashMap<String, Playlist>>>,
}

impl PlaylistManager {
    /// Create a new playlist manager
    pub fn new() -> Self {
        Self {
            playlists: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new playlist
    pub fn create_playlist(&self, name: String) -> Result<String> {
        let playlist = Playlist::new(name);
        let id = playlist.id.clone();

        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        playlists.insert(id.clone(), playlist);
        Ok(id)
    }

    /// Get a playlist by ID
    pub fn get_playlist(&self, id: &str) -> Result<Playlist> {
        let playlists = self
            .playlists
            .read()
            .map_err(|e| Error::Playlist(format!("Failed to acquire read lock: {}", e)))?;

        playlists
            .get(id)
            .cloned()
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", id)))
    }

    /// Update a playlist
    pub fn update_playlist(&self, playlist: Playlist) -> Result<()> {
        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        if !playlists.contains_key(&playlist.id) {
            return Err(Error::Playlist(format!(
                "Playlist not found: {}",
                playlist.id
            )));
        }

        playlists.insert(playlist.id.clone(), playlist);
        Ok(())
    }

    /// Delete a playlist by ID
    pub fn delete_playlist(&self, id: &str) -> Result<()> {
        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        playlists
            .remove(id)
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", id)))?;

        Ok(())
    }

    /// List all playlists
    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let playlists = self
            .playlists
            .read()
            .map_err(|e| Error::Playlist(format!("Failed to acquire read lock: {}", e)))?;

        Ok(playlists.values().cloned().collect())
    }

    /// Get playlist count
    pub fn playlist_count(&self) -> Result<usize> {
        let playlists = self
            .playlists
            .read()
            .map_err(|e| Error::Playlist(format!("Failed to acquire read lock: {}", e)))?;

        Ok(playlists.len())
    }

    /// Add a track to a playlist
    pub fn add_track_to_playlist(&self, playlist_id: &str, track: Track) -> Result<()> {
        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        let playlist = playlists
            .get_mut(playlist_id)
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", playlist_id)))?;

        playlist.add_track(track);
        Ok(())
    }

    /// Remove a track from a playlist
    pub fn remove_track_from_playlist(&self, playlist_id: &str, track_index: usize) -> Result<()> {
        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        let playlist = playlists
            .get_mut(playlist_id)
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", playlist_id)))?;

        playlist.remove_track(track_index)?;
        Ok(())
    }

    /// Move a track within a playlist
    pub fn move_track_in_playlist(
        &self,
        playlist_id: &str,
        from_index: usize,
        to_index: usize,
    ) -> Result<()> {
        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        let playlist = playlists
            .get_mut(playlist_id)
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", playlist_id)))?;

        playlist.move_track(from_index, to_index)?;
        Ok(())
    }

    /// Shuffle tracks in a playlist using Fisher-Yates algorithm
    pub fn shuffle_playlist(&self, playlist_id: &str) -> Result<()> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut playlists = self
            .playlists
            .write()
            .map_err(|e| Error::Playlist(format!("Failed to acquire write lock: {}", e)))?;

        let playlist = playlists
            .get_mut(playlist_id)
            .ok_or_else(|| Error::Playlist(format!("Playlist not found: {}", playlist_id)))?;

        let mut rng = thread_rng();
        playlist.tracks.shuffle(&mut rng);
        playlist.modified_at = chrono::Utc::now().timestamp();

        Ok(())
    }
}

impl Default for PlaylistManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_creation() {
        let track = Track::new("/path/to/song.mp3".to_string());
        assert_eq!(track.file_path, "/path/to/song.mp3");
        assert!(track.title.is_none());
        assert!(!track.id.is_empty());
    }

    #[test]
    fn test_track_with_metadata() {
        let track = Track::with_metadata(
            "/path/to/song.mp3".to_string(),
            Some("Song Title".to_string()),
            Some("Artist Name".to_string()),
            Some("Album Name".to_string()),
            Some(180.5),
        );
        assert_eq!(track.title, Some("Song Title".to_string()));
        assert_eq!(track.artist, Some("Artist Name".to_string()));
        assert_eq!(track.duration, Some(180.5));
    }

    #[test]
    fn test_playlist_creation() {
        let playlist = Playlist::new("My Playlist".to_string());
        assert_eq!(playlist.name, "My Playlist");
        assert_eq!(playlist.tracks.len(), 0);
        assert!(!playlist.id.is_empty());
    }

    #[test]
    fn test_playlist_add_track() {
        let mut playlist = Playlist::new("Test".to_string());
        let track = Track::new("/song.mp3".to_string());
        playlist.add_track(track);
        assert_eq!(playlist.tracks.len(), 1);
    }

    #[test]
    fn test_playlist_remove_track() {
        let mut playlist = Playlist::new("Test".to_string());
        playlist.add_track(Track::new("/song1.mp3".to_string()));
        playlist.add_track(Track::new("/song2.mp3".to_string()));

        let removed = playlist.remove_track(0).unwrap();
        assert_eq!(removed.file_path, "/song1.mp3");
        assert_eq!(playlist.tracks.len(), 1);
    }

    #[test]
    fn test_playlist_move_track() {
        let mut playlist = Playlist::new("Test".to_string());
        playlist.add_track(Track::new("/song1.mp3".to_string()));
        playlist.add_track(Track::new("/song2.mp3".to_string()));
        playlist.add_track(Track::new("/song3.mp3".to_string()));

        playlist.move_track(0, 2).unwrap();
        assert_eq!(playlist.tracks[2].file_path, "/song1.mp3");
    }

    #[test]
    fn test_manager_create_playlist() {
        let manager = PlaylistManager::new();
        let id = manager
            .create_playlist("Test Playlist".to_string())
            .unwrap();
        assert!(!id.is_empty());

        let playlist = manager.get_playlist(&id).unwrap();
        assert_eq!(playlist.name, "Test Playlist");
    }

    #[test]
    fn test_manager_delete_playlist() {
        let manager = PlaylistManager::new();
        let id = manager.create_playlist("Test".to_string()).unwrap();

        manager.delete_playlist(&id).unwrap();
        assert!(manager.get_playlist(&id).is_err());
    }

    #[test]
    fn test_manager_list_playlists() {
        let manager = PlaylistManager::new();
        manager.create_playlist("Playlist 1".to_string()).unwrap();
        manager.create_playlist("Playlist 2".to_string()).unwrap();

        let playlists = manager.list_playlists().unwrap();
        assert_eq!(playlists.len(), 2);
    }

    #[test]
    fn test_manager_add_track() {
        let manager = PlaylistManager::new();
        let id = manager.create_playlist("Test".to_string()).unwrap();

        let track = Track::new("/song.mp3".to_string());
        manager.add_track_to_playlist(&id, track).unwrap();

        let playlist = manager.get_playlist(&id).unwrap();
        assert_eq!(playlist.tracks.len(), 1);
    }

    #[test]
    fn test_manager_shuffle() {
        let manager = PlaylistManager::new();
        let id = manager.create_playlist("Test".to_string()).unwrap();

        // Add multiple tracks
        for i in 0..10 {
            let track = Track::new(format!("/song{}.mp3", i));
            manager.add_track_to_playlist(&id, track).unwrap();
        }

        let original = manager.get_playlist(&id).unwrap();
        manager.shuffle_playlist(&id).unwrap();
        let shuffled = manager.get_playlist(&id).unwrap();

        // Tracks should be same count but likely different order
        assert_eq!(original.tracks.len(), shuffled.tracks.len());
    }
}
