use lexique_storage::{export_learning_snapshot, initialize};
use rusqlite::Connection;

fn saved_word(connection: &mut Connection, headword: &str) -> i64 {
    use lexique_storage::*;
    save(
        connection,
        &SaveRequest {
            operation: format!("save-{}", headword.len()),
            headword: headword.into(),
            notes: "historical notes".into(),
            example_fr: "Une école.".into(),
            example_zh: String::new(),
            example_source: None,
            example_sense: Some(0),
            more_examples: vec![],
            id: None,
            revision: None,
        },
        Some(Snapshot {
            reference: Reference {
                dictionary_id: Some("mdx-99".into()),
                dictionary: "Removed synthetic source".into(),
                lookup_key: headword.into(),
                version: "1".into(),
                attribution: None,
                license: None,
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![DigestEntry {
                pronunciations: vec![],
                grammar: vec!["n".into()],
                translation: Some("学校".into()),
            }],
        }),
    )
    .unwrap()
    .record
    .id
}

#[test]
fn historical_batch_voice_survives_a_later_audio_replacement() {
    use lexique_storage::*;
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection, "école");
    add_review_batch(
        &mut connection,
        "first-batch",
        &[id, id, 999],
        1_800_000_001,
    )
    .unwrap();
    detach_word_audio(&mut connection, id, 1_800_000_002).unwrap();
    let (replacement, _) = request_selected_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Male),
        id,
        1_800_000_002,
        true,
        Some("new-voice"),
    )
    .unwrap();
    let snapshot = export_learning_snapshot(&mut connection, 1_800_000_003).unwrap();
    let encoded = serde_json::to_vec(&snapshot).unwrap();
    let decoded = read_learning_snapshot(encoded.as_slice(), 1_800_000_003).unwrap();
    assert_eq!(
        decoded.batches[0].requested_vocabulary_ids,
        vec![id, id, 999]
    );
    assert_eq!(decoded.batches[0].voice, SpeechVoice::Female);
    assert_eq!(decoded.batches[0].items[1].vocabulary_id, 999);
    assert!(decoded.batches[0].items[1].error.is_some());
    let original_key = decoded.batches[0].items[0].semantic_key.as_deref().unwrap();
    assert_ne!(original_key, replacement);
    assert!(
        decoded
            .speech_sources
            .iter()
            .any(|source| source.semantic_key == original_key && source.spec.voice == "female")
    );
    assert!(
        decoded
            .speech_owners
            .iter()
            .any(|owner| owner.vocabulary_id == id && owner.semantic_key == replacement)
    );
    assert!(
        decoded
            .speech_owners
            .iter()
            .all(|owner| owner.semantic_key != original_key)
    );
}

#[test]
fn empty_snapshot_contains_the_complete_portable_contract() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO app_metadata(key,value) VALUES('private-service','secret-device-path')",
            [],
        )
        .unwrap();
    let snapshot = export_learning_snapshot(&mut connection, 1_800_000_000).unwrap();
    let value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(value["snapshot_version"], 1);
    for collection in [
        "records",
        "save_operations",
        "cards",
        "progress",
        "ratings",
        "paused_vocabulary_ids",
        "batches",
        "speech_sources",
        "speech_owners",
        "speech_tasks",
        "audio_versions",
        "audio_selections",
        "audio_imports",
        "legacy_audio",
        "audio_operations",
        "audio_publications",
        "audio_repairs",
    ] {
        assert_eq!(value[collection], serde_json::json!([]), "{collection}");
    }
    assert_eq!(
        value["queue"],
        serde_json::json!({"context": null, "entries": []})
    );
    assert_eq!(
        value["preferences"]["interface"],
        serde_json::json!({"locale": "zh", "theme": "white"})
    );
    assert_eq!(
        value["preferences"]["speech"],
        serde_json::json!({"voice": "female", "wpm": 180})
    );
    assert_eq!(value["preferences"]["auto_enroll"], false);
    assert_eq!(value["dictionary_catalog"]["next_id"], 1);
    assert_eq!(
        value["dictionary_catalog"]["entries"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(value.as_object().unwrap().len(), 21);
    assert!(!value.to_string().contains("secret-device-path"));
}

#[test]
fn old_save_requests_and_unknown_example_sources_keep_their_original_meaning() {
    use lexique_storage::{
        DigestEntry, Example, Reference, SaveRequest, Snapshot, add_review, read_learning_snapshot,
        save,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let saved = save(
        &mut connection,
        &SaveRequest {
            operation: "legacy-save".into(),
            headword: "école".into(),
            notes: "Mon\n筆記".into(),
            example_fr: String::new(),
            example_zh: String::new(),
            example_source: None,
            example_sense: Some(0),
            more_examples: vec![Example {
                fr: "Une école.".into(),
                zh: String::new(),
                source: None,
            }],
            id: None,
            revision: None,
        },
        Some(Snapshot {
            reference: Reference {
                dictionary_id: None,
                dictionary: "synthetic".into(),
                lookup_key: "école".into(),
                version: "1".into(),
                attribution: None,
                license: Some("MIT".into()),
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![DigestEntry {
                pronunciations: vec!["e.kɔl".into()],
                grammar: vec!["n".into()],
                translation: Some("学校".into()),
            }],
        }),
    )
    .unwrap();
    add_review(&mut connection, saved.record.id).unwrap();
    let snapshot = export_learning_snapshot(&mut connection, 1_800_000_000).unwrap();
    let mut value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(value["records"][0]["headword"], "école");
    assert_eq!(value["records"][0]["notes"], "Mon\n筆記");
    assert_eq!(
        value["records"][0]["example_source"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["records"][0]["reference"]["dictionary_id"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["records"][0]["more_examples"][0]["source"],
        serde_json::Value::Null
    );
    assert!(
        value["records"][0]
            .as_object()
            .unwrap()
            .contains_key("example_source")
    );
    assert!(
        value["records"][0]["more_examples"][0]
            .as_object()
            .unwrap()
            .contains_key("source")
    );
    assert_eq!(value["cards"].as_array().unwrap().len(), 2);
    assert_eq!(value["progress"], serde_json::json!([]));
    let request = value["save_operations"][0]["request"]
        .as_object_mut()
        .unwrap();
    for field in [
        "example_fr",
        "example_zh",
        "example_sense",
        "more_examples",
        "example_source",
    ] {
        request.remove(field);
    }
    let encoded = serde_json::to_vec(&value).unwrap();
    let decoded = read_learning_snapshot(encoded.as_slice(), 1_800_000_000).unwrap();
    assert_eq!(decoded.save_operations[0].request.example_fr, "");
    assert_eq!(decoded.save_operations[0].request.example_sense, None);
    assert_eq!(decoded.save_operations[0].request.more_examples.len(), 0);
    assert_eq!(decoded.records[0].more_examples[0].source, None);
}
