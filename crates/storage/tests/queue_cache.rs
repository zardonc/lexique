use lexique_storage::*;
use rusqlite::Connection;

#[test]
fn warm_snapshot_statement_count_is_independent_of_collection_and_audio_readiness() {
    use rusqlite::trace::{TraceEvent, TraceEventCodes};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static READS: AtomicUsize = AtomicUsize::new(0);
    for mode in ["healthy", "missing-half", "missing-all", "invalid"] {
        let mut counts = Vec::new();
        for words in [100, 1000] {
            let mut connection = Connection::open_in_memory().unwrap();
            initialize(&connection).unwrap();
            let tx = connection.transaction().unwrap();
            let locator = "a".repeat(64);
            tx.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,created_at,physical_locator,context,sealed,verification,last_used) VALUES(?1,?1,4,1,?1,'word',1,'verified',1)", [&locator]).unwrap();
            for id in 1..=words {
                tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,'{}','',1,1)", rusqlite::params![id, format!("read-count-{id}")]).unwrap();
                for (card, kind) in [(id * 2 - 1, "recognition"), (id * 2, "listening")] {
                    tx.execute(
                        "INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?1,?2,?3)",
                        rusqlite::params![card, id, kind],
                    )
                    .unwrap();
                }
                if mode != "missing-all" && (mode != "missing-half" || id % 2 == 0) {
                    tx.execute("INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(?1,?2)", rusqlite::params![id, locator]).unwrap();
                }
            }
            if mode == "invalid" {
                tx.execute("UPDATE speech_versions SET verification='invalid'", [])
                    .unwrap();
            }
            tx.commit().unwrap();
            let mut cache = QueueCache::default();
            let skipped = Default::default();
            review_queue_snapshot(&connection, 1_700_000_000, "UTC", &skipped, &mut cache)
                .unwrap()
                .transaction
                .commit()
                .unwrap();
            READS.store(0, Ordering::SeqCst);
            connection.trace_v2(
                TraceEventCodes::SQLITE_TRACE_STMT,
                Some(|event| {
                    if let TraceEvent::Stmt(_, sql) = event
                        && (sql.starts_with("SELECT ") || sql.starts_with("PRAGMA "))
                    {
                        READS.fetch_add(1, Ordering::SeqCst);
                    }
                }),
            );
            {
                let snapshot =
                    review_queue_snapshot(&connection, 1_700_000_000, "UTC", &skipped, &mut cache)
                        .unwrap();
                assert!(!snapshot.metrics.rebuilt);
                assert_eq!(snapshot.metrics.loaded_rows, 0);
                snapshot.transaction.commit().unwrap();
            }
            connection.trace_v2(TraceEventCodes::empty(), None);
            counts.push(READS.load(Ordering::SeqCst));
        }
        assert!(counts[0] > 0);
        assert_eq!(
            counts[0], counts[1],
            "{mode}: warm snapshot must not issue per-card SQL"
        );
    }
}

#[test]
fn queue_revision_is_persistent_and_only_committed_changes_advance_it() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let exists = connection
        .prepare("SELECT revision,kind,card_id,vocabulary_id,locator FROM review_queue_changes");
    assert!(
        exists.is_ok(),
        "The queue needs a durable change journal, not connection-local data_version"
    );
    drop(exists);
    let initial = queue_revision(&connection).unwrap();
    connection
        .execute("UPDATE review_settings SET new_limit=new_limit", [])
        .unwrap();
    assert_eq!(queue_revision(&connection).unwrap(), initial);
    {
        let tx = connection.transaction().unwrap();
        tx.execute("UPDATE review_settings SET new_limit=new_limit+1", [])
            .unwrap();
        assert!(queue_revision(&tx).unwrap() > initial);
    }
    assert_eq!(
        queue_revision(&connection).unwrap(),
        initial,
        "rolled back changes have no revision"
    );
    connection
        .execute("UPDATE review_settings SET new_limit=new_limit+1", [])
        .unwrap();
    assert!(queue_revision(&connection).unwrap() > initial);
    assert_eq!(
        queue_changes_since(&connection, initial).unwrap()[0].kind,
        "settings"
    );
}

fn add_word(connection: &mut Connection, word: &str) -> Vec<QueueCard> {
    connection.execute("INSERT INTO vocabulary(headword,snapshot,notes,created_at,updated_at) VALUES(?1,'{}','',1,1)", [word]).unwrap();
    let vocabulary_id = connection.last_insert_rowid();
    add_review(connection, vocabulary_id)
        .unwrap()
        .into_iter()
        .map(|card| QueueCard {
            id: card.id,
            vocabulary_id,
            kind: card.kind,
        })
        .collect()
}

