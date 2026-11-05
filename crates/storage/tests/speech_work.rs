use lexique_storage::*;
use rusqlite::{Connection, params};

#[test]
fn batch_audio_candidates_match_existing_expiry_ownership_and_selection_rules() {
    let connection = fixture();
    for index in 0..25 {
        let spec = SpeechSpec::word(&format!("candidate-{index}"), SpeechVoice::Female);
        let key = register_speech(&connection, &spec).unwrap();
        connection
            .execute("INSERT INTO speech_owners VALUES(?1,1)", [&key])
            .unwrap();
        publish_speech(
            &connection,
            &key,
            &content_hash(format!("output-{index}").as_bytes()),
            1000 + index,
        )
        .unwrap();
    }
    // A shared version stays retained past expiry, while GC candidates are excluded.
    let keys = word_audio_keys(&connection, 1).unwrap();
    connection
        .execute("INSERT INTO speech_owners VALUES(?1,2)", [&keys[0]])
        .unwrap();
    connection
        .execute(
            "UPDATE speech_assets SET gc_pending=1 WHERE semantic_key=?1",
            [&keys[1]],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE speech_assets SET retain=0,expires_at=1000 WHERE semantic_key=?1",
            [&keys[2]],
        )
        .unwrap();
    for now in [1001, 1_000_000] {
        let batch = word_audio_assets(&connection, &[1, 2, 999], now).unwrap();
        for id in [1, 2, 999] {
            let expected: Vec<_> = word_audio_keys(&connection, id)
                .unwrap()
                .into_iter()
                .filter_map(|key| speech_asset(&connection, &key, now).unwrap())
                .collect();
            let actual = batch.get(&id).map_or(&[][..], Vec::as_slice);
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
        }
    }
    assert!(
        word_audio_assets(&connection, &[], 1000)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn full_speech_backlog_preserves_committed_cards_and_allows_interactive_retry() {
    let mut connection = fixture();
    let spec = serde_json::to_string(&SpeechSpec::word("backlog", SpeechVoice::Female)).unwrap();
    connection.execute("WITH RECURSIVE n(x) AS(SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<10000) INSERT INTO speech_assets(semantic_key,spec,context) SELECT printf('%064x',x),?1,'word' FROM n", [spec]).unwrap();
    connection.execute("INSERT INTO speech_tasks(semantic_key,priority,state,wake_at) SELECT semantic_key,1,'queued',1000 FROM speech_assets", []).unwrap();
    let result = add_review_batch(&mut connection, "full-backlog", &[1], 1000).unwrap();
    assert_eq!(
        result.items[0].created_cards, 2,
        "Speech capacity must not roll back card creation"
    );
    assert!(result.items[0].error.is_none());
    assert_eq!(result.items[0].audio_state.as_deref(), Some("failed"));
    assert!(
        result.items[0]
            .audio_error
            .as_deref()
            .unwrap()
            .contains("backlog")
    );
    let repeated = add_review_batch(&mut connection, "full-backlog", &[1], 1001).unwrap();
    assert_eq!(repeated.items[0].created_cards, 2);
    assert_eq!(review_cards(&connection, 1).unwrap().len(), 2);
    let queue = review_queue_filtered(&mut connection, 1001, "UTC", |card| {
        Ok(card.kind == "recognition")
    })
    .unwrap();
    assert_eq!(queue.cards.len(), 1);
    request_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Female),
        Some(1),
        1002,
        false,
    )
    .unwrap();
    assert_eq!(
        claim_speech(&mut connection, 1002, 2)
            .unwrap()
            .unwrap()
            .priority,
        0
    );
}

