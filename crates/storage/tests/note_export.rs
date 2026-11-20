use lexique_storage::{NoteExportScope, capture_note_export, initialize, save};
use rusqlite::Connection;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);

fn saved(connection: &mut Connection, word: &str, notes: &str) -> i64 {
    let request = serde_json::from_value(serde_json::json!({
        "operation": format!("fixture-{}", NEXT_OPERATION.fetch_add(1, Ordering::Relaxed)), "headword": word, "notes": notes,
        "example_fr": "", "example_zh": "", "example_sense": null,
        "more_examples": [], "id": null, "revision": null
    }))
    .unwrap();
    let snapshot = serde_json::from_value(serde_json::json!({
        "reference": {"dictionary": "Fixture", "lookup_key": word, "version": "1",
            "attribution": null, "license": null, "source_url": null, "license_url": null,
            "sha256": "a".repeat(64)},
        "digest": [{"pronunciations": [], "grammar": [], "translation": "学校"}]
    }))
    .unwrap();
    save(connection, &request, Some(snapshot))
        .unwrap()
        .record
        .id
}

#[test]
fn export_reads_saved_text_and_all_matching_records_in_notebook_order() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved(&mut connection, "école", "  中文, \"notes\"\n  ");
    saved(&mut connection, "bonjour", "other");
    let snapshot = capture_note_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    assert_eq!(snapshot.records.len(), 1);
    assert_eq!(snapshot.records[0].headword, "école");
    assert_eq!(snapshot.records[0].notes, "  中文, \"notes\"\n  ");
    assert!(snapshot.records[0].examples.is_empty());
    assert_eq!(snapshot.row_count(), 1);
}

#[test]
fn export_scopes_include_all_query_matches_and_follow_simple_and_review_due_sorts() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let school = saved(&mut connection, "école", "topic");
    let hello = saved(&mut connection, "bonjour", "topic");
    lexique_storage::add_review(&mut connection, school).unwrap();
    let grammar = serde_json::from_value(serde_json::json!({
        "operation":"grammar", "id":null, "revision":null, "title":"école", "body":"topic", "category_ids":[]
    })).unwrap();
    let grammar = lexique_storage::save_grammar(&mut connection, &grammar, 1000)
        .unwrap()
        .record;
    for sort in [
        "headword-asc",
        "headword-desc",
        "created-asc",
        "created-desc",
        "updated-asc",
        "updated-desc",
        "review-due-asc",
    ] {
        let snapshot =
            capture_note_export(&mut connection, &NoteExportScope::All { sort: sort.into() })
                .unwrap();
        let notebook = lexique_storage::list(&connection, "", sort).unwrap();
        assert_eq!(
            snapshot
                .records
                .iter()
                .map(|record| record.id)
                .collect::<Vec<_>>(),
            notebook.iter().map(|record| record.id).collect::<Vec<_>>()
        );
    }
    let snapshot = capture_note_export(
        &mut connection,
        &NoteExportScope::All {
            sort: "review-due-asc".into(),
        },
    )
    .unwrap();
    assert_eq!(
        snapshot
            .records
            .iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        [school, hello]
    );
    assert!(
        capture_note_export(&mut connection, &NoteExportScope::Single { id: grammar.id }).is_err()
    );
    for index in 0..25 {
        saved(&mut connection, &format!("word-{index:02}"), "topic");
    }
    let scope = serde_json::from_value(serde_json::json!({"kind":"search", "query":"TOPIC", "sort":"headword-asc", "category":"all"})).unwrap();
    let snapshot = capture_note_export(&mut connection, &scope).unwrap();
    assert_eq!(snapshot.records.len(), 27);
    assert_eq!(snapshot.row_count(), 27);
    assert_eq!(snapshot.records[0].headword, "bonjour");
    assert_eq!(snapshot.records[1].headword, "école");
    let scope = serde_json::from_value(serde_json::json!({"kind":"search", "query":"not present", "sort":"created-desc", "category":"all"})).unwrap();
    assert!(
        capture_note_export(&mut connection, &scope)
            .unwrap()
            .records
            .is_empty()
    );
    assert!(capture_note_export(&mut connection, &NoteExportScope::Single { id: 0 }).is_err());
    assert!(
        capture_note_export(
            &mut connection,
            &NoteExportScope::All { sort: "bad".into() }
        )
        .is_err()
    );
}

