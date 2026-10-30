use lexique_storage::{SpeechPreferences, initialize, set_speech_preferences, speech_preferences};
use rusqlite::Connection;

#[test]
fn legacy_voice_preferences_keep_gender_and_speed_when_models_change() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (legacy, gender) in [("siwis", "female"), ("tom", "male")] {
        connection
            .execute(
                "INSERT INTO app_metadata VALUES ('speech-preferences', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [format!(r#"{{"voice":"{legacy}","speed":"slow"}}"#)],
            )
            .unwrap();
        let restored = speech_preferences(&connection).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::json!({"voice": gender, "speed": "slow"})
        );
        set_speech_preferences(&connection, &restored).unwrap();
        assert_eq!(
            serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"voice": gender, "speed": "slow"})
        );
    }
}

#[test]
fn speech_preferences_reject_unknown_or_incomplete_values_and_report_corrupt_storage() {
    for value in [
        serde_json::json!({"voice": "other", "speed": "normal"}),
        serde_json::json!({"voice": "female", "speed": "fast"}),
        serde_json::json!({"voice": "female"}),
        serde_json::json!({"voice": "male", "speed": "slow", "path": "external"}),
    ] {
        assert!(serde_json::from_value::<SpeechPreferences>(value).is_err());
    }
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO app_metadata VALUES ('speech-preferences', 'broken')",
            [],
        )
        .unwrap();
    assert!(
        speech_preferences(&connection)
            .unwrap_err()
            .to_string()
            .contains("Invalid saved speech preferences")
    );
}

#[test]
fn speech_choices_survive_reopening_without_changing_other_preferences() {
    let path = std::env::temp_dir().join(format!("lexique-speech-{}.sqlite3", std::process::id()));
    assert!(!path.exists());
    {
        let connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"voice": "female", "speed": "normal"})
        );
        lexique_storage::set_sort_preference(&connection, "headword-asc").unwrap();
        let choices: SpeechPreferences =
            serde_json::from_value(serde_json::json!({"voice": "male", "speed": "slow"})).unwrap();
        set_speech_preferences(&connection, &choices).unwrap();
    }
    {
        let connection = Connection::open(&path).unwrap();
        assert_eq!(
            serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"voice": "male", "speed": "slow"})
        );
        assert_eq!(
            lexique_storage::sort_preference(&connection).unwrap(),
            "headword-asc"
        );
    }
    std::fs::remove_file(path).unwrap();
}
