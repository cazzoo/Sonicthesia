# Neothesia Collaborative Song Library Plan

## Executive Summary

This plan implements a dynamic, internet-based collaborative song library that grows organically as users contribute MIDI files. The system enables users to browse, search, download, and contribute songs while maintaining quality through duplicate detection, moderation, and community curation.

**Core Philosophy**: Make it easy to contribute, rewarding to browse, and impossible to pollute.

---

## Current State Analysis

### Existing Foundation
- **Local SQLite Database**: `song_library.db` with metadata, play statistics
- **Song Scanner**: Automatic directory scanning for MIDI files
- **Metadata Parser**: Extracts tempo, track count, note count from MIDI
- **Difficulty Calculator**: Estimates difficulty from note density and complexity
- **Play Statistics**: Tracks play count, best scores, last played

### Limitations
- Local-only storage (no sharing between users)
- Manual MIDI file acquisition
- No community contribution or curation
- No duplicate management across users
- No metadata enrichment (artist, genre, tags)

---

## Phase 1: Core Infrastructure

### 1.1 Backend API Server

**Technology Stack**:
```
Runtime: Rust (Axum framework)
Database: PostgreSQL 16+
Object Storage: S3-compatible (MinIO for self-hosted, AWS S3 for cloud)
Search: MeiliSearch (fast, typo-tolerant search)
Cache: Redis (session cache, rate limiting)
Queue: Redis/RabbitMQ (background jobs)
```

**API Structure**:
```
/api/v1/
├── /songs                    # Song CRUD operations
│   ├── GET /                 # Browse/search songs
│   ├── GET /:id              # Get song details
│   ├── POST /                # Upload new song
│   ├── PUT /:id              # Update song metadata
│   ├── DELETE /:id           # Remove song (admin)
│   └── /:id/download        # Download MIDI file
├── /variants                 # Song variant management
│   ├── GET /song/:id         # Get variants for a song
│   ├── POST /                # Upload variant
│   └── PUT /:id              # Update variant
├── /search                   # Search endpoints
│   ├── GET /songs            # Search songs
│   ├── GET /artists          # Search artists
│   └── GET /genres           # Search genres
├── /stats                    # Statistics endpoints
│   ├── GET /songs/:id        # Song statistics
│   ├── POST /songs/:id/play  # Record play
│   └── GET /trending         # Trending songs
├── /users                    # User endpoints
│   ├── POST /register        # Register user
│   ├── POST /login           # Login
│   ├── GET /me               # Get profile
│   └── GET /:id/contributions# User contributions
├── /moderation               # Moderation endpoints
│   ├── POST /report          # Report content
│   ├── GET /queue            # Moderation queue (admin)
│   └── POST /queue/:id/review# Review content (admin)
└── /sync                     # Sync endpoints
    ├── POST /library         # Sync local library
    └── GET /updates          # Get updates since timestamp
```

### 1.2 Database Schema

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100),
    avatar_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    last_login TIMESTAMP,
    is_admin BOOLEAN DEFAULT FALSE,
    is_banned BOOLEAN DEFAULT FALSE,
    contribution_count INTEGER DEFAULT 0,
    reputation_score INTEGER DEFAULT 0
);

-- Songs table (main song entity)
CREATE TABLE songs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    artist VARCHAR(255),
    album VARCHAR(255),
    year INTEGER,
    genre VARCHAR(100),
    tags TEXT[], -- PostgreSQL array for tags
    duration_secs INTEGER,
    note_count INTEGER,
    track_count INTEGER,
    tempo_bpm INTEGER,
    time_signature VARCHAR(10),
    key_signature VARCHAR(20),
    difficulty_numeric DECIMAL(3,1), -- 1.0 to 10.0
    difficulty_label VARCHAR(20), -- Beginner, Easy, Medium, Hard, Expert
    file_hash VARCHAR(64) UNIQUE NOT NULL, -- SHA-256 for duplicate detection
    file_size INTEGER,
    storage_key VARCHAR(500) NOT NULL, -- S3 object key
    preview_url VARCHAR(500), -- 30-second preview audio
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMP DEFAULT NOW(),
    approved_at TIMESTAMP,
    approved_by UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'pending', -- pending, approved, rejected, archived
    rejection_reason TEXT,
    play_count INTEGER DEFAULT 0,
    download_count INTEGER DEFAULT 0,
    avg_rating DECIMAL(3,2), -- 1.00 to 5.00
    rating_count INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    metadata_json JSONB, -- Additional metadata (measures, chord progressions, etc.)
    search_vector TSVECTOR -- Full-text search vector
);

