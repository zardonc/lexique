use lexique_storage::*;
use rusqlite::{Connection, params};

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"école","version":"1","attribution":"fixture","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'école',?1,'keep',1,1)", [snapshot]).unwrap();
    connection
}

#[test]
fn pending_initial_publication_cannot_be_bypassed_by_cached_source_or_distinct_copy() {
    let connection = fixture();
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let source =
        prepare_source_audio(&connection, &key, &content_hash(b"OggSsource"), 10, 1000).unwrap();
    verify_audio_content(&connection, &source.locator, 10).unwrap();
    complete_source_audio(&connection, &source.locator).unwrap();
    let pending = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSintended"),
        12,
        None,
        0,
        1001,
    )
    .unwrap();
    assert!(
        !reuse_audio_version(&connection, 1, &source.locator, 0, 1002).unwrap(),
        "source adoption must preserve the durable initial intent"
    );
    assert!(
        prepare_audio_version(
            &connection,
            1,
            None,
            &content_hash(b"OggSother"),
            9,
            None,
            0,
            1002
        )
        .is_err(),
        "a distinct compatibility copy must not supersede the pending intent"
    );
    let retry = prepare_audio_version(
        &connection,
        1,
        None,
        &pending.content_hash,
        12,
        None,
        0,
        1002,
    )
    .unwrap();
    assert_eq!(
        retry.locator, pending.locator,
        "the original reservation remains idempotent"
    );
    assert!(complete_audio_version(&connection, &pending.locator, 1002).unwrap());
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        pending.locator
    );
}

fn old_fixture(version: usize) -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    let migrations = [
        include_str!("../migrations/0001_initial.sql"),
        include_str!("../migrations/0002_vocabulary.sql"),
        include_str!("../migrations/0003_review.sql"),
        include_str!("../migrations/0004_ratings.sql"),
        include_str!("../migrations/0005_review_settings.sql"),
        include_str!("../migrations/0006_undo.sql"),
        include_str!("../migrations/0007_review_management.sql"),
        include_str!("../migrations/0008_examples.sql"),
        include_str!("../migrations/0009_more_examples.sql"),
        include_str!("../migrations/0010_example_sense.sql"),
        include_str!("../migrations/0011_speech_work.sql"),
        include_str!("../migrations/0012_selected_audio.sql"),
        include_str!("../migrations/0013_speech_successors.sql"),
        include_str!("../migrations/0014_audio_operations.sql"),
    ];
    for sql in &migrations[..version] {
        connection.execute_batch(sql).unwrap();
    }
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let snapshot = fixture()
        .query_row("SELECT snapshot FROM vocabulary", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'école',?1,'keep',1,1)",[snapshot]).unwrap();
    connection
}

#[test]
fn immutable_instances_selection_generations_and_previous_protection_are_atomic() {
    let connection = fixture();
    let first = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSfirst"),
        9,
        None,
        0,
        1000,
    )
    .unwrap();
    assert!(selected_audio(&connection, 1).unwrap().is_none());
    assert_eq!(pending_audio_versions(&connection, "").unwrap().len(), 1);
    assert!(
        connection
            .execute(
                "UPDATE word_audio_selections SET current_version=?1 WHERE vocabulary_id=1",
                [&first.locator]
            )
            .is_err(),
        "unverified publication must not become current"
    );
    let retry =
        prepare_audio_version(&connection, 1, None, &first.content_hash, 9, None, 0, 1001).unwrap();
    assert_eq!(
        retry.locator, first.locator,
        "an interrupted copy reuses its durable intent"
    );
    assert!(complete_audio_version(&connection, &first.locator, 1001).unwrap());
    assert!(
        prepare_audio_version(
            &connection,
            1,
            None,
            &content_hash(b"OggSother"),
            9,
            None,
            0,
            1002
        )
        .is_err(),
        "a late compatibility read cannot replace an initialized selection"
    );
    assert!(
        connection
            .execute(
                "UPDATE speech_versions SET sealed=0 WHERE locator=?1",
                [&first.locator]
            )
            .is_err()
    );
    assert!(complete_audio_version(&connection, &first.locator, 1002).unwrap());
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let stale = begin_audio_change(&connection, 1, &key).unwrap();
    let old = prepare_audio_version(
        &connection,
        1,
        Some(&key),
        &content_hash(b"OggSold"),
        7,
        Some(1003),
        stale,
        1003,
    )
    .unwrap();
    let latest = begin_audio_change(&connection, 1, &key).unwrap();
    assert!(!complete_audio_version(&connection, &old.locator, 1004).unwrap());
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        first.locator
    );
    let new = prepare_audio_version(
        &connection,
        1,
        Some(&key),
        &old.content_hash,
        7,
        Some(1003),
        latest,
        1005,
    )
    .unwrap();
    assert_ne!(
        new.locator, old.locator,
        "same hash does not reuse a publication instance"
    );
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(complete_audio_version(&connection, &new.locator, 1005).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        first.locator
    );
    assert!(complete_audio_version(&connection, &new.locator, 1006).unwrap());
    let previous:(String,i64,i64)=connection.query_row("SELECT previous_version,previous_until,revision FROM word_audio_selections WHERE vocabulary_id=1", [], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(previous, (first.locator, 1006 + 1_209_600, 2));
    assert!(complete_audio_version(&connection, &new.locator, 2000).unwrap());
    assert_eq!(
        connection
            .query_row(
                "SELECT previous_until FROM word_audio_selections",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1006 + 1_209_600
    );
    assert!(
        connection
            .execute(
                "UPDATE speech_versions SET content_hash=?1 WHERE locator=?2",
                params![content_hash(b"different"), new.locator]
            )
            .is_err()
    );
    connection.execute("DELETE FROM review_cards", []).unwrap();
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        new.locator
    );
}

