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
    assert_eq!(initialize(&connection).unwrap(), 4);
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