#[test]
fn saving_during_export_cannot_mix_text_or_scope_from_two_committed_versions() {
    use std::sync::mpsc;
    use std::time::Duration;
    let path = std::env::temp_dir().join(format!(
        "lexique-export-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .unwrap();
    let id = saved(&mut connection, "école", "before");
    saved(&mut connection, "bonjour", "before");
    let (ready_tx, ready_rx) = mpsc::channel();
    let (write_tx, write_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let writer_path = path.clone();
    let writer = std::thread::spawn(move || {
        let mut writer = Connection::open(writer_path).unwrap();
        initialize(&writer).unwrap();
        ready_tx.send(()).unwrap();
        write_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let request = serde_json::from_value(serde_json::json!({
            "operation":"concurrent", "headword":"école", "notes":"after", "example_fr":"", "example_zh":"",
            "example_sense":null, "more_examples":[], "id":id, "revision":1
        })).unwrap();
        save(&mut writer, &request, None).unwrap();
        lexique_storage::add_review(&mut writer, id).unwrap();
        saved(&mut writer, "new record", "before");
        done_tx.send(()).unwrap();
    });
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut steps = 0;
    connection
        .progress_handler(
            1,
            Some(move || {
                steps += 1;
                if steps == 20 {
                    write_tx.send(()).unwrap();
                    done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                false
            }),
        )
        .unwrap();
    let scope = serde_json::from_value(serde_json::json!({"kind":"search", "query":"before", "sort":"review-due-asc", "category":"all"})).unwrap();
    let snapshot = capture_note_export(&mut connection, &scope).unwrap();
    connection
        .progress_handler(0, None::<fn() -> bool>)
        .unwrap();
    writer.join().unwrap();
    assert_eq!(snapshot.records.len(), 2);
    assert_eq!(
        snapshot
            .records
            .iter()
            .map(|record| record.headword.as_str())
            .collect::<Vec<_>>(),
        ["bonjour", "école"]
    );
    assert_eq!(
        snapshot
            .records
            .iter()
            .map(|record| record.notes.as_str())
            .collect::<Vec<_>>(),
        ["before", "before"]
    );
    assert_eq!(snapshot.row_count(), 2);
    let after = capture_note_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    assert_eq!(after.records[0].notes, "after");
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn search_export_uses_the_current_category_filter_without_exporting_categories() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let school = saved(&mut connection, "école", "topic");
    saved(&mut connection, "bonjour", "topic");
    let category = lexique_storage::create_category(&connection, "Study").unwrap();
    let request = lexique_storage::CategoryBulkRequest {
        record_kind: "vocabulary".into(),
        ids: vec![school],
        category_id: category.id,
        action: "add".into(),
    };
    assert_eq!(
        lexique_storage::bulk_classify(&connection, &request).unwrap()[0].status,
        "saved"
    );
    let scope = serde_json::from_value(serde_json::json!({"kind":"search", "query":"topic", "sort":"headword-asc", "category":category.id})).unwrap();
    let snapshot = capture_note_export(&mut connection, &scope).unwrap();
    assert_eq!(snapshot.records.len(), 1);
    assert_eq!(snapshot.records[0].headword, "école");
    let scope = serde_json::from_value(serde_json::json!({"kind":"search", "query":"topic", "sort":"headword-asc", "category":"uncategorized"})).unwrap();
    let snapshot = capture_note_export(&mut connection, &scope).unwrap();
    assert_eq!(snapshot.records.len(), 1);
    assert_eq!(snapshot.records[0].headword, "bonjour");
    let scope = serde_json::from_value(
        serde_json::json!({"kind":"search", "query":"", "sort":"headword-asc", "category":0}),
    )
    .unwrap();
    assert!(capture_note_export(&mut connection, &scope).is_err());
}
