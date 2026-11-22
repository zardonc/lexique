//! V03 isolated candidate. Product callers and schema remain unchanged.
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use candidate as review;
pub use candidate::{QueueAudioMetadata, QueueCard, ReviewProgress};
pub use lexique_storage::{ReviewScope, StorageError, review_scope};
#[path = "../src/queue_cache.rs"]
mod queue_cache;
pub use queue_cache::*;
use rusqlite::{Connection, TransactionBehavior, params};

#[allow(dead_code)]
mod candidate {
    include!("../src/review.rs");

    type BatchOutcome = (i64, Result<Vec<ReviewCard>, String>);

    pub fn batch(
        connection: &mut Connection,
        ids: &[i64],
    ) -> Result<Vec<BatchOutcome>, StorageError> {
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        batch_transaction(tx, ids)
    }

    pub fn batch_transaction(
        tx: rusqlite::Transaction<'_>,
        ids: &[i64],
    ) -> Result<Vec<BatchOutcome>, StorageError> {
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

fn default_database_parent() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/issue-24")
}

fn database_root(parent: &Path, nonce: u128) -> PathBuf {
    static NEXT_DATABASE_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        "preflight-sqlite-{}-{nonce}-{id}",
        std::process::id()
    ))
}

#[test]
fn database_roots_stay_unique_when_clock_values_repeat() {
    let parent = default_database_parent();
    assert_ne!(database_root(&parent, 0), database_root(&parent, 0));
}

fn database(count: i64) -> (PathBuf, Connection) {
    database_at(&default_database_parent(), count)
}

fn database_at(parent: &Path, count: i64) -> (PathBuf, Connection) {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = database_root(parent, nonce);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("test.sqlite3");
    let mut connection = Connection::open(&path).unwrap();
    lexique_storage::initialize(&connection).unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"synthetic","version":"1","attribution":"synthetic","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    let tx = connection.transaction().unwrap();
    for id in 1..=count {
        tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'original',1000,1000)", params![id,format!("synthetic-{id}"),snapshot]).unwrap();
    }
    tx.commit().unwrap();
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
    let parent = std::env::var_os("V03_TIMING_DB_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(default_database_parent);
    for size in [20, 50, 100] {
        let mut contended_samples = 0;
        for repeat in 0..10 {
            let (path, mut connection) = database_at(&parent, size + 1);
            let existing = lexique_storage::add_review(&mut connection, size + 1).unwrap();
            let barrier = std::sync::Barrier::new(2);
            let contended = std::thread::scope(|scope| {
                let writer_ready = &barrier;
                // A buffered signal keeps rendezvous waits out of the timed transaction.
                let (start_writer, writer_start) = std::sync::mpsc::channel();
                let writer = scope.spawn(move || {
                    let mut other = Connection::open(&path).unwrap();
                    lexique_storage::initialize(&other).unwrap();
                    other.busy_timeout(std::time::Duration::ZERO).unwrap();
                    writer_ready.wait();
                    writer_start.recv().unwrap();
                    let start = Instant::now();
                    let note_result = other.execute(
                        "UPDATE vocabulary SET notes=?1 WHERE id=?2",
                        params!["concurrent note", size + 1],
                    );
                    let contended = note_result.is_err();
                    if let Err(error) = note_result {
                        assert_eq!(
                            error.sqlite_error_code(),
                            Some(rusqlite::ErrorCode::DatabaseBusy),
                            "{error}"
                        );
                    }
                    other
                        .busy_timeout(std::time::Duration::from_millis(250))
                        .unwrap();
                    if contended {
                        let note_result = other.execute(
                            "UPDATE vocabulary SET notes=?1 WHERE id=?2",
                            params!["concurrent note", size + 1],
                        );
                        if let Err(error) = &note_result {
                            eprintln!(
                                "V03_WRITE_FAILURE size={size} repeat={repeat} writer_ms={} error={error:?}",
                                start.elapsed().as_secs_f64() * 1000.0
                            );
                        }
                        note_result.unwrap();
                    }
                    let note_ms = start.elapsed().as_secs_f64() * 1000.0;
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
                    (start.elapsed().as_secs_f64() * 1000.0, contended, note_ms)
                });
                barrier.wait();
                let tx = connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)
                    .unwrap();
                let start = Instant::now();
                start_writer.send(()).unwrap();
                assert!(
                    candidate::batch_transaction(tx, &(1..=size).collect::<Vec<_>>())
                        .unwrap()
                        .iter()
                        .all(|(_, r)| r.is_ok())
                );
                let transaction_ms = start.elapsed().as_secs_f64() * 1000.0;
                let (writer_ms, contended, note_ms) = writer.join().unwrap_or_else(|panic| {
                    eprintln!("V03_TX size={size} repeat={repeat} transaction_ms={transaction_ms}");
                    std::panic::resume_unwind(panic)
                });
                println!(
                    "V03_PHASES size={size} repeat={repeat} note_ms={note_ms} rating_ms={}",
                    writer_ms - note_ms
                );
                println!(
                    "V03 {{\"size\":{size},\"repeat\":{repeat},\"transaction_ms\":{transaction_ms},\"writer_ms\":{writer_ms},\"contended\":{contended}}}"
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
                contended
            });
            if contended {
                contended_samples += 1;
                if contended_samples == 3 {
                    break;
                }
            }
        }
        assert_eq!(
            contended_samples, 3,
            "insufficient write contention samples"
        );
    }
}
