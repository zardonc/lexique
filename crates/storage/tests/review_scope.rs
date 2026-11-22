use lexique_storage::*;
use rusqlite::{Connection, params};

fn fixture() -> (Connection, std::path::PathBuf, i64) {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "lexique-review-scope-{}-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let category = create_category(&connection, "Voyage").unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"école","version":"1","attribution":"fixture","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    let tx = connection.transaction().unwrap();
    for (id, word) in [(1, "école"), (2, "voyage")] {
        tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'keep',1,1)", params![id,word,snapshot]).unwrap();
    }
    tx.execute(
        "INSERT INTO record_categories(record_id,category_id) VALUES(2,?1)",
        [category.id],
    )
    .unwrap();
    tx.commit().unwrap();
    add_review(&mut connection, 1).unwrap();
    add_review(&mut connection, 2).unwrap();
    let grammar = save_grammar(
        &connection,
        &GrammarSaveRequest {
            operation: "grammar".into(),
            id: None,
            revision: None,
            title: "Passé composé".into(),
            body: "Règle".into(),
            category_ids: vec![category.id],
        },
        100,
    )
    .unwrap()
    .record;
    save_grammar_card(
        &connection,
        &GrammarCardRequest {
            operation: "grammar-card".into(),
            id: grammar.id,
            revision: grammar.revision,
            question: "Question".into(),
            answer: "Réponse".into(),
        },
        101,
    )
    .unwrap();
    (connection, path, category.id)
}