#[test]
fn new_batch_rechecks_completed_audio_without_reviving_cancelled_or_failed_tasks() {
    let mut connection = fixture();
    add_review_batch(&mut connection, "first", &[1, 2], 1000).unwrap();
    let first = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    let second = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    publish_speech(&connection, &first.semantic_key, &"a".repeat(64), 1001).unwrap();
    fail_speech(&connection, second.id, 1001, "permanent failure", true).unwrap();
    let again = add_review_batch(&mut connection, "second", &[1, 2], 1002).unwrap();
    assert_eq!(again.items[0].created_cards, 0);
    assert_eq!(again.items[0].audio_state.as_deref(), Some("queued"));
    assert_eq!(again.items[1].audio_state.as_deref(), Some("failed"));
    let recheck = claim_speech(&mut connection, 1002, 2).unwrap().unwrap();
    assert_eq!(recheck.id, first.id);
    assert_eq!(recheck.attempts, 1);
    ready_speech_task(&connection, &recheck.semantic_key).unwrap();
    add_review_batch(&mut connection, "third", &[1], 1003).unwrap();
    cancel_batch(&connection, "first").unwrap();
    cancel_batch(&connection, "second").unwrap();
    cancel_batch(&connection, "third").unwrap();
    let cancelled = add_review_batch(&mut connection, "fourth", &[1], 1004).unwrap();
    assert_eq!(cancelled.items[0].audio_state.as_deref(), Some("cancelled"));
    assert!(claim_speech(&mut connection, 1004, 2).unwrap().is_none());
    request_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Female),
        Some(1),
        1005,
        false,
    )
    .unwrap();
    assert_eq!(
        claim_speech(&mut connection, 1005, 2)
            .unwrap()
            .unwrap()
            .priority,
        0
    );
}

#[test]
fn completed_audio_recheck_obeys_interactive_capacity_and_rolls_back() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let key = request_speech(&mut connection, &spec, Some(1), 1000, false).unwrap();
    claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    publish_speech(&connection, &key, &"a".repeat(64), 1001).unwrap();
    for index in 0..32 {
        request_speech(
            &mut connection,
            &SpeechSpec::word(&format!("mot{index}"), SpeechVoice::Male),
            None,
            1001,
            false,
        )
        .unwrap();
    }
    assert!(request_speech(&mut connection, &spec, Some(1), 1002, false).is_err());
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "ready"
    );
    assert!(speech_asset(&connection, &key, 1002).unwrap().is_some());
}

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let snapshot=serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"synthetic","version":"1","attribution":"synthetic","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    for (id, word) in [(1, "école"), (2, "bonjour")] {
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'original',1000,1000)",params![id,word,snapshot]).unwrap();
    }
    connection
}

#[test]
fn interactive_registration_rolls_back_ownership_and_preserves_capacity() {
    let mut connection = fixture();
    let word = SpeechSpec::word("école", SpeechVoice::Female);
    assert!(request_speech(&mut connection, &word, Some(2), 1000, true).is_err());
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM speech_assets", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let key = request_speech(&mut connection, &word, None, 1000, false).unwrap();
    let task = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    fail_speech(&connection, task.id, 1001, "permanent", true).unwrap();
    for index in 0..32 {
        request_speech(
            &mut connection,
            &SpeechSpec::word(&format!("mot{index}"), SpeechVoice::Male),
            None,
            1001,
            false,
        )
        .unwrap();
    }
    assert!(request_speech(&mut connection, &word, Some(1), 1002, true).is_err());
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "failed"
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM speech_owners", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn gc_mark_survives_recovery_and_does_not_revive_an_expired_asset() {
    let mut connection = fixture();
    let word = SpeechSpec::word("école", SpeechVoice::Female);
    let key = register_speech(&connection, &word).unwrap();
    let hash = "a".repeat(64);
    publish_speech(&connection, &key, &hash, 1000).unwrap();
    assert_eq!(
        mark_speech_gc(&connection, &key, 605800).unwrap(),
        Some(hash.clone())
    );
    request_speech(&mut connection, &word, Some(1), 605800, false).unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_none());
    assert_eq!(
        pending_speech_gc(&connection).unwrap(),
        [(key.clone(), hash.clone())]
    );
    finish_speech_gc(&connection, &key, &hash).unwrap();
    assert!(pending_speech_gc(&connection).unwrap().is_empty());
    assert!(speech_asset(&connection, &key, 605801).unwrap().is_none());
}

