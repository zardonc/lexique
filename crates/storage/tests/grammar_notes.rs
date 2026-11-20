use lexique_storage::{
    GrammarSaveRequest, get_grammar, initialize, list, list_grammar, save_grammar,
};
use rusqlite::Connection;

fn request(operation: &str) -> GrammarSaveRequest {
    GrammarSaveRequest {
        operation: operation.into(),
        id: None,
        revision: None,
        title: "  Passé composé  ".into(),
        body: "Une règle\nUne exception".into(),
        category_ids: vec![],
    }
}

#[test]
fn grammar_notes_keep_independent_identity_and_multiline_content() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = save_grammar(&connection, &request("first-note"), 100)
        .unwrap()
        .record;
    let second = save_grammar(&connection, &request("second-note"), 101)
        .unwrap()
        .record;
    assert_ne!(first.id, second.id);
    assert_eq!(
        get_grammar(&connection, first.id).unwrap().body,
        "Une règle\nUne exception"
    );
    assert_eq!(first.title, "Passé composé");
    assert_eq!(
        list_grammar(&connection, "PASSE", "updated-desc").unwrap()[0].id,
        second.id
    );
    assert!(list(&connection, "", "updated-desc").unwrap().is_empty());
    assert_eq!(
        save_grammar(&connection, &request("first-note"), 102)
            .unwrap()
            .record
            .id,
        first.id
    );
}

#[test]
fn note_category_and_card_limits_count_unicode_characters_without_losing_input() {
    use lexique_storage::{GrammarCardRequest, create_category, save_grammar_card};
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let mut draft = request("unicode-limits");
    draft.title = format!("  {}  ", "📖".repeat(200));
    draft.body = "é\n".repeat(10_000);
    let saved = save_grammar(&connection, &draft, 100).unwrap().record;
    assert_eq!(saved.title.chars().count(), 200);
    assert_eq!(saved.body, draft.body);
    assert!(create_category(&connection, &draft.title).is_ok());
    for title in [
        String::new(),
        "  ".into(),
        "📖".repeat(201),
        "Deux\nlignes".into(),
        "Deux\u{2028}lignes".into(),
    ] {
        let mut invalid = request("invalid-title");
        invalid.title = title.clone();
        assert!(save_grammar(&connection, &invalid, 101).is_err());
        assert!(create_category(&connection, &title).is_err());
    }
    let mut invalid = request("invalid-body");
    invalid.body = "📖".repeat(20_001);
    assert!(save_grammar(&connection, &invalid, 101).is_err());
    for (question, answer) in [
        ("  ".into(), "Réponse".into()),
        ("Question".into(), "\n\t".into()),
        ("📖".repeat(20_001), "Réponse".into()),
        ("Question".into(), "é".repeat(20_001)),
    ] {
        assert!(
            save_grammar_card(
                &connection,
                &GrammarCardRequest {
                    operation: "invalid-card".into(),
                    id: saved.id,
                    revision: saved.revision,
                    question,
                    answer,
                },
                102
            )
            .is_err()
        );
    }
    assert_eq!(get_grammar(&connection, saved.id).unwrap(), saved);
    let card = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "unicode-card".into(),
            id: saved.id,
            revision: saved.revision,
            question: "📖".repeat(20_000),
            answer: draft.body.clone(),
        },
        103,
    )
    .unwrap()
    .record
    .card
    .unwrap();
    assert_eq!(card.question.chars().count(), 20_000);
    assert_eq!(card.answer, draft.body);
    assert_eq!(
        list_grammar(&connection, "", "updated-desc").unwrap().len(),
        1
    );
}

