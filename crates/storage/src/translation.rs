use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationProvider {
    Disabled,
    Deepl,
    OpenaiCompatible,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TranslationLanguage {
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TranslationSource {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeeplTier {
    Free,
    Pro,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeeplPreferences {
    pub tier: DeeplTier,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenaiCompatiblePreferences {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationPreferences {
    pub active_provider: TranslationProvider,
    pub source_language: TranslationSource,
    pub target_language: TranslationLanguage,
    pub deepl: DeeplPreferences,
    pub openai_compatible: OpenaiCompatiblePreferences,
}

pub fn translation_preferences(
    connection: &Connection,
    locale: &str,
) -> Result<TranslationPreferences, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'translation-preferences'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value {
        Some(value) => serde_json::from_str(&value).map_err(|error| {
            StorageError::Invalid(format!("Invalid saved translation preferences: {error}"))
        }),
        None => Ok(TranslationPreferences {
            active_provider: TranslationProvider::Disabled,
            source_language: TranslationSource::Auto,
            target_language: if locale == "zh" {
                TranslationLanguage::ZhCn
            } else {
                TranslationLanguage::En
            },
            deepl: DeeplPreferences {
                tier: DeeplTier::Free,
            },
            openai_compatible: OpenaiCompatiblePreferences {
                base_url: String::new(),
                model: String::new(),
            },
        }),
    }
}

pub fn set_translation_preferences(
    connection: &Connection,
    preferences: &TranslationPreferences,
) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('translation-preferences', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(preferences)?],
    )?;
    Ok(())
}

pub fn preserve_translation_preferences(
    source: &Connection,
    destination: &Connection,
) -> Result<(), StorageError> {
    let value: Option<String> = source
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'translation-preferences'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(value) = value {
        destination.execute(
            "INSERT INTO app_metadata (key, value) VALUES ('translation-preferences', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [value],
        )?;
    } else {
        destination.execute(
            "DELETE FROM app_metadata WHERE key = 'translation-preferences'",
            [],
        )?;
    }
    Ok(())
}
