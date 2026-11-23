use lexique_storage::*;
use rusqlite::Connection;

fn content() -> NoteImportContent {
    NoteImportContent {
        headword: "école externe".into(),
        notes: Some("学校\n独立笔记".into()),
        examples: Some(vec![
            NoteExample {
                fr: "Une école indépendante.".into(),
                zh: "一所独立学校。".into(),
            },
            NoteExample {
                fr: "".into(),
                zh: "尚待法语原句。".into(),
            },
        ]),
    }
}

#[test]
fn imported_notes_bypass_auto_enrollment_and_survive_portable_snapshot_restore() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    set_auto_enroll(&connection, true).unwrap();
    let request = content().create_request("import-one").unwrap();
    let source = ImportSource {
        format: NoteFormat::Csv,
        filename: "école.csv".into(),
    };
    let saved = save_imported_note(&mut connection, &request, None, source.clone())
        .unwrap()
        .record;
    assert!(review_cards(&connection, saved.id).unwrap().is_empty());
    let snapshot = export_learning_snapshot(&mut connection, 1_800_000_000).unwrap();
    assert!(snapshot.speech_tasks.is_empty());
    assert!(snapshot.speech_owners.is_empty());
    assert_eq!(snapshot.snapshot_version, 5);
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let portable = read_learning_snapshot(bytes.as_slice(), 1_800_000_000).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &portable, 1_800_000_000).unwrap();
    let reopened = find(&restored, "e\u{301}cole externe").unwrap().unwrap();
    assert_eq!(reopened.import_source, Some(source));
    assert!(reopened.reference.is_none());
    assert!(content().matches(&reopened));
    assert_eq!(
        replay(&restored, &request).unwrap().unwrap().record.id,
        saved.id
    );
}

#[test]
fn portable_external_records_reject_missing_source_invalid_source_and_dictionary_content() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let request = content().create_request("external-admission").unwrap();
    let saved = save_imported_note(
        &mut connection,
        &request,
        None,
        ImportSource {
            format: NoteFormat::Csv,
            filename: "école.csv".into(),
        },
    )
    .unwrap()
    .record;
    let original =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_000).unwrap())
            .unwrap();
    let mut missing_source = original.clone();
    missing_source["records"][0]
        .as_object_mut()
        .unwrap()
        .remove("import_source");
    let mut invalid_source = original.clone();
    invalid_source["records"][0]["import_source"]["filename"] = "../école.csv".into();
    let mut dictionary_content = original.clone();
    dictionary_content["records"][0]["digest"] = serde_json::json!([{
        "pronunciations": [], "grammar": [], "translation": "fabricated"
    }]);
    let mut old_version = original;
    old_version["snapshot_version"] = 3.into();
    for invalid in [
        missing_source,
        invalid_source,
        dictionary_content,
        old_version,
    ] {
        let bytes = serde_json::to_vec(&invalid).unwrap();
        assert!(read_learning_snapshot(bytes.as_slice(), 1_800_000_000).is_err());
        assert_eq!(
            serde_json::to_value(get(&connection, saved.id).unwrap()).unwrap(),
            serde_json::to_value(&saved).unwrap()
        );
    }
}
