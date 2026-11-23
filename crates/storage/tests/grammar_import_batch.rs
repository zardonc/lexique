use lexique_storage::*;
use rusqlite::Connection;

fn content(identity: Option<&str>, title: &str) -> GrammarImportContent {
    GrammarImportContent {
        exchange_id: identity.map(String::from),
        title: title.into(),
        body: None,
        question: None,
        answer: None,
    }
}

#[test]
fn failed_second_pause_write_rolls_back_first_note_identity_card_and_receipt_then_replays() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    let items: Vec<_> = ["First", "Second"]
        .into_iter()
        .map(|title| GrammarImportItem {
            content: GrammarImportContent {
                question: Some("Q".into()),
                answer: Some("A".into()),
                ..content(Some(title), title)
            },
            existing: None,
        })
        .collect();
    db.execute_batch("CREATE TRIGGER fail_second BEFORE INSERT ON review_paused WHEN (SELECT count(*) FROM review_paused) = 1 BEGIN SELECT RAISE(FAIL, 'second pause failed'); END;").unwrap();
    assert!(
        save_grammar_import_batch(&mut db, "rollback", &items, 0, 1000)
            .unwrap_err()
            .to_string()
            .contains("second pause failed")
    );
    assert!(list_grammar(&db, "", "created-asc").unwrap().is_empty());
    assert!(find_grammar_exchange(&db, "First").unwrap().is_none());
    assert!(review_queue(&mut db, 1000, "UTC").unwrap().cards.is_empty());
    db.execute_batch("DROP TRIGGER fail_second").unwrap();
    let saved = save_grammar_import_batch(&mut db, "rollback", &items, 0, 1000).unwrap();
    assert_eq!(saved.created, 2);
    assert_eq!(saved.new_card_ids.len(), 2);
    for id in &saved.new_card_ids {
        assert!(review_paused(&db, *id).unwrap());
    }
    let replay = save_grammar_import_batch(&mut db, "rollback", &items, 0, 1001).unwrap();
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        serde_json::to_value(replay).unwrap()
    );
    assert!(save_grammar_import_batch(&mut db, "rollback", &items[..1], 1, 1001).is_err());
    assert_eq!(list_grammar(&db, "", "created-asc").unwrap().len(), 2);
}

#[test]
fn revision_or_new_identity_occupancy_rejects_whole_selected_batch() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    let saved = save_grammar_import(
        &mut db,
        "seed-stale",
        &content(Some("known"), "Rule"),
        None,
        1000,
    )
    .unwrap()
    .records
    .remove(0);
    let pending = GrammarImportItem {
        content: content(None, "Must roll back"),
        existing: None,
    };
    let stale = GrammarImportItem {
        content: content(Some("known"), "Updated"),
        existing: Some(saved.clone()),
    };
    save_grammar(
        &db,
        &GrammarSaveRequest {
            operation: "local-edit".into(),
            id: Some(saved.id),
            revision: Some(saved.revision),
            title: "Local".into(),
            body: "Local body".into(),
            category_ids: vec![],
        },
        1001,
    )
    .unwrap();
    assert!(
        save_grammar_import_batch(&mut db, "stale", &[pending.clone(), stale], 0, 1002)
            .unwrap_err()
            .to_string()
            .contains("changed after preview")
    );
    assert!(
        save_grammar_import_batch(
            &mut db,
            "occupied",
            &[
                pending,
                GrammarImportItem {
                    content: content(Some("known"), "Unseen at preview"),
                    existing: None
                }
            ],
            0,
            1002
        )
        .is_err()
    );
    assert_eq!(list_grammar(&db, "", "created-asc").unwrap().len(), 1);
    assert_eq!(get_grammar(&db, saved.id).unwrap().title, "Local");
}