#[test]
fn grammar_records_do_not_enter_word_audio_batches_or_dictionary_reference_readers() {
    use lexique_storage::{
        add_review_batch, export_learning_snapshot, referenced_custom_dictionary_ids,
        validate_learning_snapshot,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let note = save_grammar(&connection, &request("audio-boundary"), 100)
        .unwrap()
        .record;
    assert!(
        referenced_custom_dictionary_ids(&connection)
            .unwrap()
            .is_empty()
    );
    assert!(add_review_batch(&mut connection, "invalid-word-batch", &[note.id], 100).is_err());
    let snapshot = export_learning_snapshot(&mut connection, 100).unwrap();
    assert!(snapshot.batches.is_empty());
    assert!(snapshot.cards.is_empty());
    assert_eq!(snapshot.grammar_records.len(), 1);
    validate_learning_snapshot(&snapshot, 100).unwrap();
}

#[test]
fn mixed_review_planning_only_requests_audio_for_vocabulary() {
    use lexique_storage::{
        GrammarCardRequest, WordDemand, add_review, plan_review_words, review_queue,
        save_grammar_card,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let note = save_grammar(&connection, &request("grammar-audio-planning"), 100)
        .unwrap()
        .record;
    save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "grammar-audio-card".into(),
            id: note.id,
            revision: note.revision,
            question: "Une question".into(),
            answer: "Une réponse".into(),
        },
        100,
    )
    .unwrap();
    let word = word(&mut connection, &[]);
    add_review(&mut connection, word.id).unwrap();
    let queue = review_queue(&mut connection, 100, "UTC").unwrap();
    assert!(queue.cards.iter().any(|card| card.kind == "grammar"));
    assert_eq!(
        plan_review_words(&queue.cards, &queue.cards, &Default::default()),
        [WordDemand {
            vocabulary_id: word.id,
            class: 1
        }],
    );
}

#[test]
fn grammar_question_answer_cards_share_review_and_keep_progress_when_edited() {
    use lexique_storage::{
        GrammarCardRequest, Rating, RatingRequest, add_review, rate_card, review_cards,
        review_progress, review_queue, save_grammar_card, set_review_paused,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let now = 1_700_000_000;
    let note = save_grammar(&connection, &request("grammar-review-note"), now * 1000)
        .unwrap()
        .record;
    assert!(review_cards(&connection, note.id).unwrap().is_empty());
    let saved = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "join-grammar-review".into(),
            id: note.id,
            revision: note.revision,
            question: "Quand employer ce temps ?".into(),
            answer: "Une action achevée.\nUn exemple.".into(),
        },
        now * 1000,
    )
    .unwrap()
    .record;
    let card = saved.card.as_ref().unwrap();
    assert!(add_review(&mut connection, note.id).is_err());
    assert_eq!(
        review_queue(&mut connection, now, "UTC").unwrap().cards[0].kind,
        "grammar"
    );
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "grammar-first-rating".into(),
            card_id: card.id,
            expected_revision: 0,
            rating: Rating::Good,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let progress = review_progress(&connection, card.id).unwrap();
    let edited = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "edit-grammar-answer".into(),
            id: note.id,
            revision: saved.revision,
            question: "Quand employer le passé composé ?".into(),
            answer: "Une action terminée.".into(),
        },
        now * 1000 + 1,
    )
    .unwrap()
    .record;
    assert_eq!(edited.card.as_ref().unwrap().id, card.id);
    let retained = review_progress(&connection, card.id).unwrap();
    assert_eq!(retained.revision, progress.revision);
    assert_eq!(
        serde_json::to_value(retained.card).unwrap(),
        serde_json::to_value(progress.card).unwrap()
    );
    let mut changed = request("edit-grammar-note");
    changed.id = Some(note.id);
    changed.revision = Some(edited.revision);
    changed.body = "Nouvelle note".into();
    assert_eq!(
        save_grammar(&connection, &changed, now * 1000 + 2)
            .unwrap()
            .record
            .card
            .unwrap()
            .answer,
        "Une action terminée."
    );
    set_review_paused(&mut connection, note.id, true).unwrap();
    assert!(
        review_queue(&mut connection, now + 1200, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    set_review_paused(&mut connection, note.id, false).unwrap();
    assert_eq!(
        review_queue(&mut connection, now + 1200, "UTC")
            .unwrap()
            .cards[0]
            .id,
        card.id
    );
}

#[test]
fn deletion_keeps_consumed_quota_and_only_clears_the_deleted_latest_undo() {
    use lexique_storage::{
        GrammarCardRequest, Rating, RatingRequest, delete_grammar, rate_card, review_queue,
        review_settings, save_grammar_card, set_review_settings, undo_target,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let now = 1_700_000_000;
    let mut notes = vec![];
    for index in 0..3 {
        let note = save_grammar(
            &connection,
            &request(&format!("quota-note-{index}")),
            now * 1000,
        )
        .unwrap()
        .record;
        notes.push(
            save_grammar_card(
                &connection,
                &GrammarCardRequest {
                    operation: format!("quota-card-{index}"),
                    id: note.id,
                    revision: note.revision,
                    question: "Une question".into(),
                    answer: "Une réponse".into(),
                },
                now * 1000,
            )
            .unwrap()
            .record,
        );
    }
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 2;
    set_review_settings(&connection, &settings).unwrap();
    review_queue(&mut connection, now, "UTC").unwrap();
    for (index, note) in notes.iter().take(2).enumerate() {
        rate_card(
            &mut connection,
            &RatingRequest {
                operation: format!("quota-rating-{index}"),
                card_id: note.card.as_ref().unwrap().id,
                expected_revision: 0,
                rating: Rating::Easy,
                now: now + index as i64,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    }
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("quota-rating-1")
    );
    delete_grammar(&connection, notes[1].id, notes[1].revision).unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    let queue = review_queue(&mut connection, now + 2, "UTC").unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("quota"));
    settings.new_limit = 3;
    set_review_settings(&connection, &settings).unwrap();
    review_queue(&mut connection, now + 2, "UTC").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "quota-rating-2".into(),
            card_id: notes[2].card.as_ref().unwrap().id,
            expected_revision: 0,
            rating: Rating::Easy,
            now: now + 2,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    delete_grammar(&connection, notes[0].id, notes[0].revision).unwrap();
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("quota-rating-2")
    );
    delete_grammar(&connection, notes[2].id, notes[2].revision).unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    let replacement = save_grammar(&connection, &request("replacement-note"), now * 1000)
        .unwrap()
        .record;
    assert!(replacement.id > notes[2].id);
    delete_grammar(&connection, notes[2].id, notes[2].revision).unwrap();
    assert_eq!(
        get_grammar(&connection, replacement.id).unwrap().id,
        replacement.id
    );
    let replacement = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "replacement-card".into(),
            id: replacement.id,
            revision: replacement.revision,
            question: "Une question".into(),
            answer: "Une réponse".into(),
        },
        now * 1000,
    )
    .unwrap()
    .record;
    assert!(replacement.card.unwrap().id > notes[2].card.as_ref().unwrap().id);
    let queue = review_queue(&mut connection, now + 3, "UTC").unwrap();
    assert_eq!(queue.reason.as_deref(), Some("quota"));
}

