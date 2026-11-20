use std::{error::Error, fmt};

pub use lexique_core::DATABASE_SCHEMA_VERSION;
use rusqlite::Connection;

mod vocabulary;
pub use vocabulary::*;
mod note_export;
pub use note_export::*;
mod grammar;
pub use grammar::*;
mod categories;
pub use categories::*;
mod speech;
pub use speech::*;
mod conjugation;
pub use conjugation::*;
mod interface;
pub use interface::*;
mod review;
pub use review::*;
mod speech_work;
pub use speech_work::*;
mod speech_demand;
pub use speech_demand::*;
mod selected_audio;
pub use selected_audio::*;
mod audio_lifecycle;
pub use audio_lifecycle::*;
mod queue_cache;
pub use queue_cache::*;
mod dictionary_catalog;
pub use dictionary_catalog::*;
mod learning_snapshot;
pub use learning_snapshot::*;
mod snapshot_export;
pub use snapshot_export::*;
mod snapshot_import;
pub use snapshot_import::import_learning_snapshot;
mod snapshot_validation;
pub use snapshot_validation::{read_learning_snapshot, validate_learning_snapshot};

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");
const LEARNING_ROWS: &str = "SELECT 1 FROM vocabulary UNION ALL SELECT 1 FROM save_operations
     UNION ALL SELECT 1 FROM review_cards UNION ALL SELECT 1 FROM review_progress
     UNION ALL SELECT 1 FROM review_ratings UNION ALL SELECT 1 FROM review_queue_state
     UNION ALL SELECT 1 FROM review_queue_entries UNION ALL SELECT 1 FROM review_paused
     UNION ALL SELECT 1 FROM review_batches UNION ALL SELECT 1 FROM review_batch_items
     UNION ALL SELECT 1 FROM speech_assets UNION ALL SELECT 1 FROM speech_owners
     UNION ALL SELECT 1 FROM speech_tasks UNION ALL SELECT 1 FROM speech_versions
     UNION ALL SELECT 1 FROM word_audio_selections UNION ALL SELECT 1 FROM word_audio_imports
     UNION ALL SELECT 1 FROM legacy_speech UNION ALL SELECT 1 FROM word_audio_operations
     UNION ALL SELECT 1 FROM word_audio_publications UNION ALL SELECT 1 FROM word_audio_repairs
     UNION ALL SELECT 1 FROM grammar_cards UNION ALL SELECT 1 FROM learning_categories
     UNION ALL SELECT 1 FROM record_categories UNION ALL SELECT 1 FROM retained_review_usage
     UNION ALL SELECT 1 FROM deleted_grammar_operations
     UNION ALL SELECT 1 FROM app_metadata LIMIT 1";

/// Checks storage before learning workers start; the schema version alone cannot prove health.
pub fn validate_learning_database(connection: &Connection) -> Result<(), StorageError> {
    connection.prepare(LEARNING_ROWS)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StorageError::Invalid(format!(
            "Learning database integrity check failed: {integrity}"
        )));
    }
    if connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(StorageError::Invalid(
            "Learning database relationships are invalid".into(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    FutureSchema(u32),
    Invalid(String),
    RatingUnavailable(String),
    RatingConflict,
    QueueChanged,
    Json(serde_json::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite operation failed: {error}"),
            Self::Invalid(message) => write!(formatter, "{message}"),
            Self::RatingUnavailable(reason) => write!(formatter, "rating unavailable: {reason}"),
            Self::RatingConflict => write!(formatter, "Review card revision conflict"),
            Self::QueueChanged => {
                write!(formatter, "Review queue changed during audio checks; retry")
            }
            Self::Json(error) => write!(formatter, "Invalid saved vocabulary data: {error}"),
            Self::FutureSchema(version) => write!(
                formatter,
                "Unsupported database schema {version}; this app supports {DATABASE_SCHEMA_VERSION}"
            ),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::FutureSchema(_)
            | Self::Invalid(_)
            | Self::RatingUnavailable(_)
            | Self::RatingConflict => None,
            Self::QueueChanged => None,
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub fn initialize(connection: &Connection) -> Result<u32, StorageError> {
    connection.pragma_update(None, "foreign_keys", true)?;
    let version =
        connection.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))?;
    if version > DATABASE_SCHEMA_VERSION {
        return Err(StorageError::FutureSchema(version));
    }
    if version == DATABASE_SCHEMA_VERSION {
        return Ok(version);
    }
    if !connection.is_autocommit() {
        return Err(StorageError::Invalid(
            "Schema migration requires its own transaction".into(),
        ));
    }
    // A referenced parent-table rebuild requires FK checks off before BEGIN, then an explicit check.
    connection.pragma_update(None, "foreign_keys", false)?;
    let result = migrate(connection, version);
    connection.pragma_update(None, "foreign_keys", true)?;
    result
}

fn migrate(connection: &Connection, version: u32) -> Result<u32, StorageError> {
    let transaction =
        rusqlite::Transaction::new_unchecked(connection, rusqlite::TransactionBehavior::Immediate)?;
    if version < 1 {
        transaction.execute_batch(INITIAL_SCHEMA)?;
    }
    if version < 2 {
        transaction.execute_batch(include_str!("../migrations/0002_vocabulary.sql"))?;
    }
    if version < 3 {
        transaction.execute_batch(include_str!("../migrations/0003_review.sql"))?;
    }
    if version < 4 {
        transaction.execute_batch(include_str!("../migrations/0004_ratings.sql"))?;
    }
    if version < 5 {
        transaction.execute_batch(include_str!("../migrations/0005_review_settings.sql"))?;
    }
    if version < 6 {
        transaction.execute_batch(include_str!("../migrations/0006_undo.sql"))?;
    }
    if version < 7 {
        transaction.execute_batch(include_str!("../migrations/0007_review_management.sql"))?;
    }
    if version < 8 {
        transaction.execute_batch(include_str!("../migrations/0008_examples.sql"))?;
    }
    if version < 9 {
        transaction.execute_batch(include_str!("../migrations/0009_more_examples.sql"))?;
    }
    if version < 10 {
        transaction.execute_batch(include_str!("../migrations/0010_example_sense.sql"))?;
    }
    if version < 11 {
        transaction.execute_batch(include_str!("../migrations/0011_speech_work.sql"))?;
    }
    if version < 12 {
        transaction.execute_batch(include_str!("../migrations/0012_selected_audio.sql"))?;
    }
    if version < 13 {
        transaction.execute_batch(include_str!("../migrations/0013_speech_successors.sql"))?;
    }
    if version < 14 {
        transaction.execute_batch(include_str!("../migrations/0014_audio_operations.sql"))?;
    }
    if version < 15 {
        transaction.execute_batch(include_str!("../migrations/0015_audio_lifecycle.sql"))?;
    }
    if version < 16 {
        transaction.execute_batch(include_str!("../migrations/0016_queue_revision.sql"))?;
    }
    if version < 17 {
        transaction.execute_batch(include_str!("../migrations/0017_audio_demands.sql"))?;
    }
    if version < 18 {
        transaction.execute_batch(include_str!("../migrations/0018_example_sources.sql"))?;
    }
    if version < 19 {
        transaction.execute_batch(include_str!("../migrations/0019_grammar_categories.sql"))?;
    }
    if transaction
        .prepare("PRAGMA foreign_key_check")?
        .exists([])?
    {
        return Err(StorageError::Invalid(
            "Schema migration failed its foreign-key check".into(),
        ));
    }
    transaction.commit()?;
    Ok(DATABASE_SCHEMA_VERSION)
}
