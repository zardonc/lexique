use lexique_storage::{DigestEntry, Reference, SaveRequest, Snapshot, get, initialize, save};
use lexique_storage::{find, list, set_sort_preference, sort_preference};
use rusqlite::Connection;

fn snapshot() -> Snapshot {
    Snapshot {
        reference: Reference {
            dictionary: "FreeDict".into(),
            lookup_key: "école".into(),
            version: "2025.11.23".into(),
            attribution: "Wiktionary contributors".into(),
            license: "CC-BY-SA-3.0".into(),
            source_url: "https://download.freedict.org/".into(),
            license_url: "https://creativecommons.org/licenses/by-sa/3.0/".into(),
            sha256: "a".repeat(64),
        },
        digest: vec![DigestEntry {
            pronunciations: vec!["e.kɔl".into()],
            grammar: vec!["n".into()],
            translation: Some("学校".into()),
        }],
    }
}

fn create(operation: &str, headword: &str, notes: &str) -> SaveRequest {
    SaveRequest {
        operation: operation.into(),
        headword: headword.into(),
        notes: notes.into(),
        example_fr: String::new(),
        example_zh: String::new(),
        example_sense: None,
        more_examples: vec![],
        id: None,
        revision: None,
    }
}

#[test]
fn explicit_save_creates_one_record_and_duplicate_collection_preserves_it() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let saved = save(
        &mut connection,
        &create("create-1", "école", ""),
        Some(snapshot()),
    )
    .unwrap();
    assert_eq!(saved.outcome, "created");
    let duplicate = save(
        &mut connection,
        &create("create-2", "école", "other"),
        Some(snapshot()),
    )
    .unwrap();
    assert_eq!(duplicate.outcome, "existing");
    assert_eq!(duplicate.record.id, saved.record.id);
    assert_eq!(
        save(&mut connection, &create("create-2", "école", "other"), None)
            .unwrap()
            .outcome,
        "existing"
    );
    assert_eq!(get(&connection, saved.record.id).unwrap().notes, "");
    assert_eq!(
        duplicate.record.digest[0].translation.as_deref(),
        Some("学校")
    );
}

fn update(operation: &str, record: &lexique_storage::VocabularyRecord, notes: &str) -> SaveRequest {
    SaveRequest {
        operation: operation.into(),
        headword: record.headword.clone(),
        notes: notes.into(),
        example_fr: record.example_fr.clone(),
        example_zh: record.example_zh.clone(),
        example_sense: record.example_sense,
        more_examples: record.more_examples.clone(),
        id: Some(record.id),
        revision: Some(record.revision),
    }
}