-- Song variants (different arrangements of same song)
CREATE TABLE song_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    variant_type VARCHAR(50) NOT NULL, -- 'simplified', 'original', 'advanced', 'jazz', 'rock'
    title VARCHAR(255) NOT NULL,
    difficulty_numeric DECIMAL(3,1),
    difficulty_label VARCHAR(20),
    file_hash VARCHAR(64) UNIQUE NOT NULL,
    file_size INTEGER,
    storage_key VARCHAR(500) NOT NULL,
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMP DEFAULT NOW(),
    approved_at TIMESTAMP,
    status VARCHAR(20) DEFAULT 'pending',
    play_count INTEGER DEFAULT 0,
    avg_rating DECIMAL(3,2),
    notes TEXT -- Variant-specific notes
);

-- User ratings
CREATE TABLE song_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(song_id, user_id)
);

-- Play history (aggregated statistics)
CREATE TABLE play_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES song_variants(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    played_at TIMESTAMP DEFAULT NOW(),
    duration_played INTEGER, -- Seconds played
    completed BOOLEAN,
    accuracy DECIMAL(5,2),
    max_streak INTEGER,
    score INTEGER,
    difficulty_played VARCHAR(20),
    device_id VARCHAR(100) -- For anonymous tracking
);

-- User library (songs downloaded/saved)
CREATE TABLE user_library (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES song_variants(id) ON DELETE SET NULL,
    downloaded_at TIMESTAMP DEFAULT NOW(),
    local_path VARCHAR(500),
    last_played_at TIMESTAMP,
    play_count INTEGER DEFAULT 0,
    best_score INTEGER,
    best_accuracy DECIMAL(5,2),
    best_stars INTEGER,
    UNIQUE(user_id, song_id, variant_id)
);

-- User playlists
CREATE TABLE playlists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    song_count INTEGER DEFAULT 0
);

CREATE TABLE playlist_songs (
    playlist_id UUID REFERENCES playlists(id) ON DELETE CASCADE,
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES song_variants(id) ON DELETE SET NULL,
    position INTEGER NOT NULL,
    added_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (playlist_id, song_id)
);

-- Reports and moderation
CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID REFERENCES users(id) ON DELETE SET NULL,
    target_type VARCHAR(20) NOT NULL, -- 'song', 'variant', 'user', 'rating'
    target_id UUID NOT NULL,
    reason VARCHAR(50) NOT NULL, -- 'duplicate', 'inappropriate', 'copyright', 'spam', 'low_quality'
    description TEXT,
    status VARCHAR(20) DEFAULT 'pending', -- pending, reviewing, resolved, dismissed
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMP,
    resolution TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Moderation queue
CREATE TABLE moderation_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_type VARCHAR(20) NOT NULL, -- 'song', 'variant', 'rating'
    item_id UUID NOT NULL,
    priority INTEGER DEFAULT 0, -- Higher = more urgent
    reason VARCHAR(100),
    assigned_to UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'pending', -- pending, in_progress, completed
    created_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP
);

-- Duplicate detection cache
CREATE TABLE duplicate_candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    song_id UUID REFERENCES songs(id) ON DELETE CASCADE,
    candidate_id UUID NOT NULL, -- Potentially duplicate song ID
    similarity_score DECIMAL(5,4), -- 0.0000 to 1.0000
    match_type VARCHAR(20), -- 'exact_hash', 'similar_notes', 'similar_metadata'
    detected_at TIMESTAMP DEFAULT NOW(),
    resolved BOOLEAN DEFAULT FALSE,
    resolution VARCHAR(20) -- 'duplicate', 'variant', 'different'
);