#[test]
fn version_metadata_and_generation_overflow_fail_without_changing_selection() {
    let connection = fixture();
    for (hash, size, time) in [
        ("bad".into(), 4, 1000),
        (content_hash(b"OggS"), 10_000_001, 1000),
        (content_hash(b"OggS"), 4, i64::MAX),
    ] {
        assert!(prepare_audio_version(&connection, 1, None, &hash, size, None, 0, time).is_err());
    }
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let wrong =
        register_speech(&connection, &SpeechSpec::word("chat", SpeechVoice::Female)).unwrap();
    assert!(begin_audio_change(&connection, 1, &wrong).is_err());
    assert!(
        prepare_audio_version(
            &connection,
            1,
            Some(&wrong),
            &content_hash(b"OggS"),
            4,
            None,
            0,
            1000
        )
        .is_err()
    );
    begin_audio_change(&connection, 1, &key).unwrap();
    connection
        .execute(
            "UPDATE word_audio_selections SET desired_generation=9223372036854775807",
            [],
        )
        .unwrap();
    assert!(begin_audio_change(&connection, 1, &key).is_err());
    assert!(selected_audio(&connection, 1).unwrap().is_none());
}

#[test]
fn recovery_pages_past_missing_versions_without_repeating_them() {
    let connection = fixture();
    for i in 0..205 {
        let id = i + 1;
        if id > 1 {
            connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,'{}','',1,1)", params![id, format!("mot{id}")]).unwrap();
        }
        prepare_audio_version(
            &connection,
            id,
            None,
            &content_hash(format!("OggS{i}").as_bytes()),
            8,
            None,
            0,
            1000,
        )
        .unwrap();
    }
    let mut cursor = String::new();
    let mut seen = std::collections::HashSet::new();
    loop {
        let page = pending_audio_versions(&connection, &cursor).unwrap();
        assert!(page.len() <= 100);
        if page.is_empty() {
            break;
        }
        for version in page {
            assert!(seen.insert(version.locator.clone()));
            cursor = version.locator;
        }
    }
    assert_eq!(seen.len(), 205);
}

#[test]
fn failed_selection_request_rolls_back_its_execution_demand() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let (key, generation) =
        request_selected_speech(&mut connection, &spec, 1, 1000, true, None).unwrap();
    assert_eq!(generation, 1);
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "queued"
    );
    connection
        .execute(
            "UPDATE word_audio_selections SET desired_generation=9223372036854775807",
            [],
        )
        .unwrap();
    let other = SpeechSpec::word("école", SpeechVoice::Male);
    assert!(request_selected_speech(&mut connection, &other, 1, 1001, true, None).is_err());
    assert!(
        speech_task_state(&connection, &other.key().unwrap())
            .unwrap()
            .is_none()
    );
    assert_eq!(
        connection
            .query_row("SELECT desired_key FROM word_audio_selections", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        key
    );
}