#[test]
fn save_request_requires_all_example_fields() {
    let request = serde_json::to_value(create("required-fields", "école", "note")).unwrap();
    for field in ["example_fr", "example_zh", "example_sense", "more_examples"] {
        let mut missing = request.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<SaveRequest>(missing).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn unassigned_legacy_examples_are_preserved_but_new_edits_require_a_meaning() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let original = save(
        &mut connection,
        &create("legacy-word", "école", ""),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    let mut new_example = update("new-example", &original, "");
    new_example.example_fr = "Bonjour".into();
    assert!(save(&mut connection, &new_example, None).is_err());
    connection
        .execute(
            "UPDATE vocabulary SET example_fr = ?1 WHERE id = ?2",
            rusqlite::params!["Legacy", original.id],
        )
        .unwrap();
    let legacy = get(&connection, original.id).unwrap();
    let preserved = save(
        &mut connection,
        &update("notes-only", &legacy, "note"),
        None,
    )
    .unwrap()
    .record;
    assert_eq!(preserved.example_fr, "Legacy");
    let mut edit = update("edit-legacy", &preserved, "note");
    edit.example_fr = "Changed".into();
    assert!(save(&mut connection, &edit, None).is_err());
    edit.example_sense = Some(0);
    assert_eq!(
        save(&mut connection, &edit, None)
            .unwrap()
            .record
            .example_fr,
        "Changed"
    );
}

#[test]
fn example_meaning_survives_upgrade_and_rejects_out_of_range() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let original = save(
        &mut connection,
        &create("meaning-create", "école", "note"),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    connection
        .execute_batch("ALTER TABLE vocabulary DROP COLUMN example_sense; PRAGMA user_version = 9;")
        .unwrap();
    initialize(&connection).unwrap();
    let migrated = get(&connection, original.id).unwrap();
    assert_eq!(migrated.example_sense, None);
    let mut request = update("meaning-edit", &migrated, "note");
    request.example_sense = Some(1);
    assert!(save(&mut connection, &request, None).is_err());
    assert_eq!(get(&connection, original.id).unwrap().example_sense, None);
    request.example_sense = Some(0);
    let saved = save(&mut connection, &request, None).unwrap();
    assert_eq!(saved.record.example_sense, Some(0));
    assert_eq!(
        save(&mut connection, &request, None).unwrap().outcome,
        "replayed"
    );
}

#[test]
fn review_example_is_saved_with_revision_and_safe_retry() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let original = save(
        &mut connection,
        &create("example-create", "école", "note"),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    let mut value = serde_json::to_value(update("example-edit", &original, "note")).unwrap();
    value["example_fr"] = "Je vais à l'école.".into();
    value["example_zh"] = "".into();
    value["example_sense"] = 0.into();
    let request: SaveRequest = serde_json::from_value(value).unwrap();
    let saved = save(&mut connection, &request, None).unwrap();
    assert_eq!(
        serde_json::to_value(&saved.record).unwrap()["example_fr"],
        "Je vais à l'école."
    );
    assert_eq!(
        save(&mut connection, &request, None).unwrap().outcome,
        "replayed"
    );
    assert_eq!(
        save(
            &mut connection,
            &update("stale-example", &original, "stale"),
            None
        )
        .unwrap()
        .outcome,
        "conflict"
    );
    assert_eq!(get(&connection, original.id).unwrap().notes, "note");
}

#[test]
fn pre_example_save_operation_can_be_retried_after_upgrade() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let request = create("legacy-operation", "école", "note");
    let saved = save(&mut connection, &request, Some(snapshot())).unwrap();
    let mut legacy = serde_json::to_value(&request).unwrap();
    legacy.as_object_mut().unwrap().remove("example_fr");
    legacy.as_object_mut().unwrap().remove("example_zh");
    legacy.as_object_mut().unwrap().remove("example_sense");
    connection
        .execute(
            "UPDATE save_operations SET request = ?1 WHERE operation = ?2",
            rusqlite::params![legacy.to_string(), request.operation],
        )
        .unwrap();
    let replay = save(&mut connection, &request, None).unwrap();
    assert_eq!(replay.outcome, "replayed");
    assert_eq!(replay.record.id, saved.record.id);
    assert_eq!(replay.record.notes, "note");
}

#[test]
fn canonical_identity_is_separate_from_accent_insensitive_literal_search() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (index, word) in [
        "cote",
        "côte",
        "côté",
        "Côté",
        "aujourd'hui",
        "aujourd’hui",
        "école",
    ]
    .iter()
    .enumerate()
    {
        save(
            &mut connection,
            &create(&format!("word-{index}"), word, "Éléphant 100%_\\"),
            Some(snapshot()),
        )
        .unwrap();
    }
    let duplicate = save(
        &mut connection,
        &create("equivalent", "e\u{301}cole", "ignored"),
        Some(snapshot()),
    )
    .unwrap();
    assert_eq!(duplicate.outcome, "existing");
    assert_eq!(list(&connection, "", "updated-desc").unwrap().len(), 7);
    assert_eq!(list(&connection, "COTE", "headword-asc").unwrap().len(), 4);
    assert_eq!(
        list(&connection, "elephant", "headword-asc").unwrap().len(),
        7
    );
    assert_eq!(list(&connection, "%_\\", "headword-asc").unwrap().len(), 7);
    assert!(
        list(&connection, "ele phant", "headword-asc")
            .unwrap()
            .is_empty()
    );
    assert_eq!(list(&connection, "学校", "headword-asc").unwrap().len(), 7);
    assert_eq!(
        find(&connection, "e\u{301}cole").unwrap().unwrap().headword,
        "école"
    );
}

#[test]
fn retry_after_lost_response_never_reapplies_over_a_later_edit() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let request = create("first-create", "école", "");
    let original = save(&mut connection, &request, Some(snapshot()))
        .unwrap()
        .record;
    let request2 = update("first-update", &original, "line one\nÉcole 😀");
    let edited = save(&mut connection, &request2, None).unwrap().record;
    let newest = save(
        &mut connection,
        &update("newer-update", &edited, "newer"),
        None,
    )
    .unwrap()
    .record;
    for retry in [&request, &request2] {
        let replay = save(&mut connection, retry, None).unwrap();
        assert_eq!(replay.outcome, "replayed");
        assert_eq!(replay.record.notes, "newer");
        assert_eq!(replay.record.revision, newest.revision);
    }
    let stale = update("stale-update", &original, "must not overwrite");
    assert_eq!(
        save(&mut connection, &stale, None).unwrap().outcome,
        "conflict"
    );
    assert_eq!(
        save(&mut connection, &stale, None).unwrap().outcome,
        "conflict"
    );
    assert!(
        save(
            &mut connection,
            &create("first-create", "école", "different payload"),
            None
        )
        .is_err()
    );
    let cleared = save(&mut connection, &update("clear", &newest, ""), None)
        .unwrap()
        .record;
    assert_eq!(cleared.id, original.id);
    assert_eq!(cleared.notes, "");
    assert_eq!(cleared.reference, original.reference);
    assert_eq!(cleared.digest, original.digest);
    assert!(cleared.updated_at > original.updated_at);
}

