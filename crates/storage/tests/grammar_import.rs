use lexique_storage::*;
use rusqlite::Connection;

fn incoming() -> GrammarImportContent {
    GrammarImportContent {
        exchange_id: Some("external-rule-214".into()),
        title: "  Passé composé  ".into(),
        body: Some("  été, \"中文\"\r\n  ".into()),
        question: Some("  Quand ?\n".into()),
        answer: Some("  Hier.\n".into()),
    }
}

#[test]
fn imported_identity_and_paused_card_survive_restart_and_explicit_start() {
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ticket-214/storage");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("restart-{}.sqlite3", std::process::id()));
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let result =
        save_grammar_import(&mut connection, "import-paired", &incoming(), None, 1000).unwrap();
    let record = &result.records[0];
    assert_eq!(record.title, "Passé composé");
    assert_eq!(record.body, "  été, \"中文\"\r\n  ");
    assert!(record.category_ids.is_empty());
    assert_eq!(record.card.as_ref().unwrap().question, "  Quand ?\n");
    assert_eq!(record.card.as_ref().unwrap().answer, "  Hier.\n");
    assert!(review_paused(&connection, record.id).unwrap());
    let id = record.id;
    drop(connection);
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    assert_eq!(get_grammar(&connection, id).unwrap(), *record);
    let exported =
        capture_grammar_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    assert_eq!(exported.records[0].exchange_id, "external-rule-214");
    assert!(review_paused(&connection, id).unwrap());
    assert_eq!(
        save_grammar_import(&mut connection, "import-paired", &incoming(), None, 1001)
            .unwrap()
            .records[0],
        *record
    );
    assert!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    set_review_paused(&mut connection, id, false).unwrap();
    assert!(!review_paused(&connection, id).unwrap());
    let queue = review_queue(&mut connection, 1000, "UTC").unwrap();
    assert_eq!(queue.cards[0].id, record.card.as_ref().unwrap().id);
    assert_eq!(queue.cards[0].kind, "grammar");
}

#[test]
fn retry_replays_but_separate_identifier_free_imports_remain_independent() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let mut content = incoming();
    content.exchange_id = None;
    let first = save_grammar_import(&mut connection, "first", &content, None, 1000).unwrap();
    let retry = save_grammar_import(&mut connection, "first", &content, None, 1001).unwrap();
    assert_eq!(first.records, retry.records);
    content.title = "Changed request".into();
    assert!(save_grammar_import(&mut connection, "first", &content, None, 1001).is_err());
    content = incoming();
    content.exchange_id = Some(String::new());
    let second = save_grammar_import(&mut connection, "second", &content, None, 1002).unwrap();
    assert_ne!(first.records[0].id, second.records[0].id);
    let exported = capture_grammar_export(
        &mut connection,
        &NoteExportScope::All {
            sort: "created-asc".into(),
        },
    )
    .unwrap();
    assert_eq!(exported.records.len(), 2);
    assert_ne!(
        exported.records[0].exchange_id,
        exported.records[1].exchange_id
    );
}

#[test]
fn pause_write_failure_rolls_back_note_identity_card_and_receipt_then_retries() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_pause BEFORE INSERT ON review_paused BEGIN SELECT RAISE(FAIL, 'pause write failed'); END;").unwrap();
    assert!(
        save_grammar_import(&mut connection, "atomic", &incoming(), None, 1000)
            .unwrap_err()
            .to_string()
            .contains("pause write failed")
    );
    assert!(
        list_grammar(&connection, "", "created-asc")
            .unwrap()
            .is_empty()
    );
    assert!(
        find_grammar_exchange(&connection, "external-rule-214")
            .unwrap()
            .is_none()
    );
    assert!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    connection
        .execute_batch("DROP TRIGGER reject_pause")
        .unwrap();
    let result = save_grammar_import(&mut connection, "atomic", &incoming(), None, 1000).unwrap();
    assert!(review_paused(&connection, result.records[0].id).unwrap());
    connection.execute_batch("CREATE TRIGGER reject_start BEFORE DELETE ON review_paused BEGIN SELECT RAISE(FAIL, 'start write failed'); END;").unwrap();
    assert!(set_review_paused(&mut connection, result.records[0].id, false).is_err());
    assert_eq!(
        get_grammar(&connection, result.records[0].id).unwrap(),
        result.records[0]
    );
    assert!(review_paused(&connection, result.records[0].id).unwrap());
    assert!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
}

