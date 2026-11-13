use lexique_storage::{
    InterfacePreferences, initialize, interface_preferences, set_interface_preferences,
    set_sort_preference, sort_preference,
};
use rusqlite::Connection;

#[test]
fn interface_choices_survive_reopening_without_changing_other_preferences() {
    let path =
        std::env::temp_dir().join(format!("lexique-interface-{}.sqlite3", std::process::id()));
    assert!(!path.exists());
    {
        let connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            serde_json::to_value(interface_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"locale": "zh", "theme": "white"})
        );
        set_sort_preference(&connection, "headword-asc").unwrap();
        let choices: InterfacePreferences =
            serde_json::from_value(serde_json::json!({"locale": "fr", "theme": "dark"})).unwrap();
        set_interface_preferences(&connection, &choices).unwrap();
    }
    {
        let connection = Connection::open(&path).unwrap();
        assert_eq!(
            serde_json::to_value(interface_preferences(&connection).unwrap()).unwrap(),
            serde_json::json!({"locale": "fr", "theme": "dark"})
        );
        assert_eq!(sort_preference(&connection).unwrap(), "headword-asc");
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_saved_interface_values_and_unreadable_storage_never_return_defaults() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for value in [
        r#"{"locale":"de","theme":"white"}"#,
        r#"{"locale":"zh","theme":"blue"}"#,
        r#"{"locale":"zh"}"#,
        r#"{"theme":"white"}"#,
        r#"{"locale":"zh","theme":"white","path":"external"}"#,
        r#"{"locale":null,"theme":"white"}"#,
        r#"{"locale":"fr","locale":"zh","theme":"dark"}"#,
        "broken",
    ] {
        assert!(serde_json::from_str::<InterfacePreferences>(value).is_err());
        connection
            .execute(
                "INSERT INTO app_metadata VALUES ('interface-preferences', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [value],
            )
            .unwrap();
        assert!(
            interface_preferences(&connection)
                .unwrap_err()
                .to_string()
                .contains("Invalid saved interface preferences")
        );
    }
    connection.execute("DROP TABLE app_metadata", []).unwrap();
    assert!(
        interface_preferences(&connection)
            .unwrap_err()
            .to_string()
            .contains("SQLite operation failed")
    );
}

#[test]
fn all_interface_choices_save_and_a_failed_write_preserves_both_values_until_retry() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for locale in ["zh", "en", "fr"] {
        for theme in ["white", "sepia", "dark"] {
            let expected = serde_json::json!({"locale": locale, "theme": theme});
            let choices: InterfacePreferences = serde_json::from_value(expected.clone()).unwrap();
            set_interface_preferences(&connection, &choices).unwrap();
            assert_eq!(
                serde_json::to_value(interface_preferences(&connection).unwrap()).unwrap(),
                expected
            );
        }
    }
    let retry: InterfacePreferences =
        serde_json::from_value(serde_json::json!({"locale": "en", "theme": "sepia"})).unwrap();
    connection.execute_batch("PRAGMA query_only=ON").unwrap();
    assert!(set_interface_preferences(&connection, &retry).is_err());
    assert_eq!(
        serde_json::to_value(interface_preferences(&connection).unwrap()).unwrap(),
        serde_json::json!({"locale": "fr", "theme": "dark"})
    );
    connection.execute_batch("PRAGMA query_only=OFF").unwrap();
    set_interface_preferences(&connection, &retry).unwrap();
    assert_eq!(
        serde_json::to_value(interface_preferences(&connection).unwrap()).unwrap(),
        serde_json::json!({"locale": "en", "theme": "sepia"})
    );
}

#[test]
fn interface_writes_preserve_learning_content_ratings_and_selected_audio() {
    use lexique_storage as storage;
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let request: storage::SaveRequest = serde_json::from_value(serde_json::json!({
        "operation": "interface-vocabulary", "headword": "école", "notes": "keep these notes",
        "example_fr": "Voici mon école.", "example_zh": "这是我的学校。", "example_source": null,
        "example_sense": 0, "more_examples": [], "id": null, "revision": null
    }))
    .unwrap();
    let snapshot = serde_json::from_value(serde_json::json!({
        "reference": {"dictionary": "synthetic", "lookup_key": "école", "version": "1",
            "attribution": "fixture", "license": "MIT", "source_url": "https://example.org",
            "license_url": "https://example.org", "sha256": "a".repeat(64)},
        "digest": [{"pronunciations": [], "grammar": [], "translation": "学校"}]
    }))
    .unwrap();
    let record = storage::save(&mut connection, &request, Some(snapshot))
        .unwrap()
        .record;
    let cards = storage::add_review(&mut connection, record.id).unwrap();
    storage::rate_card(
        &mut connection,
        &storage::RatingRequest {
            operation: "interface-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: lexique_core::scheduling::Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let progress =
        serde_json::to_value(storage::review_progress(&connection, cards[0].id).unwrap()).unwrap();
    let version = storage::prepare_audio_version(
        &connection,
        record.id,
        None,
        storage::AudioPublicationContent {
            hash: &"b".repeat(64),
            bytes: 12,
            generated_at: None,
        },
        0,
        1_700_000_000,
    )
    .unwrap();
    assert!(storage::complete_audio_version(&connection, &version.locator, 1_700_000_000).unwrap());
    set_interface_preferences(
        &connection,
        &serde_json::from_value(serde_json::json!({"locale": "fr", "theme": "sepia"})).unwrap(),
    )
    .unwrap();
    let preserved = storage::get(&connection, record.id).unwrap();
    assert_eq!(preserved.notes, "keep these notes");
    assert_eq!(preserved.example_fr, "Voici mon école.");
    assert_eq!(preserved.revision, 1);
    assert_eq!(
        storage::review_cards(&connection, record.id).unwrap(),
        cards
    );
    assert_eq!(
        serde_json::to_value(storage::review_progress(&connection, cards[0].id).unwrap()).unwrap(),
        progress
    );
    assert_eq!(
        storage::undo_target(&connection).unwrap().as_deref(),
        Some("interface-rating")
    );
    assert_eq!(
        storage::selected_audio(&connection, record.id)
            .unwrap()
            .unwrap()
            .locator,
        version.locator
    );
}