#[test]
fn character_limits_count_unicode_scalars_and_reject_without_truncation() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let notes = format!("{}e\u{301}", "😀".repeat(19_998));
    let original = save(
        &mut connection,
        &create("boundary", &"é".repeat(200), &notes),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    assert!(
        save(
            &mut connection,
            &update("too-long", &original, &format!("{notes}a")),
            None
        )
        .is_err()
    );
    assert_eq!(get(&connection, original.id).unwrap().notes, notes);
    for (index, word) in [String::new(), "a\nb".into(), "é".repeat(201)]
        .iter()
        .enumerate()
    {
        assert!(
            save(
                &mut connection,
                &create(&format!("bad-{index}"), word, ""),
                Some(snapshot())
            )
            .is_err()
        );
    }
    let mut renamed = update("rename", &original, "");
    renamed.headword = "different".into();
    assert!(save(&mut connection, &renamed, None).is_err());
}

#[test]
fn failed_writes_roll_back_record_and_operation_so_retry_can_succeed() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let original = save(
        &mut connection,
        &create("initial", "école", "keep"),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    let request = update("retry-update", &original, "new");
    // Fault injection only; exercise the public save API against a read-only database.
    connection.execute_batch("PRAGMA query_only = ON").unwrap();
    let error = save(&mut connection, &request, None)
        .unwrap_err()
        .to_string();
    assert!(error.contains("SQLite"));
    assert_eq!(get(&connection, original.id).unwrap().notes, "keep");
    connection.execute_batch("PRAGMA query_only = OFF").unwrap();
    assert_eq!(
        save(&mut connection, &request, None).unwrap().outcome,
        "updated"
    );
}

#[test]
fn failed_commit_rolls_back_both_content_and_retry_identity() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let original = save(
        &mut connection,
        &create("commit-original", "école", "keep"),
        Some(snapshot()),
    )
    .unwrap()
    .record;
    let request = update("commit-retry", &original, "new");
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(save(&mut connection, &request, None).is_err());
    assert_eq!(get(&connection, original.id).unwrap().notes, "keep");
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    let saved = save(&mut connection, &request, None).unwrap();
    assert_eq!(saved.outcome, "updated");
    assert_eq!(saved.record.revision, original.revision + 1);
}

fn database_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "lexique-m1-{label}-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn file_database_reopens_with_record_identity_content_and_sort_preference() {
    let path = database_path("reopen");
    let original;
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(sort_preference(&connection).unwrap(), "updated-desc");
        let created = save(
            &mut connection,
            &create("persistent", "école", "l’école\n学校"),
            Some(snapshot()),
        )
        .unwrap()
        .record;
        let mut request = update("persistent-example", &created, &created.notes);
        request.example_fr = "Je vais à l'école.".into();
        request.example_zh = "我去学校。".into();
        request.example_sense = Some(0);
        request.more_examples.push(lexique_storage::Example {
            fr: "Une autre phrase.".into(),
            zh: "另一句话。".into(),
        });
        original = save(&mut connection, &request, None).unwrap().record;
        set_sort_preference(&connection, "headword-asc").unwrap();
    }
    {
        let connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let restored = get(&connection, original.id).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(original).unwrap()
        );
        assert_eq!(sort_preference(&connection).unwrap(), "headword-asc");
        assert!(set_sort_preference(&connection, "not-valid").is_err());
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn concurrent_creators_share_one_record_and_stale_writer_cannot_overwrite() {
    let path = database_path("race");
    let connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|index| {
            let path = path.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut connection = Connection::open(path).unwrap();
                connection
                    .busy_timeout(std::time::Duration::from_secs(3))
                    .unwrap();
                barrier.wait();
                save(
                    &mut connection,
                    &create(&format!("race-{index}"), "école", ""),
                    Some(snapshot()),
                )
                .unwrap()
                .record
                .id
            })
        })
        .collect();
    let ids: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(ids[0], ids[1]);
    assert_eq!(list(&connection, "", "created-asc").unwrap().len(), 1);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn sorting_is_stable_in_each_direction_and_search_keeps_the_order() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (index, word) in ["zèbre", "école", "avion"].iter().enumerate() {
        save(
            &mut connection,
            &create(&format!("sort-{index}"), word, "match"),
            Some(snapshot()),
        )
        .unwrap();
    }
    let words = list(&connection, "MATCH", "headword-asc").unwrap();
    assert_eq!(
        words
            .iter()
            .map(|r| r.headword.as_str())
            .collect::<Vec<_>>(),
        ["avion", "école", "zèbre"]
    );
    for field in ["headword", "created", "updated"] {
        let asc: Vec<_> = list(&connection, "", &format!("{field}-asc"))
            .unwrap()
            .iter()
            .map(|r| r.id)
            .collect();
        let mut desc: Vec<_> = list(&connection, "match", &format!("{field}-desc"))
            .unwrap()
            .iter()
            .map(|r| r.id)
            .collect();
        desc.reverse();
        assert_eq!(asc, desc);
    }
}

