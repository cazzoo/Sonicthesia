use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::song_library::models::{
    calculate_difficulty, FilterState, HighScoreEntry, SongEntry, SongMetadata, SortPreference,
};
use crate::song_library::scanner::SongScanner;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Song not found: {0}")]
    SongNotFound(i64),
    #[error("Database path not set")]
    PathNotSet,
    #[error("Database initialization failed: {0}")]
    InitializationFailed(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub trait SongRepository: Send + Sync {
    fn upsert_song(&self, metadata: &SongMetadata, file_path: &Path) -> Result<i64>;
    fn remove_song(&self, song_id: i64) -> Result<()>;
    fn get_song(&self, song_id: i64) -> Result<Option<SongEntry>>;
    fn list_songs(&self, sort: &SortPreference, filter: &FilterState) -> Result<Vec<SongEntry>>;
    fn update_stats(&self, song_id: i64, score: Option<f32>) -> Result<()>;
    fn update_genre(&self, song_id: i64, genre: Option<String>) -> Result<()>;
    fn update_labels(&self, song_id: i64, labels: Vec<String>) -> Result<()>;
    fn reset_score(&self, song_id: i64) -> Result<()>;
    fn song_count(&self) -> Result<usize>;
    fn save_high_score(
        &self,
        song_id: i64,
        score: u64,
        accuracy: f64,
        streak: u32,
        stars: u32,
        perfect_count: u32,
        good_count: u32,
        okay_count: u32,
        miss_count: u32,
    ) -> Result<()>;
    fn get_high_scores(&self, song_id: i64, limit: i64) -> Result<Vec<HighScoreEntry>>;
    fn get_best_score(&self, song_id: i64) -> Result<Option<HighScoreEntry>>;
    fn scan_directories(&self, _directories: &[PathBuf]) -> Result<usize> {
        Ok(0)
    }
}

const CURRENT_SCHEMA_VERSION: i32 = 3;

struct DatabaseConnection {
    conn: Connection,
}

impl DatabaseConnection {
    fn new(db_path: &Path) -> Result<Self> {
        let db_path_str = db_path.parent().ok_or_else(|| {
            DatabaseError::InitializationFailed("Invalid database path".to_string())
        })?;

        std::fs::create_dir_all(db_path_str).map_err(|e| {
            DatabaseError::InitializationFailed(format!(
                "Failed to create database directory: {}",
                e
            ))
        })?;

        let conn = Connection::open(db_path).map_err(|e| {
            DatabaseError::InitializationFailed(format!("Failed to open database: {}", e))
        })?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| {
                DatabaseError::InitializationFailed(format!("Failed to enable WAL mode: {}", e))
            })?;

        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| {
                DatabaseError::InitializationFailed(format!(
                    "Failed to set synchronous mode: {}",
                    e
                ))
            })?;

        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| {
                DatabaseError::InitializationFailed(format!("Failed to enable foreign keys: {}", e))
            })?;

        Ok(Self { conn })
    }

    fn initialize_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        let current_version: Option<i32> = self
            .conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .ok();

        if current_version.is_none() {
            self.create_schema()?;
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![CURRENT_SCHEMA_VERSION],
            )?;
        } else {
            let version = current_version.unwrap();
            if version < CURRENT_SCHEMA_VERSION {
                self.run_migrations(version)?;
            }
        }

        Ok(())
    }

    fn run_migrations(&mut self, from_version: i32) -> Result<()> {
        if from_version < 2 {
            self.migrate_to_v2()?;
        }
        if from_version < 3 {
            self.migrate_to_v3()?;
        }

        Ok(())
    }

    fn migrate_to_v2(&mut self) -> Result<()> {
        self.conn
            .execute("ALTER TABLE songs ADD COLUMN genre TEXT", [])?;

        self.conn
            .execute("ALTER TABLE songs ADD COLUMN labels TEXT", [])?;

        self.conn
            .execute("UPDATE schema_version SET version = 2", [])?;

        Ok(())
    }

    fn migrate_to_v3(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS achievements (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                unlocked_at INTEGER,
                progress REAL DEFAULT 0.0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS challenges (
                id TEXT PRIMARY KEY,
                challenge_type TEXT NOT NULL,
                target REAL NOT NULL,
                progress REAL DEFAULT 0.0,
                completed INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS high_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                song_id INTEGER NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
                score INTEGER NOT NULL,
                accuracy REAL NOT NULL,
                streak INTEGER NOT NULL,
                stars INTEGER NOT NULL,
                perfect_count INTEGER NOT NULL DEFAULT 0,
                good_count INTEGER NOT NULL DEFAULT 0,
                okay_count INTEGER NOT NULL DEFAULT 0,
                miss_count INTEGER NOT NULL DEFAULT 0,
                played_at INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_high_scores_song_id ON high_scores(song_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_high_scores_score ON high_scores(score DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS learning_progress (
                path_id TEXT NOT NULL,
                stage_id TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                completed_at INTEGER,
                PRIMARY KEY (path_id, stage_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS skill_metrics (
                date INTEGER PRIMARY KEY,
                avg_accuracy REAL DEFAULT 0.0,
                avg_streak INTEGER DEFAULT 0,
                max_streak INTEGER DEFAULT 0,
                songs_played INTEGER DEFAULT 0,
                total_play_time INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "ALTER TABLE songs ADD COLUMN best_stars INTEGER DEFAULT 0",
            [],
        )?;

        self.conn.execute(
            "ALTER TABLE songs ADD COLUMN best_streak INTEGER DEFAULT 0",
            [],
        )?;

        self.conn
            .execute("UPDATE schema_version SET version = 3", [])?;

        Ok(())
    }

    fn create_schema(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                difficulty INTEGER NOT NULL,
                duration INTEGER NOT NULL,
                track_count INTEGER NOT NULL,
                note_count INTEGER NOT NULL,
                tempo_changes INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                file_modified INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                indexed_at INTEGER NOT NULL,
                genre TEXT,
                labels TEXT,
                best_stars INTEGER DEFAULT 0,
                best_streak INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS song_stats (
                song_id INTEGER PRIMARY KEY REFERENCES songs(id) ON DELETE CASCADE,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_score REAL,
                best_score REAL,
                last_played_at INTEGER,
                first_played_at INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS achievements (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                unlocked_at INTEGER,
                progress REAL DEFAULT 0.0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS challenges (
                id TEXT PRIMARY KEY,
                challenge_type TEXT NOT NULL,
                target REAL NOT NULL,
                progress REAL DEFAULT 0.0,
                completed INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS high_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                song_id INTEGER NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
                score INTEGER NOT NULL,
                accuracy REAL NOT NULL,
                streak INTEGER NOT NULL,
                stars INTEGER NOT NULL,
                perfect_count INTEGER NOT NULL DEFAULT 0,
                good_count INTEGER NOT NULL DEFAULT 0,
                okay_count INTEGER NOT NULL DEFAULT 0,
                miss_count INTEGER NOT NULL DEFAULT 0,
                played_at INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS learning_progress (
                path_id TEXT NOT NULL,
                stage_id TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                completed_at INTEGER,
                PRIMARY KEY (path_id, stage_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS skill_metrics (
                date INTEGER PRIMARY KEY,
                avg_accuracy REAL DEFAULT 0.0,
                avg_streak INTEGER DEFAULT 0,
                max_streak INTEGER DEFAULT 0,
                songs_played INTEGER DEFAULT 0,
                total_play_time INTEGER DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_songs_name ON songs(name)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_songs_difficulty ON songs(difficulty)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_song_stats_last_played ON song_stats(last_played_at)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_song_stats_play_count ON song_stats(play_count)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_songs_file_path ON songs(file_path)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_high_scores_song_id ON high_scores(song_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_high_scores_score ON high_scores(score DESC)",
            [],
        )?;

        Ok(())
    }
}

pub struct SqliteSongRepository {
    _db: Arc<Mutex<DatabaseConnection>>,
    db_path: PathBuf,
}

impl SqliteSongRepository {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let mut db_conn = DatabaseConnection::new(&db_path)?;
        db_conn.initialize_schema()?;

        Ok(Self {
            _db: Arc::new(Mutex::new(db_conn)),
            db_path,
        })
    }

    pub fn with_default_path() -> Result<Self> {
        let db_path = Self::default_db_path()?;
        Self::new(db_path)
    }

    fn default_db_path() -> Result<PathBuf> {
        let base_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("neothesia")
        } else {
            return Err(DatabaseError::PathNotSet);
        };

        Ok(base_dir.join("song_library.db"))
    }

    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(Into::into)
    }

    fn build_where_clause(filter: &FilterState) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(min_diff) = filter.difficulty_min {
            conditions.push("s.difficulty >= ?".to_string());
            params.push(Box::new(min_diff));
        }

        if let Some(max_diff) = filter.difficulty_max {
            conditions.push("s.difficulty <= ?".to_string());
            params.push(Box::new(max_diff));
        }

        if filter.played_only {
            conditions.push("stats.play_count > 0".to_string());
        }

        if filter.unplayed_only {
            conditions.push("stats.play_count = 0 OR stats.play_count IS NULL".to_string());
        }

        if let Some(query) = &filter.search_query {
            if !query.trim().is_empty() {
                conditions.push("s.name LIKE ?".to_string());
                params.push(Box::new(format!("%{}%", query)));
            }
        }

        if let Some(genre) = &filter.genre {
            if !genre.trim().is_empty() {
                conditions.push("s.genre LIKE ?".to_string());
                params.push(Box::new(format!("%{}%", genre)));
            }
        }

        if let Some(min_score) = filter.score_min {
            conditions.push("stats.best_score >= ?".to_string());
            params.push(Box::new(min_score));
        }

        if let Some(max_score) = filter.score_max {
            conditions.push("stats.best_score <= ?".to_string());
            params.push(Box::new(max_score));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }

    fn build_order_clause(sort: &SortPreference) -> &'static str {
        match sort {
            SortPreference::NameAsc => "ORDER BY s.name ASC",
            SortPreference::NameDesc => "ORDER BY s.name DESC",
            SortPreference::DifficultyAsc => "ORDER BY s.difficulty ASC",
            SortPreference::DifficultyDesc => "ORDER BY s.difficulty DESC",
            SortPreference::PlayCountDesc => "ORDER BY stats.play_count DESC",
            SortPreference::PlayCountAsc => "ORDER BY stats.play_count ASC",
            SortPreference::LastPlayedDesc => "ORDER BY stats.last_played_at DESC",
            SortPreference::LastPlayedAsc => "ORDER BY stats.last_played_at ASC",
            SortPreference::LastScoreDesc => "ORDER BY stats.last_score DESC",
            SortPreference::LastScoreAsc => "ORDER BY stats.last_score ASC",
            SortPreference::GenreAsc => "ORDER BY s.genre ASC",
            SortPreference::GenreDesc => "ORDER BY s.genre DESC",
        }
    }
}

impl SongRepository for SqliteSongRepository {
    fn upsert_song(&self, metadata: &SongMetadata, file_path: &Path) -> Result<i64> {
        let conn = self.get_connection()?;

        let (file_size, file_modified) = match std::fs::metadata(file_path) {
            Ok(file_metadata) => {
                let modified = file_metadata
                    .modified()
                    .map_err(|e| {
                        DatabaseError::InitializationFailed(format!(
                            "Failed to get file modified time: {}",
                            e
                        ))
                    })?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| {
                        DatabaseError::InitializationFailed(format!(
                            "Failed to convert file time: {}",
                            e
                        ))
                    })?
                    .as_secs() as i64;
                (file_metadata.len(), modified)
            }
            Err(_) => (0, Utc::now().timestamp()),
        };

        let now = Utc::now().timestamp();

        let difficulty = calculate_difficulty(metadata);
        let labels_json =
            serde_json::to_string(&metadata.labels).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO songs (
                file_path, name, difficulty, duration, track_count, note_count, tempo_changes,
                file_size, file_modified, created_at, indexed_at, genre, labels
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(file_path) DO UPDATE SET
                name = excluded.name,
                difficulty = excluded.difficulty,
                duration = excluded.duration,
                track_count = excluded.track_count,
                note_count = excluded.note_count,
                tempo_changes = excluded.tempo_changes,
                file_size = excluded.file_size,
                file_modified = excluded.file_modified,
                indexed_at = excluded.indexed_at,
                genre = excluded.genre,
                labels = excluded.labels
            ",
            params![
                file_path.to_string_lossy(),
                metadata.name,
                difficulty,
                metadata.duration_secs as i64,
                metadata.track_count as i64,
                metadata.note_count as i64,
                metadata.tempo_changes as i64,
                file_size,
                file_modified,
                now,
                now,
                metadata.genre,
                labels_json,
            ],
        )?;

        let song_id: i64 = conn.query_row(
            "SELECT id FROM songs WHERE file_path = ?1",
            params![file_path.to_string_lossy()],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO song_stats (song_id, play_count) VALUES (?1, 0)",
            params![song_id],
        )?;

        Ok(song_id)
    }

    fn remove_song(&self, song_id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        let changes = conn.execute("DELETE FROM songs WHERE id = ?1", params![song_id])?;

        if changes == 0 {
            return Err(DatabaseError::SongNotFound(song_id));
        }

        Ok(())
    }

    fn get_song(&self, song_id: i64) -> Result<Option<SongEntry>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT
                s.id,
                s.file_path,
                s.name,
                s.difficulty,
                s.duration,
                s.track_count,
                COALESCE(stats.play_count, 0) as play_count,
                stats.last_score,
                stats.best_score,
                stats.last_played_at,
                s.created_at,
                s.genre,
                s.labels
            FROM songs s
            LEFT JOIN song_stats stats ON s.id = stats.song_id
            WHERE s.id = ?1",
        )?;

        let result = stmt.query_row(params![song_id], |row| {
            let labels_json: Option<String> = row.get(12)?;
            let labels = labels_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            Ok(SongEntry {
                id: row.get(0)?,
                file_path: PathBuf::from(row.get::<_, String>(1)?),
                name: row.get(2)?,
                difficulty: row.get(3)?,
                duration_secs: row.get(4)?,
                track_count: row.get(5)?,
                play_count: row.get(6)?,
                last_score: row.get(7)?,
                best_score: row.get(8)?,
                last_played_at: row
                    .get::<_, Option<i64>>(9)?
                    .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
                created_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap(),
                genre: row.get(11)?,
                labels,
            })
        });

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_songs(&self, sort: &SortPreference, filter: &FilterState) -> Result<Vec<SongEntry>> {
        let conn = self.get_connection()?;

        let (where_clause, params) = Self::build_where_clause(filter);
        let order_clause = Self::build_order_clause(sort);

        let query = format!(
            "SELECT
                s.id,
                s.file_path,
                s.name,
                s.difficulty,
                s.duration,
                s.track_count,
                COALESCE(stats.play_count, 0) as play_count,
                stats.last_score,
                stats.best_score,
                stats.last_played_at,
                s.created_at,
                s.genre,
                s.labels
            FROM songs s
            LEFT JOIN song_stats stats ON s.id = stats.song_id
            {}
            {}",
            where_clause, order_clause
        );

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let entries = stmt
            .query_map(param_refs.as_slice(), |row| {
                let labels_json: Option<String> = row.get(12)?;
                let labels = labels_json
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                Ok(SongEntry {
                    id: row.get(0)?,
                    file_path: PathBuf::from(row.get::<_, String>(1)?),
                    name: row.get(2)?,
                    difficulty: row.get(3)?,
                    duration_secs: row.get(4)?,
                    track_count: row.get(5)?,
                    play_count: row.get(6)?,
                    last_score: row.get(7)?,
                    best_score: row.get(8)?,
                    last_played_at: row
                        .get::<_, Option<i64>>(9)?
                        .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
                    created_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap(),
                    genre: row.get(11)?,
                    labels,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    fn update_stats(&self, song_id: i64, score: Option<f32>) -> Result<()> {
        let conn = self.get_connection()?;

        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO song_stats (song_id, play_count, last_score, best_score, last_played_at, first_played_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(song_id) DO UPDATE SET
                play_count = play_count + 1,
                last_score = CASE
                    WHEN excluded.last_score IS NOT NULL
                    THEN excluded.last_score
                    ELSE last_score
                END,
                best_score = CASE
                    WHEN excluded.best_score IS NOT NULL AND (best_score IS NULL OR excluded.best_score > best_score)
                    THEN excluded.best_score
                    ELSE best_score
                END,
                last_played_at = excluded.last_played_at,
                first_played_at = CASE
                    WHEN first_played_at IS NULL
                    THEN excluded.first_played_at
                    ELSE first_played_at
                END",
            params![
                song_id,
                1,
                score,
                score,
                now,
                now,
            ],
        )?;

        Ok(())
    }

    fn song_count(&self) -> Result<usize> {
        let conn = self.get_connection()?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM songs", [], |row| row.get(0))?;

        Ok(count as usize)
    }

    fn scan_directories(&self, directories: &[PathBuf]) -> Result<usize> {
        let scanner = SongScanner::new();
        let summary = scanner.index_directories(directories, self, None);

        if !summary.errors.is_empty() {
            log::warn!(
                "Song library scan completed with {} errors:",
                summary.errors.len()
            );
            for error in summary.errors.iter().take(10) {
                log::warn!("  {}", error);
            }
            if summary.errors.len() > 10 {
                log::warn!("  ... and {} more", summary.errors.len() - 10);
            }
        }

        Ok(summary.songs_added)
    }

    fn update_genre(&self, song_id: i64, genre: Option<String>) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "UPDATE songs SET genre = ?1 WHERE id = ?2",
            params![genre, song_id],
        )?;

        Ok(())
    }

    fn update_labels(&self, song_id: i64, labels: Vec<String>) -> Result<()> {
        let conn = self.get_connection()?;

        let labels_json = serde_json::to_string(&labels).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "UPDATE songs SET labels = ?1 WHERE id = ?2",
            params![labels_json, song_id],
        )?;

        Ok(())
    }

    fn reset_score(&self, song_id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        conn.execute(
            "UPDATE song_stats SET play_count = 0, last_score = NULL, best_score = NULL WHERE song_id = ?1",
            params![song_id],
        )?;

        Ok(())
    }

    fn save_high_score(
        &self,
        song_id: i64,
        score: u64,
        accuracy: f64,
        streak: u32,
        stars: u32,
        perfect_count: u32,
        good_count: u32,
        okay_count: u32,
        miss_count: u32,
    ) -> Result<()> {
        let conn = self.get_connection()?;
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT INTO high_scores (
                song_id, score, accuracy, streak, stars,
                perfect_count, good_count, okay_count, miss_count, played_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                song_id,
                score as i64,
                accuracy,
                streak as i64,
                stars as i64,
                perfect_count as i64,
                good_count as i64,
                okay_count as i64,
                miss_count as i64,
                now,
            ],
        )?;

        conn.execute(
            "UPDATE songs SET best_stars = MAX(COALESCE(best_stars, 0), ?1), best_streak = MAX(COALESCE(best_streak, 0), ?2) WHERE id = ?3",
            params![stars as i64, streak as i64, song_id],
        )?;

        Ok(())
    }

    fn get_high_scores(&self, song_id: i64, limit: i64) -> Result<Vec<HighScoreEntry>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, song_id, score, accuracy, streak, stars,
                    perfect_count, good_count, okay_count, miss_count, played_at
             FROM high_scores
             WHERE song_id = ?1
             ORDER BY score DESC
             LIMIT ?2",
        )?;

        let entries = stmt
            .query_map(params![song_id, limit], |row| {
                Ok(HighScoreEntry {
                    id: row.get(0)?,
                    song_id: row.get(1)?,
                    score: row.get::<_, i64>(2)? as u64,
                    accuracy: row.get(3)?,
                    streak: row.get::<_, i64>(4)? as u32,
                    stars: row.get::<_, i64>(5)? as u32,
                    perfect_count: row.get::<_, i64>(6)? as u32,
                    good_count: row.get::<_, i64>(7)? as u32,
                    okay_count: row.get::<_, i64>(8)? as u32,
                    miss_count: row.get::<_, i64>(9)? as u32,
                    played_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    fn get_best_score(&self, song_id: i64) -> Result<Option<HighScoreEntry>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, song_id, score, accuracy, streak, stars,
                    perfect_count, good_count, okay_count, miss_count, played_at
             FROM high_scores
             WHERE song_id = ?1
             ORDER BY score DESC
             LIMIT 1",
        )?;

        let result = stmt.query_row(params![song_id], |row| {
            Ok(HighScoreEntry {
                id: row.get(0)?,
                song_id: row.get(1)?,
                score: row.get::<_, i64>(2)? as u64,
                accuracy: row.get(3)?,
                streak: row.get::<_, i64>(4)? as u32,
                stars: row.get::<_, i64>(5)? as u32,
                perfect_count: row.get::<_, i64>(6)? as u32,
                good_count: row.get::<_, i64>(7)? as u32,
                okay_count: row.get::<_, i64>(8)? as u32,
                miss_count: row.get::<_, i64>(9)? as u32,
                played_at: DateTime::from_timestamp(row.get(10)?, 0).unwrap(),
            })
        });

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo() -> SqliteSongRepository {
        let db_path = PathBuf::from("/tmp/test_song_library.db");
        let _ = std::fs::remove_file(&db_path);
        SqliteSongRepository::new(db_path).unwrap()
    }

    #[test]
    fn test_upsert_and_get_song() {
        let repo = create_test_repo();

        let metadata = SongMetadata {
            name: "Test Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 1000,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        let file_path = Path::new("/test/path.mid");

        let song_id = repo.upsert_song(&metadata, file_path).unwrap();
        let song = repo.get_song(song_id).unwrap().unwrap();

        assert_eq!(song.name, "Test Song");
        assert_eq!(song.duration_secs, 180);
        assert_eq!(song.track_count, 3);
    }

    #[test]
    fn test_remove_song() {
        let repo = create_test_repo();

        let metadata = SongMetadata {
            name: "Test Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 1000,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        let file_path = Path::new("/test/path.mid");

        let song_id = repo.upsert_song(&metadata, file_path).unwrap();
        repo.remove_song(song_id).unwrap();

        let song = repo.get_song(song_id).unwrap();
        assert!(song.is_none());
    }

    #[test]
    fn test_update_stats() {
        let repo = create_test_repo();

        let metadata = SongMetadata {
            name: "Test Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 1000,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        let file_path = Path::new("/test/path.mid");

        let song_id = repo.upsert_song(&metadata, file_path).unwrap();

        repo.update_stats(song_id, Some(85.5)).unwrap();

        let song = repo.get_song(song_id).unwrap().unwrap();
        assert_eq!(song.play_count, 1);
        assert_eq!(song.last_score, Some(85.5));
        assert_eq!(song.best_score, Some(85.5));
        assert!(song.last_played_at.is_some());
    }

    #[test]
    fn test_list_songs_with_sort() {
        let repo = create_test_repo();

        let metadata1 = SongMetadata {
            name: "Alpha Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 500,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        let metadata2 = SongMetadata {
            name: "Zeta Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 2000,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        repo.upsert_song(&metadata1, Path::new("/test/alpha.mid"))
            .unwrap();
        repo.upsert_song(&metadata2, Path::new("/test/zeta.mid"))
            .unwrap();

        let songs = repo
            .list_songs(&SortPreference::NameAsc, &FilterState::default())
            .unwrap();
        assert_eq!(songs[0].name, "Alpha Song");
        assert_eq!(songs[1].name, "Zeta Song");

        let songs = repo
            .list_songs(&SortPreference::NameDesc, &FilterState::default())
            .unwrap();
        assert_eq!(songs[0].name, "Zeta Song");
        assert_eq!(songs[1].name, "Alpha Song");
    }

    #[test]
    fn test_song_count() {
        let repo = create_test_repo();

        assert_eq!(repo.song_count().unwrap(), 0);

        let metadata = SongMetadata {
            name: "Test Song".to_string(),
            duration_secs: 180,
            track_count: 3,
            note_count: 1000,
            tempo_changes: 5,
            genre: None,
            labels: Vec::new(),
        };

        repo.upsert_song(&metadata, Path::new("/test/path.mid"))
            .unwrap();
        assert_eq!(repo.song_count().unwrap(), 1);
    }
}
