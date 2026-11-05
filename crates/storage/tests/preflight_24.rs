//! V03 isolated candidate. Product callers and schema remain unchanged.
use std::{path::PathBuf, time::Instant};

use candidate as review;
pub use candidate::{QueueAudioMetadata, QueueCard, ReviewProgress};
pub use lexique_storage::StorageError;
#[path = "../src/queue_cache.rs"]
mod queue_cache;
pub use queue_cache::*;
use rusqlite::{Connection, params};

#[allow(dead_code)]
mod candidate {
    include!("../src/review.rs");

    pub fn batch(
        connection: &mut Connection,
        ids: &[i64],
    ) -> Result<Vec<(i64, Result<Vec<ReviewCard>, String>)>, StorageError> {
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut outcomes = vec![];
        for &id in ids {
            tx.execute_batch("SAVEPOINT word")?;
            let result = if id <= 0 || id > 9_007_199_254_740_991 {
                Err(StorageError::Invalid("Invalid vocabulary ID".into()))
            } else {
                ensure_review_cards(&tx, id).and_then(|()| review_cards(&tx, id))
            };
            match result {
                Ok(cards) => outcomes.push((id, Ok(cards))),
                Err(error) => {
                    tx.execute_batch("ROLLBACK TO word")?;
                    outcomes.push((id, Err(error.to_string())));
                }
            }
            tx.execute_batch("RELEASE word")?;
        }
        tx.commit()?;
        Ok(outcomes)
    }
}

fn database(count: i64) -> (PathBuf, Connection) {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/issue-24")
        .join(format!("preflight-sqlite-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("test.sqlite3");
    let connection = Connection::open(&path).unwrap();
    lexique_storage::initialize(&connection).unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"synthetic","version":"1","attribution":"synthetic","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    for id in 1..=count {
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'original',1000,1000)", params![id,format!("synthetic-{id}"),snapshot]).unwrap();
    }
    (path, connection)
}

#[test]
fn v03_unique_identity_partial_failures_progress_pause_and_replay() {
    let (_, mut connection) = database(3);
    let cards = lexique_storage::add_review(&mut connection, 1).unwrap();
    lexique_storage::rate_card(
        &mut connection,
        &lexique_storage::RatingRequest {
            operation: "preflight-rating".into(),
            card_id: cards[0].id,
            expected_revision: 0,
            rating: lexique_core::scheduling::Rating::Good,
            now: 1_700_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    lexique_storage::set_review_paused(&mut connection, 1, true).unwrap();
    let progress = lexique_storage::review_progress(&connection, cards[0].id).unwrap();
    let inputs = [1, 2, 999, -1, 0, 9_007_199_254_740_992, 2, 3];
    let outcomes = candidate::batch(&mut connection, &inputs).unwrap();
    assert_eq!(outcomes.iter().filter(|(_, r)| r.is_ok()).count(), 4);
    assert_eq!(
        lexique_storage::review_cards(&connection, 1).unwrap(),
        cards
    );
    assert_eq!(
        serde_json::to_value(lexique_storage::review_progress(&connection, cards[0].id).unwrap())
            .unwrap(),
        serde_json::to_value(progress).unwrap()
    );
    assert!(lexique_storage::review_paused(&connection, 1).unwrap());
    assert_eq!(
        lexique_storage::get(&connection, 1).unwrap().notes,
        "original"
    );
    assert_eq!(
        candidate::batch(&mut connection, &inputs).unwrap(),
        outcomes
    );
    for id in 2..=3 {
        let cards = lexique_storage::review_cards(&connection, id).unwrap();
        assert_eq!(cards.len(), 2);
        assert!(cards.iter().all(|card| {
            lexique_storage::review_progress(&connection, card.id)
                .unwrap()
                .card
                .due_at
                .is_none()
        }));
    }
}

#[test]
fn v03_per_word_failure_rolls_back_first_card_and_commit_failure_reports_no_success() {
    let (path, mut connection) = database(3);
    connection.execute_batch("CREATE TRIGGER fail_word BEFORE INSERT ON review_cards WHEN NEW.vocabulary_id=2 AND NEW.kind='listening' BEGIN SELECT RAISE(ABORT,'injected second card failure'); END").unwrap();
    let outcomes = candidate::batch(&mut connection, &[1, 2, 3]).unwrap();
    assert!(outcomes[0].1.is_ok() && outcomes[1].1.is_err() && outcomes[2].1.is_ok());
    assert!(
        lexique_storage::review_cards(&connection, 2)
            .unwrap()
            .is_empty()
    );
    connection.execute_batch("DROP TRIGGER fail_word").unwrap();
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(candidate::batch(&mut connection, &[2]).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(
        lexique_storage::review_cards(&connection, 2)
            .unwrap()
            .is_empty()
    );
    drop(connection);
    let mut reopened = Connection::open(path).unwrap();
    assert_eq!(
        candidate::batch(&mut reopened, &[2]).unwrap()[0]
            .1
            .as_ref()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn v03_candidate_transaction_sizes_and_competing_notes_and_ratings() {
    for size in [20, 50, 100] {
        for repeat in 0..3 {
            let (path, mut connection) = database(size + 1);
            let existing = lexique_storage::add_review(&mut connection, size + 1).unwrap();
            let barrier = std::sync::Barrier::new(2);
            std::thread::scope(|scope| {
                let writer = scope.spawn(|| {
                    let mut other = Connection::open(&path).unwrap();
                    lexique_storage::initialize(&other).unwrap();
                    other
                        .busy_timeout(std::time::Duration::from_millis(250))
                        .unwrap();
                    barrier.wait();
                    let start = Instant::now();
                    other
                        .execute(
                            "UPDATE vocabulary SET notes=?1 WHERE id=?2",
                            params!["concurrent note", size + 1],
                        )
                        .unwrap();
                    lexique_storage::rate_card(
                        &mut other,
                        &lexique_storage::RatingRequest {
                            operation: format!("competing-{size}-{repeat}"),
                            card_id: existing[0].id,
                            expected_revision: 0,
                            rating: lexique_core::scheduling::Rating::Good,
                            now: 1_700_000_000,
                            timezone: "UTC".into(),
                        },
                    )
                    .unwrap();
                    start.elapsed().as_secs_f64() * 1000.0
                });
                barrier.wait();
                let start = Instant::now();
                assert!(
                    candidate::batch(&mut connection, &(1..=size).collect::<Vec<_>>())
                        .unwrap()
                        .iter()
                        .all(|(_, r)| r.is_ok())
                );
                let transaction_ms = start.elapsed().as_secs_f64() * 1000.0;
                let writer_ms = writer.join().unwrap();
                println!(
                    "V03 {{\"size\":{size},\"repeat\":{repeat},\"transaction_ms\":{transaction_ms},\"writer_ms\":{writer_ms}}}"
                );
                assert!(
                    transaction_ms <= 100.0,
                    "candidate exceeds pre-agreed lock budget"
                );
                assert!(
                    writer_ms <= 250.0,
                    "candidate exceeds pre-agreed interaction budget"
                );
                assert_eq!(
                    lexique_storage::get(&connection, size + 1).unwrap().notes,
                    "concurrent note"
                );
            });
        }
    }
}