#[test]
fn review_due_sort_prioritizes_learned_then_enrolled_then_other_records() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let mut ids = Vec::new();
    for (index, word) in ["zèbre", "école", "avion", "bateau"].iter().enumerate() {
        ids.push(
            save(
                &mut connection,
                &create(&format!("due-{index}"), word, "match"),
                Some(snapshot()),
            )
            .unwrap()
            .record
            .id,
        );
    }
    let cards = lexique_storage::add_review(&mut connection, ids[0]).unwrap();
    lexique_storage::add_review(&mut connection, ids[1]).unwrap();
    lexique_storage::add_review(&mut connection, ids[2]).unwrap();
    let now = 1_700_000_000;
    lexique_storage::review_queue(&mut connection, now, "UTC").unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "due-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: lexique_core::scheduling::Rating::Good,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let words: Vec<_> = list(&connection, "match", "review-due-asc")
        .unwrap()
        .into_iter()
        .map(|record| record.headword)
        .collect();
    assert_eq!(words, ["zèbre", "avion", "école", "bateau"]);
    connection
        .execute(
            "UPDATE review_progress SET state = 'invalid' WHERE card_id = ?1",
            [cards[0].id],
        )
        .unwrap();
    assert!(list(&connection, "match", "review-due-asc").is_err());
}

#[test]
fn review_due_sort_uses_earliest_card_and_survives_reopen_and_pause() {
    let path = database_path("review-due");
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let mut ids = Vec::new();
        for (index, word) in ["zèbre", "avion", "bateau", "chat", "bonjour"]
            .iter()
            .enumerate()
        {
            ids.push(
                save(
                    &mut connection,
                    &create(
                        &format!("persist-due-{index}"),
                        word,
                        if index < 2 { "match filtered" } else { "match" },
                    ),
                    Some(snapshot()),
                )
                .unwrap()
                .record
                .id,
            );
        }
        let z = lexique_storage::add_review(&mut connection, ids[0]).unwrap();
        let a = lexique_storage::add_review(&mut connection, ids[1]).unwrap();
        let b = lexique_storage::add_review(&mut connection, ids[4]).unwrap();
        lexique_storage::add_review(&mut connection, ids[2]).unwrap();
        let now = 1_700_000_000;
        lexique_storage::review_queue(&mut connection, now, "UTC").unwrap();
        for (operation, card_id, time) in [("rate-z", z[0].id, now), ("rate-a", a[0].id, now)] {
            lexique_storage::rate_card(
                &mut connection,
                &lexique_storage::RatingRequest {
                    operation: operation.into(),
                    card_id,
                    expected_revision: 0,
                    rating: lexique_core::scheduling::Rating::Good,
                    now: time,
                    timezone: "UTC".into(),
                },
            )
            .unwrap();
        }
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: "rate-b".into(),
                card_id: b[0].id,
                expected_revision: 0,
                rating: lexique_core::scheduling::Rating::Good,
                now: now + 3_600,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        let later = now + 86_400;
        lexique_storage::review_queue(&mut connection, later, "UTC").unwrap();
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: "rate-z-listening".into(),
                card_id: z[1].id,
                expected_revision: 0,
                rating: lexique_core::scheduling::Rating::Good,
                now: later,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        lexique_storage::set_review_paused(&mut connection, ids[0], true).unwrap();
        let mut settings = lexique_storage::review_settings(&connection).unwrap();
        settings.new_limit = 0;
        settings.review_limit = 0;
        lexique_storage::set_review_settings(&connection, &settings).unwrap();
        set_sort_preference(&connection, "review-due-asc").unwrap();
    }
    {
        let connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(sort_preference(&connection).unwrap(), "review-due-asc");
        let words: Vec<_> = list(&connection, "match", &sort_preference(&connection).unwrap())
            .unwrap()
            .into_iter()
            .map(|record| record.headword)
            .collect();
        assert_eq!(words, ["avion", "zèbre", "bonjour", "bateau", "chat"]);
    }
    if let Some(destination) = std::env::var_os("LEXIQUE_NATIVE_REVIEW_SORT_FIXTURE") {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .unwrap();
        std::io::copy(&mut std::fs::File::open(&path).unwrap(), &mut output).unwrap();
    }
    std::fs::remove_file(path).unwrap();
}
