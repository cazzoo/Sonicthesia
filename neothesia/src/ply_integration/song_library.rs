//! PLY-based song library integration for Neothesia
//!
//! This module provides integration between Neothesia's song library
//! and the PLY engine architecture, enabling enhanced library features
//! like visual browsing, search, and statistics.

use std::collections::HashMap;
use std::path::PathBuf;
use crate::song_library::{SongLibraryDatabase, SongEntry, SortPreference, FilterState, SongRepository};

/// PLY-integrated song library manager
/// 
/// Provides enhanced library features with PLY-aware extensions
/// for visual browsing, search, and statistics tracking.
pub struct PlySongLibraryManager {
    /// Song repository
    repository: SongLibraryDatabase,
    /// PLY-specific extensions
    ply_extensions: LibraryExtensions,
}

/// PLY-specific library extensions
pub struct LibraryExtensions {
    /// Visual state for library browsing
    pub visual_state: LibraryVisualState,
    /// Statistics cache
    pub statistics_cache: HashMap<i64, CachedStatistics>,
}

/// Visual state for library browsing
#[derive(Debug, Default, Clone)]
pub struct LibraryVisualState {
    /// Current scroll position
    pub scroll_position: f32,
    /// Selected song index
    pub selected_index: Option<usize>,
    /// Hovered song index
    pub hovered_index: Option<usize>,
    /// View mode (grid, list, details)
    pub view_mode: LibraryViewMode,
}

/// Library view mode
#[derive(Debug, Clone, Copy)]
pub enum LibraryViewMode {
    /// Grid view
    Grid,
    /// List view
    List,
    /// Detailed view
    Details,
}

impl Default for LibraryViewMode {
    fn default() -> Self {
        Self::List
    }
}

/// Cached statistics for a song
#[derive(Debug, Clone)]
pub struct CachedStatistics {
    /// Song ID
    pub song_id: i64,
    /// Whether this song is a favorite
    pub is_favorite: bool,
    /// Custom rating (1-5 stars)
    pub custom_rating: Option<u8>,
}

impl PlySongLibraryManager {
    /// Create a new PLY song library manager
    pub fn new(repository: SongLibraryDatabase) -> Self {
        Self {
            repository,
            ply_extensions: LibraryExtensions {
                visual_state: LibraryVisualState::default(),
                statistics_cache: HashMap::new(),
            },
        }
    }

    /// Get the song repository
    pub fn repository(&self) -> &SongLibraryDatabase {
        &self.repository
    }

    /// Get mutable reference to the song repository
    pub fn repository_mut(&mut self) -> &mut SongLibraryDatabase {
        &mut self.repository
    }

    /// Get visual state
    pub fn visual_state(&self) -> &LibraryVisualState {
        &self.ply_extensions.visual_state
    }

    /// Get mutable reference to visual state
    pub fn visual_state_mut(&mut self) -> &mut LibraryVisualState {
        &mut self.ply_extensions.visual_state
    }

    /// List songs with sorting and filtering
    pub fn list_songs(&self, sort: &SortPreference, filter: &FilterState) -> Result<Vec<SongEntry>, String> {
        self.repository.list_songs(sort, filter)
            .map_err(|e| e.to_string())
    }

    /// Get a specific song
    pub fn get_song(&self, song_id: i64) -> Result<Option<SongEntry>, String> {
        self.repository.get_song(song_id)
            .map_err(|e| e.to_string())
    }

    /// Update song statistics after a play session
    pub fn update_play_statistics(&mut self, song_id: i64, accuracy: f32) -> Result<(), String> {
        self.repository.update_stats(song_id, Some(accuracy))
            .map_err(|e| e.to_string())
    }

    /// Set a song as favorite
    pub fn set_favorite(&mut self, song_id: i64, favorite: bool) {
        self.ply_extensions.statistics_cache
            .entry(song_id)
            .or_insert_with(|| CachedStatistics {
                song_id,
                is_favorite: false,
                custom_rating: None,
            })
            .is_favorite = favorite;
    }

    /// Check if a song is a favorite
    pub fn is_favorite(&self, song_id: i64) -> bool {
        self.ply_extensions.statistics_cache
            .get(&song_id)
            .map(|s| s.is_favorite)
            .unwrap_or(false)
    }

    /// Set custom rating for a song
    pub fn set_rating(&mut self, song_id: i64, rating: Option<u8>) {
        let stats = self.ply_extensions.statistics_cache
            .entry(song_id)
            .or_insert_with(|| CachedStatistics {
                song_id,
                is_favorite: false,
                custom_rating: None,
            });
        stats.custom_rating = rating.map(|r| r.clamp(1, 5));
    }

    /// Get custom rating for a song
    pub fn get_rating(&self, song_id: i64) -> Option<u8> {
        self.ply_extensions.statistics_cache
            .get(&song_id)
            .and_then(|s| s.custom_rating)
    }

    /// Get song count
    pub fn song_count(&self) -> Result<usize, String> {
        self.repository.song_count()
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_manager_creation() {
        // Note: This test would need a real SongLibraryDatabase to work properly
        // For now, we just test the structure
        let extensions = LibraryExtensions {
            visual_state: LibraryVisualState::default(),
            statistics_cache: HashMap::new(),
        };
        
        assert_eq!(extensions.visual_state.scroll_position, 0.0);
        assert_eq!(extensions.visual_state.selected_index, None);
    }

    #[test]
    fn test_view_mode() {
        let mode = LibraryViewMode::default();
        assert!(matches!(mode, LibraryViewMode::List));
    }

    #[test]
    fn test_cached_statistics() {
        let stats = CachedStatistics {
            song_id: 123,
            is_favorite: true,
            custom_rating: Some(5),
        };
        
        assert_eq!(stats.song_id, 123);
        assert!(stats.is_favorite);
        assert_eq!(stats.custom_rating, Some(5));
    }
}
