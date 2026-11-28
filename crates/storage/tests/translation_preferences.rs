use lexique_storage::{
    TranslationPreferences, export_learning_snapshot, initialize, preserve_translation_preferences,
    set_translation_preferences, translation_preferences,
};
use rusqlite::Connection;

#[test]
fn fresh_preferences_follow_interface_locale_and_invalid_json_is_reported() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let defaults =
        serde_json::to_value(translation_preferences(&connection, "zh").unwrap()).unwrap();
    assert_eq!(defaults["active_provider"], "disabled");
    assert_eq!(defaults["source_language"], "auto");
    assert_eq!(defaults["target_language"], "zh-CN");
    assert_eq!(
        serde_json::to_value(translation_preferences(&connection, "fr").unwrap()).unwrap()["target_language"],
        "en"
    );
    connection
        .execute(
            "INSERT INTO app_metadata VALUES ('translation-preferences', 'broken')",
            [],
        )
        .unwrap();
    assert!(translation_preferences(&connection, "zh").is_err());
}

#[test]
fn settings_are_private_to_the_device_across_snapshot_and_restore() {
    let mut source = Connection::open_in_memory().unwrap();
    let destination = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    initialize(&destination).unwrap();
    let json = serde_json::json!({
        "active_provider": "openai_compatible", "source_language": "fr", "target_language": "en",
        "deepl": {"tier": "pro"}, "openai_compatible": {"base_url": "https://private.example/v1", "model": "private-model"}
    });
    let settings: TranslationPreferences = serde_json::from_value(json.clone()).unwrap();
    set_translation_preferences(&source, &settings).unwrap();
    let snapshot =
        serde_json::to_string(&export_learning_snapshot(&mut source, 1).unwrap()).unwrap();
    assert!(!snapshot.contains("private.example"));
    assert!(!snapshot.contains("private-model"));
    preserve_translation_preferences(&source, &destination).unwrap();
    assert_eq!(
        serde_json::to_value(translation_preferences(&destination, "zh").unwrap()).unwrap(),
        json
    );
    source
        .execute(
            "UPDATE app_metadata SET value='broken' WHERE key='translation-preferences'",
            [],
        )
        .unwrap();
    preserve_translation_preferences(&source, &destination).unwrap();
    assert!(translation_preferences(&destination, "zh").is_err());
}