fn word(connection: &mut Connection, category_ids: &[i64]) -> lexique_storage::VocabularyRecord {
    let request = serde_json::from_value(serde_json::json!({
        "operation": "shared-category-word", "headword": "école", "notes": "Une note",
        "example_fr": "", "example_zh": "", "example_sense": null, "more_examples": [],
        "id": null, "revision": null, "category_ids": category_ids,
    }))
    .unwrap();
    let snapshot = serde_json::from_value(serde_json::json!({
        "reference": { "dictionary": "Fixture", "lookup_key": "école", "version": "1",
            "attribution": null, "license": null, "source_url": null, "license_url": null,
            "sha256": "a".repeat(64) },
        "digest": [{ "pronunciations": [], "grammar": [], "translation": "学校" }],
    }))
    .unwrap();
    lexique_storage::save(connection, &request, Some(snapshot))
        .unwrap()
        .record
}

#[test]
fn grammar_deletion_preserves_unrelated_vocabulary_batch_history() {
    use lexique_storage::{
        add_review_batch, batch_status, delete_grammar, export_learning_snapshot,
        import_learning_snapshot, read_learning_snapshot, review_cards,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let record = word(&mut connection, &[]);
    let ids = [record.id, record.id + 1];
    let original = serde_json::to_value(
        add_review_batch(&mut connection, "batch-history-delete", &ids, 1000).unwrap(),
    )
    .unwrap();
    let cards = review_cards(&connection, record.id).unwrap();
    let note = save_grammar(&connection, &request("batch-history-note"), 1001)
        .unwrap()
        .record;
    assert_eq!(note.id, ids[1]);
    delete_grammar(&connection, note.id, note.revision).unwrap();
    assert_eq!(
        serde_json::to_value(batch_status(&connection, "batch-history-delete").unwrap()).unwrap(),
        original
    );
    assert_eq!(review_cards(&connection, record.id).unwrap(), cards);
    let snapshot = export_learning_snapshot(&mut connection, 1002).unwrap();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let snapshot = read_learning_snapshot(bytes.as_slice(), 1002).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &snapshot, 1002).unwrap();
    assert_eq!(
        serde_json::to_value(
            add_review_batch(&mut restored, "batch-history-delete", &ids, 1002).unwrap()
        )
        .unwrap(),
        original
    );
    assert_eq!(review_cards(&restored, record.id).unwrap(), cards);
}

