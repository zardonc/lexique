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

const DEFAULT_BLOCK_ORDER: [&str; 23] = [
    "P", "PC", "I", "F", "C", "S", "PQP", "J", "PA", "FA", "CP", "SP", "T", "SQP", "Y", "YP", "W",
    "WP", "G", "K", "KP", "GP", "GPP",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(try_from = "Vec<String>")]
pub struct ConjugationBlockOrder(Vec<String>);

impl Default for ConjugationBlockOrder {
    fn default() -> Self {
        Self(
            DEFAULT_BLOCK_ORDER
                .iter()
                .map(|code| (*code).into())
                .collect(),
        )
    }
}

impl TryFrom<Vec<String>> for ConjugationBlockOrder {
    type Error = String;

    fn try_from(order: Vec<String>) -> Result<Self, Self::Error> {
        if order.len() != DEFAULT_BLOCK_ORDER.len()
            || DEFAULT_BLOCK_ORDER
                .iter()
                .any(|code| order.iter().filter(|value| value.as_str() == *code).count() != 1)
        {
            return Err("Block order must contain all 23 tense blocks exactly once".into());
        }
        Ok(Self(order))
    }
}

pub fn conjugation_block_order(
    connection: &Connection,
) -> Result<ConjugationBlockOrder, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'conjugation-block-order'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value {
        Some(value) => serde_json::from_str(&value).map_err(|error| {
            StorageError::Invalid(format!("Invalid saved conjugation block order: {error}"))
        }),
        None => Ok(ConjugationBlockOrder::default()),
    }
}

pub fn set_conjugation_block_order(
    connection: &Connection,
    order: &ConjugationBlockOrder,
) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('conjugation-block-order', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(order)?],
    )?;
    Ok(())
}