#[test]
fn a_committed_queue_snapshot_does_not_reload_its_own_positions() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    add_word(&mut connection, "école");
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    let (first, _, metrics) =
        review_queue_cached(&mut connection, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert!(metrics.rebuilt);
    let (warm, _, metrics) =
        review_queue_cached(&mut connection, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(warm.cards, first.cards);
    assert!(
        !metrics.rebuilt,
        "own committed position/context changes must not rebuild the next read"
    );
    assert_eq!(metrics.loaded_rows, 0);
    assert_eq!(metrics.parsed_progress, 0);
}

#[test]
fn progress_audio_and_context_changes_match_the_uncached_queue() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let cards = add_word(&mut connection, "école");
    add_word(&mut connection, "maison");
    let now = 1_700_000_000;
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    review_queue_cached(&mut connection, now, "UTC", &skipped, &mut cache).unwrap();
    review_queue_cached(&mut connection, now, "UTC", &skipped, &mut cache).unwrap();
    connection.execute("INSERT INTO review_progress VALUES(?1,1,?2)", rusqlite::params![cards[0].id, serde_json::json!({"phase":"learning_first","memory":null,"last_review_at":now-120,"due_at":now+60}).to_string()]).unwrap();
    let (queue, _, metrics) =
        review_queue_cached(&mut connection, now, "UTC", &skipped, &mut cache).unwrap();
    assert!(!metrics.rebuilt);
    assert_eq!(metrics.loaded_rows, 1);
    assert_eq!(metrics.parsed_progress, 1);
    let expected = review_queue_filtered(&mut connection, now, "UTC", |card| {
        Ok(card.kind != "listening")
    })
    .unwrap();
    assert_eq!(queue.cards, expected.cards);
    assert_eq!(queue.reason, expected.reason);
    assert_eq!(queue.available_at, expected.available_at);
    let version = prepare_audio_version(
        &connection,
        cards[1].vocabulary_id,
        None,
        lexique_storage::AudioPublicationContent {
            hash: &content_hash(b"OggSfixture"),
            bytes: 11,
            generated_at: None,
        },
        0,
        now,
    )
    .unwrap();
    complete_audio_version(&connection, &version.locator, now).unwrap();
    let (queue, metadata, metrics) =
        review_queue_cached(&mut connection, now, "UTC", &skipped, &mut cache).unwrap();
    assert!(!metrics.rebuilt);
    assert_eq!(metrics.parsed_progress, 0);
    assert!(metadata.ready.contains(&cards[1].vocabulary_id));
    assert!(queue.cards.iter().any(|card| card.id == cards[1].id));
    mark_audio_unavailable(&connection, &version.locator, "missing").unwrap();
    let (queue, metadata, _) =
        review_queue_cached(&mut connection, now, "UTC", &skipped, &mut cache).unwrap();
    assert!(!metadata.ready.contains(&cards[1].vocabulary_id));
    assert!(!queue.cards.iter().any(|card| card.id == cards[1].id));
    let (queue, _, metrics) =
        review_queue_cached(&mut connection, now + 60, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(
        metrics.parsed_progress, 0,
        "time wakes reuse decoded authoritative progress"
    );
    assert!(queue.cards.iter().any(|card| card.id == cards[0].id));
}

#[test]
fn a_second_connection_and_reopen_keep_committed_revisions_visible() {
    let path = std::env::temp_dir().join(format!(
        "lexique-revision-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut reader = Connection::open(&path).unwrap();
    initialize(&reader).unwrap();
    let cards = add_word(&mut reader, "école");
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    review_queue_cached(&mut reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    let before = queue_revision(&reader).unwrap();
    let writer = Connection::open(&path).unwrap();
    writer
        .execute("UPDATE review_settings SET new_limit=0", [])
        .unwrap();
    let (queue, _, metrics) =
        review_queue_cached(&mut reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert!(queue.cards.is_empty());
    assert_eq!(metrics.parsed_progress, 0);
    assert!(cache.revision() > before);
    let committed = cache.revision();
    drop(writer);
    drop(reader);
    let mut reader = Connection::open(&path).unwrap();
    initialize(&reader).unwrap();
    assert_eq!(queue_revision(&reader).unwrap(), committed);
    assert_eq!(
        queue_changes_since(&reader, before).unwrap()[0].kind,
        "settings"
    );
    let tx = reader.transaction().unwrap();
    tx.execute(
        "DELETE FROM review_queue_entries WHERE card_id=?1",
        [cards[0].id],
    )
    .unwrap();
    tx.execute("DELETE FROM review_cards WHERE id=?1", [cards[0].id])
        .unwrap();
    tx.commit().unwrap();
    let (_, _, metrics) =
        review_queue_cached(&mut reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(metrics.reason.as_deref(), Some("collection"));
    drop(reader);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_pruned_revision_gap_rebuilds_once_instead_of_skipping_changes() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    add_word(&mut connection, "école");
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    review_queue_cached(&mut connection, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    let tx = connection.transaction().unwrap();
    for _ in 0..1025 {
        tx.execute("UPDATE review_settings SET new_limit=new_limit+1", [])
            .unwrap();
    }
    tx.commit().unwrap();
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM review_queue_changes", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1024);
    let (_, _, metrics) =
        review_queue_cached(&mut connection, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert_eq!(metrics.reason.as_deref(), Some("revision_gap"));
    let (_, _, metrics) =
        review_queue_cached(&mut connection, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
    assert!(!metrics.rebuilt);
    assert_eq!(metrics.loaded_rows, 0);
}

#[test]
fn a_queue_snapshot_keeps_admission_records_and_revision_coherent_during_repair_commits() {
    let path = std::env::temp_dir().join(format!(
        "lexique-coherent-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut reader = Connection::open(&path).unwrap();
    initialize(&reader).unwrap();
    reader.pragma_update(None, "journal_mode", "WAL").unwrap();
    let cards = add_word(&mut reader, "école");
    let version = prepare_audio_version(
        &reader,
        1,
        None,
        lexique_storage::AudioPublicationContent {
            hash: &content_hash(b"OggS"),
            bytes: 4,
            generated_at: None,
        },
        0,
        1_700_000_000,
    )
    .unwrap();
    complete_audio_version(&reader, &version.locator, 1_700_000_000).unwrap();
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    {
        let warm =
            review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
        warm.transaction.commit().unwrap();
        let snapshot =
            review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
        let revision = cache.revision();
        let writer = Connection::open(&path).unwrap();
        writer.execute_batch("BEGIN IMMEDIATE; UPDATE speech_versions SET verification='missing'; UPDATE review_settings SET new_limit=0; UPDATE vocabulary SET notes='changed',revision=revision+1; COMMIT").unwrap();
        assert!(queue_revision(&writer).unwrap() > revision);
        assert_eq!(queue_revision(&snapshot.transaction).unwrap(), revision);
        assert_eq!(snapshot.queue.cards, cards);
        assert_eq!(snapshot.audio.pending_count, 0);
        let old_notes: String = snapshot
            .transaction
            .query_row("SELECT notes FROM vocabulary WHERE id=1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(old_notes, "");
        assert!(
            selected_audio_batch(&snapshot.transaction, &[1])
                .unwrap()
                .contains_key(&1)
        );
        snapshot.transaction.commit().unwrap();
        let next =
            review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
        assert!(next.queue.cards.is_empty());
        assert_eq!(next.audio.pending_count, 1);
        assert!(
            next.demand_candidates.is_empty(),
            "quota-blocked repairs must not enter today's plan"
        );
        assert_eq!(cache.revision(), queue_revision(&writer).unwrap());
        next.transaction.commit().unwrap();
        drop(writer);
    }
    drop(reader);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn queue_snapshots_finish_under_continuous_audio_revision_churn() {
    use std::sync::mpsc;
    let path = std::env::temp_dir().join(format!(
        "lexique-churn-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut reader = Connection::open(&path).unwrap();
    initialize(&reader).unwrap();
    reader.pragma_update(None, "journal_mode", "WAL").unwrap();
    reader
        .busy_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    let tx = reader.transaction().unwrap();
    tx.execute(
        "UPDATE review_settings SET new_limit=1000,review_limit=1000",
        [],
    )
    .unwrap();
    for id in 1..=500 {
        tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,'{}','',1,1)", rusqlite::params![id, format!("mot{id}")]).unwrap();
        for (card, kind) in [(id * 2 - 1, "recognition"), (id * 2, "listening")] {
            tx.execute(
                "INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?1,?2,?3)",
                rusqlite::params![card, id, kind],
            )
            .unwrap();
        }
    }
    let locator = "a".repeat(64);
    tx.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,created_at,physical_locator,context,sealed,verification,last_used) VALUES(?1,?1,4,1,?1,'word',1,'verified',1)", [&locator]).unwrap();
    tx.execute(
        "INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(1,?1)",
        [&locator],
    )
    .unwrap();
    tx.execute("UPDATE speech_versions SET verification='missing'", [])
        .unwrap();
    tx.commit().unwrap();
    let mut cache = QueueCache::default();
    let skipped = Default::default();
    review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache)
        .unwrap()
        .transaction
        .commit()
        .unwrap();
    let writer = Connection::open(&path).unwrap();
    writer
        .busy_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    let transaction = review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache)
        .unwrap()
        .transaction;
    let (committed, observed) = mpsc::sync_channel(0);
    let worker = std::thread::spawn(move || {
        for index in 0..1000 {
            writer.execute("UPDATE speech_versions SET verification=CASE WHEN verification='missing' THEN 'invalid' ELSE 'missing' END", []).unwrap();
            if index == 0 {
                committed.send(queue_revision(&writer).unwrap()).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    assert!(
        observed.recv().unwrap() > queue_revision(&transaction).unwrap(),
        "the competing writer must commit during snapshot reads"
    );
    assert_eq!(queue_revision(&transaction).unwrap(), cache.revision());
    transaction.commit().unwrap();
    for _ in 0..40 {
        let snapshot =
            review_queue_snapshot(&reader, 1_700_000_000, "UTC", &skipped, &mut cache).unwrap();
        assert_eq!(
            queue_revision(&snapshot.transaction).unwrap(),
            cache.revision()
        );
        assert_eq!(snapshot.queue.cards.len(), 500);
        assert_eq!(snapshot.demand_candidates.len(), 1000);
        snapshot.transaction.commit().unwrap();
    }
    worker.join().unwrap();
    drop(reader);
    std::fs::remove_file(path).unwrap();
}