#[test]
fn legacy_metadata_is_registered_in_atomic_bounded_chunks_without_rewriting_unchanged_hashes() {
    let mut connection = fixture();
    let hashes = vec![(1, "a".repeat(64)), (2, "b".repeat(64))];
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(register_legacy_speeches(&mut connection, &hashes).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM legacy_speech", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    register_legacy_speeches(&mut connection, &hashes).unwrap();
    let before = connection.total_changes();
    static TRANSACTIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    fn count_transactions(event: rusqlite::trace::TraceEvent<'_>) {
        if let rusqlite::trace::TraceEvent::Stmt(_, sql) = event
            && sql.starts_with("BEGIN")
        {
            TRANSACTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    connection.trace_v2(
        rusqlite::trace::TraceEventCodes::SQLITE_TRACE_STMT,
        Some(count_transactions),
    );
    register_legacy_speeches(&mut connection, &hashes).unwrap();
    connection.trace_v2(rusqlite::trace::TraceEventCodes::empty(), None);
    assert_eq!(connection.total_changes(), before);
    assert_eq!(TRANSACTIONS.load(std::sync::atomic::Ordering::Relaxed), 0);
    let changed = vec![(1, "c".repeat(64)), (2, "d".repeat(64))];
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(register_legacy_speeches(&mut connection, &changed).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    let retained: String = connection
        .query_row(
            "SELECT content_hash FROM legacy_speech WHERE vocabulary_id=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(retained, hashes[0].1);
    register_legacy_speeches(&mut connection, &changed).unwrap();
    let updated: String = connection
        .query_row(
            "SELECT content_hash FROM legacy_speech WHERE vocabulary_id=1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(updated, changed[0].1);
    assert!(register_legacy_speeches(&mut connection, &[(1, "invalid".into())]).is_err());
    assert!(
        speech_asset(
            &connection,
            &SpeechSpec::word("école", SpeechVoice::Female)
                .key()
                .unwrap(),
            1000
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn retry_budget_survives_worker_recovery_and_queue_refresh() {
    let mut connection = fixture();
    let key = request_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Female),
        None,
        1000,
        false,
    )
    .unwrap();
    let first = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    assert_eq!(first.deadline_at, 1120);
    resume_speech(&connection, first.id, 1010).unwrap();
    let second = claim_speech(&mut connection, 1010, 2).unwrap().unwrap();
    assert_eq!(second.attempts, 2);
    assert_eq!(second.deadline_at, 1120);
    resume_speech(&connection, second.id, 1120).unwrap();
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "failed"
    );
    assert!(claim_speech(&mut connection, 1121, 2).unwrap().is_none());
}

#[test]
fn later_chunks_keep_the_committed_voice_and_cancellation_snapshot() {
    let path = std::env::temp_dir().join(format!(
        "lexique-batch-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut connection = Connection::open(&path).unwrap();
        initialize(&connection).unwrap();
        let seed = fixture();
        let snapshot: String = seed
            .query_row("SELECT snapshot FROM vocabulary WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        let tx = connection.transaction().unwrap();
        for id in 1..=150 {
            tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'keep',1000,1000)",params![id,format!("mot{id}"),snapshot]).unwrap();
        }
        tx.commit().unwrap();
        let mut other = Connection::open(&path).unwrap();
        initialize(&other).unwrap();
        let ids = (1..=150).collect::<Vec<_>>();
        let mut commits = 0;
        let status = add_review_batch_observed(&mut connection, "snapshot", &ids, 1000, || {
            commits += 1;
            if commits == 1 {
                set_speech_preferences(
                    &other,
                    &SpeechPreferences {
                        voice: SpeechVoice::Male,
                        speed: SpeechSpeed::Normal,
                    },
                )
                .unwrap();
                cancel_batch(&other, "snapshot").unwrap();
                assert!(add_review_batch(&mut other, "snapshot", &[1], 1001).is_err());
            }
        })
        .unwrap();
        assert_eq!(commits, 2);
        assert!(status.cancelled);
        assert!(status.items.iter().all(
            |item| item.created_cards == 2 && item.audio_state.as_deref() == Some("cancelled")
        ));
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM speech_assets WHERE json_extract(spec,'$.voice')='female'",[],|r|r.get::<_,i64>(0)).unwrap(),150);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM review_cards", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            300
        );
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn batch_commit_is_idempotent_and_audio_demand_does_not_block_cards() {
    let mut connection = fixture();
    let result = add_review_batch(&mut connection, "batch-1", &[1, 2, 999, 1], 1000).unwrap();
    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0].created_cards, 2);
    assert_eq!(result.items[0].audio_state.as_deref(), Some("queued"));
    assert!(result.items[2].error.is_some());
    assert_eq!(review_cards(&connection, 1).unwrap().len(), 2);
    assert_eq!(
        review_progress(&connection, review_cards(&connection, 1).unwrap()[0].id)
            .unwrap()
            .card
            .due_at,
        None
    );
    let again = add_review_batch(&mut connection, "batch-1", &[1, 2, 999, 1], 1001).unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(again).unwrap()
    );
    assert!(add_review_batch(&mut connection, "batch-1", &[1], 1001).is_err());
}

#[test]
fn committed_ownership_protects_temp_audio_without_review_cards() {
    let connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let key = spec.key().unwrap();
    register_speech(&connection, &spec).unwrap();
    publish_speech(&connection, &key, &"a".repeat(64), 1000).unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_none());
    attach_word_audio_at(&connection, 1, 605799).unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_some());
    assert!(review_cards(&connection, 1).unwrap().is_empty());
    assert!(expired_speech(&connection, 605800, 100).unwrap().is_empty());
}

#[test]
fn tasks_preserve_attempt_budget_cancel_and_reserved_interactive_capacity() {
    let mut connection = fixture();
    add_review_batch(&mut connection, "batch-1", &[1, 2], 1000).unwrap();
    let first = claim_speech(&mut connection, 1000, 1).unwrap().unwrap();
    assert_eq!(first.priority, 1);
    assert!(claim_speech(&mut connection, 1000, 1).unwrap().is_none());
    let p0 = SpeechSpec::word("rouge", SpeechVoice::Male);
    demand_speech(&connection, &p0, 0, 1000, true).unwrap();
    assert_eq!(
        claim_speech(&mut connection, 1000, 1)
            .unwrap()
            .unwrap()
            .priority,
        0
    );
    fail_speech(
        &connection,
        first.id,
        1001,
        "temporary worker failure",
        false,
    )
    .unwrap();
    for (now, attempt) in [(1003, 2), (1011, 3), (1031, 4)] {
        let next = claim_speech(&mut connection, now, 1).unwrap().unwrap();
        assert_eq!(next.id, first.id);
        assert_eq!(next.attempts, attempt);
        fail_speech(&connection, next.id, now, "temporary worker failure", false).unwrap();
    }
    assert_eq!(
        batch_status(&connection, "batch-1").unwrap().items[0]
            .audio_state
            .as_deref(),
        Some("failed")
    );
    cancel_batch(&connection, "batch-1").unwrap();
    assert_eq!(
        batch_status(&connection, "batch-1").unwrap().items[1]
            .audio_state
            .as_deref(),
        Some("cancelled")
    );
}

#[test]
fn failed_commit_does_not_create_cards_or_claim_success() {
    let mut connection = fixture();
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(add_review_batch(&mut connection, "batch-1", &[1, 2], 1000).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(review_cards(&connection, 1).unwrap().is_empty());
    assert!(batch_status(&connection, "batch-1").is_err());
}

#[test]
fn publication_intent_preserves_previous_asset_until_verified_commit() {
    let mut connection = fixture();
    add_review_batch(&mut connection, "first", &[1], 1000).unwrap();
    let task = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    publish_speech(&connection, &task.semantic_key, &"a".repeat(64), 1000).unwrap();
    prepare_speech_publication(&connection, &task.semantic_key, &"b".repeat(64), 1010).unwrap();
    assert_eq!(
        speech_asset(&connection, &task.semantic_key, 1010)
            .unwrap()
            .unwrap()
            .content_hash,
        "a".repeat(64)
    );
    assert!(complete_speech_publication(&connection, &task.semantic_key, &"c".repeat(64)).is_err());
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(complete_speech_publication(&connection, &task.semantic_key, &"b".repeat(64)).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert!(
        pending_publication(&connection, &task.semantic_key)
            .unwrap()
            .is_some()
    );
    complete_speech_publication(&connection, &task.semantic_key, &"b".repeat(64)).unwrap();
    assert_eq!(
        speech_asset(&connection, &task.semantic_key, 1010)
            .unwrap()
            .unwrap()
            .content_hash,
        "b".repeat(64)
    );
}

#[test]
fn gc_rechecks_word_ownership_but_does_not_permanently_protect_sentences() {
    let connection = fixture();
    let word = SpeechSpec::word("école", SpeechVoice::Female);
    let key = register_speech(&connection, &word).unwrap();
    publish_speech(&connection, &key, &"a".repeat(64), 1000).unwrap();
    assert_eq!(
        expired_speech(&connection, 605800, 100).unwrap(),
        std::slice::from_ref(&key)
    );
    attach_word_audio_at(&connection, 1, 605799).unwrap();
    assert!(mark_speech_gc(&connection, &key, 605800).unwrap().is_none());
    let mut sentence = SpeechSpec::word("bonjour", SpeechVoice::Female);
    sentence.context = "sentence".into();
    let key = register_speech(&connection, &sentence).unwrap();
    publish_speech(&connection, &key, &"b".repeat(64), 1000).unwrap();
    connection
        .execute("INSERT INTO speech_owners VALUES(?1,2)", [&key])
        .unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_none());
    assert!(mark_speech_gc(&connection, &key, 605800).unwrap().is_some());
    prepare_speech_publication(&connection, &key, &"c".repeat(64), 605800).unwrap();
    let pending = lexique_storage::pending_source_audio(&connection, "").unwrap();
    assert_eq!(
        pending.len(),
        1,
        "a marked old instance does not block a fresh publication"
    );
    finish_speech_gc(&connection, &key, &"b".repeat(64)).unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_none());
}

#[test]
fn duplicate_batches_do_not_reset_terminal_audio_or_change_voice() {
    let mut connection = fixture();
    add_review_batch(&mut connection, "one", &[1], 1000).unwrap();
    let task = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    fail_speech(&connection, task.id, 1001, "permanent", true).unwrap();
    add_review_batch(&mut connection, "two", &[1], 1002).unwrap();
    assert_eq!(
        batch_status(&connection, "two").unwrap().items[0]
            .audio_state
            .as_deref(),
        Some("failed")
    );
    set_speech_preferences(
        &connection,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            speed: SpeechSpeed::Slow,
        },
    )
    .unwrap();
    add_review_batch(&mut connection, "one", &[1], 1003).unwrap();
    assert_eq!(
        batch_status(&connection, "one").unwrap().items[0]
            .audio_state
            .as_deref(),
        Some("failed")
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM speech_assets", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn a_favorite_at_expiry_does_not_revive_expired_temporary_audio() {
    let connection = fixture();
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    publish_speech(&connection, &key, &"a".repeat(64), 1000).unwrap();
    attach_word_audio_at(&connection, 1, 605800).unwrap();
    assert!(speech_asset(&connection, &key, 605800).unwrap().is_none());
    publish_speech(&connection, &key, &"b".repeat(64), 605801).unwrap();
    assert!(
        speech_asset(&connection, &key, 1_210_601)
            .unwrap()
            .is_some()
    );
}

#[test]
fn backend_scale_uses_committed_hundred_word_chunks_and_bounded_claims() {
    for count in [100, 1000, 10000] {
        let mut connection = fixture();
        connection.execute("DELETE FROM vocabulary", []).unwrap();
        let tx = connection.transaction().unwrap();
        let snapshot=serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"synthetic","version":"1","attribution":"synthetic","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
        for id in 1..=count {
            tx.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'',1000,1000)",params![id,format!("mot{id}"),snapshot]).unwrap();
        }
        tx.commit().unwrap();
        let ids = (1..=count).collect::<Vec<_>>();
        let mut chunks = 0;
        let mut previous = std::time::Instant::now();
        let mut max_chunk_ms: f64 = 0.0;
        let start = std::time::Instant::now();
        let status = add_review_batch_observed(&mut connection, "scale", &ids, 1000, || {
            chunks += 1;
            max_chunk_ms = max_chunk_ms.max(previous.elapsed().as_secs_f64() * 1000.0);
            previous = std::time::Instant::now();
        })
        .unwrap();
        println!(
            "PRODUCT_BATCH count={count} elapsed_ms={} committed_chunks={chunks} max_chunk_ms={max_chunk_ms}",
            start.elapsed().as_secs_f64() * 1000.0
        );
        assert_eq!(chunks, count / 100);
        assert_eq!(status.items.len(), count as usize);
        assert!(
            status
                .items
                .iter()
                .all(|item| item.created_cards == 2 && item.error.is_none())
        );
        assert_eq!(
            claim_speech(&mut connection, 1000, 2)
                .unwrap()
                .unwrap()
                .priority,
            1
        );
        assert_eq!(
            claim_speech(&mut connection, 1000, 2)
                .unwrap()
                .unwrap()
                .priority,
            1
        );
        assert!(claim_speech(&mut connection, 1000, 2).unwrap().is_none());
        let p0 = request_speech(
            &mut connection,
            &SpeechSpec::word("interaction", SpeechVoice::Male),
            None,
            1000,
            false,
        )
        .unwrap();
        assert_eq!(
            claim_speech(&mut connection, 1000, 2)
                .unwrap()
                .unwrap()
                .semantic_key,
            p0
        );
        cancel_batch(&connection, "scale").unwrap();
        assert_eq!(
            batch_status(&connection, "scale")
                .unwrap()
                .items
                .iter()
                .filter(|item| item.audio_state.as_deref() == Some("cancelled"))
                .count(),
            count as usize - 2
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM review_cards", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2 * count
        );
    }
}