#[test]
fn vocabulary_batch_history_replays_and_resumes_when_missing_ids_become_grammar() {
    use lexique_storage::{
        add_review_batch, add_review_batch_observed, batch_status, delete_grammar,
        export_learning_snapshot, import_learning_snapshot, read_learning_snapshot,
    };
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let record = word(&mut connection, &[]);
    let mut ids = vec![record.id, record.id + 1];
    ids.extend(1000..1098);
    ids.push(record.id + 2);
    let interrupted = catch_unwind(AssertUnwindSafe(|| {
        add_review_batch_observed(&mut connection, "batch-history-resume", &ids, 1000, || {
            panic!("interrupt after the first committed chunk");
        })
    }));
    assert!(interrupted.is_err());
    let committed = batch_status(&connection, "batch-history-resume").unwrap();
    assert_eq!(committed.items.len(), 100);
    let note = save_grammar(&connection, &request("batch-history-committed-note"), 1001)
        .unwrap()
        .record;
    assert_eq!(note.id, ids[1]);
    let pending = save_grammar(&connection, &request("batch-history-pending-note"), 1002)
        .unwrap()
        .record;
    assert_eq!(pending.id, *ids.last().unwrap());
    assert!(
        add_review_batch(&mut connection, "batch-history-resume", &ids, 1003)
            .err()
            .unwrap()
            .to_string()
            .contains("Word audio batches cannot contain grammar notes")
    );
    delete_grammar(&connection, pending.id, pending.revision).unwrap();
    let resumed = add_review_batch(&mut connection, "batch-history-resume", &ids, 1004).unwrap();
    assert_eq!(resumed.items.len(), ids.len());
    assert_eq!(
        serde_json::to_value(&resumed.items[..100]).unwrap(),
        serde_json::to_value(&committed.items).unwrap()
    );
    assert!(resumed.items[1].error.is_some());
    assert!(get_grammar(&connection, note.id).unwrap().card.is_none());
    let snapshot = export_learning_snapshot(&mut connection, 1005).unwrap();
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let snapshot = read_learning_snapshot(bytes.as_slice(), 1005).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &snapshot, 1005).unwrap();
    assert_eq!(
        serde_json::to_value(
            add_review_batch(&mut restored, "batch-history-resume", &ids, 1005).unwrap()
        )
        .unwrap(),
        serde_json::to_value(resumed).unwrap()
    );
}

