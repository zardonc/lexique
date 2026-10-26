use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechVoice {
    #[default]
    Siwis,
    Tom,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechSpeed {
    #[default]
    Normal,
    Slow,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechPreferences {
    pub voice: SpeechVoice,
    pub speed: SpeechSpeed,
}

pub fn speech_preferences(connection: &Connection) -> Result<SpeechPreferences, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'speech-preferences'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value {
        Some(value) => serde_json::from_str(&value).map_err(|error| {
            StorageError::Invalid(format!("Invalid saved speech preferences: {error}"))
        }),
        None => Ok(SpeechPreferences::default()),
    }
}

pub fn set_speech_preferences(
    connection: &Connection,
    preferences: &SpeechPreferences,
) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('speech-preferences', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(preferences)?],
    )?;
    Ok(())
}
