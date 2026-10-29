use std::{error::Error, fmt};

use lexique_core::DATABASE_SCHEMA_VERSION;
use rusqlite::Connection;

mod vocabulary;
pub use vocabulary::*;
mod speech;
pub use speech::*;
mod review;
pub use review::*;

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    FutureSchema(u32),
    Invalid(String),
    RatingUnavailable(String),
    RatingConflict,
    Json(serde_json::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite operation failed: {error}"),
            Self::Invalid(message) => write!(formatter, "{message}"),
            Self::RatingUnavailable(reason) => write!(formatter, "rating unavailable: {reason}"),
            Self::RatingConflict => write!(formatter, "Review card revision conflict"),
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
    let transaction = connection.unchecked_transaction()?;
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
    transaction.commit()?;
    Ok(DATABASE_SCHEMA_VERSION)
}