#[test]
fn schema_eleven_upgrade_is_non_destructive_and_rolls_back_on_conflict() {
    let connection = old_fixture(11);
    connection
        .execute_batch("CREATE TABLE speech_versions(unexpected TEXT);")
        .unwrap();
    assert!(initialize(&connection).is_err());
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        11
    );
    assert_eq!(
        connection
            .query_row("SELECT notes FROM vocabulary", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
    connection
        .execute("DROP TABLE speech_versions", [])
        .unwrap();
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    assert!(selected_audio(&connection, 1).unwrap().is_none());
    assert_eq!(pending_audio_imports(&connection).unwrap(), vec![1]);
    assert_eq!(audio_migration_status(&connection).unwrap().pending, 1);
    finish_audio_import(&connection, 1, Some("read denied")).unwrap();
    assert_eq!(audio_migration_status(&connection).unwrap().failed, 1);
    assert!(pending_audio_imports(&connection).unwrap().is_empty());
    finish_audio_import(&connection, 1, None).unwrap();
    assert_eq!(audio_migration_status(&connection).unwrap().failed, 0);
}

#[test]
fn compatibility_imports_resume_in_recent_first_bounded_chunks() {
    let connection = fixture();
    for id in 2..=32 {
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,'{}','',1,?1)",params![id,format!("word-{id}")]).unwrap();
    }
    connection
        .execute(
            "INSERT INTO word_audio_imports(vocabulary_id) SELECT id FROM vocabulary",
            [],
        )
        .unwrap();
    assert_eq!(
        pending_audio_imports(&connection).unwrap(),
        (17..=32).rev().collect::<Vec<_>>()
    );
    for id in 17..=32 {
        finish_audio_import(&connection, id, None).unwrap();
    }
    initialize(&connection).unwrap();
    assert_eq!(
        pending_audio_imports(&connection).unwrap(),
        (1..=16).rev().collect::<Vec<_>>()
    );
    assert_eq!(audio_migration_status(&connection).unwrap().pending, 16);
}

#[test]
fn a_new_regeneration_during_execution_gets_a_successor_before_selection() {
    let mut connection = fixture();
    let original = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSoriginal"),
        12,
        None,
        0,
        999,
    )
    .unwrap();
    complete_audio_version(&connection, &original.locator, 999).unwrap();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let (key, first) =
        request_selected_speech(&mut connection, &spec, 1, 1000, true, None).unwrap();
    claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    let (_, latest) = request_selected_speech(&mut connection, &spec, 1, 1001, true, None).unwrap();
    assert_eq!(latest, first + 1);
    prepare_speech_publication(&connection, &key, &content_hash(b"OggSold-execution"), 1002)
        .unwrap();
    complete_speech_publication(&connection, &key, &content_hash(b"OggSold-execution")).unwrap();
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "queued",
        "an execution already in flight cannot satisfy a newer explicit regeneration"
    );
    assert!(
        ready_audio_selections(&connection, &key)
            .unwrap()
            .is_empty(),
        "publication must not bind the older output to the newest generation"
    );
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        original.locator
    );
    let successor = claim_speech(&mut connection, 1002, 2).unwrap().unwrap();
    assert!(successor.refresh);
    assert_eq!(successor.attempts, 1);
    assert_eq!(successor.deadline_at, 1122);
    publish_speech(&connection, &key, &content_hash(b"OggSsuccessor"), 1003).unwrap();
    assert_eq!(
        speech_task_state(&connection, &key).unwrap().unwrap().0,
        "ready"
    );
    assert_eq!(
        ready_audio_selections(&connection, &key).unwrap(),
        [(1, latest)]
    );
}

#[test]
fn a_successor_survives_recovery_without_reusing_the_old_execution_budget() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let (key, _) = request_selected_speech(&mut connection, &spec, 1, 1000, true, None).unwrap();
    let original = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    request_selected_speech(&mut connection, &spec, 1, 1001, true, None).unwrap();
    resume_speech(&connection, original.id, 1002).unwrap();
    resume_speech(&connection, original.id, 1003).unwrap();
    let successor = claim_speech(&mut connection, 1003, 2).unwrap().unwrap();
    assert_eq!(successor.semantic_key, key);
    assert!(successor.refresh);
    assert_eq!(
        successor.attempts, 1,
        "the explicit successor starts a new execution, not a retry of the interrupted one"
    );
    assert_eq!(successor.deadline_at, 1123);
    fail_speech(&connection, successor.id, 1004, "temporary", false).unwrap();
    let retry = claim_speech(&mut connection, 1006, 2).unwrap().unwrap();
    assert_eq!(retry.attempts, 2);
    assert_eq!(
        retry.deadline_at, 1123,
        "automatic retries retain the successor's budget"
    );
}

