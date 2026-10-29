use lexique_core::scheduling::Rating;
use lexique_storage::{
    DigestEntry, Reference, SaveRequest, Snapshot, add_review, initialize, review_cards, save,
};
use rusqlite::Connection;

fn saved_word(connection: &mut Connection) -> i64 {
    let request = SaveRequest {
        operation: "create-review-word".into(),
        headword: "école".into(),
        notes: "my notes".into(),
        id: None,
        revision: None,
    };
    let snapshot = Snapshot {
        reference: Reference {
            dictionary: "FreeDict".into(),
            lookup_key: "école".into(),
            version: "1".into(),
            attribution: "source".into(),
            license: "CC-BY-SA-3.0".into(),
            source_url: "https://example.org".into(),
            license_url: "https://example.org".into(),
            sha256: "a".repeat(64),
        },
        digest: vec![DigestEntry {
            pronunciations: vec![],
            grammar: vec![],
            translation: Some("学校".into()),
        }],
    };
    save(connection, &request, Some(snapshot))
        .unwrap()
        .record
        .id
}

#[test]
fn undo_rating_restores_queue_and_can_be_retried() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = saved_word(&mut connection);
    let cards = add_review(&mut connection, word).unwrap();
    let before = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "undo-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    lexique_storage::undo_rating(&mut connection, "undo-1", "undo-rating").unwrap();
    lexique_storage::undo_rating(&mut connection, "undo-1", "undo-rating").unwrap();
    let retry = lexique_storage::RatingRequest {
        operation: "undo-rating".into(),
        card_id: cards[0].id,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    assert!(lexique_storage::rate_card(&mut connection, &retry).is_err());
    let progress = lexique_storage::review_progress(&connection, cards[0].id).unwrap();
    assert_eq!(progress.card.phase, lexique_core::scheduling::Phase::New);
    let after = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(after.cards, before.cards);
    let next = lexique_storage::RatingRequest {
        operation: "after-undo".into(),
        card_id: cards[0].id,
        expected_revision: progress.revision,
        rating: Rating::Easy,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    lexique_storage::rate_card(&mut connection, &next).unwrap();
    lexique_storage::undo_rating(&mut connection, "undo-1", "undo-rating").unwrap();
    assert_eq!(
        lexique_storage::undo_target(&connection)
            .unwrap()
            .as_deref(),
        Some("after-undo")
    );
    assert!(
        lexique_storage::undo_rating(&mut connection, "different-undo", "undo-rating").is_err()
    );
}

#[test]
fn undo_rating_outside_queue_keeps_independent_settings() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = saved_word(&mut connection);
    let cards = add_review(&mut connection, word).unwrap();
    lexique_storage::review_queue_filtered(&mut connection, 1_700_000_000, "UTC", |card| {
        Ok(card.id != cards[0].id)
    })
    .unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "direct-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let mut settings = lexique_storage::review_settings(&connection).unwrap();
    settings.new_limit = 1;
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    lexique_storage::undo_rating(&mut connection, "direct-undo", "direct-rating").unwrap();
    assert_eq!(
        lexique_storage::review_settings(&connection)
            .unwrap()
            .new_limit,
        1
    );
    assert_eq!(
        lexique_storage::review_progress(&connection, cards[0].id)
            .unwrap()
            .card
            .phase,
        lexique_core::scheduling::Phase::New
    );
}

#[test]
fn undo_restores_queue_after_timezone_change_within_learning_day() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = saved_word(&mut connection);
    let cards = add_review(&mut connection, word).unwrap();
    let before = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "timezone-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "Europe/Paris".into(),
        },
    )
    .unwrap();
    lexique_storage::undo_rating(&mut connection, "timezone-undo", "timezone-rating").unwrap();
    let after =
        lexique_storage::review_queue(&mut connection, 1_700_000_000, "Europe/Paris").unwrap();
    assert_eq!(after.cards, before.cards);
}

