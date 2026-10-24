use std::{error::Error, fmt};

use lexique_core::DATABASE_SCHEMA_VERSION;
use rusqlite::Connection;

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_initial.sql");

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    FutureSchema(u32),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite initialization failed: {error}"),
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
            Self::FutureSchema(_) => None,
        }
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
    transaction.execute_batch(INITIAL_SCHEMA)?;
    transaction.commit()?;
    Ok(DATABASE_SCHEMA_VERSION)
}