#[test]
fn cancelling_a_batch_does_not_cancel_its_running_interactive_subscriber() {
    let mut connection = fixture();
    add_review_batch(&mut connection, "batch", &[1], 1000).unwrap();
    let running = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    request_speech(&mut connection, &running.spec, Some(1), 1001, false).unwrap();
    cancel_batch(&connection, "batch").unwrap();
    fail_speech(&connection, running.id, 1002, "temporary", false).unwrap();
    assert_eq!(
        speech_task_state(&connection, &running.semantic_key)
            .unwrap()
            .unwrap()
            .0,
        "retry_wait",
        "the active interactive subscriber still needs the shared execution"
    );
    let retry = claim_speech(&mut connection, 1004, 2).unwrap().unwrap();
    assert_eq!(retry.id, running.id);
    assert_eq!(retry.attempts, 2);
}

#[test]
fn schema_twelve_upgrade_preserves_selection_and_rolls_back_the_new_task_fields() {
    let connection = old_fixture(12);
    let original = "a".repeat(64);
    connection.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,created_at,vocabulary_id,expected_revision,generation,sealed) VALUES(?1,?2,8,999,1,0,0,1)",params![original,content_hash(b"OggSkeep")]).unwrap();
    connection.execute("INSERT INTO word_audio_selections(vocabulary_id,current_version,revision) VALUES(1,?1,1)",[&original]).unwrap();
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(initialize(&connection).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        12
    );
    assert!(
        connection
            .prepare("SELECT successor_requested FROM speech_tasks")
            .is_err()
    );
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        original
    );
    assert_eq!(get(&connection, 1).unwrap().notes, "keep");
}

#[test]
fn retrying_the_same_audio_operation_does_not_create_a_successor() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let first =
        request_selected_speech(&mut connection, &spec, 1, 1000, true, Some("replace-1")).unwrap();
    claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    let retry =
        request_selected_speech(&mut connection, &spec, 1, 1001, true, Some("replace-1")).unwrap();
    assert_eq!(
        retry, first,
        "a lost-response retry must keep its original selection intent"
    );
    assert!(
        !connection
            .query_row("SELECT successor_requested FROM speech_tasks", [], |r| r
                .get::<_, bool>(
                0
            ))
            .unwrap()
    );
    assert!(
        request_selected_speech(&mut connection, &spec, 1, 1002, false, Some("replace-1")).is_err()
    );
    let other = SpeechSpec::word("école", SpeechVoice::Male);
    assert!(
        request_selected_speech(&mut connection, &other, 1, 1002, true, Some("replace-1")).is_err()
    );
    let latest =
        request_selected_speech(&mut connection, &spec, 1, 1002, true, Some("replace-2")).unwrap();
    assert_eq!(latest.1, first.1 + 1);
}

#[test]
fn failed_or_cancelled_publication_cannot_replace_the_selected_audio() {
    for cancelled in [false, true] {
        let mut connection = fixture();
        let original = prepare_audio_version(
            &connection,
            1,
            None,
            &content_hash(b"OggSoriginal"),
            12,
            None,
            0,
            999,
        )
        .unwrap();
        complete_audio_version(&connection, &original.locator, 999).unwrap();
        let spec = SpeechSpec::word("école", SpeechVoice::Female);
        let (key, generation) =
            request_selected_speech(&mut connection, &spec, 1, 1000, true, None).unwrap();
        let task = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
        let pending = prepare_audio_version(
            &connection,
            1,
            Some(&key),
            &content_hash(b"OggSpending"),
            11,
            Some(1001),
            generation,
            1001,
        )
        .unwrap();
        if cancelled {
            connection
                .execute(
                    "UPDATE speech_tasks SET cancel_requested=1 WHERE id=?1",
                    [task.id],
                )
                .unwrap();
        }
        fail_speech(&connection, task.id, 1002, "publication interrupted", true).unwrap();
        assert!(
            !complete_audio_version(&connection, &pending.locator, 1003).unwrap(),
            "terminal execution must only register unbound content"
        );
        assert_eq!(
            selected_audio(&connection, 1).unwrap().unwrap().locator,
            original.locator
        );
        assert!(
            pending_audio_versions(&connection, "").unwrap().is_empty(),
            "recovery must finish the abandoned intent"
        );
    }
}