#[test]
fn undo_prior_learning_day_does_not_change_current_day_quota_after_travel() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = saved_word(&mut connection);
    let cards = add_review(&mut connection, first).unwrap();
    connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES ('second', '{}', '', 1, 1)", []).unwrap();
    let second_id = connection.last_insert_rowid();
    let second = add_review(&mut connection, second_id).unwrap();
    lexique_storage::set_review_settings(
        &connection,
        &lexique_storage::ReviewSettings {
            retention_percent: 90,
            new_limit: 1,
            review_limit: 50,
        },
    )
    .unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "travel-undo-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let before = lexique_storage::review_availability(
        &connection,
        second[0].id,
        1_700_000_000,
        "Asia/Tokyo",
    )
    .unwrap();
    assert_eq!(before.reason, None);
    lexique_storage::undo_rating(&mut connection, "travel-undo", "travel-undo-rating").unwrap();
    let after = lexique_storage::review_availability(
        &connection,
        second[0].id,
        1_700_000_000,
        "Asia/Tokyo",
    )
    .unwrap();
    assert_eq!(after.reason, None);
}

#[test]
fn undo_restores_minute_and_long_term_states() {
    for (initial, name) in [(Rating::Good, "minute"), (Rating::Easy, "long")] {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let word = saved_word(&mut connection);
        let card = add_review(&mut connection, word).unwrap()[0].id;
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: format!("{name}-first"),
                card_id: card,
                expected_revision: 0,
                rating: initial,
                now: 1_700_000_000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        let before = lexique_storage::review_progress(&connection, card).unwrap();
        let due = before.card.due_at.unwrap();
        lexique_storage::review_queue(&mut connection, due, "UTC").unwrap();
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: format!("{name}-second"),
                card_id: card,
                expected_revision: before.revision,
                rating: Rating::Good,
                now: due,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        lexique_storage::undo_rating(
            &mut connection,
            &format!("{name}-undo"),
            &format!("{name}-second"),
        )
        .unwrap();
        let restored = lexique_storage::review_progress(&connection, card).unwrap();
        assert_eq!(
            serde_json::to_value(restored.card).unwrap(),
            serde_json::to_value(before.card).unwrap()
        );
    }
}

#[test]
fn undo_legacy_position_only_snapshot_rebuilds_queue_and_reports_retry() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = saved_word(&mut connection);
    let card = add_review(&mut connection, word).unwrap()[0].id;
    lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "legacy-rating".into(),
            card_id: card,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    connection.execute("UPDATE review_ratings SET queue_before = json_remove(queue_before, '$.ordered_cards') WHERE operation = 'legacy-rating'", []).unwrap();
    assert!(lexique_storage::undo_rating(&mut connection, "legacy-undo", "legacy-rating").unwrap());
    assert!(lexique_storage::undo_rating(&mut connection, "legacy-undo", "legacy-rating").unwrap());
    let after = lexique_storage::review_progress(&connection, card).unwrap();
    assert_eq!(after.card.phase, lexique_core::scheduling::Phase::New);
    assert!(
        lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC")
            .unwrap()
            .cards
            .iter()
            .any(|entry| entry.id == card)
    );
}

