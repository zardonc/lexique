use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConjugationMode {
    Verbal,
    Main,
    #[default]
    Ending,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConjugationPreferences {
    pub mode: ConjugationMode,
}

pub fn conjugation_preferences(
    connection: &Connection,
) -> Result<ConjugationPreferences, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'conjugation-preferences'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value {
        Some(value) => serde_json::from_str(&value).map_err(|error| {
            StorageError::Invalid(format!("Invalid saved conjugation preferences: {error}"))
        }),
        None => Ok(ConjugationPreferences::default()),
    }
}

pub fn set_conjugation_preferences(
    connection: &Connection,
    preferences: &ConjugationPreferences,
) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('conjugation-preferences', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(preferences)?],
    )?;
    Ok(())
}