#[test]
fn a_previous_execution_cannot_bind_after_a_retry_claims_the_same_demand() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let (key, generation) =
        request_selected_speech(&mut connection, &spec, 1, 1000, true, None).unwrap();
    let first = claim_speech(&mut connection, 1000, 2).unwrap().unwrap();
    let pending = prepare_audio_version(
        &connection,
        1,
        Some(&key),
        &content_hash(b"OggSpending"),
        11,
        Some(1001),
        generation,
        1001,
    )
    .unwrap();
    fail_speech(&connection, first.id, 1002, "interrupted", false).unwrap();
    claim_speech(&mut connection, 1004, 2).unwrap().unwrap();
    publish_speech(&connection, &key, &content_hash(b"OggSretry"), 1005).unwrap();
    assert!(
        !complete_audio_version(&connection, &pending.locator, 1006).unwrap(),
        "old bytes must not borrow a later execution identity"
    );
    assert!(selected_audio(&connection, 1).unwrap().is_none());
}

#[test]
fn verified_content_can_outlive_its_creating_word_and_keep_another_binding() {
    let connection = fixture();
    let version = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSshared"),
        10,
        None,
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &version.locator, 1000).unwrap();
    connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) SELECT 2,'shared',snapshot,'keep other word',1,1 FROM vocabulary WHERE id=1", []).unwrap();
    connection
        .execute(
            "INSERT INTO word_audio_selections(vocabulary_id,current_version) VALUES(2,?1)",
            [&version.locator],
        )
        .unwrap();
    connection
        .execute(
            "DELETE FROM word_audio_selections WHERE vocabulary_id=1",
            [],
        )
        .unwrap();
    connection
        .execute("DELETE FROM vocabulary WHERE id=1", [])
        .unwrap();
    assert_eq!(
        selected_audio(&connection, 2).unwrap().unwrap().locator,
        version.locator
    );
    assert_eq!(get(&connection, 2).unwrap().notes, "keep other word");
}

#[test]
fn shared_ownership_previous_protection_and_last_detachment_use_exact_deadlines() {
    let mut connection = fixture();
    let first = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSfirst"),
        9,
        None,
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &first.locator, 1000).unwrap();
    connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) SELECT 2,'shared',snapshot,'other',1,1 FROM vocabulary WHERE id=1",[]).unwrap();
    assert!(reuse_audio_version(&connection, 2, &first.locator, 0, 1001).unwrap());
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let generation = begin_audio_change(&connection, 1, &key).unwrap();
    let second = prepare_audio_version(
        &connection,
        1,
        Some(&key),
        &content_hash(b"OggSsecond"),
        10,
        Some(1002),
        generation,
        1002,
    )
    .unwrap();
    complete_audio_version(&connection, &second.locator, 1002).unwrap();
    assert!(
        mark_audio_gc(&connection, &first.locator, 2_000_000)
            .unwrap()
            .is_none(),
        "another current owner is permanent"
    );
    detach_word_audio(&mut connection, 2, 2_000_000).unwrap();
    let until = connection
        .query_row(
            "SELECT orphan_until FROM speech_versions WHERE locator=?1",
            [&first.locator],
            |r| r.get::<_, i64>(0),
        )
        .unwrap();
    assert_eq!(until, 2_604_800);
    detach_word_audio(&mut connection, 2, 2_000_001).unwrap();
    assert!(
        mark_audio_gc(&connection, &first.locator, until - 1)
            .unwrap()
            .is_none()
    );
    assert!(
        mark_audio_gc(&connection, &first.locator, until)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        second.locator
    );
    assert_eq!(get(&connection, 1).unwrap().notes, "keep");
    assert!(review_cards(&connection, 1).unwrap().is_empty());
}