#[test]
fn selected_updates_preserve_omitted_body_and_card_then_explicit_empty_clears_body() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    let original = GrammarImportContent {
        body: Some("Saved body".into()),
        question: Some("Question".into()),
        answer: Some("Answer".into()),
        ..content(Some("batch-known"), "Original")
    };
    let saved = save_grammar_import(&mut db, "seed", &original, None, 1000)
        .unwrap()
        .records
        .remove(0);
    let items = [GrammarImportItem {
        content: content(Some("batch-known"), "Renamed"),
        existing: Some(saved.clone()),
    }];
    let result = save_grammar_import_batch(&mut db, "update", &items, 2, 1001).unwrap();
    assert_eq!((result.created, result.updated, result.skipped), (0, 1, 2));
    let updated = get_grammar(&db, saved.id).unwrap();
    assert_eq!(updated.title, "Renamed");
    assert_eq!(updated.body, "Saved body");
    assert_eq!(updated.card, saved.card);
    assert!(review_paused(&db, saved.id).unwrap());
    assert!(result.new_card_ids.is_empty());
    let clear = [GrammarImportItem {
        content: GrammarImportContent {
            body: Some(String::new()),
            question: Some(" \n".into()),
            ..content(Some("batch-known"), "Renamed")
        },
        existing: Some(updated),
    }];
    save_grammar_import_batch(&mut db, "clear", &clear, 0, 1002).unwrap();
    assert_eq!(get_grammar(&db, saved.id).unwrap().body, "");
    assert_eq!(get_grammar(&db, saved.id).unwrap().card, saved.card);
}

#[test]
fn paired_updates_preserve_active_and_paused_learning_and_categories_without_reenrollment() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    let category = create_category(&db, "Local category").unwrap();
    let mut items = Vec::new();
    for (index, paused) in [false, true].into_iter().enumerate() {
        let identity = format!("learned-{index}");
        let input = GrammarImportContent {
            question: Some("Old Q".into()),
            answer: Some("Old A".into()),
            ..content(Some(&identity), "Same title")
        };
        let saved = save_grammar_import(&mut db, &format!("seed-{index}"), &input, None, 1000)
            .unwrap()
            .records
            .remove(0);
        let saved = save_grammar(
            &db,
            &GrammarSaveRequest {
                operation: format!("category-{index}"),
                id: Some(saved.id),
                revision: Some(saved.revision),
                title: saved.title,
                body: "Original body".into(),
                category_ids: vec![category.id],
            },
            1001,
        )
        .unwrap()
        .record;
        set_review_paused(&mut db, saved.id, false).unwrap();
        review_queue(&mut db, 1000, "UTC").unwrap();
        rate_card(
            &mut db,
            &RatingRequest {
                operation: format!("rating-{index}"),
                card_id: saved.card.as_ref().unwrap().id,
                expected_revision: 0,
                rating: Rating::Good,
                now: 1000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        set_review_paused(&mut db, saved.id, paused).unwrap();
        items.push(GrammarImportItem {
            content: GrammarImportContent {
                question: Some(" New Q \n".into()),
                answer: Some(" New A \n".into()),
                ..content(Some(&identity), "Renamed")
            },
            existing: Some(saved),
        });
    }
    let mut before =
        serde_json::to_value(export_learning_snapshot(&mut db, 2000).unwrap()).unwrap();
    let result = save_grammar_import_batch(&mut db, "learned-update", &items, 0, 1002).unwrap();
    assert_eq!(result.updated, 2);
    assert!(result.new_card_ids.is_empty());
    for (record, item) in result.records.iter().zip(&items) {
        let original = item.existing.as_ref().unwrap();
        assert_eq!(record.body, "Original body");
        assert_eq!(record.category_ids, vec![category.id]);
        assert_eq!(
            record.card.as_ref().unwrap().id,
            original.card.as_ref().unwrap().id
        );
        assert_eq!(record.card.as_ref().unwrap().question, " New Q \n");
        assert_eq!(record.card.as_ref().unwrap().answer, " New A \n");
    }
    let mut after = serde_json::to_value(export_learning_snapshot(&mut db, 2000).unwrap()).unwrap();
    before.as_object_mut().unwrap().remove("grammar_records");
    after.as_object_mut().unwrap().remove("grammar_records");
    for (key, value) in before.as_object().unwrap() {
        assert_eq!(&after[key], value, "retained snapshot field {key}");
    }
}
