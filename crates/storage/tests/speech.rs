use lexique_storage::{SpeechPreferences, initialize, set_speech_preferences, speech_preferences};
use rusqlite::Connection;

#[test]
fn legacy_voice_and_speed_preferences_restore_as_wpm_without_rewriting_on_read() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (legacy, gender, speed, wpm) in [
        ("siwis", "female", "normal", 180),
        ("siwis", "female", "slow", 100),
        ("tom", "male", "normal", 180),
        ("tom", "male", "slow", 100),
    ] {
        let saved = format!(r#"{{"voice":"{legacy}","speed":"{speed}"}}"#);
        connection
            .execute(
                "INSERT INTO app_metadata VALUES ('speech-preferences', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [&saved],
            )
            .unwrap();
        let restored = speech_preferences(&connection).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::json!({"voice": gender, "wpm": wpm})
        );
        let unchanged: String = connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key = 'speech-preferences'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(unchanged, saved);
        set_speech_preferences(&connection, &restored).unwrap();
        assert_eq!(
            serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"voice": gender, "wpm": wpm})
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
        serde_json::json!({"voice": "female", "wpm": 60}),
        serde_json::json!({"voice": "female", "wpm": 260}),
        serde_json::json!({"voice": "female", "wpm": 90}),
        serde_json::json!({"voice": "female", "wpm": 180.5}),
        serde_json::json!({"voice": "female", "wpm": 180.0}),
        serde_json::json!({"voice": "female", "wpm": "180"}),
        serde_json::json!({"voice": "female", "wpm": 180, "speed": "normal"}),
        serde_json::json!({"voice": "female", "wpm": 180, "path": "external"}),
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
            serde_json::json!({"voice": "female", "wpm": 180})
        );
        lexique_storage::set_sort_preference(&connection, "headword-asc").unwrap();
        let choices: SpeechPreferences =
            serde_json::from_value(serde_json::json!({"voice": "male", "wpm": 100})).unwrap();
        set_speech_preferences(&connection, &choices).unwrap();
    }
    {
        let connection = Connection::open(&path).unwrap();
        assert_eq!(
            serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"voice": "male", "wpm": 100})
        );
        assert_eq!(
            lexique_storage::sort_preference(&connection).unwrap(),
            "headword-asc"
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn all_wpm_steps_save_and_failed_writes_preserve_the_saved_preferences() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for voice in ["female", "male"] {
        for wpm in (80..=240).step_by(20) {
            let expected = serde_json::json!({"voice": voice, "wpm": wpm});
            let choices: SpeechPreferences = serde_json::from_value(expected.clone()).unwrap();
            set_speech_preferences(&connection, &choices).unwrap();
            assert_eq!(
                serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
                expected
            );
        }
    }
    let invalid = SpeechPreferences {
        voice: lexique_storage::SpeechVoice::Female,
        wpm: 90,
    };
    assert!(set_speech_preferences(&connection, &invalid).is_err());
    connection.execute_batch("PRAGMA query_only=ON").unwrap();
    assert!(set_speech_preferences(&connection, &SpeechPreferences::default()).is_err());
    assert_eq!(
        serde_json::to_value(speech_preferences(&connection).unwrap()).unwrap(),
        serde_json::json!({"voice": "male", "wpm": 240})
    );
}

#[test]
fn playback_voice_comes_from_recorded_audio_and_corrupt_metadata_stays_an_error() {
    use lexique_storage::{SpeechSpec, SpeechVoice, register_speech, speech_audio_voice};
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    set_speech_preferences(
        &connection,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            wpm: 240,
        },
    )
    .unwrap();
    assert_eq!(
        speech_audio_voice(&connection, Some(&key)).unwrap(),
        Some(SpeechVoice::Female)
    );
    assert_eq!(speech_audio_voice(&connection, None).unwrap(), None);
    let mut old = SpeechSpec::word("bonjour", SpeechVoice::Male);
    old.model = "legacy".into();
    let old_key = register_speech(&connection, &old).unwrap();
    assert_eq!(
        speech_audio_voice(&connection, Some(&old_key)).unwrap(),
        None
    );
    connection
        .execute(
            "UPDATE speech_assets SET spec='broken' WHERE semantic_key=?1",
            [&key],
        )
        .unwrap();
    assert!(speech_audio_voice(&connection, Some(&key)).is_err());
}