#[test]
fn releasing_an_older_previous_version_preserves_other_protection_without_renewing_it() {
    let connection = fixture();
    let first = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSfirst"),
        9,
        None,
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &first.locator, 1000).unwrap();
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let mut versions = vec![first.locator.clone()];
    for at in [1001, 1002] {
        let generation = begin_audio_change(&connection, 1, &key).unwrap();
        let v = prepare_audio_version(
            &connection,
            1,
            Some(&key),
            &content_hash(format!("OggS{at}").as_bytes()),
            8,
            Some(at),
            generation,
            at,
        )
        .unwrap();
        complete_audio_version(&connection, &v.locator, at).unwrap();
        versions.push(v.locator);
    }
    assert!(
        mark_audio_gc(&connection, &versions[0], 1001 + AUDIO_TEMP_SECONDS - 1)
            .unwrap()
            .is_none()
    );
    assert!(
        mark_audio_gc(&connection, &versions[0], 1001 + AUDIO_TEMP_SECONDS)
            .unwrap()
            .is_some()
    );
    assert!(
        mark_audio_gc(&connection, &versions[1], 1002 + AUDIO_PREVIOUS_SECONDS - 1)
            .unwrap()
            .is_none()
    );
    assert!(
        mark_audio_gc(&connection, &versions[1], 1002 + AUDIO_PREVIOUS_SECONDS)
            .unwrap()
            .is_some()
    );
    assert!(
        mark_audio_gc(&connection, &versions[2], i64::MAX)
            .unwrap()
            .is_none()
    );
}

#[test]
fn identical_republication_uses_a_new_physical_identity_and_ignores_late_gc() {
    let connection = fixture();
    let key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let hash = content_hash(b"OggScontent");
    let first = prepare_source_audio(&connection, &key, &hash, 11, 1000).unwrap();
    verify_audio_content(&connection, &first.locator, 11).unwrap();
    complete_source_audio(&connection, &first.locator).unwrap();
    let gc = mark_audio_gc(&connection, &first.locator, 605800)
        .unwrap()
        .unwrap();
    assert!(!reuse_audio_version(&connection, 1, &first.locator, 0, 605800).unwrap());
    assert!(!finish_audio_gc(&connection, &first.locator, gc.generation + 1).unwrap());
    let next = prepare_source_audio(&connection, &key, &hash, 11, 605801).unwrap();
    assert_ne!(next.locator, first.locator);
    assert_ne!(next.physical_locator, first.physical_locator);
    verify_audio_content(&connection, &next.locator, 11).unwrap();
    complete_source_audio(&connection, &next.locator).unwrap();
    assert!(finish_audio_gc(&connection, &first.locator, gc.generation).unwrap());
    assert_eq!(
        speech_asset(&connection, &key, 605801)
            .unwrap()
            .unwrap()
            .locator,
        next.locator
    );
    assert!(reuse_audio_version(&connection, 1, &next.locator, 0, 605801).unwrap());
    assert!(!finish_audio_gc(&connection, &first.locator, gc.generation).unwrap());
    assert!(
        connection
            .execute(
                "UPDATE speech_versions SET gc_state='live' WHERE locator=?1",
                [&first.locator]
            )
            .is_err()
    );
}

#[test]
fn pending_publication_prevents_gc_and_missing_intent_can_be_retired() {
    let connection = fixture();
    let pending = prepare_audio_version(
        &connection,
        1,
        None,
        &content_hash(b"OggSpending"),
        11,
        None,
        0,
        1000,
    )
    .unwrap();
    assert!(
        mark_audio_gc(&connection, &pending.locator, 2_000_000)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        audio_usage(&connection, 1000).unwrap().publication_bytes,
        11
    );
    abandon_audio_publication(&connection, &pending.locator, Some("Missing file")).unwrap();
    assert!(pending_audio_versions(&connection, "").unwrap().is_empty());
    assert!(
        mark_audio_gc(&connection, &pending.locator, 2_000_000)
            .unwrap()
            .is_some()
    );
}