#[test]
fn deleted_note_save_replays_stay_rejected_after_snapshot_restore() {
    use lexique_storage::{
        delete_grammar, export_learning_snapshot, import_learning_snapshot, read_learning_snapshot,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let draft = request("deleted-note-create");
    let note = save_grammar(&connection, &draft, 100).unwrap().record;
    delete_grammar(&connection, note.id, note.revision).unwrap();
    assert!(save_grammar(&connection, &draft, 101).is_err());
    assert!(
        list_grammar(&connection, "", "updated-desc")
            .unwrap()
            .is_empty()
    );

    let snapshot = export_learning_snapshot(&mut connection, 101).unwrap();
    assert!(snapshot.grammar_records.is_empty());
    assert!(snapshot.grammar_save_operations.is_empty());
    assert_eq!(
        snapshot.deleted_grammar_operations.as_slice(),
        std::slice::from_ref(&draft.operation)
    );
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let json = std::str::from_utf8(&bytes).unwrap();
    assert!(!json.contains(note.title.as_str()));
    assert!(!json.contains(note.body.as_str()));
    let mut duplicate: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    duplicate["deleted_grammar_operations"] = serde_json::json!([draft.operation, draft.operation]);
    assert!(
        read_learning_snapshot(serde_json::to_vec(&duplicate).unwrap().as_slice(), 101).is_err()
    );
    let snapshot = read_learning_snapshot(bytes.as_slice(), 101).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &snapshot, 101).unwrap();
    assert!(save_grammar(&restored, &draft, 102).is_err());
    let replacement = save_grammar(&restored, &request("replacement-after-delete"), 102)
        .unwrap()
        .record;
    assert!(replacement.id > note.id);
    assert!(save_grammar(&restored, &draft, 103).is_err());
    assert_eq!(
        list_grammar(&restored, "", "updated-desc").unwrap().len(),
        1
    );
}

#[test]
fn older_restores_keep_deleted_operations_and_restored_receipts_remain_replayable() {
    use lexique_storage::{
        delete_grammar, deleted_grammar_operations, export_learning_snapshot,
        import_learning_snapshot, preserve_deleted_grammar_operations, read_learning_snapshot,
    };
    let mut current = Connection::open_in_memory().unwrap();
    initialize(&current).unwrap();
    let draft = request("deleted-after-old-backup");
    let note = save_grammar(&current, &draft, 100).unwrap().record;
    let before_deletion = export_learning_snapshot(&mut current, 100).unwrap();
    delete_grammar(&current, note.id, note.revision).unwrap();
    let operations = deleted_grammar_operations(&current).unwrap();

    for version in [1, 2] {
        let mut restored = Connection::open_in_memory().unwrap();
        initialize(&restored).unwrap();
        let empty = export_learning_snapshot(&mut restored, 101).unwrap();
        let mut value = serde_json::to_value(empty).unwrap();
        value["snapshot_version"] = version.into();
        for field in [
            "grammar_records",
            "grammar_save_operations",
            "deleted_grammar_operations",
            "categories",
            "retained_review_usage",
            "identities",
        ] {
            value.as_object_mut().unwrap().remove(field);
        }
        for field in ["grammar_sort", "undo_blocked"] {
            value["preferences"].as_object_mut().unwrap().remove(field);
        }
        let legacy =
            read_learning_snapshot(serde_json::to_vec(&value).unwrap().as_slice(), 101).unwrap();
        import_learning_snapshot(&mut restored, &legacy, 101).unwrap();
        preserve_deleted_grammar_operations(&restored, &operations).unwrap();
        assert!(save_grammar(&restored, &draft, 102).is_err());
        assert!(
            list_grammar(&restored, "", "updated-desc")
                .unwrap()
                .is_empty()
        );
    }

    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &before_deletion, 101).unwrap();
    preserve_deleted_grammar_operations(&restored, &operations).unwrap();
    assert_eq!(save_grammar(&restored, &draft, 102).unwrap().record, note);
    let snapshot = export_learning_snapshot(&mut restored, 102).unwrap();
    assert!(snapshot.deleted_grammar_operations.is_empty());
    read_learning_snapshot(serde_json::to_vec(&snapshot).unwrap().as_slice(), 102).unwrap();
}

