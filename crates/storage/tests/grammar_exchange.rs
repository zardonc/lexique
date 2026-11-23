use lexique_storage::*;
use rusqlite::Connection;

fn note(
    connection: &Connection,
    operation: &str,
    id: Option<i64>,
    revision: Option<i64>,
    title: &str,
) -> GrammarRecord {
    save_grammar(
        connection,
        &GrammarSaveRequest {
            operation: operation.into(),
            id,
            revision,
            title: title.into(),
            body: "  été, \"中文\"\r\n  ".into(),
            category_ids: vec![],
        },
        1000,
    )
    .unwrap()
    .record
}

#[test]
fn grammar_exchange_identity_survives_rename_reopen_and_snapshot_restore() {
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    let first = note(&source, "first", None, None, "Une règle");
    let second = note(&source, "second", None, None, "Une règle");
    let scope = NoteExportScope::All {
        sort: "created-asc".into(),
    };
    let before = capture_grammar_export(&mut source, &scope).unwrap();
    assert_ne!(before.records[0].exchange_id, before.records[1].exchange_id);
    note(
        &source,
        "rename",
        Some(first.id),
        Some(first.revision),
        "Autre règle",
    );
    initialize(&source).unwrap();
    let after = capture_grammar_export(&mut source, &scope).unwrap();
    assert_eq!(before.records[0].exchange_id, after.records[0].exchange_id);
    assert_eq!(after.records[1].id, second.id);
    let snapshot = export_learning_snapshot(&mut source, 2000).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &snapshot, 2000).unwrap();
    assert_eq!(
        after,
        capture_grammar_export(&mut restored, &scope).unwrap()
    );
}

#[test]
fn schema_20_upgrade_and_legacy_snapshot_restore_preserve_all_learning_state() {
    let now = 1_700_000_000;
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ticket-213/storage");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("migration-{}.sqlite3", std::process::id()));
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let record = note(&connection, "upgrade-note", None, None, "Une règle");
    let category = create_category(&connection, "Temps").unwrap();
    let saved = save_grammar(
        &connection,
        &GrammarSaveRequest {
            operation: "classified".into(),
            id: Some(record.id),
            revision: Some(record.revision),
            title: record.title,
            body: record.body,
            category_ids: vec![category.id],
        },
        1001,
    )
    .unwrap()
    .record;
    let saved = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "learned-card".into(),
            id: saved.id,
            revision: saved.revision,
            question: "  Pourquoi ?\n  ".into(),
            answer: "中文, \"réponse\"".into(),
        },
        1002,
    )
    .unwrap()
    .record;
    review_queue(&mut connection, now, "UTC").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "rating".into(),
            card_id: saved.card.as_ref().unwrap().id,
            expected_revision: 0,
            rating: Rating::Easy,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    set_review_paused(&mut connection, saved.id, true).unwrap();
    let mut legacy =
        serde_json::to_value(export_learning_snapshot(&mut connection, now).unwrap()).unwrap();
    legacy["snapshot_version"] = 4.into();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("grammar_exchange_identities");
    connection.execute_batch("DROP TRIGGER grammar_exchange_identity_insert; DROP TABLE grammar_exchange_identities; PRAGMA user_version=20;").unwrap();
    drop(connection);
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let scope = NoteExportScope::All {
        sort: "created-asc".into(),
    };
    let migrated = capture_grammar_export(&mut connection, &scope).unwrap();
    assert_eq!(migrated.records.len(), 1);
    assert!(!migrated.records[0].exchange_id.is_empty());
    let mut upgraded =
        serde_json::to_value(export_learning_snapshot(&mut connection, now).unwrap()).unwrap();
    upgraded["snapshot_version"] = 4.into();
    upgraded
        .as_object_mut()
        .unwrap()
        .remove("grammar_exchange_identities");
    assert_eq!(upgraded, legacy);
    drop(connection);
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    assert_eq!(
        migrated,
        capture_grammar_export(&mut connection, &scope).unwrap()
    );
    for version in [3, 4] {
        let mut value = legacy.clone();
        value["snapshot_version"] = version.into();
        let snapshot =
            read_learning_snapshot(serde_json::to_vec(&value).unwrap().as_slice(), now).unwrap();
        let mut restored = Connection::open_in_memory().unwrap();
        initialize(&restored).unwrap();
        import_learning_snapshot(&mut restored, &snapshot, now).unwrap();
        let identities = capture_grammar_export(&mut restored, &scope).unwrap();
        assert!(!identities.records[0].exchange_id.is_empty());
        let snapshot = export_learning_snapshot(&mut restored, now).unwrap();
        assert_eq!(snapshot.grammar_records[0], saved);
        assert_eq!(snapshot.paused_vocabulary_ids, vec![saved.id]);
        assert_eq!(snapshot.ratings.len(), 1);
        assert_eq!(snapshot.progress.len(), 1);
        initialize(&restored).unwrap();
        assert_eq!(
            identities,
            capture_grammar_export(&mut restored, &scope).unwrap()
        );
    }
}

#[test]
fn grammar_export_uses_saved_scope_order_and_retained_text() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = note(&connection, "scope-first", None, None, "Zulu");
    let second = note(&connection, "scope-second", None, None, "Alpha");
    let all = capture_grammar_export(
        &mut connection,
        &NoteExportScope::All {
            sort: "headword-asc".into(),
        },
    )
    .unwrap();
    assert_eq!(
        all.records
            .iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        [second.id, first.id]
    );
    assert_eq!(
        (&all.records[0].question, &all.records[0].answer),
        (&String::new(), &String::new())
    );
    let scope = serde_json::from_value(serde_json::json!({ "kind":"search", "query":"zulu", "sort":"created-desc", "category":"uncategorized" })).unwrap();
    let retained = capture_grammar_export(&mut connection, &scope).unwrap();
    note(
        &connection,
        "scope-edit",
        Some(first.id),
        Some(first.revision),
        "Replaced",
    );
    assert_eq!(retained.records[0].title, "Zulu");
    assert!(
        capture_grammar_export(&mut connection, &scope)
            .unwrap()
            .records
            .is_empty()
    );
    assert_eq!(
        capture_grammar_export(&mut connection, &NoteExportScope::Single { id: first.id })
            .unwrap()
            .records[0]
            .title,
        "Replaced"
    );
}

#[test]
fn snapshot_rejects_missing_duplicate_and_unknown_exchange_identities_before_restore() {
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    note(&source, "validation-first", None, None, "Règle");
    note(&source, "validation-second", None, None, "Règle");
    let baseline =
        serde_json::to_value(export_learning_snapshot(&mut source, 2000).unwrap()).unwrap();
    for mutation in 0..5 {
        let mut value = baseline.clone();
        match mutation {
            0 => {
                value
                    .as_object_mut()
                    .unwrap()
                    .remove("grammar_exchange_identities");
            }
            1 => {
                value["grammar_exchange_identities"][1]["exchange_id"] =
                    value["grammar_exchange_identities"][0]["exchange_id"].clone();
            }
            2 => {
                value["grammar_exchange_identities"][1]["record_id"] = 999.into();
            }
            3 => {
                value["grammar_exchange_identities"][1]["exchange_id"] = "".into();
            }
            _ => {
                value["snapshot_version"] = 6.into();
            }
        }
        assert!(
            read_learning_snapshot(serde_json::to_vec(&value).unwrap().as_slice(), 2000).is_err(),
            "mutation {mutation}"
        );
    }
}