#[test]
fn soft_capacity_evicts_unprotected_content_without_changing_expiry_or_permanent_audio() {
    let connection = fixture();
    let permanent = prepare_audio_version(
        &connection,
        1,
        None,
        &"a".repeat(64),
        10_000_000,
        None,
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &permanent.locator, 1000).unwrap();
    for index in 0..23 {
        let spec = SpeechSpec::word(&format!("temporary-{index}"), SpeechVoice::Female);
        let key = register_speech(&connection, &spec).unwrap();
        let version =
            prepare_source_audio(&connection, &key, &"b".repeat(64), 10_000_000, 1000 + index)
                .unwrap();
        verify_audio_content(&connection, &version.locator, 10_000_000).unwrap();
        complete_source_audio(&connection, &version.locator).unwrap();
        audio_last_used(&connection, &version.locator, 2000).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT expires_at FROM speech_versions WHERE locator=?1",
                    [&version.locator],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1000 + index + AUDIO_TEMP_SECONDS
        );
    }
    let usage = audio_usage(&connection, 2000).unwrap();
    assert_eq!(usage.permanent_bytes, 10_000_000);
    assert_eq!(usage.unowned_bytes, 230_000_000);
    assert!(usage.over_capacity);
    assert!(
        mark_audio_gc(&connection, &permanent.locator, 2000)
            .unwrap()
            .is_none()
    );
    let work = mark_audio_gc_batch(&connection, 2000, 100).unwrap();
    assert_eq!(
        work.len(),
        3,
        "A prepared batch must mark only enough temporary bytes to reach the target"
    );
    assert!(
        mark_audio_gc_batch(&connection, 2000, 100)
            .unwrap()
            .is_empty(),
        "Pending deletion already accounts for enough bytes"
    );
    for gc in work {
        finish_audio_gc(&connection, &gc.version.locator, gc.generation).unwrap();
    }
    assert!(!audio_usage(&connection, 2000).unwrap().over_capacity);
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().locator,
        permanent.locator
    );
}

#[test]
fn schema_fourteen_rebuild_preserves_bindings_pending_intents_and_rolls_back_atomically() {
    let connection = old_fixture(14);
    let key = SpeechSpec::word("école", SpeechVoice::Female)
        .key()
        .unwrap();
    let spec = serde_json::to_string(&SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    connection.execute("INSERT INTO speech_assets(semantic_key,spec,context,content_hash,generated_at,expires_at,pending_hash,pending_generated_at,pending_execution) VALUES(?1,?2,'word',?3,1000,605800,?4,1001,3)",params![key,spec,"d".repeat(64),"e".repeat(64)]).unwrap();
    connection.execute("INSERT INTO speech_tasks(semantic_key,priority,state,wake_at,execution_generation) VALUES(?1,0,'failed',1000,3)",[&key]).unwrap();
    connection.execute("INSERT INTO speech_versions(locator,content_hash,byte_length,created_at,vocabulary_id,expected_revision,generation,sealed) VALUES(?1,?2,8,1000,1,0,7,1)",params!["a".repeat(64),"b".repeat(64)]).unwrap();
    connection.execute("INSERT INTO word_audio_selections(vocabulary_id,current_version,revision,desired_generation) VALUES(1,?1,2,7)",["a".repeat(64)]).unwrap();
    connection.commit_hook(Some(|| true)).unwrap();
    assert!(initialize(&connection).is_err());
    connection.commit_hook(None::<fn() -> bool>).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        14
    );
    assert!(
        connection
            .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
            .unwrap()
    );
    assert!(
        connection
            .prepare("SELECT physical_locator FROM speech_versions")
            .is_err()
    );
    assert_eq!(
        initialize(&connection).unwrap(),
        lexique_core::DATABASE_SCHEMA_VERSION
    );
    assert!(
        !connection
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
    assert_eq!(get(&connection, 1).unwrap().notes, "keep");
    assert_eq!(
        selected_audio(&connection, 1).unwrap().unwrap().generation,
        7
    );
    let pending = pending_source_audio(&connection, "").unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].execution_generation, Some(3));
    assert_eq!(pending[0].import_locator.as_deref(), Some(key.as_str()));
    assert_eq!(pending[0].physical_locator, pending[0].locator);
    verify_audio_content(&connection, &pending[0].locator, 8).unwrap();
    assert!(
        !complete_source_audio(&connection, &pending[0].locator).unwrap(),
        "terminal execution stays unbound"
    );
    assert!(pending_source_audio(&connection, "").unwrap().is_empty());
    assert_eq!(
        speech_asset(&connection, &key, 1002)
            .unwrap()
            .unwrap()
            .content_hash,
        "d".repeat(64)
    );
}