-- Tags and genres
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE genres (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Sync state (for client-side sync)
CREATE TABLE sync_state (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(100),
    last_sync_at TIMESTAMP,
    sync_token VARCHAR(255),
    PRIMARY KEY (user_id, device_id)
);

-- Indexes for performance
CREATE INDEX idx_songs_status ON songs(status);
CREATE INDEX idx_songs_genre ON songs(genre);
CREATE INDEX idx_songs_difficulty ON songs(difficulty_numeric);
CREATE INDEX idx_songs_play_count ON songs(play_count DESC);
CREATE INDEX idx_songs_avg_rating ON songs(avg_rating DESC);
CREATE INDEX idx_songs_submitted_at ON songs(submitted_at DESC);
CREATE INDEX idx_songs_search ON songs USING GIN(search_vector);
CREATE INDEX idx_songs_tags ON songs USING GIN(tags);
CREATE INDEX idx_play_history_song ON play_history(song_id);
CREATE INDEX idx_play_history_user ON play_history(user_id);
CREATE INDEX idx_user_library_user ON user_library(user_id);
CREATE INDEX idx_reports_status ON reports(status);
CREATE INDEX idx_moderation_queue_status ON moderation_queue(status);
```

### 1.3 Object Storage Structure

```
s3://neothesia-library/
├── songs/
│   ├── {song_id}/
│   │   ├── original.mid          # Original MIDI file
│   │   ├── metadata.json         # Parsed metadata
│   │   ├── preview.mp3           # 30-second preview (generated)
│   │   └── analysis.json         # Difficulty, measures, etc.
│   └── ...
├── variants/
│   ├── {variant_id}/
│   │   ├── variant.mid
│   │   └── metadata.json
│   └── ...
├── temp/
│   └── {upload_id}/             # Temporary upload storage
└── backups/
    └── {date}/                  # Daily backups
```

---

## Phase 2: Duplicate Detection & Content Protection

### 2.1 Multi-Layer Duplicate Detection

**Layer 1: File Hash (Exact Duplicate)**
```rust
// SHA-256 hash of raw file content
fn compute_file_hash(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// Check: If hash exists, reject upload
```

**Layer 2: MIDI Content Analysis (Near Duplicate)**
```rust
struct MidiFingerprint {
    note_sequence: Vec<u8>,     // Simplified note sequence (pitch only)
    rhythm_pattern: Vec<u32>,   // Duration pattern
    total_notes: usize,
    duration_secs: u32,
}

fn compute_midi_fingerprint(midi: &MidiFile) -> MidiFingerprint {
    // Extract simplified note sequence
    // Normalize timing (remove tempo differences)
    // Create fingerprint for comparison
}

fn similarity_score(a: &MidiFingerprint, b: &MidiFingerprint) -> f64 {
    // Compare note sequences (Levenshtein distance)
    // Compare rhythm patterns
    // Return 0.0 to 1.0 similarity
}

// Threshold: >0.85 similarity = likely duplicate
// Threshold: >0.70 similarity = possible variant
```

**Layer 3: Metadata Similarity**
```rust
fn metadata_similarity(a: &SongMetadata, b: &SongMetadata) -> f64 {
    let title_sim = string_similarity(&a.title, &b.title);
    let artist_sim = string_similarity(&a.artist, &b.artist);
    let duration_sim = 1.0 - (a.duration_secs as f64 - b.duration_secs as f64).abs() / 300.0;
    
    (title_sim * 0.5 + artist_sim * 0.3 + duration_sim * 0.2).clamp(0.0, 1.0)
}

fn string_similarity(a: &str, b: &str) -> f64 {
    // Normalize strings (lowercase, remove special chars)
    // Use Jaro-Winkler distance
    // Return 0.0 to 1.0
}
```

**Layer 4: Acoustic Fingerprint (Optional)**
```rust
// Convert MIDI to audio preview
// Compute chromaprint fingerprint
// Compare with existing fingerprints
// Detect same song, different arrangement
```

### 2.2 Duplicate Resolution Flow

```
Upload Flow:
1. User uploads MIDI file
2. Compute SHA-256 hash
3. Check hash against database
   - EXACT MATCH → Reject with link to existing song
   - NO MATCH → Continue
4. Parse MIDI and compute fingerprint
5. Find similar fingerprints (>0.70 similarity)
   - HIGH SIMILARITY (>0.85) → Flag as potential duplicate
   - MEDIUM SIMILARITY (0.70-0.85) → Suggest as variant
   - LOW SIMILARITY (<0.70) → Proceed as new song
6. If flagged, add to moderation queue
7. If approved, proceed with upload
```

**Duplicate Resolution UI**:
```
┌─────────────────────────────────────────┐
│  ⚠️  Potential Duplicate Detected       │
│                                         │
│  Your upload matches an existing song:  │
│                                         │
│  🎵 "Für Elise" by Beethoven            │
│  Similarity: 92%                        │
│  Existing: 3 variants, 1,247 plays      │
│                                         │
│  Options:                               │
│  ○ Add as variant (simplified version)  │
│  ○ Submit as new (requires review)      │
│  ○ Cancel upload                        │
│                                         │
│  [View Existing] [Continue] [Cancel]    │
└─────────────────────────────────────────┘
```

### 2.3 Content Moderation

**Automated Checks**:
```rust
struct ContentChecks {
    // File validation
    is_valid_midi: bool,
    file_size_ok: bool,          // < 10MB
    
    // Content analysis
    has_notes: bool,             // Not empty
    note_count_ok: bool,         // > 10 notes
    duration_ok: bool,           // > 10 seconds, < 30 minutes
    
    // Metadata validation
    title_valid: bool,
    artist_valid: bool,
    no_profanity: bool,
    
    // Quality checks
    difficulty_calculable: bool,
    tracks_parseable: bool,
}

fn validate_upload(content: &[u8], metadata: &SongMetadata) -> ContentChecks {
    // Run all checks
    // Return validation results
}
```

**Manual Review Queue**:
- New users (first 5 uploads)
- Songs flagged as potential duplicates
- Songs reported by users
- Songs with unusual metadata

### 2.4 Rate Limiting & Spam Prevention

```rust
struct RateLimits {
    // Upload limits
    uploads_per_hour: u32,       // 5 per hour
    uploads_per_day: u32,        // 20 per day
    max_file_size: usize,        // 10MB
    
    // API limits
    requests_per_minute: u32,    // 60 per minute
    downloads_per_hour: u32,     // 50 per hour
    
    // User limits
    new_user_upload_delay: Duration, // 1 hour after registration
    reputation_threshold: i32,   // Min reputation for instant approval
}

fn check_rate_limit(user: &User, action: Action) -> Result<(), RateLimitError> {
    // Check Redis for current counts
    // Apply limits based on user reputation
    // Return error if exceeded
}
```

---

## Phase 3: Client Integration

### 3.1 Library Sync Protocol

**Sync State**:
```rust
struct SyncState {
    last_sync_at: DateTime<Utc>,
    sync_token: String,          // Opaque token for incremental sync
    local_songs: HashSet<String>, // Song IDs in local library
    pending_uploads: Vec<Upload>, // Songs to upload
    pending_downloads: Vec<String>, // Song IDs to download
}
```

**Sync Flow**:
```
1. Client sends sync request with:
   - sync_token (from last sync)
   - list of local song hashes
   - timestamp of last sync

2. Server responds with:
   - new sync_token
   - songs added since last sync
   - songs updated since last sync
   - songs removed since last sync
   - user's library changes

3. Client:
   - Downloads new/updated songs
   - Removes deleted songs (if still in sync)
   - Uploads pending local songs
   - Updates local database
```

### 3.2 Local Database Extensions

```sql
-- Add to existing song_library.db

-- Remote song reference
ALTER TABLE songs ADD COLUMN remote_id TEXT;
ALTER TABLE songs ADD COLUMN remote_version INTEGER;
ALTER TABLE songs ADD COLUMN last_synced_at TIMESTAMP;
ALTER TABLE songs ADD COLUMN sync_status TEXT; -- 'synced', 'pending_upload', 'pending_download', 'conflict'

-- Download queue
CREATE TABLE download_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_id TEXT NOT NULL,
    song_name TEXT,
    status TEXT DEFAULT 'pending', -- 'pending', 'downloading', 'completed', 'failed'
    progress REAL DEFAULT 0.0,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Upload queue
CREATE TABLE upload_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    local_song_id INTEGER REFERENCES songs(id),
    status TEXT DEFAULT 'pending',
    progress REAL DEFAULT 0.0,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### 3.3 Client API Integration

```rust
// New module: neothesia/src/song_library/cloud.rs

pub struct CloudLibraryClient {
    api_base_url: String,
    auth_token: Option<String>,
    client: reqwest::Client,
}

impl CloudLibraryClient {
    // Authentication
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()>;
    pub async fn register(&mut self, email: &str, username: &str, password: &str) -> Result<()>;
    pub async fn logout(&mut self);
    
    // Library sync
    pub async fn sync_library(&self, state: &SyncState) -> Result<SyncResponse>;
    
    // Song operations
    pub async fn search_songs(&self, query: &str, filters: &SearchFilters) -> Result<Vec<Song>>;
    pub async fn get_song_details(&self, id: &str) -> Result<SongDetails>;
    pub async fn download_song(&self, id: &str, dest: &Path) -> Result<()>;
    pub async fn upload_song(&self, path: &Path, metadata: &SongMetadata) -> Result<UploadResult>;
    
    // Statistics
    pub async fn record_play(&self, song_id: &str, stats: &PlayStats) -> Result<()>;
    pub async fn rate_song(&self, song_id: &str, rating: u8, review: Option<&str>) -> Result<()>;
    
    // User
    pub async fn get_user_profile(&self) -> Result<UserProfile>;
    pub async fn get_user_library(&self) -> Result<Vec<UserLibraryEntry>>;
}
```

### 3.4 Offline Support

**Offline Mode**:
- Download songs for offline use
- Queue plays for later sync
- Cache search results
- Store user library locally

**Conflict Resolution**:
```rust
enum ConflictResolution {
    UseLocal,      // Keep local version
    UseRemote,     // Download remote version
    KeepBoth,      // Keep both (rename local)
    Merge,         // Merge metadata (keep best stats)
}

fn resolve_conflict(local: &Song, remote: &Song) -> ConflictResolution {
    // If local has better stats, keep local
    // If remote is newer version, use remote
    // If both modified, keep both
}
```

---

## Phase 4: Search & Discovery

### 4.1 Search Engine Integration

**MeiliSearch Configuration**:
```json
{
  "searchableAttributes": [
    "title",
    "artist",
    "album",
    "tags",
    "genre"
  ],
  "filterableAttributes": [
    "genre",
    "difficulty_label",
    "difficulty_numeric",
    "duration_secs",
    "play_count",
    "avg_rating",
    "tags",
    "year"
  ],
  "sortableAttributes": [
    "play_count",
    "avg_rating",
    "difficulty_numeric",
    "duration_secs",
    "submitted_at"
  ],
  "rankingRules": [
    "words",
    "typo",
    "proximity",
    "attribute",
    "sort",
    "exactness"
  ]
}
```

### 4.2 Search Filters

```rust
struct SearchFilters {
    query: Option<String>,           // Full-text search
    genre: Option<String>,
    difficulty_min: Option<f32>,     // 1.0 to 10.0
    difficulty_max: Option<f32>,
    duration_min: Option<u32>,       // Seconds
    duration_max: Option<u32>,
    rating_min: Option<f32>,         // 1.0 to 5.0
    has_variants: Option<bool>,
    tags: Vec<String>,
    year_min: Option<u32>,
    year_max: Option<u32>,
    
    // Sorting
    sort_by: SortField,
    sort_order: SortOrder,
    
    // Pagination
    page: u32,
    per_page: u32,
}

enum SortField {
    Relevance,       // Default (search relevance)
    PlayCount,
    Rating,
    Difficulty,
    Duration,
    Newest,
    Oldest,
}

enum SortOrder {
    Asc,
    Desc,
}
```

### 4.3 Discovery Features

**Trending Songs**:
```rust
struct TrendingConfig {
    time_window: Duration,      // Last 7 days
    min_plays: u32,             // Min 10 plays
    weight_play_count: f32,     // 0.4
    weight_rating: f32,         // 0.3
    weight_recency: f32,        // 0.3
}

fn calculate_trending_score(song: &Song, config: &TrendingConfig) -> f64 {
    let play_score = (song.recent_plays as f64 / config.min_plays as f64).min(1.0);
    let rating_score = song.avg_rating.unwrap_or(0.0) / 5.0;
    let recency_score = 1.0 - (now - song.submitted_at).num_days() as f64 / 30.0;
    
    (play_score * config.weight_play_count as f64
     + rating_score * config.weight_rating as f64
     + recency_score * config.weight_recency as f64)
}
```

**Recommendations**:
```rust
struct RecommendationEngine {
    user_play_history: Vec<PlayHistoryEntry>,
    user_ratings: Vec<Rating>,
    user_library: Vec<Song>,
}

impl RecommendationEngine {
    fn get_recommendations(&self, limit: usize) -> Vec<Song> {
        // Based on:
        // 1. Songs similar to highly-rated songs
        // 2. Songs in genres user plays often
        // 3. Songs at user's skill level
        // 4. Trending songs user hasn't played
        // 5. Songs by artists user has played
    }
}
```

**Curated Collections**:
```rust
struct Collection {
    id: String,
    name: String,
    description: String,
    curator: String,           // User or "system"
    songs: Vec<Song>,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Examples:
// - "Classical Essentials" (system curated)
// - "Jazz Standards" (community curated)
// - "Beginner Friendly" (difficulty filtered)
// - "User's Top 10" (user curated)
```

---

## Phase 5: Management Operations

### 5.1 Admin Dashboard

**Song Management**:
```
┌─────────────────────────────────────────┐
│  📊 Song Management Dashboard           │
│                                         │
│  Pending Review: 23                     │
│  Reports: 5                             │
│  Duplicates: 12                         │
│                                         │
│  Recent Activity:                       │
│  • New upload: "Moonlight Sonata"       │
│  • Report: "Spam content"               │
│  • Duplicate: "Für Elise" (92% match)   │
│                                         │
│  [Review Queue] [Reports] [Duplicates]  │
└─────────────────────────────────────────┘
```

**Moderation Queue**:
```rust
struct ModerationItem {
    id: String,
    item_type: ItemType,       // Song, Variant, Rating
    item_data: ItemData,
    priority: i32,
    reason: String,
    similar_items: Vec<SimilarItem>,
    submitted_by: User,
    created_at: DateTime<Utc>,
}

enum ModerationAction {
    Approve,
    Reject { reason: String },
    Merge { target_id: String },
    Archive,
    Delete,
}
```

### 5.2 Version Management

**Song Versioning**:
```rust
struct SongVersion {
    version: u32,
    file_hash: String,
    storage_key: String,
    metadata: SongMetadata,
    uploaded_by: User,
    uploaded_at: DateTime<Utc>,
    change_notes: String,
}

struct Song {
    id: String,
    current_version: u32,
    versions: Vec<SongVersion>,
    // ... other fields
}

// Version history preserved
// Users can download specific versions
// Latest version shown by default
```

**Update Flow**:
```
1. User uploads new version of existing song
2. System checks permissions (owner or admin)
3. Creates new version entry
4. Updates current_version pointer
5. Notifies users who have song in library
6. Optional: Auto-download new version
```

### 5.3 Variant Management

**Variant Types**:
```rust
enum VariantType {
    Simplified,    // Fewer notes, easier
    Original,      // Original arrangement
    Advanced,      // More notes, harder
    Jazz,          // Jazz interpretation
    Rock,          // Rock arrangement
    Classical,     // Classical arrangement
    Custom,        // User-defined
}

struct VariantMetadata {
    variant_type: VariantType,
    difficulty: DifficultyRating,
    arrangement_notes: String,   // "Left hand only", "Simplified chords"
    original_composer: String,
    arranger: String,            // Who arranged this variant
}
```

**Variant UI**:
```
┌─────────────────────────────────────────┐
│  🎵 Für Elise — Beethoven               │
│                                         │
│  Available Variants:                    │
│                                         │
│  ★★★★★ Original (5.2)                   │
│  1,247 plays | 3:45                     │
│  [Download] [Play]                      │
│                                         │
│  ★★★★☆ Simplified (3.1)                 │
│  892 plays | 3:30                       │
│  [Download] [Play]                      │
│                                         │
│  ★★★☆☆ Jazz Arrangement (7.8)          │
│  234 plays | 4:12                       │
│  [Download] [Play]                      │
│                                         │
│  [+ Add Variant]                        │
└─────────────────────────────────────────┘
```

### 5.4 Bulk Operations

```rust
// Import from local library
async fn bulk_upload(
    client: &CloudLibraryClient,
    songs: &[Song],
    progress_callback: impl Fn(UploadProgress),
) -> Result<UploadSummary> {
    // Check each song for duplicates
    // Upload non-duplicates
    // Report results
}

// Export to local library
async fn bulk_download(
    client: &CloudLibraryClient,
    song_ids: &[String],
    dest_dir: &Path,
    progress_callback: impl Fn(DownloadProgress),
) -> Result<DownloadSummary> {
    // Download songs
    // Update local database
}

// Merge duplicate songs
async fn merge_duplicates(
    admin: &AdminClient,
    primary_id: &str,
    duplicate_ids: &[String],
) -> Result<()> {
    // Merge play counts
    // Merge ratings
    // Merge variants
    // Archive duplicates
}
```

---

## Phase 6: Community Features

### 6.1 User Profiles

```rust
struct UserProfile {
    id: String,
    username: String,
    display_name: String,
    avatar_url: Option<String>,
    bio: Option<String>,
    
    // Statistics
    contributions: ContributionStats,
    play_stats: PlayStats,
    reputation: ReputationScore,
    
    // Collections
    public_playlists: Vec<Playlist>,
    favorite_genres: Vec<String>,
    
    // Badges
    badges: Vec<Badge>,
}

struct ContributionStats {
    songs_uploaded: u32,
    variants_uploaded: u32,
    ratings_given: u32,
    helpful_reviews: u32,
    reports_submitted: u32,
}

struct Badge {
    id: String,
    name: String,
    description: String,
    icon: String,
    earned_at: DateTime<Utc>,
}
```

**Badge Examples**:
```
- "First Upload" — Upload your first song
- "Contributor" — Upload 10 songs
- "Curator" — Upload 50 songs
- "Reviewer" — Write 25 helpful reviews
- "Bug Hunter" — Report 10 issues
- "Top Rated" — Have a song reach 4.5+ rating
- "Trending" — Have a song reach trending
- "Genre Expert" — Upload 20 songs in one genre
```

### 6.2 Ratings & Reviews

```rust
struct Rating {
    song_id: String,
    user_id: String,
    rating: u8,                // 1-5 stars
    review: Option<String>,
    helpful_votes: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct Review {
    id: String,
    song_id: String,
    user: UserProfile,
    rating: u8,
    content: String,
    pros: Vec<String>,
    cons: Vec<String>,
    difficulty_experience: DifficultyRating, // User's experience of difficulty
    helpful_votes: u32,
    replies: Vec<Reply>,
    created_at: DateTime<Utc>,
}
```

**Review Moderation**:
- Auto-filter profanity
- Report button for inappropriate content
- Helpful vote system
- Sort by most helpful

### 6.3 Playlists

```rust
struct Playlist {
    id: String,
    name: String,
    description: String,
    owner: UserProfile,
    is_public: bool,
    songs: Vec<PlaylistEntry>,
    followers: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct PlaylistEntry {
    song: Song,
    variant: Option<Variant>,
    added_at: DateTime<Utc>,
    notes: Option<String>,     // "Great for practicing chords"
}
```

**Playlist Features**:
- Create public/private playlists
- Share playlist links
- Follow other users' playlists
- "Add to playlist" from song page
- Collaborative playlists (multiple editors)

### 6.4 Leaderboards (Optional)

```rust
struct Leaderboard {
    song_id: String,
    entries: Vec<LeaderboardEntry>,
    updated_at: DateTime<Utc>,
}

struct LeaderboardEntry {
    rank: u32,
    user: UserProfile,
    score: u64,
    accuracy: f64,
    max_streak: u32,
    played_at: DateTime<Utc>,
}
```

**Leaderboard Types**:
- Per-song high scores
- Weekly/Monthly/All-time
- By difficulty level
- Global/Country/Friends

---

## Phase 7: Metadata Enrichment

### 7.1 Auto-Detection

```rust
struct AutoDetectedMetadata {
    // From MIDI file
    title: Option<String>,           // From MIDI metadata
    artist: Option<String>,
    tempo_bpm: u32,
    time_signature: (u8, u8),
    key_signature: Option<String>,
    
    // Computed
    difficulty: DifficultyRating,
    note_density: f32,
    chord_complexity: f32,
    rhythm_complexity: f32,
    
    // Analysis
    measures: Vec<Measure>,
    chord_progressions: Vec<ChordProgression>,
    hand_parts: HandParts,           // Left/right hand separation
}

fn analyze_midi(midi: &MidiFile) -> AutoDetectedMetadata {
    // Parse MIDI metadata
    // Count notes and compute density
    // Analyze chord patterns
    // Compute difficulty
    // Separate hand parts
}
```

### 7.2 Genre Classification

```rust
// Rule-based classification
fn classify_genre(metadata: &AutoDetectedMetadata) -> Option<String> {
    // Based on:
    // - Tempo (fast = rock/pop, slow = ballad/classical)
    // - Time signature (3/4 = waltz, 4/4 = common)
    // - Chord progressions (jazz patterns, classical patterns)
    // - Note density (high = fast/complex)
    
    // Or use ML model trained on labeled data
}
```

### 7.3 Tag Suggestions

```rust
fn suggest_tags(song: &Song) -> Vec<TagSuggestion> {
    let mut suggestions = Vec::new();
    
    // Based on metadata
    if song.difficulty_numeric < 3.0 {
        suggestions.push(TagSuggestion::new("beginner", 0.9));
    }
    
    // Based on analysis
    if song.has_chords {
        suggestions.push(TagSuggestion::new("chords", 0.8));
    }
    
    // Based on similar songs
    let similar_tags = find_similar_songs_tags(song);
    suggestions.extend(similar_tags);
    
    // Based on user tags (most common)
    let user_tags = get_user_tags_for_similar(song);
    suggestions.extend(user_tags);
    
    suggestions
}
```

---

## Technical Implementation Details

### 8.1 New Crates

```
neothesia-cloud/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   ├── handlers.rs
│   │   └── middleware.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── song.rs
│   │   ├── user.rs
│   │   ├── playlist.rs
│   │   └── moderation.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── migrations/
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── s3.rs
│   │   └── local.rs
│   ├── search/
│   │   ├── mod.rs
│   │   └── meilisearch.rs
│   ├── duplicate_detection/
│   │   ├── mod.rs
│   │   ├── hash.rs
│   │   ├── fingerprint.rs
│   │   └── similarity.rs
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── difficulty.rs
│   │   ├── genre.rs
│   │   └── metadata.rs
│   └── auth/
│       ├── mod.rs
│       ├── jwt.rs
│       └── oauth.rs
└── tests/
    └── ...
```

### 8.2 Client Integration

```
neothesia/src/
├── song_library/
│   ├── mod.rs
│   ├── database.rs        # Existing
│   ├── models.rs          # Existing
│   ├── scanner.rs         # Existing
│   ├── parser.rs          # Existing
│   └── cloud.rs           # NEW - Cloud sync client
├── scene/
│   └── cloud_library/     # NEW - Cloud library scene
│       ├── mod.rs
│       ├── browse.rs
│       ├── search.rs
│       ├── upload.rs
│       └── profile.rs
└── config/
    └── cloud.rs           # NEW - Cloud config
```

### 8.3 Configuration

```rust
// In config/mod.rs
pub struct CloudConfig {
    pub enabled: bool,
    pub api_url: String,
    pub auto_sync: bool,
    pub sync_interval: Duration,
    pub offline_mode: bool,
    pub upload_on_add: bool,
    pub download_on_browse: bool,
    pub cache_size_mb: usize,
}

// In config file (neothesia.ron)
CloudConfig(
    enabled: true,
    api_url: "https://api.neothesia.io/v1",
    auto_sync: true,
    sync_interval: 300,  // 5 minutes
    offline_mode: false,
    upload_on_add: false,
    download_on_browse: true,
    cache_size_mb: 500,
)
```

### 8.4 API Authentication

```rust
// JWT-based authentication
struct AuthToken {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

// OAuth2 support (optional)
enum OAuthProvider {
    Google,
    GitHub,
    Discord,
}

// API key for anonymous access
struct ApiKey {
    key: String,
    rate_limit: u32,
    permissions: Vec<Permission>,
}
```

---

## Deployment & Infrastructure

### 9.1 Deployment Options

**Option 1: Self-Hosted**
```
- Docker Compose setup
- PostgreSQL + MinIO + MeiliSearch
- Nginx reverse proxy
- Let's Encrypt SSL
- Cost: ~$20-50/month (VPS)
```

**Option 2: Cloud (AWS)**
```
- ECS/EKS for API
- RDS PostgreSQL
- S3 for storage
- ElastiCache Redis
- CloudFront CDN
- Cost: ~$50-200/month (depending on scale)
```

**Option 3: Hybrid**
```
- Core API on VPS
- S3-compatible storage (Backblaze B2, Wasabi)
- CDN (Cloudflare)
- Cost: ~$30-100/month
```

### 9.2 Scaling Strategy

```
Phase 1: Single server
- All services on one machine
- Handle 100 concurrent users

Phase 2: Horizontal scaling
- Separate API servers
- Load balancer
- Handle 1,000 concurrent users

Phase 3: Microservices
- Separate search, storage, API
- Read replicas for database
- Handle 10,000+ concurrent users
```

### 9.3 Monitoring & Analytics

```rust
// Metrics to track
struct Metrics {
    // API metrics
    request_count: u64,
    request_duration: Histogram,
    error_rate: f64,
    
    // Storage metrics
    storage_used: u64,
    upload_count: u64,
    download_count: u64,
    
    // User metrics
    active_users: u64,
    new_registrations: u64,
    retention_rate: f64,
    
    // Content metrics
    total_songs: u64,
    pending_moderation: u64,
    duplicate_rate: f64,
}
```

---

## Security Considerations

### 10.1 Data Protection

```
- HTTPS everywhere
- JWT token expiration (24h access, 7d refresh)
- Password hashing (Argon2)
- Rate limiting per IP/user
- Input sanitization
- SQL injection prevention (parameterized queries)
- XSS prevention (CSP headers)
- CORS configuration
```

### 10.2 File Security

```
- Virus scanning (ClamAV)
- File type validation (magic bytes, not extension)
- File size limits (10MB)
- Content validation (parse MIDI before storage)
- Sandboxed processing
- Signed URLs for downloads (expire in 1 hour)
```

### 10.3 Privacy

```
- GDPR compliance
- Data export endpoint
- Account deletion
- Anonymized analytics
- Opt-out of tracking
- Clear privacy policy
```

---

## Implementation Timeline (AI Agent)

### Phase 1: Backend Foundation — **5-7 days**
1. Database schema setup
2. Basic API endpoints (CRUD)
3. File upload/download
4. User authentication

### Phase 2: Duplicate Detection — **3-4 days**
1. File hashing
2. MIDI fingerprinting
3. Similarity detection
4. Duplicate resolution flow

### Phase 3: Client Integration — **4-5 days**
1. Cloud client library
2. Sync protocol
3. Local database updates
4. Offline support

### Phase 4: Search & Discovery — **3-4 days**
1. MeiliSearch integration
2. Search filters
3. Trending/recommendations
4. Curated collections

### Phase 5: Community Features — **3-4 days**
1. User profiles
2. Ratings & reviews
3. Playlists
4. Badges & reputation

### Phase 6: Management & Polish — **2-3 days**
1. Admin dashboard
2. Moderation tools
3. Bulk operations
4. Deployment setup

**Total Estimated Time: 20-27 days (AI agent)**

---

## Success Metrics

### Adoption
- **Registered Users**: Number of accounts created
- **Active Users**: Monthly active users
- **Upload Rate**: Songs uploaded per day
- **Download Rate**: Songs downloaded per day

### Content Quality
- **Duplicate Rate**: % of uploads flagged as duplicates
- **Approval Rate**: % of uploads approved
- **Average Rating**: Mean rating across all songs
- **Report Rate**: Reports per 1000 downloads

### Engagement
- **Session Duration**: Average time in library
- **Search Usage**: Searches per session
- **Playlist Creation**: Playlists created per user
- **Review Rate**: Reviews per download

### Technical
- **API Latency**: p95 response time
- **Uptime**: % availability
- **Storage Growth**: GB per month
- **Sync Success**: % of successful syncs

---

## Conclusion

This collaborative song library transforms Neothesia from a standalone application into a growing ecosystem where:

1. **Users contribute** — Upload songs, create variants, write reviews
2. **Quality is maintained** — Duplicate detection, moderation, community curation
3. **Discovery is easy** — Smart search, trending, recommendations
4. **Library grows** — More users = more songs = more value
5. **Community thrives** — Playlists, badges, leaderboards

The system is designed to scale from a small community to thousands of users while maintaining quality and preventing abuse.
