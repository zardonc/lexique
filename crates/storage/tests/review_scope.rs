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