#[test]
fn known_identity_never_overwrites_or_accepts_a_stale_preview() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = save_grammar_import(&mut connection, "new", &incoming(), None, 1000)
        .unwrap()
        .records
        .remove(0);
    assert!(
        save_grammar_import(&mut connection, "occupied", &incoming(), None, 1001)
            .unwrap_err()
            .to_string()
            .contains("changed after preview")
    );
    let mut content = incoming();
    content.body = None;
    content.question = Some(" ".into());
    content.answer = None;
    let skipped =
        save_grammar_import(&mut connection, "same", &content, Some(&first), 1001).unwrap();
    assert_eq!(skipped.skipped, 1);
    assert!(skipped.records.is_empty());
    content.title = "New title".into();
    assert!(save_grammar_import(&mut connection, "changed", &content, Some(&first), 1001).is_err());
    assert_eq!(get_grammar(&connection, first.id).unwrap(), first);
    assert!(review_paused(&connection, first.id).unwrap());
}

#[test]
fn validation_uses_grammar_limits_and_never_truncates_or_saves_partial_pairs() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for content in [
        GrammarImportContent {
            title: " ".into(),
            ..incoming()
        },
        GrammarImportContent {
            title: "é".repeat(201),
            ..incoming()
        },
        GrammarImportContent {
            body: Some("é".repeat(20001)),
            ..incoming()
        },
        GrammarImportContent {
            question: Some("é".repeat(20001)),
            ..incoming()
        },
        GrammarImportContent {
            answer: Some("é".repeat(20001)),
            ..incoming()
        },
        GrammarImportContent {
            question: None,
            ..incoming()
        },
        GrammarImportContent {
            answer: Some(" \n".into()),
            ..incoming()
        },
    ] {
        assert!(save_grammar_import(&mut connection, "invalid", &content, None, 1000).is_err());
    }
    assert!(
        list_grammar(&connection, "", "created-asc")
            .unwrap()
            .is_empty()
    );
    let content = GrammarImportContent {
        title: format!("  {}  ", "é".repeat(200)),
        body: Some("é".repeat(20000)),
        question: Some("é".repeat(20000)),
        answer: Some("中".repeat(20000)),
        ..incoming()
    };
    let saved = save_grammar_import(&mut connection, "limits", &content, None, 1000)
        .unwrap()
        .records
        .remove(0);
    assert_eq!(saved.title, "é".repeat(200));
    assert_eq!(saved.body, "é".repeat(20000));
    assert_eq!(saved.card.unwrap().answer, "中".repeat(20000));
}

#[test]
fn skipping_a_known_note_preserves_categories_active_card_progress_and_history() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let record = save_grammar_import(&mut connection, "new-learning", &incoming(), None, 1000)
        .unwrap()
        .records
        .remove(0);
    let category = create_category(&connection, "Temps").unwrap();
    let record = save_grammar(
        &connection,
        &GrammarSaveRequest {
            operation: "classify".into(),
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
    set_review_paused(&mut connection, record.id, false).unwrap();
    review_queue(&mut connection, 1000, "UTC").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "learn-imported".into(),
            card_id: record.card.as_ref().unwrap().id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let before = export_learning_snapshot(&mut connection, 2000).unwrap();
    let result = save_grammar_import(
        &mut connection,
        "known-learning",
        &incoming(),
        Some(&record),
        1002,
    )
    .unwrap();
    assert_eq!(result.skipped, 1);
    assert_eq!(
        serde_json::to_value(export_learning_snapshot(&mut connection, 2000).unwrap()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}

#[test]
fn invalid_exchange_identity_reports_exchange_identity_error() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let long = "a".repeat(129);
    for bad in ["bad id", "bad_id", long.as_str()] {
        let mut content = incoming();
        content.exchange_id = Some(bad.into());
        for error in [
            content.validate().unwrap_err().to_string(),
            save_grammar_import(&mut connection, "invalid-id", &content, None, 1000)
                .unwrap_err()
                .to_string(),
        ] {
            assert!(
                error.to_ascii_lowercase().contains("exchange identity"),
                "unexpected error for {bad:?}: {error}"
            );
            assert!(
                !error.to_ascii_lowercase().contains("save operation"),
                "misleading operation error for {bad:?}: {error}"
            );
        }
    }
}
