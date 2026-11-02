use lexique_storage::*;
use rusqlite::{Connection, params};

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
    register_legacy_speeches(&mut connection, &hashes).unwrap();
    assert_eq!(connection.total_changes(), before);
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
        [key.clone()]
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
    assert!(prepare_speech_publication(&connection, &key, &"c".repeat(64), 605800).is_err());
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