#[test]
fn deletion_retries_finish_only_the_requested_identity() {
    use lexique_storage::{StorageError, create_category, delete_category, delete_grammar, get};
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let category = create_category(&connection, "Temps").unwrap();
    let word = word(&mut connection, &[category.id]);
    let note = save_grammar(&connection, &request("delete-retry"), 100)
        .unwrap()
        .record;
    assert!(delete_grammar(&connection, word.id, word.revision).is_err());
    assert!(delete_grammar(&connection, note.id, note.revision + 1).is_err());
    assert_eq!(get_grammar(&connection, note.id).unwrap(), note);
    delete_grammar(&connection, note.id, note.revision).unwrap();
    let replacement = save_grammar(&connection, &request("delete-retry-replacement"), 101)
        .unwrap()
        .record;
    delete_grammar(&connection, note.id, note.revision).unwrap();
    assert_eq!(
        get_grammar(&connection, replacement.id).unwrap(),
        replacement
    );
    delete_category(&connection, category.id).unwrap();
    let retained = get(&connection, word.id).unwrap();
    let replacement_category = create_category(&connection, "Temps").unwrap();
    delete_category(&connection, category.id).unwrap();
    assert_eq!(
        serde_json::to_value(get(&connection, word.id).unwrap()).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    assert!(replacement_category.id > category.id);
    for invalid in [-1, 0, 9_007_199_254_740_992] {
        assert!(matches!(
            get_grammar(&connection, invalid),
            Err(StorageError::Invalid(_))
        ));
        assert!(delete_grammar(&connection, invalid, 1).is_err());
        assert!(delete_grammar(&connection, replacement.id, invalid).is_err());
        assert!(delete_category(&connection, invalid).is_err());
    }
    assert_eq!(
        get_grammar(&connection, replacement.id).unwrap(),
        replacement
    );
}

#[test]
fn shared_categories_are_unique_and_changes_preserve_notes_and_review() {
    use lexique_storage::{
        add_review, create_category, delete_category, get, learning_categories, rename_category,
        review_progress,
    };
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    assert!(learning_categories(&connection).unwrap().is_empty());
    let category = create_category(&connection, " Pronoms ").unwrap();
    assert_eq!(category.name, "Pronoms");
    assert!(create_category(&connection, "pronoms").is_err());
    let accent = create_category(&connection, "É").unwrap();
    assert!(create_category(&connection, "e\u{301}").is_err());
    assert_ne!(
        create_category(&connection, "a").unwrap().id,
        create_category(&connection, "à").unwrap().id
    );
    let mut draft = request("categorized-grammar");
    draft.category_ids = vec![category.id, accent.id];
    let note = save_grammar(&connection, &draft, 100).unwrap().record;
    let word = word(&mut connection, &[category.id]);
    let card = add_review(&mut connection, word.id).unwrap()[0].id;
    let progress = review_progress(&connection, card).unwrap();
    rename_category(&connection, category.id, "Règles").unwrap();
    assert_eq!(
        get(&connection, word.id).unwrap().category_ids,
        vec![category.id]
    );
    assert_eq!(
        get_grammar(&connection, note.id).unwrap().category_ids,
        vec![category.id, accent.id]
    );
    let mut edited = request("failing-note-membership-save");
    edited.id = Some(note.id);
    edited.revision = Some(note.revision);
    edited.title = "Changed".into();
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(save_grammar(&connection, &edited, 200).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(get_grammar(&connection, note.id).unwrap(), note);
    delete_category(&connection, category.id).unwrap();
    assert!(get(&connection, word.id).unwrap().category_ids.is_empty());
    assert_eq!(
        get_grammar(&connection, note.id).unwrap().category_ids,
        vec![accent.id]
    );
    assert_eq!(
        review_progress(&connection, card).unwrap().revision,
        progress.revision
    );
    let replacement = create_category(&connection, "Règles").unwrap();
    assert!(replacement.id > category.id);
    assert!(rename_category(&connection, category.id, "Stale").is_err());
    assert_eq!(
        learning_categories(&connection)
            .unwrap()
            .iter()
            .find(|item| item.id == replacement.id)
            .unwrap()
            .name,
        "Règles"
    );
}

#[test]
fn bulk_classification_preserves_other_memberships_and_reports_each_record() {
    use lexique_storage::{CategoryBulkRequest, bulk_classify, create_category};
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = create_category(&connection, "Temps").unwrap();
    let second = create_category(&connection, "Erreurs").unwrap();
    let mut draft = request("bulk-grammar");
    draft.category_ids = vec![first.id];
    let note = save_grammar(&connection, &draft, 100).unwrap().record;
    let word = word(&mut connection, &[]);
    let results = bulk_classify(
        &connection,
        &CategoryBulkRequest {
            record_kind: "grammar".into(),
            ids: vec![note.id, word.id, 9999],
            category_id: second.id,
            action: "add".into(),
        },
    )
    .unwrap();
    assert_eq!(
        results
            .iter()
            .map(|item| item.status.as_str())
            .collect::<Vec<_>>(),
        ["saved", "failed", "failed"]
    );
    assert_eq!(
        get_grammar(&connection, note.id).unwrap().category_ids,
        vec![first.id, second.id]
    );
    assert!(
        lexique_storage::get(&connection, word.id)
            .unwrap()
            .category_ids
            .is_empty()
    );
    let before = get_grammar(&connection, note.id).unwrap();
    connection.commit_hook(Some(|| true)).unwrap();
    assert_eq!(
        bulk_classify(
            &connection,
            &CategoryBulkRequest {
                record_kind: "grammar".into(),
                ids: vec![note.id],
                category_id: first.id,
                action: "remove".into(),
            }
        )
        .unwrap()[0]
            .status,
        "failed"
    );
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(get_grammar(&connection, note.id).unwrap(), before);
    assert_eq!(
        bulk_classify(
            &connection,
            &CategoryBulkRequest {
                record_kind: "grammar".into(),
                ids: vec![note.id],
                category_id: first.id,
                action: "remove".into(),
            }
        )
        .unwrap()[0]
            .status,
        "saved"
    );
    assert_eq!(
        get_grammar(&connection, note.id).unwrap().category_ids,
        vec![second.id]
    );
}

#[test]
fn complete_backup_restores_notes_classification_quota_undo_and_identity_watermarks() {
    use lexique_storage::*;
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let now = 1_700_000_000;
    let category = create_category(&connection, "Règles").unwrap();
    let removed_category = create_category(&connection, "Ancienne catégorie").unwrap();
    delete_category(&connection, removed_category.id).unwrap();
    let word = word(&mut connection, &[category.id]);
    let mut notes = vec![];
    for index in 0..3 {
        let mut draft = request(&format!("backup-note-{index}"));
        draft.category_ids = vec![category.id];
        draft.body = if index == 2 {
            "removed-grammar-body-distinct"
        } else {
            "La règle\nL’exemple"
        }
        .into();
        let note = save_grammar(&connection, &draft, now * 1000)
            .unwrap()
            .record;
        notes.push(
            save_grammar_card(
                &connection,
                &GrammarCardRequest {
                    operation: format!("backup-card-{index}"),
                    id: note.id,
                    revision: note.revision,
                    question: "Une question".into(),
                    answer: if index == 2 {
                        "removed-grammar-answer-distinct"
                    } else {
                        "Une réponse\nUn exemple"
                    }
                    .into(),
                },
                now * 1000,
            )
            .unwrap()
            .record,
        );
    }
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 2;
    set_review_settings(&connection, &settings).unwrap();
    set_sort_preference(&connection, "headword-desc").unwrap();
    set_grammar_sort_preference(&connection, "created-asc").unwrap();
    review_queue(&mut connection, now, "UTC").unwrap();
    for (index, note) in [&notes[0], &notes[2]].into_iter().enumerate() {
        rate_card(
            &mut connection,
            &RatingRequest {
                operation: format!("backup-rating-{index}"),
                card_id: note.card.as_ref().unwrap().id,
                expected_revision: 0,
                rating: Rating::Easy,
                now: now + index as i64,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    }
    delete_grammar(&connection, notes[2].id, notes[2].revision).unwrap();
    let snapshot = export_learning_snapshot(&mut connection, now + 2).unwrap();
    assert_eq!(snapshot.snapshot_version, 3);
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let content = String::from_utf8(bytes.clone()).unwrap();
    assert!(!content.contains("removed-grammar-body-distinct"));
    assert!(!content.contains("removed-grammar-answer-distinct"));
    let snapshot = read_learning_snapshot(bytes.as_slice(), now + 2).unwrap();
    let mut restored = Connection::open_in_memory().unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(&mut restored, &snapshot, now + 2).unwrap();
    assert_eq!(get_grammar(&restored, notes[0].id).unwrap(), notes[0]);
    assert_eq!(
        get(&restored, word.id).unwrap().category_ids,
        vec![category.id]
    );
    assert_eq!(sort_preference(&restored).unwrap(), "headword-desc");
    assert_eq!(grammar_sort_preference(&restored).unwrap(), "created-asc");
    assert_eq!(undo_target(&restored).unwrap(), None);
    assert_eq!(
        review_queue(&mut restored, now + 2, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert!(create_category(&restored, "Nouvelle catégorie").unwrap().id > removed_category.id);
    let fresh = save_grammar(&restored, &request("restored-new-note"), now * 1000)
        .unwrap()
        .record;
    assert!(fresh.id > notes[2].id);
    let fresh = save_grammar_card(
        &restored,
        &GrammarCardRequest {
            operation: "restored-new-card".into(),
            id: fresh.id,
            revision: fresh.revision,
            question: "Question".into(),
            answer: "Réponse".into(),
        },
        now * 1000,
    )
    .unwrap()
    .record;
    assert!(fresh.card.unwrap().id > notes[2].card.as_ref().unwrap().id);
}

#[test]
fn grammar_due_sort_uses_raw_due_dates_then_new_and_unenrolled_records() {
    use lexique_storage::*;
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let now = 1_700_000_000;
    let unenrolled = save_grammar(&connection, &request("sort-unenrolled"), 1)
        .unwrap()
        .record;
    let mut cards = vec![];
    for index in 0..2 {
        let mut draft = request(&format!("sort-note-{index}"));
        draft.title = format!("Title {index}");
        let note = save_grammar(&connection, &draft, 10 + index)
            .unwrap()
            .record;
        cards.push(
            save_grammar_card(
                &connection,
                &GrammarCardRequest {
                    operation: format!("sort-card-{index}"),
                    id: note.id,
                    revision: note.revision,
                    question: "Question".into(),
                    answer: "Answer".into(),
                },
                20 + index,
            )
            .unwrap()
            .record,
        );
    }
    review_queue(&mut connection, now, "UTC").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "sort-rating".into(),
            card_id: cards[1].card.as_ref().unwrap().id,
            expected_revision: 0,
            rating: Rating::Easy,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    set_review_paused(&mut connection, cards[1].id, true).unwrap();
    let sorted = list_grammar(&connection, "", "review-due-asc").unwrap();
    assert_eq!(
        sorted.iter().map(|record| record.id).collect::<Vec<_>>(),
        [cards[1].id, cards[0].id, unenrolled.id]
    );
    assert!(
        list_grammar(&connection, "Answer", "updated-desc")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn vocabulary_and_grammar_share_quota_and_review_records_remain_distinct() {
    use lexique_storage::*;
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = word(&mut connection, &[]);
    let word_cards = add_review(&mut connection, word.id).unwrap();
    let note = save_grammar(&connection, &request("mixed-note"), 100)
        .unwrap()
        .record;
    let note = save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "mixed-card".into(),
            id: note.id,
            revision: note.revision,
            question: "Une question".into(),
            answer: "Une réponse".into(),
        },
        101,
    )
    .unwrap()
    .record;
    let records = get_review_records(&connection, &[word.id, note.id, 9999]).unwrap();
    assert!(matches!(
        records.get(&word.id),
        Some(ReviewRecord::Vocabulary(_))
    ));
    assert!(matches!(
        records.get(&note.id),
        Some(ReviewRecord::Grammar(_))
    ));
    assert!(!records.contains_key(&9999));
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    set_review_settings(&connection, &settings).unwrap();
    let now = 1_700_000_000;
    review_queue(&mut connection, now, "UTC").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "mixed-word-rating".into(),
            card_id: word_cards[0].id,
            expected_revision: 0,
            rating: Rating::Easy,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let queue = review_queue(&mut connection, now + 1, "UTC").unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("quota"));
    undo_rating(&mut connection, "mixed-undo", "mixed-word-rating").unwrap();
    assert_eq!(
        review_queue(&mut connection, now + 1, "UTC").unwrap().cards[0].id,
        word_cards[0].id
    );
    set_review_paused(&mut connection, word.id, true).unwrap();
    assert!(
        review_queue(&mut connection, now + 1, "UTC")
            .unwrap()
            .cards
            .iter()
            .any(|card| card.kind == "grammar")
    );
}
