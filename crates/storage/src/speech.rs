use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeechVoice {
    #[default]
    #[serde(alias = "siwis")]
    Female,
    #[serde(alias = "tom")]
    Male,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpeechSpeed {
    Normal,
    Slow,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(try_from = "SavedSpeechPreferences")]
pub struct SpeechPreferences {
    pub voice: SpeechVoice,
    pub wpm: u16,
}

#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum SavedSpeechPreferences {
    Wpm {
        voice: SpeechVoice,
        wpm: u16,
    },
    Legacy {
        voice: SpeechVoice,
        speed: SpeechSpeed,
    },
}

impl Default for SpeechPreferences {
    fn default() -> Self {
        Self {
            voice: SpeechVoice::Female,
            wpm: 180,
        }
    }
}

impl SpeechPreferences {
    fn validate(&self) -> Result<(), StorageError> {
        if !(80..=240).contains(&self.wpm) || self.wpm % 20 != 0 {
            return Err(StorageError::Invalid(
                "Speech WPM must be 80–240 in steps of 20".into(),
            ));
        }
        Ok(())
    }
}

impl TryFrom<SavedSpeechPreferences> for SpeechPreferences {
    type Error = StorageError;

    fn try_from(saved: SavedSpeechPreferences) -> Result<Self, Self::Error> {
        let preferences = match saved {
            SavedSpeechPreferences::Wpm { voice, wpm } => Self { voice, wpm },
            SavedSpeechPreferences::Legacy { voice, speed } => Self {
                voice,
                wpm: match speed {
                    SpeechSpeed::Normal => 180,
                    SpeechSpeed::Slow => 100,
                },
            },
        };
        preferences.validate()?;
        Ok(preferences)
    }
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
    preferences.validate()?;
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('speech-preferences', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [serde_json::to_string(preferences)?],
    )?;
    Ok(())
}
