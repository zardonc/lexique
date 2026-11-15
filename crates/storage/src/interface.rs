use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceLocale {
    #[default]
    Zh,
    En,
    Fr,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceTheme {
    #[default]
    White,
    Sepia,
    Dark,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterfacePreferences {
    pub locale: InterfaceLocale,
    pub theme: InterfaceTheme,
}

pub fn interface_preferences(
    connection: &Connection,
) -> Result<InterfacePreferences, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'interface-preferences'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value {
        Some(value) => serde_json::from_str(&value).map_err(|error| {
            StorageError::Invalid(format!("Invalid saved interface preferences: {error}"))
        }),
        None => Ok(InterfacePreferences::default()),
    }
}

pub fn set_interface_preferences(
    connection: &Connection,
    preferences: &InterfacePreferences,
) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('interface-preferences', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(preferences)?],
    )?;
    Ok(())
}