#[test]
fn saved_category_scope_preserves_vocabulary_cards_and_reopens_without_grammar() {
    let (mut connection, path, category) = fixture();
    assert_eq!(review_scope(&connection).unwrap(), ReviewScope::All {});
    assert!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .cards
            .iter()
            .any(|card| card.kind == "grammar")
    );
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    let expected = review_cards(&connection, 2).unwrap();
    let queue = review_queue(&mut connection, 1000, "UTC").unwrap();
    assert_eq!(
        queue.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
        expected.iter().map(|card| card.id).collect::<Vec<_>>()
    );
    assert!(
        queue
            .cards
            .iter()
            .all(|card| card.vocabulary_id == 2 && card.kind != "grammar")
    );
    drop(connection);
    let mut reopened = Connection::open(&path).unwrap();
    assert_eq!(review_scope(&reopened).unwrap(), scope);
    assert_eq!(
        review_queue(&mut reopened, 1000, "UTC").unwrap().cards,
        queue.cards
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn deleting_selected_categories_persists_remaining_conditions_and_requires_a_choice_on_reopen() {
    let (mut connection, path, category) = fixture();
    let remaining = create_category(&connection, "Remaining").unwrap().id;
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![category, remaining],
            include_uncategorized: true,
        },
    )
    .unwrap();
    delete_category(&connection, category).unwrap();
    assert_eq!(
        review_scope(&connection).unwrap(),
        ReviewScope::Vocabulary {
            category_ids: vec![remaining],
            include_uncategorized: true,
        }
    );
    delete_category(&connection, remaining).unwrap();
    assert_eq!(
        review_scope(&connection).unwrap(),
        ReviewScope::Vocabulary {
            category_ids: vec![],
            include_uncategorized: true,
        }
    );
    let last = create_category(&connection, "Last").unwrap().id;
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![last],
            include_uncategorized: false,
        },
    )
    .unwrap();
    delete_category(&connection, last).unwrap();
    assert_eq!(
        review_scope(&connection).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    assert_eq!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("needs_selection")
    );
    drop(connection);
    let reopened = Connection::open(&path).unwrap();
    assert_eq!(
        review_scope(&reopened).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn renaming_a_selected_category_keeps_identity_memberships_progress_and_scope() {
    let (mut connection, path, category) = fixture();
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    let card = review_cards(&connection, 2).unwrap()[0].id;
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "before-rename".into(),
            card_id: card,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let progress = review_progress(&connection, card).unwrap();
    let record = get(&connection, 2).unwrap();
    let queue = review_queue(&mut connection, 1001, "UTC").unwrap();
    assert_eq!(
        rename_category(&connection, category, "Travel").unwrap(),
        LearningCategory {
            id: category,
            name: "Travel".into()
        }
    );
    let unrelated = create_category(&connection, "Other").unwrap();
    rename_category(&connection, unrelated.id, "Elsewhere").unwrap();
    assert!(rename_category(&connection, category, "Elsewhere").is_err());
    assert_eq!(
        serde_json::to_value(get(&connection, 2).unwrap()).unwrap(),
        serde_json::to_value(record).unwrap()
    );
    assert_eq!(
        serde_json::to_value(review_progress(&connection, card).unwrap()).unwrap(),
        serde_json::to_value(&progress).unwrap()
    );
    assert_eq!(
        review_queue(&mut connection, 1001, "UTC").unwrap().cards,
        queue.cards
    );
    drop(connection);
    let reopened = Connection::open(&path).unwrap();
    assert_eq!(review_scope(&reopened).unwrap(), scope);
    assert!(
        learning_categories(&reopened)
            .unwrap()
            .contains(&LearningCategory {
                id: category,
                name: "Travel".into()
            })
    );
    assert_eq!(
        serde_json::to_value(review_progress(&reopened, card).unwrap()).unwrap(),
        serde_json::to_value(&progress).unwrap()
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn stale_saved_category_identities_are_revalidated_and_persisted_on_reentry() {
    let (connection, path, category) = fixture();
    // A preference written before live deletion revalidation was available.
    connection.execute("INSERT INTO app_metadata(key,value) VALUES('review-scope',?1)", [serde_json::json!({"kind":"vocabulary", "category_ids":[category, 999], "include_uncategorized":false}).to_string()]).unwrap();
    drop(connection);
    let reopened = Connection::open(&path).unwrap();
    assert_eq!(
        review_scope(&reopened).unwrap(),
        ReviewScope::Vocabulary {
            category_ids: vec![category],
            include_uncategorized: false
        }
    );
    drop(reopened);
    let reopened = Connection::open(&path).unwrap();
    delete_category(&reopened, category).unwrap();
    assert_eq!(
        review_scope(&reopened).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    assert!(
        set_review_scope(
            &reopened,
            &ReviewScope::Vocabulary {
                category_ids: vec![999],
                include_uncategorized: false
            }
        )
        .is_err()
    );
    assert_eq!(
        review_scope(&reopened).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn saved_memberships_update_cached_candidates_counts_and_audio_demand_without_changing_progress() {
    let (mut connection, path, category) = fixture();
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![category],
            include_uncategorized: false,
        },
    )
    .unwrap();
    let mut cache = QueueCache::default();
    let skipped = std::collections::HashSet::new();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(
        queue
            .cards
            .iter()
            .map(|card| card.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(audio.missing_words, vec![2]);
    let cards = review_cards(&connection, 1).unwrap();
    let request = CategoryBulkRequest {
        record_kind: "vocabulary".into(),
        ids: vec![1],
        category_id: category,
        action: "add".into(),
    };
    assert_eq!(
        bulk_classify(&connection, &request).unwrap()[0].status,
        "saved"
    );
    {
        let snapshot =
            review_queue_snapshot(&connection, 1000, "UTC", &skipped, &mut cache).unwrap();
        assert_eq!(
            snapshot
                .queue
                .cards
                .iter()
                .map(|card| card.vocabulary_id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(snapshot.audio.pending_count, 2);
        assert_eq!(snapshot.audio.missing_words, vec![1, 2]);
        assert_eq!(
            snapshot
                .demand_candidates
                .iter()
                .map(|card| card.vocabulary_id)
                .collect::<Vec<_>>(),
            vec![1, 1, 2, 2]
        );
        snapshot.transaction.commit().unwrap();
    }
    let failed = CategoryBulkRequest {
        category_id: 999,
        ..request
    };
    assert_eq!(
        bulk_classify(&connection, &failed).unwrap()[0].status,
        "failed"
    );
    let (unchanged, _, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(
        unchanged
            .cards
            .iter()
            .map(|card| card.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let remove = CategoryBulkRequest {
        category_id: category,
        action: "remove".into(),
        ..failed
    };
    assert_eq!(
        bulk_classify(&connection, &remove).unwrap()[0].status,
        "saved"
    );
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(
        queue
            .cards
            .iter()
            .map(|card| card.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(audio.missing_words, vec![2]);
    assert_eq!(review_cards(&connection, 1).unwrap(), cards);
    for card in cards {
        assert_eq!(review_progress(&connection, card.id).unwrap().revision, 0);
    }
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn current_presentation_finishes_after_membership_removal_without_bypassing_other_eligibility() {
    let (mut connection, path, category) = fixture();
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![category],
            include_uncategorized: false,
        },
    )
    .unwrap();
    let card = review_queue(&mut connection, 1000, "UTC").unwrap().cards[0].id;
    let revision = review_progress(&connection, card).unwrap().revision;
    let request = RatingRequest {
        operation: "finish-presentation".into(),
        card_id: card,
        expected_revision: revision,
        rating: Rating::Good,
        now: 1000,
        timezone: "UTC".into(),
    };
    bulk_classify(
        &connection,
        &CategoryBulkRequest {
            record_kind: "vocabulary".into(),
            ids: vec![2],
            category_id: category,
            action: "remove".into(),
        },
    )
    .unwrap();
    assert_eq!(
        review_availability(&connection, card, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("scope")
    );
    assert!(
        matches!(rate_card(&mut connection, &request), Err(StorageError::RatingUnavailable(reason)) if reason == "scope")
    );
    delete_category(&connection, category).unwrap();
    assert_eq!(
        review_scope(&connection).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    assert_eq!(
        review_presentation_availability(&connection, card, 1000, "UTC", Some(revision))
            .unwrap()
            .reason,
        None
    );
    set_review_paused(&mut connection, 2, true).unwrap();
    assert!(
        matches!(rate_presented_card(&mut connection, &request, Some(revision)), Err(StorageError::RatingUnavailable(reason)) if reason == "paused")
    );
    set_review_paused(&mut connection, 2, false).unwrap();
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 0;
    set_review_settings(&connection, &settings).unwrap();
    assert!(
        matches!(rate_presented_card(&mut connection, &request, Some(revision)), Err(StorageError::RatingUnavailable(reason)) if reason == "quota")
    );
    settings.new_limit = 20;
    set_review_settings(&connection, &settings).unwrap();
    let result = rate_presented_card(&mut connection, &request, Some(revision)).unwrap();
    assert_eq!(
        rate_presented_card(&mut connection, &request, Some(revision)).unwrap(),
        result
    );
    assert_eq!(
        review_progress(&connection, card).unwrap().revision,
        revision + 1
    );
    assert_eq!(
        review_presentation_availability(&connection, card, 1001, "UTC", Some(revision))
            .unwrap()
            .reason
            .as_deref(),
        Some("scope")
    );
    let future = RatingRequest {
        operation: "future-presentation".into(),
        expected_revision: revision + 1,
        now: 1001,
        ..request
    };
    assert!(
        matches!(rate_presented_card(&mut connection, &future, Some(revision + 1)), Err(StorageError::RatingUnavailable(reason)) if reason == "due")
    );
    let clock = RatingRequest {
        operation: "clock-presentation".into(),
        now: 999,
        ..future
    };
    assert!(
        matches!(rate_presented_card(&mut connection, &clock, Some(revision + 1)), Err(StorageError::RatingUnavailable(reason)) if reason == "clock")
    );
    assert!(
        review_queue(&mut connection, 1001, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn only_successfully_saved_note_memberships_change_the_scoped_queue_and_survive_reopen() {
    let (mut connection, path, category) = fixture();
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    let mut cache = QueueCache::default();
    let skipped = std::collections::HashSet::new();
    let (before, _, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    let record = get(&connection, 2).unwrap();
    let cards = review_cards(&connection, 2).unwrap();
    let mut request = SaveRequest {
        operation: "save-membership".into(),
        id: Some(record.id),
        revision: Some(record.revision),
        headword: record.headword,
        notes: record.notes,
        example_fr: record.example_fr,
        example_zh: record.example_zh,
        example_source: record.example_source,
        example_sense: record.example_sense,
        more_examples: record.more_examples,
        category_ids: vec![999],
    };
    assert!(save(&mut connection, &request, None).is_err());
    assert_eq!(get(&connection, 2).unwrap().category_ids, vec![category]);
    let (unchanged, _, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(unchanged.cards, before.cards);
    request.category_ids.clear();
    assert_eq!(
        save(&mut connection, &request, None).unwrap().outcome,
        "updated"
    );
    let (after, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert!(after.cards.is_empty());
    assert_eq!(after.reason.as_deref(), Some("empty_scope"));
    assert_eq!(audio.pending_count, 0);
    assert_eq!(review_cards(&connection, 2).unwrap(), cards);
    assert_eq!(get(&connection, 2).unwrap().notes, "keep");
    drop(connection);
    let reopened = Connection::open(&path).unwrap();
    assert_eq!(review_scope(&reopened).unwrap(), scope);
    assert!(get(&reopened, 2).unwrap().category_ids.is_empty());
    for card in cards {
        assert_eq!(review_progress(&reopened, card.id).unwrap().revision, 0);
    }
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn scoped_admission_and_rating_use_the_remaining_global_quota() {
    let (mut connection, path, category) = fixture();
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    settings.review_limit = 1;
    set_review_settings(&connection, &settings).unwrap();
    let outside = review_cards(&connection, 1).unwrap()[0].id;
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    assert_eq!(
        review_queue(&mut connection, 1000, "UTC").unwrap().cards[0].id,
        outside
    );
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![category],
            include_uncategorized: false,
        },
    )
    .unwrap();
    assert_eq!(
        review_queue(&mut connection, 1000, "UTC").unwrap().cards[0].id,
        selected
    );
    assert_eq!(
        review_availability(&connection, outside, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("scope")
    );
    let mut rating = RatingRequest {
        operation: "scoped-rating".into(),
        card_id: outside,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1000,
        timezone: "UTC".into(),
    };
    assert!(
        matches!(rate_card(&mut connection, &rating), Err(StorageError::RatingUnavailable(reason)) if reason == "scope")
    );
    rating.card_id = selected;
    let result = rate_card(&mut connection, &rating).unwrap();
    assert_eq!(review_progress(&connection, selected).unwrap().revision, 1);
    set_review_scope(&connection, &ReviewScope::All {}).unwrap();
    assert_eq!(rate_card(&mut connection, &rating).unwrap(), result);
    let queue = review_queue(&mut connection, 1000, "UTC").unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("due"));
    assert_eq!(
        review_availability(&connection, outside, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn category_window_metadata_and_audio_demand_share_scope_and_normal_eligibility() {
    let (mut connection, path, category) = fixture();
    let mut cache = QueueCache::default();
    let skipped = std::collections::HashSet::new();
    review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    {
        let snapshot =
            review_queue_snapshot(&connection, 1000, "UTC", &skipped, &mut cache).unwrap();
        assert_eq!(snapshot.queue.cards.len(), 1);
        assert_eq!(snapshot.queue.cards[0].vocabulary_id, 2);
        assert_eq!(snapshot.queue.cards[0].kind, "recognition");
        assert_eq!(snapshot.audio.pending_count, 1);
        assert_eq!(snapshot.audio.missing_words, vec![2]);
        assert_eq!(snapshot.demand_candidates.len(), 2);
        assert!(
            snapshot
                .demand_candidates
                .iter()
                .all(|card| card.vocabulary_id == 2)
        );
        let demands = plan_review_words(
            &snapshot.queue.cards,
            &snapshot.demand_candidates,
            &snapshot.audio.ready,
        );
        assert!(demands.iter().all(|word| word.vocabulary_id == 2));
        snapshot.transaction.commit().unwrap();
    }
    assert_eq!(cache.scope(), &scope);
    assert!(
        review_card_inventory(&connection)
            .unwrap()
            .iter()
            .all(|card| card.vocabulary_id == 2)
    );
    assert_eq!(
        queue_audio_metadata(&connection, 1000, &skipped)
            .unwrap()
            .missing_words,
        vec![2]
    );
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "future-card".into(),
            card_id: selected,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let (queue, _, _) =
        review_queue_cached(&mut connection, 1001, "UTC", &skipped, &mut cache).unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("due"));
    set_review_paused(&mut connection, 2, true).unwrap();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1001, "UTC", &skipped, &mut cache).unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(audio.pending_count, 0);
    assert_eq!(review_scope(&connection).unwrap(), scope);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_scope_and_failed_saves_do_not_replace_the_applied_preference() {
    let (connection, path, category) = fixture();
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    for ids in [
        vec![],
        vec![0],
        vec![category, category],
        vec![9_007_199_254_740_992],
        vec![999],
    ] {
        assert!(
            set_review_scope(
                &connection,
                &ReviewScope::Vocabulary {
                    category_ids: ids,
                    include_uncategorized: false
                }
            )
            .is_err()
        );
        assert_eq!(review_scope(&connection).unwrap(), scope);
    }
    connection.execute_batch("CREATE TRIGGER fail_scope_save BEFORE UPDATE ON app_metadata WHEN OLD.key='review-scope' BEGIN SELECT RAISE(ABORT,'scope disk failure'); END;").unwrap();
    assert!(set_review_scope(&connection, &ReviewScope::All {}).is_err());
    assert_eq!(review_scope(&connection).unwrap(), scope);
    connection.execute_batch("DROP TRIGGER fail_scope_save; UPDATE app_metadata SET value='{}' WHERE key='review-scope';").unwrap();
    assert!(review_scope(&connection).is_err());
    assert!(serde_json::from_str::<ReviewScope>(r#"{"kind":"all","category_ids":[1]}"#).is_err());
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn category_union_deduplicates_overlap_and_combines_uncategorized_without_resetting_cards() {
    let (mut connection, path, category) = fixture();
    let school = create_category(&connection, "École").unwrap().id;
    let outside = create_category(&connection, "Other").unwrap().id;
    let tx = connection.transaction().unwrap();
    for (id, word) in [(4, "classe"), (5, "gare"), (6, "ailleurs")] {
        tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) SELECT ?1,?2,snapshot,'keep',1,1 FROM vocabulary WHERE id=1", params![id, word]).unwrap();
    }
    for (id, category_id) in [(4, school), (5, category), (5, school), (6, outside)] {
        tx.execute(
            "INSERT INTO record_categories(record_id,category_id) VALUES(?1,?2)",
            params![id, category_id],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    for id in [4, 5, 6] {
        add_review(&mut connection, id).unwrap();
    }
    for id in [1, 2, 4, 5, 6] {
        let version = prepare_audio_version(
            &connection,
            id,
            None,
            AudioPublicationContent {
                hash: &content_hash(format!("OggS{id}").as_bytes()),
                bytes: 5,
                generated_at: Some(1000),
            },
            0,
            1000,
        )
        .unwrap();
        assert!(complete_audio_version(&connection, &version.locator, 1000).unwrap());
    }
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category, school],
        include_uncategorized: false,
    };
    set_review_scope(&connection, &scope).unwrap();
    let mut cache = QueueCache::default();
    let skipped = std::collections::HashSet::new();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(queue.cards.len(), 6);
    assert_eq!(audio.ready, std::collections::HashSet::from([2, 4, 5]));
    assert_eq!(
        queue
            .cards
            .iter()
            .filter(|card| card.vocabulary_id == 5)
            .map(|card| card.kind.as_str())
            .collect::<Vec<_>>(),
        vec!["recognition", "listening"]
    );
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category, school],
        include_uncategorized: true,
    };
    set_review_scope(&connection, &scope).unwrap();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    let mut records: Vec<_> = queue.cards.iter().map(|card| card.vocabulary_id).collect();
    records.sort_unstable();
    assert_eq!(records, vec![1, 1, 2, 2, 4, 4, 5, 5]);
    assert_eq!(
        queue
            .cards
            .iter()
            .map(|card| card.id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        8
    );
    assert_eq!(audio.ready, std::collections::HashSet::from([1, 2, 4, 5]));
    assert_eq!(audio.pending_count, 0);
    let overlap = review_cards(&connection, 5).unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "union-recognition".into(),
            card_id: overlap[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    assert_eq!(
        review_progress(&connection, overlap[0].id)
            .unwrap()
            .revision,
        1
    );
    assert_eq!(
        review_progress(&connection, overlap[1].id)
            .unwrap()
            .revision,
        0
    );
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![],
            include_uncategorized: true,
        },
    )
    .unwrap();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(
        queue.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
        review_cards(&connection, 1)
            .unwrap()
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(audio.ready, std::collections::HashSet::from([1]));
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    settings.review_limit = 1;
    set_review_settings(&connection, &settings).unwrap();
    assert_eq!(
        review_availability(&connection, queue.cards[0].id, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    set_review_scope(&connection, &scope).unwrap();
    drop(connection);
    let mut reopened = Connection::open(&path).unwrap();
    assert_eq!(review_scope(&reopened).unwrap(), scope);
    assert_eq!(
        review_progress(&reopened, overlap[0].id).unwrap().revision,
        1
    );
    assert_eq!(
        review_progress(&reopened, overlap[1].id).unwrap().revision,
        0
    );
    assert_eq!(
        review_availability(&reopened, overlap[1].id, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert_eq!(
        review_queue(&mut reopened, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("due")
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn combined_scope_keeps_audio_waiting_due_times_and_global_quota_diagnostics() {
    let (mut connection, path, category) = fixture();
    let empty = create_category(&connection, "Empty").unwrap().id;
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category, empty],
        include_uncategorized: true,
    };
    set_review_scope(&connection, &scope).unwrap();
    let first = review_cards(&connection, 1).unwrap();
    let second = review_cards(&connection, 2).unwrap();
    let mut cache = QueueCache::default();
    let skipped = std::collections::HashSet::from([first[0].id, second[0].id]);
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason, None);
    assert_eq!(audio.pending_count, 2);
    assert_eq!(audio.missing_words, vec![1, 2]);
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 0;
    settings.review_limit = 0;
    set_review_settings(&connection, &settings).unwrap();
    let skipped = std::collections::HashSet::new();
    let (queue, audio, _) =
        review_queue_cached(&mut connection, 1000, "UTC", &skipped, &mut cache).unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("quota"));
    assert_eq!(audio.pending_count, 2);
    assert_eq!(
        review_availability(&connection, first[0].id, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    settings.new_limit = 20;
    settings.review_limit = 50;
    set_review_settings(&connection, &settings).unwrap();
    for card in [&first[0], &second[0]] {
        rate_card(
            &mut connection,
            &RatingRequest {
                operation: format!("future-{}", card.id),
                card_id: card.id,
                expected_revision: 0,
                rating: Rating::Good,
                now: 1000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    }
    {
        let snapshot =
            review_queue_snapshot(&connection, 1001, "UTC", &skipped, &mut cache).unwrap();
        assert!(snapshot.queue.cards.is_empty());
        assert_eq!(snapshot.queue.reason.as_deref(), Some("due"));
        assert_eq!(snapshot.queue.available_at, Some(1600));
        assert_eq!(snapshot.audio.pending_count, 2);
        assert_eq!(snapshot.audio.missing_words, vec![1, 2]);
        assert_eq!(
            snapshot
                .demand_candidates
                .iter()
                .map(|card| card.id)
                .collect::<Vec<_>>(),
            vec![first[1].id, second[1].id]
        );
        snapshot.transaction.commit().unwrap();
    }
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(
        review_availability(&connection, first[0].id, 1001, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("due")
    );
    assert_eq!(
        review_progress(&connection, first[1].id).unwrap().revision,
        0
    );
    assert_eq!(
        review_progress(&connection, second[1].id).unwrap().revision,
        0
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn combined_scope_distinguishes_unenrolled_matches_pause_and_needs_selection() {
    let (mut connection, path, category) = fixture();
    let empty = create_category(&connection, "Unenrolled").unwrap();
    let tx = connection.transaction().unwrap();
    tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) SELECT 4,'sans carte',snapshot,'keep',1,1 FROM vocabulary WHERE id=1", []).unwrap();
    tx.execute(
        "INSERT INTO record_categories(record_id,category_id) VALUES(4,?1)",
        [empty.id],
    )
    .unwrap();
    tx.commit().unwrap();
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![category, empty.id],
        include_uncategorized: true,
    };
    set_review_scope(&connection, &scope).unwrap();
    for id in [1, 2] {
        set_review_paused(&mut connection, id, true).unwrap();
    }
    let queue = review_queue(&mut connection, 1000, "UTC").unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(queue.reason.as_deref(), Some("paused"));
    assert_eq!(queue.available_at, None);
    assert_eq!(review_scope(&connection).unwrap(), scope);
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![empty.id],
            include_uncategorized: false,
        },
    )
    .unwrap();
    assert_eq!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("empty_scope")
    );
    assert!(review_cards(&connection, 4).unwrap().is_empty());
    set_review_scope(&connection, &ReviewScope::NeedsSelection {}).unwrap();
    assert_eq!(
        review_queue(&mut connection, 1000, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("needs_selection")
    );
    drop(connection);
    let mut reopened = Connection::open(&path).unwrap();
    assert_eq!(
        review_scope(&reopened).unwrap(),
        ReviewScope::NeedsSelection {}
    );
    assert!(
        review_queue(&mut reopened, 1000, "UTC")
            .unwrap()
            .cards
            .is_empty()
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn scoped_audio_readiness_reports_only_selected_vocabulary() {
    let (mut connection, path, category) = fixture();
    for id in [1, 2] {
        let version = prepare_audio_version(
            &connection,
            id,
            None,
            AudioPublicationContent {
                hash: &content_hash(format!("OggS{id}").as_bytes()),
                bytes: 5,
                generated_at: Some(1000),
            },
            0,
            1000,
        )
        .unwrap();
        assert!(complete_audio_version(&connection, &version.locator, 1000).unwrap());
    }
    set_review_scope(
        &connection,
        &ReviewScope::Vocabulary {
            category_ids: vec![category],
            include_uncategorized: false,
        },
    )
    .unwrap();
    let (queue, audio, _) = review_queue_cached(
        &mut connection,
        1000,
        "UTC",
        &std::collections::HashSet::new(),
        &mut QueueCache::default(),
    )
    .unwrap();
    assert_eq!(queue.cards.len(), 2);
    assert_eq!(audio.ready, std::collections::HashSet::from([2]));
    assert_eq!(audio.pending_count, 0);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}
