use lexique_storage::*;
use rusqlite::{Connection, params};
use std::collections::HashSet;

const NOW: i64 = 1_700_000_000;

fn fixture() -> (Connection, std::path::PathBuf, ReviewScope) {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "lexique-scope-undo-{}-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut connection = Connection::open(&path).unwrap();
    initialize(&connection).unwrap();
    let outside = create_category(&connection, "École").unwrap();
    let selected = create_category(&connection, "Voyage").unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"école","version":"1","attribution":"fixture","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    let tx = connection.transaction().unwrap();
    for (id, word, category) in [
        (1, "école", outside.id),
        (2, "voyage", selected.id),
        (3, "train", selected.id),
    ] {
        tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'keep',1,1)", params![id, word, snapshot]).unwrap();
        tx.execute(
            "INSERT INTO record_categories(record_id,category_id) VALUES(?1,?2)",
            params![id, category],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    for id in 1..=3 {
        add_review(&mut connection, id).unwrap();
    }
    let scope = ReviewScope::Vocabulary {
        category_ids: vec![selected.id],
        include_uncategorized: false,
    };
    (connection, path, scope)
}

fn rate(connection: &mut Connection, card_id: i64, operation: &str, now: i64) {
    let expected_revision = review_progress(connection, card_id).unwrap().revision;
    rate_card(
        connection,
        &RatingRequest {
            operation: operation.into(),
            card_id,
            expected_revision,
            rating: Rating::Good,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
}

fn grammar(connection: &Connection) -> GrammarRecord {
    let saved = save_grammar(
        connection,
        &GrammarSaveRequest {
            operation: "grammar".into(),
            id: None,
            revision: None,
            title: "Passé composé".into(),
            body: "Règle".into(),
            category_ids: vec![],
        },
        NOW,
    )
    .unwrap()
    .record;
    save_grammar_card(
        connection,
        &GrammarCardRequest {
            operation: "grammar-card".into(),
            id: saved.id,
            revision: saved.revision,
            question: "Question".into(),
            answer: "Réponse".into(),
        },
        NOW,
    )
    .unwrap()
    .record
}

#[test]
fn global_grammar_undo_refunds_its_quota_without_exposing_grammar_in_vocabulary_scope() {
    let (mut connection, path, scope) = fixture();
    let note = grammar(&connection);
    let card = note.card.as_ref().unwrap().id;
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    settings.review_limit = 1;
    settings.new_cards_ignore_review_limit = true;
    set_review_settings(&connection, &settings).unwrap();
    rate(&mut connection, card, "grammar-rating", NOW);
    set_review_scope(&connection, &scope).unwrap();
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("grammar-rating")
    );
    assert_eq!(
        review_availability(&connection, selected, NOW, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    undo_rating(&mut connection, "undo-grammar", "grammar-rating").unwrap();
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(get_grammar(&connection, note.id).unwrap(), note);
    assert!(
        review_availability(&connection, selected, NOW, "UTC")
            .unwrap()
            .reason
            .is_none()
    );
    let queue = review_queue(&mut connection, NOW, "UTC").unwrap();
    assert_eq!(queue.cards.len(), 1);
    assert_eq!(queue.cards[0].id, selected);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn concurrent_newer_global_rating_rejects_an_older_in_scope_target_and_keeps_retries_safe() {
    let (mut connection, path, scope) = fixture();
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let outside = review_cards(&connection, 1).unwrap()[0].id;
    rate(&mut connection, selected, "in-scope", NOW);
    let target = undo_target(&connection).unwrap().unwrap();
    let mut other = Connection::open(&path).unwrap();
    rate(&mut other, outside, "newer-outside", NOW);
    set_review_scope(&connection, &scope).unwrap();
    let selected_progress =
        serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap();
    let newer = serde_json::to_value(review_progress(&other, outside).unwrap()).unwrap();
    assert!(matches!(
        undo_rating(&mut connection, "stale-undo", &target),
        Err(StorageError::Invalid(_))
    ));
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("newer-outside")
    );
    assert_eq!(
        serde_json::to_value(review_progress(&other, outside).unwrap()).unwrap(),
        newer
    );
    assert_eq!(
        serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap(),
        selected_progress
    );
    undo_rating(&mut connection, "global-undo", "newer-outside").unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    assert!(undo_rating(&mut connection, "older-undo", "in-scope").is_err());
    set_review_scope(&connection, &ReviewScope::All {}).unwrap();
    rate(&mut other, outside, "after-undo", NOW);
    set_review_scope(&connection, &scope).unwrap();
    let after = serde_json::to_value(review_progress(&other, outside).unwrap()).unwrap();
    undo_rating(&mut connection, "global-undo", "newer-outside").unwrap();
    assert_eq!(
        serde_json::to_value(review_progress(&other, outside).unwrap()).unwrap(),
        after
    );
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("after-undo")
    );
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(
        serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap(),
        selected_progress
    );
    drop(other);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn post_undo_cached_reads_keep_scope_order_skips_pause_due_times_and_audio_requirements() {
    let (mut connection, path, scope) = fixture();
    set_review_scope(&connection, &scope).unwrap();
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let next = review_cards(&connection, 3).unwrap()[0].id;
    let skipped = HashSet::from([selected]);
    let mut cache = QueueCache::default();
    let (before, _, _) =
        review_queue_cached(&mut connection, NOW, "UTC", &HashSet::new(), &mut cache).unwrap();
    assert_eq!(
        before.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
        vec![selected, next]
    );
    rate(&mut connection, selected, "scoped", NOW);
    undo_rating(&mut connection, "undo-scoped", "scoped").unwrap();
    let (after, _, _) =
        review_queue_cached(&mut connection, NOW, "UTC", &HashSet::new(), &mut cache).unwrap();
    assert_eq!(after.cards, before.cards);
    {
        let snapshot =
            review_queue_snapshot(&connection, NOW, "UTC", &skipped, &mut cache).unwrap();
        assert_eq!(
            snapshot
                .queue
                .cards
                .iter()
                .map(|card| card.id)
                .collect::<Vec<_>>(),
            vec![next]
        );
        assert_eq!(snapshot.audio.pending_count, 2);
        assert!(
            snapshot
                .demand_candidates
                .iter()
                .all(|card| card.id != selected && card.vocabulary_id != 1)
        );
        snapshot.transaction.commit().unwrap();
    }
    // The restored card remains paused after an independent management edit.
    rate(&mut connection, selected, "paused", NOW);
    set_review_paused(&mut connection, 2, true).unwrap();
    undo_rating(&mut connection, "undo-paused", "paused").unwrap();
    assert!(review_paused(&connection, 2).unwrap());
    assert_eq!(
        review_queue_cached(&mut connection, NOW, "UTC", &HashSet::new(), &mut cache)
            .unwrap()
            .0
            .cards
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        vec![next]
    );
    set_review_paused(&mut connection, 2, false).unwrap();
    rate(&mut connection, selected, "learning", NOW);
    let before_learning =
        serde_json::to_value(review_progress(&connection, selected).unwrap().card).unwrap();
    rate(&mut connection, selected, "learning-repeat", NOW + 600);
    undo_rating(&mut connection, "undo-learning-repeat", "learning-repeat").unwrap();
    assert_eq!(
        serde_json::to_value(review_progress(&connection, selected).unwrap().card).unwrap(),
        before_learning
    );
    let (future, _, _) = review_queue_cached(
        &mut connection,
        NOW + 300,
        "UTC",
        &HashSet::new(),
        &mut cache,
    )
    .unwrap();
    assert!(
        future
            .cards
            .iter()
            .all(|card| card.id != selected && card.kind != "listening" && card.vocabulary_id != 1)
    );
    assert_eq!(
        review_availability(&connection, selected, NOW + 300, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("due")
    );
    assert_eq!(review_scope(&connection).unwrap(), scope);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn failed_undo_commit_keeps_progress_quota_and_scope_until_the_same_request_succeeds() {
    let (mut connection, path, scope) = fixture();
    let outside = review_cards(&connection, 1).unwrap()[0].id;
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    set_review_settings(&connection, &settings).unwrap();
    rate(&mut connection, outside, "outside", NOW);
    set_review_scope(&connection, &scope).unwrap();
    let progress = serde_json::to_value(review_progress(&connection, outside).unwrap()).unwrap();
    connection.execute_batch("CREATE TABLE undo_failure_parent(id INTEGER PRIMARY KEY);
        CREATE TABLE undo_failure_child(parent INTEGER REFERENCES undo_failure_parent(id) DEFERRABLE INITIALLY DEFERRED);
        CREATE TRIGGER fail_undo_commit AFTER UPDATE OF undo_operation ON review_ratings WHEN NEW.undo_operation IS NOT NULL
        BEGIN INSERT INTO undo_failure_child VALUES(1); END;").unwrap();
    assert!(matches!(
        undo_rating(&mut connection, "retry-undo", "outside"),
        Err(StorageError::Database(_))
    ));
    assert_eq!(
        serde_json::to_value(review_progress(&connection, outside).unwrap()).unwrap(),
        progress
    );
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("outside")
    );
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(
        review_availability(&connection, selected, NOW, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    connection
        .execute_batch("DROP TRIGGER fail_undo_commit")
        .unwrap();
    undo_rating(&mut connection, "retry-undo", "outside").unwrap();
    undo_rating(&mut connection, "retry-undo", "outside").unwrap();
    assert!(
        review_availability(&connection, selected, NOW, "UTC")
            .unwrap()
            .reason
            .is_none()
    );
    assert_eq!(review_progress(&connection, outside).unwrap().revision, 2);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn scoped_undo_refunds_interday_review_usage_but_not_unrelated_or_intraday_usage() {
    let (mut connection, path, scope) = fixture();
    let outside = review_cards(&connection, 1).unwrap()[0].id;
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let next = review_cards(&connection, 3).unwrap()[0].id;
    let day_two = NOW + 86_400;
    rate(&mut connection, outside, "yesterday", NOW);
    rate(&mut connection, selected, "today-new", day_two);
    rate(&mut connection, outside, "today-interday", day_two);
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 2;
    settings.review_limit = 2;
    set_review_settings(&connection, &settings).unwrap();
    set_review_scope(&connection, &scope).unwrap();
    assert_eq!(
        review_availability(&connection, next, day_two, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    undo_rating(&mut connection, "refund-interday", "today-interday").unwrap();
    assert!(
        review_availability(&connection, next, day_two, "UTC")
            .unwrap()
            .reason
            .is_none()
    );
    rate(&mut connection, next, "today-second-new", day_two);
    let next_progress = serde_json::to_value(review_progress(&connection, next).unwrap()).unwrap();
    rate(&mut connection, selected, "today-intraday", day_two + 600);
    undo_rating(&mut connection, "undo-intraday", "today-intraday").unwrap();
    assert_eq!(review_settings(&connection).unwrap(), settings);
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(
        serde_json::to_value(review_progress(&connection, next).unwrap()).unwrap(),
        next_progress
    );
    set_review_scope(&connection, &ReviewScope::All {}).unwrap();
    let new_listening = review_cards(&connection, 1).unwrap()[1].id;
    assert_eq!(
        review_availability(&connection, new_listening, day_two + 600, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn invalid_or_deleted_global_latest_never_falls_back_to_an_older_scoped_rating() {
    let (mut connection, path, scope) = fixture();
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let next = review_cards(&connection, 3).unwrap()[0].id;
    rate(&mut connection, selected, "older-in-scope", NOW);
    let older = serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap();
    let note = grammar(&connection);
    let latest = note.card.as_ref().unwrap().id;
    rate(&mut connection, latest, "latest-grammar", NOW);
    set_review_scope(&connection, &scope).unwrap();
    // Simulate a committed progress revision that the rating cannot reverse.
    connection
        .execute(
            "UPDATE review_progress SET revision = revision + 1 WHERE card_id = ?1",
            [latest],
        )
        .unwrap();
    assert!(matches!(
        undo_rating(&mut connection, "conflicting-undo", "latest-grammar"),
        Err(StorageError::RatingConflict)
    ));
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("latest-grammar")
    );
    assert_eq!(
        serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap(),
        older
    );
    delete_grammar(&connection, note.id, note.revision).unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    assert!(undo_rating(&mut connection, "fallback-undo", "older-in-scope").is_err());
    assert_eq!(
        serde_json::to_value(review_progress(&connection, selected).unwrap()).unwrap(),
        older
    );
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 2;
    set_review_settings(&connection, &settings).unwrap();
    assert_eq!(
        review_availability(&connection, next, NOW, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    drop(connection);
    let connection = Connection::open(&path).unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    assert_eq!(review_scope(&connection).unwrap(), scope);
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn global_category_undo_reopens_refunds_quota_and_keeps_applied_scope() {
    let (mut connection, path, scope) = fixture();
    let outside = review_cards(&connection, 1).unwrap()[0].id;
    let selected = review_cards(&connection, 2).unwrap()[0].id;
    let before = review_progress(&connection, outside).unwrap();
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 1;
    settings.review_limit = 1;
    set_review_settings(&connection, &settings).unwrap();
    review_queue(&mut connection, NOW, "UTC").unwrap();
    rate(&mut connection, outside, "outside", NOW);
    set_review_scope(&connection, &scope).unwrap();
    assert_eq!(
        review_availability(&connection, selected, NOW, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    drop(connection);
    let mut connection = Connection::open(&path).unwrap();
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("outside")
    );
    undo_rating(&mut connection, "undo-outside", "outside").unwrap();
    undo_rating(&mut connection, "undo-outside", "outside").unwrap();
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(review_settings(&connection).unwrap(), settings);
    assert_eq!(get(&connection, 1).unwrap().notes, "keep");
    assert_eq!(
        serde_json::to_value(review_progress(&connection, outside).unwrap().card).unwrap(),
        serde_json::to_value(before.card).unwrap()
    );
    let queue = review_queue(&mut connection, NOW, "UTC").unwrap();
    assert_eq!(queue.cards.len(), 1);
    assert_eq!(queue.cards[0].id, selected);
    assert_eq!(
        review_availability(&connection, outside, NOW, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("scope")
    );
    drop(connection);
    let mut connection = Connection::open(&path).unwrap();
    assert_eq!(undo_target(&connection).unwrap(), None);
    assert_eq!(review_scope(&connection).unwrap(), scope);
    assert_eq!(
        review_queue(&mut connection, NOW, "UTC").unwrap().cards,
        queue.cards
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}