#[test]
fn legacy_schema_upgrade_undo_retry_survives_reopening() {
    let path = std::env::temp_dir().join(format!(
        "lexique-legacy-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let word = saved_word(&mut connection);
        let card = add_review(&mut connection, word).unwrap()[0].id;
        lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: "legacy-file".into(),
                card_id: card,
                expected_revision: 0,
                rating: Rating::Good,
                now: 1_700_000_000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        connection.execute("UPDATE review_ratings SET queue_before = json_remove(queue_before, '$.ordered_cards')", []).unwrap();
        connection.execute_batch("DROP INDEX review_ratings_undo_operation; ALTER TABLE review_ratings DROP COLUMN undo_operation; ALTER TABLE review_ratings DROP COLUMN undo_queue_rebuilt; PRAGMA user_version = 5;").unwrap();
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        lexique_storage::review_queue(&mut connection, 1_700_086_400, "UTC").unwrap();
        assert!(
            lexique_storage::undo_rating(&mut connection, "legacy-file-undo", "legacy-file")
                .unwrap()
        );
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert!(
            lexique_storage::undo_rating(&mut connection, "legacy-file-undo", "legacy-file")
                .unwrap()
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn undo_rejects_invalid_saved_card_state_without_changing_progress() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let word = saved_word(&mut connection);
    let card = add_review(&mut connection, word).unwrap()[0].id;
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "invalid-before".into(),
            card_id: card,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    connection.execute("UPDATE review_ratings SET before_state = '{\"phase\":\"review\",\"memory\":null,\"last_review_at\":null,\"due_at\":null}' WHERE operation = 'invalid-before'", []).unwrap();
    let before = lexique_storage::review_progress(&connection, card).unwrap();
    assert!(
        lexique_storage::undo_rating(&mut connection, "invalid-undo", "invalid-before").is_err()
    );
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(lexique_storage::review_progress(&connection, card).unwrap()).unwrap()
    );
}

#[test]
fn undo_survives_reopening_and_failed_commit_across_learning_days() {
    let path = std::env::temp_dir().join(format!(
        "lexique-undo-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let card;
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let word = saved_word(&mut connection);
        card = add_review(&mut connection, word).unwrap()[0].id;
        lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: "persist-rating".into(),
                card_id: card,
                expected_revision: 0,
                rating: Rating::Good,
                now: 1_700_000_000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(
            lexique_storage::undo_target(&connection)
                .unwrap()
                .as_deref(),
            Some("persist-rating")
        );
        lexique_storage::review_queue(&mut connection, 1_700_086_400, "UTC").unwrap();
        connection.commit_hook(Some(|| true)).unwrap();
        assert!(
            lexique_storage::undo_rating(&mut connection, "persist-undo", "persist-rating")
                .is_err()
        );
        connection.commit_hook(None::<fn() -> bool>).unwrap();
        assert_eq!(
            lexique_storage::review_progress(&connection, card)
                .unwrap()
                .card
                .phase,
            lexique_core::scheduling::Phase::LearningLast
        );
        lexique_storage::undo_rating(&mut connection, "persist-undo", "persist-rating").unwrap();
        assert_eq!(
            lexique_storage::review_progress(&connection, card)
                .unwrap()
                .card
                .phase,
            lexique_core::scheduling::Phase::New
        );
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(lexique_storage::undo_target(&connection).unwrap(), None);
        lexique_storage::undo_rating(&mut connection, "persist-undo", "persist-rating").unwrap();
        assert!(
            lexique_storage::review_queue(&mut connection, 1_700_086_400, "UTC")
                .unwrap()
                .cards
                .iter()
                .any(|entry| entry.id == card)
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn queue_preserves_order_and_defers_a_rated_cards_sibling() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = saved_word(&mut connection);
    let first_cards = add_review(&mut connection, first).unwrap();
    connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES ('second', '{}', '', 1, 1)", []).unwrap();
    let second = connection.last_insert_rowid();
    let second_cards = add_review(&mut connection, second).unwrap();
    let before = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(
        before.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
        vec![
            first_cards[0].id,
            first_cards[1].id,
            second_cards[0].id,
            second_cards[1].id
        ]
    );
    let repeated = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(before.cards, repeated.cards);
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "queue-first".into(),
            card_id: first_cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let saved: String = connection
        .query_row(
            "SELECT queue_before FROM review_ratings WHERE operation = 'queue-first'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        snapshot["ordered_cards"],
        serde_json::json!(before.cards.iter().map(|card| card.id).collect::<Vec<_>>())
    );
    let after = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(
        after.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
        vec![second_cards[0].id, second_cards[1].id]
    );
}

#[test]
fn queue_limits_admission_without_forgetting_original_order() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let mut settings = lexique_storage::review_settings(&connection).unwrap();
    settings.new_limit = 1;
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    let queue = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(queue.cards.len(), 1);
    assert_eq!(queue.cards[0].id, cards[0].id);
    settings.new_limit = 2;
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    let expanded = lexique_storage::review_queue(&mut connection, 1_700_000_000, "UTC").unwrap();
    assert_eq!(
        expanded
            .cards
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        vec![cards[0].id, cards[1].id]
    );
}

#[test]
fn queue_excludes_unavailable_audio_before_using_quota() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let mut settings = lexique_storage::review_settings(&connection).unwrap();
    settings.new_limit = 1;
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    let queue =
        lexique_storage::review_queue_filtered(&mut connection, 1_700_000_000, "UTC", |card| {
            Ok(card.kind != "recognition")
        })
        .unwrap();
    assert_eq!(queue.cards.len(), 1);
    assert_eq!(queue.cards[0].id, cards[1].id);
}

#[test]
fn mixed_queue_uses_admitted_counts_and_survives_reopening() {
    let path = std::env::temp_dir().join(format!(
        "lexique-queue-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let expected;
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let id = saved_word(&mut connection);
        let cards = add_review(&mut connection, id).unwrap();
        lexique_storage::rate_card(
            &mut connection,
            &lexique_storage::RatingRequest {
                operation: "mixed-old".into(),
                card_id: cards[0].id,
                expected_revision: 0,
                rating: Rating::Easy,
                now: 1_700_000_000,
                timezone: "UTC".into(),
            },
        )
        .unwrap();
        for number in 0..3 {
            connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES (?1, '{}', '', 1, 1)", [format!("mixed-{number}")]).unwrap();
            let id = connection.last_insert_rowid();
            add_review(&mut connection, id).unwrap();
        }
        let settings = lexique_storage::ReviewSettings {
            retention_percent: 90,
            new_limit: 2,
            review_limit: 1,
        };
        lexique_storage::set_review_settings(&connection, &settings).unwrap();
        let queue = lexique_storage::review_queue(&mut connection, 1_702_592_000, "UTC").unwrap();
        expected = queue.cards.iter().map(|card| card.id).collect::<Vec<_>>();
        assert_eq!(expected.len(), 3);
        assert_eq!(expected[1], cards[0].id);
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let queue = lexique_storage::review_queue(&mut connection, 1_702_592_000, "UTC").unwrap();
        assert_eq!(
            queue.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
            expected
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn minute_learning_takes_priority_and_daily_quota_resets() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let settings = lexique_storage::ReviewSettings {
        retention_percent: 90,
        new_limit: 1,
        review_limit: 1,
    };
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "minute-first".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Again,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let waiting = lexique_storage::review_queue(&mut connection, 1_700_000_030, "UTC").unwrap();
    assert!(waiting.cards.is_empty());
    assert_eq!(waiting.reason.as_deref(), Some("due"));
    assert_eq!(waiting.available_at, Some(1_700_000_060));
    let due = lexique_storage::review_queue(&mut connection, 1_700_000_060, "UTC").unwrap();
    assert_eq!(due.cards[0].id, cards[0].id);
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "minute-repeat".into(),
            card_id: cards[0].id,
            expected_revision: 1,
            rating: Rating::Again,
            now: 1_700_000_060,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let exhausted = lexique_storage::review_queue(&mut connection, 1_700_000_120, "UTC").unwrap();
    assert!(exhausted.cards.is_empty());
    assert_eq!(exhausted.reason.as_deref(), Some("quota"));
    let tomorrow = lexique_storage::review_queue(&mut connection, 1_700_086_400, "UTC").unwrap();
    assert_eq!(tomorrow.cards[0].id, cards[0].id);
    assert!(tomorrow.cards.iter().any(|card| card.id == cards[1].id));
}

#[test]
fn raising_review_limit_interleaves_newly_admitted_cards() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let old = add_review(&mut connection, id).unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "expand-old".into(),
            card_id: old[0].id,
            expected_revision: 0,
            rating: Rating::Easy,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let mut settings = lexique_storage::ReviewSettings {
        retention_percent: 90,
        new_limit: 2,
        review_limit: 0,
    };
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    let before = lexique_storage::review_queue(&mut connection, 1_702_592_000, "UTC").unwrap();
    assert_eq!(before.cards.len(), 1);
    connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES ('expand-new', '{}', '', 1, 1)", []).unwrap();
    let id = connection.last_insert_rowid();
    add_review(&mut connection, id).unwrap();
    let before = lexique_storage::review_queue(&mut connection, 1_702_592_000, "UTC").unwrap();
    assert_eq!(before.cards.len(), 2);
    settings.review_limit = 1;
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    let after = lexique_storage::review_queue(&mut connection, 1_702_592_000, "UTC").unwrap();
    assert_eq!(
        after.cards.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![before.cards[0].id, old[0].id, before.cards[1].id]
    );
}

#[test]
fn review_settings_control_future_ratings_and_preserve_daily_usage() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let defaults = lexique_storage::review_settings(&connection).unwrap();
    assert_eq!(
        (
            defaults.retention_percent,
            defaults.new_limit,
            defaults.review_limit
        ),
        (90, 20, 50)
    );
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let request = lexique_storage::RatingRequest {
        operation: "settings-first".into(),
        card_id: cards[0].id,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    lexique_storage::rate_card(&mut connection, &request).unwrap();
    let settings = lexique_storage::ReviewSettings {
        retention_percent: 80,
        new_limit: 0,
        review_limit: 0,
    };
    lexique_storage::set_review_settings(&connection, &settings).unwrap();
    assert_eq!(
        lexique_storage::review_settings(&connection).unwrap(),
        settings
    );
    assert_eq!(
        lexique_storage::review_availability(&connection, cards[0].id, request.now + 600, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    let next = lexique_storage::RatingRequest {
        operation: "settings-next".into(),
        expected_revision: 1,
        now: request.now + 600,
        ..request
    };
    assert!(lexique_storage::rate_card(&mut connection, &next).is_err());
    let raised = lexique_storage::ReviewSettings {
        review_limit: 1,
        ..settings
    };
    lexique_storage::set_review_settings(&connection, &raised).unwrap();
    let scheduled = lexique_storage::rate_card(&mut connection, &next).unwrap();
    assert_eq!(scheduled["retention_percent"], 80);
    let replayed = lexique_storage::rate_card(&mut connection, &next).unwrap();
    assert_eq!(replayed, scheduled);
    let invalid = lexique_storage::ReviewSettings {
        retention_percent: 69,
        ..raised
    };
    assert!(lexique_storage::set_review_settings(&connection, &invalid).is_err());
    assert_eq!(
        lexique_storage::review_settings(&connection).unwrap(),
        raised
    );
}

#[test]
fn rating_a_card_is_persistent_and_retry_is_idempotent() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let request = lexique_storage::RatingRequest {
        operation: "rating-1".into(),
        card_id: cards[0].id,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    let first = lexique_storage::rate_card(&mut connection, &request).unwrap();
    assert_eq!(first["card"]["due_at"], 1_700_000_600);
    let progress = lexique_storage::review_progress(&connection, cards[0].id).unwrap();
    assert_eq!(progress.revision, 1);
    assert_eq!(progress.card.due_at, Some(1_700_000_600));
    let retry = lexique_storage::rate_card(&mut connection, &request).unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(retry).unwrap()
    );
    let delayed = lexique_storage::RatingRequest {
        now: request.now + 100,
        ..request
    };
    assert!(lexique_storage::rate_card(&mut connection, &delayed).is_ok());
    let stale = lexique_storage::RatingRequest {
        operation: "rating-2".into(),
        ..delayed
    };
    assert!(lexique_storage::rate_card(&mut connection, &stale).is_err());
}

#[test]
fn failed_rating_commit_can_be_retried_without_advancing_progress() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let card = add_review(&mut connection, id).unwrap().remove(0);
    let request = lexique_storage::RatingRequest {
        operation: "failed-rating".into(),
        card_id: card.id,
        expected_revision: 0,
        rating: Rating::Again,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(lexique_storage::rate_card(&mut connection, &request).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    let result = lexique_storage::rate_card(&mut connection, &request).unwrap();
    assert_eq!(result["card"]["due_at"], 1_700_000_060);
}

#[test]
fn rating_defers_the_other_new_card_until_the_next_learning_day() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let first = lexique_storage::RatingRequest {
        operation: "first".into(),
        card_id: cards[0].id,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    lexique_storage::rate_card(&mut connection, &first).unwrap();
    let availability =
        lexique_storage::review_availability(&connection, cards[1].id, first.now, "UTC").unwrap();
    assert_eq!(availability.reason.as_deref(), Some("deferred"));
    assert!(availability.available_at.unwrap() > first.now);
    let sibling = lexique_storage::RatingRequest {
        operation: "sibling".into(),
        card_id: cards[1].id,
        ..first
    };
    assert!(lexique_storage::rate_card(&mut connection, &sibling).is_err());
    let tomorrow = lexique_storage::RatingRequest {
        now: 1_700_086_400,
        ..sibling
    };
    assert!(lexique_storage::rate_card(&mut connection, &tomorrow).is_ok());
}

#[test]
fn sibling_deferral_uses_current_timezone_after_travel() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let cards = add_review(&mut connection, id).unwrap();
    let first = lexique_storage::RatingRequest {
        operation: "travel-first".into(),
        card_id: cards[0].id,
        expected_revision: 0,
        rating: Rating::Good,
        now: 1_700_000_000,
        timezone: "UTC".into(),
    };
    lexique_storage::rate_card(&mut connection, &first).unwrap();
    let second = lexique_storage::RatingRequest {
        operation: "travel-second".into(),
        card_id: cards[1].id,
        timezone: "Asia/Tokyo".into(),
        ..first
    };
    assert!(lexique_storage::rate_card(&mut connection, &second).is_err());
}

#[test]
fn new_card_limit_counts_successful_ratings_once_per_learning_day() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let mut last = None;
    for number in 0..21 {
        connection.execute("INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at) VALUES (?1, '{}', '', 1, 1)", [format!("word-{number}")]).unwrap();
        let id = connection.last_insert_rowid();
        let card = add_review(&mut connection, id).unwrap().remove(0);
        let request = lexique_storage::RatingRequest {
            operation: format!("quota-{number}"),
            card_id: card.id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        };
        if number < 20 {
            assert!(lexique_storage::rate_card(&mut connection, &request).is_ok());
            assert!(lexique_storage::rate_card(&mut connection, &request).is_ok());
        } else {
            last = Some(request);
        }
    }
    let request = last.unwrap();
    assert!(lexique_storage::rate_card(&mut connection, &request).is_err());
    let tomorrow = lexique_storage::RatingRequest {
        now: request.now + 86_400,
        ..request
    };
    assert!(lexique_storage::rate_card(&mut connection, &tomorrow).is_ok());
}

#[test]
fn minute_step_ratings_use_the_separate_review_limit() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    let card = add_review(&mut connection, id).unwrap().remove(0);
    for revision in 0..52 {
        let request = lexique_storage::RatingRequest {
            operation: format!("repeat-{revision}"),
            card_id: card.id,
            expected_revision: revision,
            rating: Rating::Again,
            now: 1_700_000_000 + revision * 60,
            timezone: "UTC".into(),
        };
        assert_eq!(
            lexique_storage::rate_card(&mut connection, &request).is_ok(),
            revision <= 50
        );
    }
}

#[test]
fn adding_a_saved_word_creates_two_cards_once_without_changing_the_record() {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../migrations/0002_vocabulary.sql"))
        .unwrap();
    let id = saved_word(&mut connection);
    let original = lexique_storage::get(&connection, id).unwrap();
    assert_eq!(initialize(&connection).unwrap(), 6);
    assert!(review_cards(&connection, id).unwrap().is_empty());
    let first = add_review(&mut connection, id).unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(
        first
            .iter()
            .map(|card| card.kind.as_str())
            .collect::<Vec<_>>(),
        ["recognition", "listening"]
    );
    let retry = add_review(&mut connection, id).unwrap();
    assert_eq!(first, retry);
    assert_eq!(
        serde_json::to_value(lexique_storage::get(&connection, id).unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
}

#[test]
fn adding_an_unknown_word_fails_without_creating_cards() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    assert!(add_review(&mut connection, 123).is_err());
    assert!(review_cards(&connection, 123).unwrap().is_empty());
}

#[test]
fn failed_commit_leaves_no_cards_and_retry_creates_both() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let id = saved_word(&mut connection);
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(add_review(&mut connection, id).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(review_cards(&connection, id).unwrap().is_empty());
    assert_eq!(add_review(&mut connection, id).unwrap().len(), 2);
}

#[test]
fn cards_keep_their_identity_after_reopening_the_database() {
    let path = std::env::temp_dir().join(format!(
        "lexique-review-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let cards;
    let id;
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        id = saved_word(&mut connection);
        cards = add_review(&mut connection, id).unwrap();
        let request = lexique_storage::RatingRequest {
            operation: "reopen-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        };
        lexique_storage::rate_card(&mut connection, &request).unwrap();
    }
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        assert_eq!(review_cards(&connection, id).unwrap(), cards);
        assert_eq!(add_review(&mut connection, id).unwrap(), cards);
        let progress = lexique_storage::review_progress(&connection, cards[0].id).unwrap();
        assert_eq!(progress.revision, 1);
        assert_eq!(progress.card.due_at, Some(1_700_000_600));
        let sibling = lexique_storage::RatingRequest {
            operation: "reopen-sibling".into(),
            card_id: cards[1].id,
            expected_revision: 0,
            rating: Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        };
        assert!(lexique_storage::rate_card(&mut connection, &sibling).is_err());
    }
    std::fs::remove_file(path).unwrap();
}
