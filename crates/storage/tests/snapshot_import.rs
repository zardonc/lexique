use lexique_core::scheduling::{Rating, Scheduled};
use lexique_storage::*;
use rusqlite::{Connection, params};

const CAPTURED_AT: i64 = 1_800_000_100;

fn saved_word(connection: &mut Connection, operation: &str, headword: &str) -> i64 {
    let id = save(
        connection,
        &SaveRequest {
            operation: operation.into(),
            headword: headword.into(),
            notes: "  Une note\navec son espacement.  ".into(),
            example_fr: "L’école est ouverte.".into(),
            example_zh: "学校开着。".into(),
            example_source: None,
            example_sense: Some(0),
            more_examples: vec![Example {
                fr: "Une autre école.".into(),
                zh: String::new(),
                source: None,
            }],
            id: None,
            revision: None,
        },
        Some(Snapshot {
            reference: Reference {
                dictionary_id: Some("mdx-99".into()),
                dictionary: "Removed dictionary".into(),
                lookup_key: headword.into(),
                version: "original".into(),
                attribution: None,
                license: None,
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![DigestEntry {
                pronunciations: vec!["ekɔl".into()],
                grammar: vec!["n.f.".into()],
                translation: Some("学校".into()),
            }],
        }),
    )
    .unwrap()
    .record
    .id;
    connection
        .execute(
            "UPDATE vocabulary SET created_at=?1,updated_at=?2 WHERE id=?3",
            params![1_800_000_000_123_i64, 1_800_000_001_456_i64, id],
        )
        .unwrap();
    id
}

#[test]
fn imported_record_and_preferences_are_readable_and_saved_operations_remain_idempotent() {
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    let id = saved_word(&mut source, "save-original", "école");
    set_interface_preferences(
        &source,
        &InterfacePreferences {
            locale: InterfaceLocale::Fr,
            theme: InterfaceTheme::Dark,
        },
    )
    .unwrap();
    source
        .execute(
            "INSERT INTO app_metadata(key,value) VALUES('private-service','private-device-value')",
            [],
        )
        .unwrap();
    let snapshot = export_learning_snapshot(&mut source, CAPTURED_AT).unwrap();
    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    import_learning_snapshot(&mut target, &snapshot, CAPTURED_AT).unwrap();

    let record = get(&target, id).unwrap();
    assert_eq!(record.headword, "école");
    assert_eq!(record.notes, "  Une note\navec son espacement.  ");
    assert_eq!(record.reference.dictionary_id.as_deref(), Some("mdx-99"));
    assert_eq!(record.digest[0].translation.as_deref(), Some("学校"));
    assert_eq!(record.example_fr, "L’école est ouverte.");
    assert_eq!(record.example_zh, "学校开着。");
    assert_eq!(record.example_sense, Some(0));
    assert_eq!(record.more_examples[0].fr, "Une autre école.");
    assert!(record.example_source.is_none() && record.more_examples[0].source.is_none());
    assert_eq!(record.created_at, 1_800_000_000_123);
    assert_eq!(record.updated_at, 1_800_000_001_456);
    assert_eq!(record.revision, 1);
    let replayed = save(&mut target, &snapshot.save_operations[0].request, None).unwrap();
    assert_eq!(replayed.outcome, "replayed");
    assert_eq!(replayed.record.id, id);
    assert_eq!(list(&target, "", "headword-asc").unwrap().len(), 1);
    assert!(matches!(
        interface_preferences(&target).unwrap().locale,
        InterfaceLocale::Fr
    ));
    assert!(matches!(
        interface_preferences(&target).unwrap().theme,
        InterfaceTheme::Dark
    ));
    assert_eq!(initialize(&target).unwrap(), 18);
    assert!(
        !target
            .prepare("SELECT 1 FROM app_metadata WHERE key='private-service'")
            .unwrap()
            .exists([])
            .unwrap()
    );
}

fn rate(
    connection: &mut Connection,
    operation: &str,
    card_id: i64,
    rating: Rating,
    now: i64,
) -> serde_json::Value {
    let expected_revision = review_progress(connection, card_id).unwrap().revision;
    rate_card(
        connection,
        &RatingRequest {
            operation: operation.into(),
            card_id,
            expected_revision,
            rating,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap()
}

fn queue_rows(connection: &Connection) -> Vec<(i64, i64)> {
    connection
        .prepare("SELECT card_id,position FROM review_queue_entries ORDER BY position")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn imported_equal_time_history_keeps_undo_order_quota_queue_and_absent_progress() {
    const NOW: i64 = 1_794_564_000;
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    let ecole = saved_word(&mut source, "save-ecole", "école");
    let etre = saved_word(&mut source, "save-etre", "être");
    let suivre = saved_word(&mut source, "save-suivre", "suivre");
    let double = saved_word(&mut source, "save-double", "double");
    let ecole_cards = add_review(&mut source, ecole).unwrap();
    let etre_cards = add_review(&mut source, etre).unwrap();
    let suivre_cards = add_review(&mut source, suivre).unwrap();
    let double_cards = add_review(&mut source, double).unwrap();
    set_review_settings(
        &source,
        &ReviewSettings {
            retention_percent: 87,
            new_limit: 2,
            review_limit: 3,
            new_cards_ignore_review_limit: false,
        },
    )
    .unwrap();
    rate(
        &mut source,
        "old-review",
        suivre_cards[0].id,
        Rating::Easy,
        NOW - 86_400,
    );
    let first = rate(&mut source, "z-new", ecole_cards[0].id, Rating::Good, NOW);
    rate(
        &mut source,
        "a-undone",
        etre_cards[0].id,
        Rating::Again,
        NOW,
    );
    assert!(!undo_rating(&mut source, "undo-original", "a-undone").unwrap());
    set_review_paused(&mut source, double, true).unwrap();
    source
        .execute(
            "INSERT INTO review_queue_state(id,learning_day,timezone) VALUES(1,'2026-11-13','UTC')",
            [],
        )
        .unwrap();
    for (card, position) in [
        (ecole_cards[1].id, 1_000_000),
        (suivre_cards[0].id, 2_000_000),
        (double_cards[1].id, 3_000_000),
    ] {
        source
            .execute(
                "INSERT INTO review_queue_entries(card_id,position) VALUES(?1,?2)",
                params![card, position],
            )
            .unwrap();
    }
    let latest = rate(
        &mut source,
        "m-latest",
        ecole_cards[1].id,
        Rating::Again,
        NOW,
    );
    let snapshot = export_learning_snapshot(&mut source, NOW + 1).unwrap();
    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    import_learning_snapshot(&mut target, &snapshot, NOW + 1).unwrap();

    assert_eq!(review_cards(&target, ecole).unwrap(), ecole_cards);
    assert_eq!(review_cards(&target, etre).unwrap(), etre_cards);
    assert_eq!(review_cards(&target, suivre).unwrap(), suivre_cards);
    assert_eq!(review_cards(&target, double).unwrap(), double_cards);
    let order: Vec<String> = target
        .prepare("SELECT operation FROM review_ratings ORDER BY rowid")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(order, ["old-review", "z-new", "a-undone", "m-latest"]);
    let saved_result: String = target
        .query_row(
            "SELECT result FROM review_ratings WHERE operation='z-new'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&saved_result).unwrap(),
        first
    );
    assert_eq!(
        review_progress(&target, ecole_cards[0].id)
            .unwrap()
            .card
            .due_at,
        Some(NOW + 600)
    );
    assert_eq!(
        serde_json::to_value(review_progress(&target, ecole_cards[1].id).unwrap().card).unwrap(),
        serde_json::to_value(
            serde_json::from_value::<Scheduled>(latest.clone())
                .unwrap()
                .card
        )
        .unwrap()
    );
    let absent: Vec<i64> = target.prepare("SELECT c.id FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE p.card_id IS NULL ORDER BY c.id")
        .unwrap().query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(
        absent,
        [
            etre_cards[1].id,
            suivre_cards[1].id,
            double_cards[0].id,
            double_cards[1].id
        ]
    );
    assert_eq!(
        queue_rows(&target),
        [
            (suivre_cards[0].id, 2_000_000),
            (double_cards[1].id, 3_000_000)
        ]
    );
    assert_eq!(
        review_availability(&target, etre_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert_eq!(
        review_availability(&target, double_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("paused")
    );
    let restored_rating = &snapshot.ratings[3];
    assert_eq!(
        rate_card(&mut target, &restored_rating.request).unwrap(),
        latest
    );
    assert_eq!(
        review_progress(&target, ecole_cards[1].id)
            .unwrap()
            .revision,
        1
    );
    assert!(
        rate_card(
            &mut target,
            &RatingRequest {
                operation: "m-latest".into(),
                card_id: ecole_cards[1].id,
                expected_revision: 0,
                rating: Rating::Easy,
                now: NOW,
                timezone: "UTC".into(),
            }
        )
        .is_err()
    );
    assert_eq!(undo_target(&target).unwrap().as_deref(), Some("m-latest"));
    assert!(!undo_rating(&mut target, "undo-restored", "m-latest").unwrap());
    assert_eq!(
        queue_rows(&target),
        [
            (ecole_cards[1].id, 1_000_000),
            (suivre_cards[0].id, 2_000_000),
            (double_cards[1].id, 3_000_000)
        ]
    );
    assert_eq!(undo_target(&target).unwrap(), None);
    assert!(!undo_rating(&mut target, "undo-restored", "m-latest").unwrap());
    assert!(undo_rating(&mut target, "undo-restored", "z-new").is_err());
    assert_eq!(
        review_availability(&target, etre_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason,
        None
    );
    assert_eq!(
        review_progress(&target, ecole_cards[1].id)
            .unwrap()
            .revision,
        2
    );
    assert_eq!(
        serde_json::to_value(review_progress(&target, ecole_cards[1].id).unwrap().card).unwrap(),
        serde_json::json!({"phase":"new","memory":null,"last_review_at":null,"due_at":null})
    );
    let next = rate(
        &mut target,
        "after-restore",
        ecole_cards[1].id,
        Rating::Good,
        NOW + 2,
    );
    assert_eq!(next["card"]["due_at"], NOW + 602);
}

#[test]
fn imported_partial_batches_keep_voice_identity_failures_and_speech_resume_budgets() {
    const NOW: i64 = 1_794_564_000;
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    let ecole = saved_word(&mut source, "save-ecole", "école");
    let etre = saved_word(&mut source, "save-etre", "être");
    let suivre = saved_word(&mut source, "save-suivre", "suivre");
    add_review_batch(&mut source, "z-first", &[ecole, 99_999, etre, ecole], NOW).unwrap();
    set_speech_preferences(
        &source,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            wpm: 140,
        },
    )
    .unwrap();
    add_review_batch(&mut source, "a-last", &[suivre, ecole], NOW).unwrap();
    source
        .execute(
            "DELETE FROM review_batch_items WHERE operation='a-last' AND vocabulary_id=?1",
            [ecole],
        )
        .unwrap();
    cancel_batch(&source, "a-last").unwrap();
    let female_key = SpeechSpec::word("école", SpeechVoice::Female)
        .key()
        .unwrap();
    let retry_key = SpeechSpec::word("être", SpeechVoice::Female).key().unwrap();
    let cancelled_key = SpeechSpec::word("suivre", SpeechVoice::Male).key().unwrap();
    source
        .execute(
            "UPDATE speech_tasks SET state='running',attempts=2,wake_at=?1,deadline_at=?2,
         cancel_requested=0,refresh_requested=1,successor_requested=1,interactive_requested=1,
         execution_generation=7,demand_class=1,requested_at=?3,error='interrupted request'
         WHERE semantic_key=?4",
            params![NOW, NOW + 90, NOW - 30, female_key],
        )
        .unwrap();
    source
        .execute(
            "UPDATE speech_tasks SET state='retry_wait',attempts=3,wake_at=?1,deadline_at=?2,
         execution_generation=4,demand_class=2,requested_at=?3,error='model unavailable'
         WHERE semantic_key=?4",
            params![NOW + 8, NOW + 45, NOW - 20, retry_key],
        )
        .unwrap();
    source.execute(
        "UPDATE speech_tasks SET attempts=4,deadline_at=?1,error='cancelled demand' WHERE semantic_key=?2",
        params![NOW - 1, cancelled_key],
    ).unwrap();
    let first_status = serde_json::to_value(batch_status(&source, "z-first").unwrap()).unwrap();
    let last_status = serde_json::to_value(batch_status(&source, "a-last").unwrap()).unwrap();
    let snapshot = export_learning_snapshot(&mut source, NOW + 1).unwrap();
    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    import_learning_snapshot(&mut target, &snapshot, NOW + 1).unwrap();

    assert_eq!(
        serde_json::to_value(batch_status(&target, "z-first").unwrap()).unwrap(),
        first_status
    );
    assert_eq!(
        serde_json::to_value(latest_batch(&target).unwrap().unwrap()).unwrap(),
        last_status
    );
    let first = batch_status(&target, "z-first").unwrap();
    assert_eq!(
        first
            .items
            .iter()
            .map(|item| item.vocabulary_id)
            .collect::<Vec<_>>(),
        [ecole, 99_999, etre]
    );
    assert_eq!(first.items[0].created_cards, 2);
    assert_eq!(first.items[1].created_cards, 0);
    assert_eq!(
        first.items[1].error.as_deref(),
        Some("SQLite operation failed: Query returned no rows")
    );
    assert_eq!(first.items[2].audio_state.as_deref(), Some("retry_wait"));
    assert_eq!(
        first.items[2].audio_error.as_deref(),
        Some("model unavailable")
    );
    assert!(word_audio_keys(&target, ecole).unwrap().is_empty());
    assert_eq!(speech_owner(&target, &female_key).unwrap(), Some(ecole));
    assert_eq!(
        speech_owner(
            &target,
            &SpeechSpec::word("école", SpeechVoice::Male).key().unwrap()
        )
        .unwrap(),
        Some(ecole)
    );
    assert_eq!(
        speech_audio_voice(&target, Some(&female_key)).unwrap(),
        Some(SpeechVoice::Female)
    );
    assert_eq!(
        speech_audio_voice(&target, Some(&cancelled_key)).unwrap(),
        Some(SpeechVoice::Male)
    );
    assert_eq!(speech_preferences(&target).unwrap().wpm, 140);
    let interrupted = interrupted_speech(&target).unwrap();
    assert_eq!(interrupted.len(), 1);
    assert_eq!(interrupted[0].1, female_key);
    let task: (i64, i64, bool, bool, bool, bool, i64, i64, i64) = target.query_row(
        "SELECT attempts,deadline_at,cancel_requested,refresh_requested,successor_requested,
         interactive_requested,execution_generation,demand_class,requested_at FROM speech_tasks WHERE semantic_key=?1",
        [&female_key], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?,row.get(7)?,row.get(8)?)),
    ).unwrap();
    assert_eq!(task, (2, NOW + 90, false, true, true, true, 7, 1, NOW - 30));
    let retry: (i64,i64,i64,i64) = target.query_row(
        "SELECT attempts,wake_at,deadline_at,execution_generation FROM speech_tasks WHERE semantic_key=?1",
        [&retry_key], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
    ).unwrap();
    assert_eq!(retry, (3, NOW + 8, NOW + 45, 4));
    assert_eq!(
        speech_task_state(&target, &cancelled_key).unwrap(),
        Some(("cancelled".into(), Some("cancelled demand".into())))
    );
    let cancelled = latest_batch(&target).unwrap().unwrap();
    assert!(cancelled.cancelled);
    assert_eq!(cancelled.items.len(), 1);
    assert_eq!(
        add_review_batch(&mut target, "a-last", &[suivre, ecole], NOW + 2)
            .unwrap()
            .items
            .len(),
        2
    );
    assert!(add_review_batch(&mut target, "a-last", &[ecole, suivre], NOW + 2).is_err());
    resume_speech(&target, interrupted[0].0, NOW + 2).unwrap();
    let resumed = claim_speech(&mut target, NOW + 2, 2).unwrap().unwrap();
    assert_eq!(resumed.semantic_key, female_key);
    assert_eq!(resumed.attempts, 1);
    assert_eq!(resumed.deadline_at, NOW + 122);
    assert_eq!(resumed.execution_generation, 8);
    assert!(resumed.refresh);
    assert_eq!(
        speech_task_state(&target, &cancelled_key)
            .unwrap()
            .unwrap()
            .0,
        "cancelled"
    );
}

#[test]
fn imported_audio_keeps_shared_selection_legacy_nulls_absolute_protection_and_unpublished_intent() {
    const NOW: i64 = 1_794_564_000;
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    let ecole = saved_word(&mut source, "save-ecole", "école");
    let etre = saved_word(&mut source, "save-etre", "être");
    let suivre = saved_word(&mut source, "save-suivre", "suivre");
    let double = saved_word(&mut source, "save-double", "double");
    let venir = saved_word(&mut source, "save-venir", "venir");
    let finir = saved_word(&mut source, "save-finir", "finir");
    let ouvrir = saved_word(&mut source, "save-ouvrir", "ouvrir");
    for id in [ecole, etre, suivre, double, venir, finir, ouvrir] {
        add_review(&mut source, id).unwrap();
    }
    let mut snapshot = export_learning_snapshot(&mut source, NOW + 1).unwrap();
    let follow_spec = SpeechSpec::word("suivre", SpeechVoice::Female);
    let follow_key = follow_spec.key().unwrap();
    let double_spec = SpeechSpec::word("double", SpeechVoice::Male);
    let double_key = double_spec.key().unwrap();
    let shared = "a".repeat(64);
    let previous = "b".repeat(64);
    let expired = "c".repeat(64);
    let missing = "d".repeat(64);
    let pending = "e".repeat(64);
    let invalid = "f".repeat(64);
    let deleted = "0".repeat(64);
    let version = |locator: &str, key: Option<&str>, hash: &str| SnapshotAudioVersion {
        locator: locator.into(),
        semantic_key: key.map(str::to_owned),
        content_hash: hash.into(),
        byte_length: Some(640),
        generated_at: Some(NOW - 3600),
        created_at: NOW - 3600,
        origin_vocabulary_id: None,
        expected_revision: 0,
        generation: 0,
        sealed: true,
        execution_generation: None,
        context: "word".into(),
        verification: "verified".into(),
        gc_state: "live".into(),
        expires_at: Some(NOW + 601_200),
        orphan_until: None,
        payload_status: AudioPayloadStatus::Included,
        exclusion_reason: None,
    };
    let mut current_version = version(&shared, None, &shared);
    current_version.generated_at = None;
    current_version.created_at = 0;
    current_version.expires_at = None;
    current_version.orphan_until = Some(NOW + 99);
    let protected_version = version(&previous, None, &previous);
    let mut expired_version = version(&expired, None, &expired);
    expired_version.generated_at = Some(NOW - 1_400_000);
    expired_version.created_at = NOW - 1_400_000;
    expired_version.expires_at = Some(NOW - 795_200);
    expired_version.origin_vocabulary_id = Some(99_999);
    expired_version.gc_state = "marked".into();
    expired_version.payload_status = AudioPayloadStatus::Excluded;
    expired_version.exclusion_reason = Some(AudioExclusionReason::Expired);
    let mut missing_version = version(&missing, Some(&follow_key), &shared);
    missing_version.byte_length = None;
    missing_version.generated_at = None;
    missing_version.created_at = 0;
    missing_version.expires_at = None;
    missing_version.orphan_until = Some(NOW + 123);
    missing_version.verification = "missing".into();
    missing_version.payload_status = AudioPayloadStatus::SourceMissing;
    let mut pending_version = version(&pending, Some(&follow_key), &shared);
    pending_version.generated_at = Some(NOW - 20);
    pending_version.created_at = NOW - 20;
    pending_version.expires_at = Some(NOW + 604_780);
    pending_version.origin_vocabulary_id = Some(suivre);
    pending_version.expected_revision = 5;
    pending_version.generation = 3;
    pending_version.execution_generation = Some(7);
    pending_version.payload_status = AudioPayloadStatus::Excluded;
    pending_version.exclusion_reason = Some(AudioExclusionReason::Unpublished);
    let mut invalid_version = version(&invalid, Some(&double_key), &shared);
    invalid_version.verification = "invalid".into();
    invalid_version.payload_status = AudioPayloadStatus::SourceInvalid;
    let mut deleted_version = version(&deleted, Some(&double_key), &deleted);
    deleted_version.gc_state = "deleted".into();
    deleted_version.origin_vocabulary_id = Some(99_999);
    deleted_version.payload_status = AudioPayloadStatus::Excluded;
    deleted_version.exclusion_reason = Some(AudioExclusionReason::NotSelected);
    snapshot.audio_versions = vec![
        current_version,
        protected_version,
        expired_version,
        missing_version,
        pending_version,
        invalid_version,
        deleted_version,
    ];
    snapshot.speech_sources = vec![
        SnapshotSpeechSource {
            semantic_key: follow_key.clone(),
            spec: follow_spec,
            context: "word".into(),
            current_version: Some(missing.clone()),
            pending_version: Some(pending.clone()),
            retain: true,
        },
        SnapshotSpeechSource {
            semantic_key: double_key.clone(),
            spec: double_spec,
            context: "word".into(),
            current_version: Some(invalid.clone()),
            pending_version: None,
            retain: false,
        },
    ];
    snapshot.speech_owners = vec![
        SnapshotSpeechOwner {
            semantic_key: follow_key.clone(),
            vocabulary_id: suivre,
        },
        SnapshotSpeechOwner {
            semantic_key: double_key.clone(),
            vocabulary_id: double,
        },
    ];
    snapshot.speech_tasks = vec![
        SnapshotSpeechTask {
            id: 91,
            semantic_key: follow_key.clone(),
            priority: 0,
            state: "running".into(),
            attempts: 3,
            wake_at: NOW - 10,
            deadline_at: Some(NOW + 20),
            cancel_requested: true,
            refresh_requested: false,
            successor_requested: false,
            interactive_requested: true,
            execution_generation: 7,
            demand_class: 2,
            requested_at: NOW - 15,
            error: Some("publication interrupted".into()),
        },
        SnapshotSpeechTask {
            id: 92,
            semantic_key: double_key.clone(),
            priority: 1,
            state: "failed".into(),
            attempts: 4,
            wake_at: NOW - 30,
            deadline_at: Some(NOW - 1),
            cancel_requested: false,
            refresh_requested: true,
            successor_requested: false,
            interactive_requested: false,
            execution_generation: 8,
            demand_class: 3,
            requested_at: NOW - 40,
            error: Some("budget exhausted".into()),
        },
    ];
    snapshot.audio_selections = vec![
        SnapshotAudioSelection {
            vocabulary_id: ecole,
            current_version: Some(shared.clone()),
            previous_version: Some(previous.clone()),
            previous_until: Some(NOW + 40),
            revision: 10,
            desired_generation: 0,
            desired_key: None,
            current_generation: 0,
        },
        SnapshotAudioSelection {
            vocabulary_id: etre,
            current_version: Some(shared.clone()),
            previous_version: Some(expired.clone()),
            previous_until: Some(NOW - 1),
            revision: 11,
            desired_generation: 0,
            desired_key: None,
            current_generation: 0,
        },
        SnapshotAudioSelection {
            vocabulary_id: suivre,
            current_version: Some(missing.clone()),
            previous_version: None,
            previous_until: None,
            revision: 5,
            desired_generation: 3,
            desired_key: Some(follow_key.clone()),
            current_generation: 0,
        },
        SnapshotAudioSelection {
            vocabulary_id: double,
            current_version: Some(invalid.clone()),
            previous_version: None,
            previous_until: None,
            revision: 2,
            desired_generation: 2,
            desired_key: Some(double_key.clone()),
            current_generation: 1,
        },
    ];
    snapshot.audio_imports = vec![
        SnapshotAudioImport {
            vocabulary_id: ecole,
            state: "finished".into(),
            error: None,
        },
        SnapshotAudioImport {
            vocabulary_id: venir,
            state: "pending".into(),
            error: None,
        },
        SnapshotAudioImport {
            vocabulary_id: finir,
            state: "failed".into(),
            error: Some("original file absent".into()),
        },
    ];
    snapshot.legacy_audio = vec![
        SnapshotLegacyAudio {
            vocabulary_id: venir,
            content_hash: shared.clone(),
            payload_status: AudioPayloadStatus::Included,
            exclusion_reason: None,
        },
        SnapshotLegacyAudio {
            vocabulary_id: finir,
            content_hash: shared.clone(),
            payload_status: AudioPayloadStatus::SourceMissing,
            exclusion_reason: None,
        },
        SnapshotLegacyAudio {
            vocabulary_id: ouvrir,
            content_hash: shared.clone(),
            payload_status: AudioPayloadStatus::SourceInvalid,
            exclusion_reason: None,
        },
        SnapshotLegacyAudio {
            vocabulary_id: etre,
            content_hash: previous.clone(),
            payload_status: AudioPayloadStatus::Excluded,
            exclusion_reason: Some(AudioExclusionReason::NotSelected),
        },
    ];
    snapshot.audio_operations = vec![SnapshotAudioOperation {
        operation: "change-original".into(),
        vocabulary_id: suivre,
        semantic_key: follow_key.clone(),
        generation: 3,
        refresh: true,
    }];
    snapshot.audio_publications = vec![SnapshotAudioPublication {
        locator: pending.clone(),
        vocabulary_id: suivre,
        expected_revision: 5,
        generation: 3,
        execution_generation: Some(7),
        error: Some("interrupted publish".into()),
    }];
    snapshot.audio_repairs = vec![
        SnapshotAudioRepair {
            vocabulary_id: double,
            failure_identity: format!("{invalid}:invalid"),
            semantic_key: Some(double_key.clone()),
            generation: Some(2),
            error: Some("budget exhausted".into()),
        },
        SnapshotAudioRepair {
            vocabulary_id: finir,
            failure_identity: "absent:missing".into(),
            semantic_key: None,
            generation: None,
            error: Some("unknown original voice".into()),
        },
    ];
    validate_learning_snapshot(&snapshot, NOW + 1).unwrap();
    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    import_learning_snapshot(&mut target, &snapshot, NOW + 1).unwrap();

    let selected = selected_audio(&target, ecole).unwrap().unwrap();
    assert_eq!(selected.locator, shared);
    assert_eq!(selected.physical_locator, shared);
    assert_eq!(selected.byte_length, 640);
    assert_eq!(selected.generated_at, None);
    assert_eq!(selected.expected_revision, 10);
    assert_eq!(selected.generation, 0);
    assert_eq!(
        selected_audio(&target, etre).unwrap().unwrap().locator,
        shared
    );
    let missing_audio = selected_audio(&target, suivre).unwrap().unwrap();
    assert_eq!(missing_audio.locator, missing);
    assert_eq!(missing_audio.byte_length, 0);
    assert_eq!(missing_audio.generated_at, None);
    assert_eq!(missing_audio.verification, "missing");
    assert_eq!(
        selected_audio_batch(&target, &[ecole, etre, suivre, double])
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        selected_audio(&target, double)
            .unwrap()
            .unwrap()
            .verification,
        "invalid"
    );
    let protected: (Option<String>, Option<i64>) = target.query_row(
        "SELECT previous_version,previous_until FROM word_audio_selections WHERE vocabulary_id=?1", [ecole],
        |row| Ok((row.get(0)?,row.get(1)?)),
    ).unwrap();
    assert_eq!(protected, (Some(previous.clone()), Some(NOW + 40)));
    type RetiredVersion = (
        String,
        i64,
        Option<i64>,
        Option<String>,
        Option<i64>,
        i64,
        Option<String>,
    );
    let retired: RetiredVersion = target.query_row(
        "SELECT gc_state,gc_generation,expires_at,gc_error,vocabulary_id,last_used,import_locator
         FROM speech_versions WHERE locator=?1", [&expired],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?)),
    ).unwrap();
    assert_eq!(
        retired,
        (
            "marked".into(),
            0,
            Some(NOW - 795_200),
            None,
            Some(99_999),
            NOW + 1,
            None
        )
    );
    let raw_missing: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = target.query_row(
        "SELECT byte_length,generated_at,expires_at,orphan_until FROM speech_versions WHERE locator=?1", [&missing],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
    ).unwrap();
    assert_eq!(raw_missing, (None, None, None, Some(NOW + 123)));
    type SourceProjection = (
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<String>,
        Option<i64>,
        Option<i64>,
        bool,
    );
    let source_projection: SourceProjection = target.query_row(
        "SELECT content_hash,generated_at,expires_at,pending_hash,pending_generated_at,pending_execution,retain
         FROM speech_assets WHERE semantic_key=?1", [&follow_key],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?)),
    ).unwrap();
    assert_eq!(
        source_projection,
        (
            Some(shared.clone()),
            None,
            None,
            Some(shared.clone()),
            Some(NOW - 20),
            Some(7),
            true
        )
    );
    assert_eq!(
        speech_asset(&target, &follow_key, NOW + 1)
            .unwrap()
            .unwrap()
            .generated_at,
        0
    );
    assert_eq!(
        word_audio_assets(&target, &[suivre], NOW + 1).unwrap()[&suivre][0].generated_at,
        0
    );
    assert_eq!(
        pending_publication(&target, &follow_key).unwrap(),
        Some((shared.clone(), NOW - 20))
    );
    assert_eq!(
        pending_source_audio(&target, "").unwrap()[0].locator,
        pending
    );
    let intent = pending_word_audio_versions(&target, suivre).unwrap();
    assert_eq!(intent.len(), 1);
    assert_eq!(intent[0].locator, pending);
    assert_eq!(intent[0].expected_revision, 5);
    assert_eq!(intent[0].generation, 3);
    assert_eq!(intent[0].execution_generation, Some(7));
    let unpublished: (bool,String,Option<i64>,Option<String>) = target.query_row(
        "SELECT sealed,verification,expires_at,import_locator FROM speech_versions WHERE locator=?1", [&pending],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?)),
    ).unwrap();
    assert_eq!(
        unpublished,
        (true, "verified".into(), Some(NOW + 604_780), None)
    );
    assert_eq!(
        selected_speech_operation(&target, "change-original", suivre, true).unwrap(),
        Some((follow_key.clone(), 3))
    );
    assert!(selected_speech_operation(&target, "change-original", suivre, false).is_err());
    assert!(selected_speech_operation(&target, "change-original", double, true).is_err());
    assert_eq!(
        request_selected_speech(
            &mut target,
            &SpeechSpec::word("suivre", SpeechVoice::Female),
            suivre,
            NOW + 2,
            true,
            Some("change-original")
        )
        .unwrap(),
        (follow_key.clone(), 3)
    );
    assert!(
        !demand_word_repair(
            &mut target,
            &WordDemand {
                vocabulary_id: double,
                class: 1
            },
            NOW + 2
        )
        .unwrap()
    );
    assert_eq!(
        speech_task_state(&target, &double_key).unwrap(),
        Some(("failed".into(), Some("budget exhausted".into())))
    );
    let failed_budget: (i64, Option<i64>, i64) = target
        .query_row(
            "SELECT attempts,deadline_at,execution_generation FROM speech_tasks WHERE id=92",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(failed_budget, (4, Some(NOW - 1), 8));
    assert_eq!(audio_migration_status(&target).unwrap().pending, 1);
    assert_eq!(audio_migration_status(&target).unwrap().failed, 1);
    assert_eq!(pending_audio_imports(&target).unwrap(), [venir]);
    let legacy_hash: String = target
        .query_row(
            "SELECT content_hash FROM legacy_speech WHERE vocabulary_id=?1",
            [finir],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(legacy_hash, shared);
    let deleted_state: String = target
        .query_row(
            "SELECT gc_state FROM speech_versions WHERE locator=?1",
            [&deleted],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(deleted_state, "deleted");
    assert_eq!(pending_audio_gc(&target).unwrap().len(), 1);
    assert_eq!(
        pending_audio_gc(&target).unwrap()[0].version.locator,
        expired
    );
    assert_eq!(review_cards(&target, suivre).unwrap().len(), 2);
    assert_eq!(
        review_progress(&target, review_cards(&target, suivre).unwrap()[1].id)
            .unwrap()
            .revision,
        0
    );
}

#[test]
fn snapshot_import_rolls_back_partial_writes_when_sqlite_runs_out_of_pages() {
    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    let pages: i64 = target
        .pragma_query_value(None, "page_count", |row| row.get(0))
        .unwrap();
    let free_pages: i64 = target
        .pragma_query_value(None, "freelist_count", |row| row.get(0))
        .unwrap();
    target.pragma_update(None, "max_page_count", pages).unwrap();
    let changes = target.total_changes();

    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    saved_word(&mut source, "save-small", "petit");
    for index in 0..free_pages + 2 {
        saved_word(
            &mut source,
            &format!("save-large-{index}"),
            &format!("grand-{index}"),
        );
    }
    let mut snapshot = export_learning_snapshot(&mut source, CAPTURED_AT).unwrap();
    for record in snapshot.records.iter_mut().skip(1) {
        record.notes = "x".repeat(20_000);
    }
    validate_learning_snapshot(&snapshot, CAPTURED_AT).unwrap();
    let error = import_learning_snapshot(&mut target, &snapshot, CAPTURED_AT).unwrap_err();
    assert!(matches!(
        error,
        StorageError::Database(rusqlite::Error::SqliteFailure(failure, _))
            if failure.code == rusqlite::ErrorCode::DiskFull
    ));
    assert!(target.total_changes() > changes);
    assert!(target.is_autocommit());
    assert!(list(&target, "", "headword-asc").unwrap().is_empty());
    assert!(review_card_inventory(&target).unwrap().is_empty());
    assert!(latest_batch(&target).unwrap().is_none());
    assert_eq!(review_settings(&target).unwrap().retention_percent, 90);
    assert_eq!(review_settings(&target).unwrap().new_limit, 20);
    assert_eq!(review_settings(&target).unwrap().review_limit, 50);
    assert!(
        !target
            .prepare("SELECT 1 FROM save_operations UNION ALL SELECT 1 FROM app_metadata LIMIT 1")
            .unwrap()
            .exists([])
            .unwrap()
    );
    assert_eq!(initialize(&target).unwrap(), 18);
    assert!(
        !target
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .exists([])
            .unwrap()
    );
}

#[test]
fn snapshot_import_rejects_populated_or_incompatible_destinations_without_writes() {
    let mut source = Connection::open_in_memory().unwrap();
    initialize(&source).unwrap();
    saved_word(&mut source, "save-source", "école");
    let snapshot = export_learning_snapshot(&mut source, CAPTURED_AT).unwrap();

    let mut target = Connection::open_in_memory().unwrap();
    initialize(&target).unwrap();
    let preserved = saved_word(&mut target, "save-destination", "reste");
    let changes = target.total_changes();
    let error = import_learning_snapshot(&mut target, &snapshot, CAPTURED_AT).unwrap_err();
    assert!(
        matches!(error, StorageError::Invalid(message) if message.contains("empty destination"))
    );
    assert_eq!(target.total_changes(), changes);
    assert_eq!(get(&target, preserved).unwrap().headword, "reste");
    assert_eq!(list(&target, "", "headword-asc").unwrap().len(), 1);
    assert!(target.is_autocommit());

    let mut metadata_target = Connection::open_in_memory().unwrap();
    initialize(&metadata_target).unwrap();
    metadata_target
        .execute(
            "INSERT INTO app_metadata(key,value) VALUES('existing-setting','synthetic-value')",
            [],
        )
        .unwrap();
    let changes = metadata_target.total_changes();
    let error = import_learning_snapshot(&mut metadata_target, &snapshot, CAPTURED_AT).unwrap_err();
    assert!(
        matches!(error, StorageError::Invalid(message) if message.contains("empty destination"))
    );
    assert_eq!(metadata_target.total_changes(), changes);
    assert_eq!(
        metadata_target
            .query_row(
                "SELECT value FROM app_metadata WHERE key='existing-setting'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "synthetic-value"
    );

    for schema in [0_u32, 17, 19, 99] {
        let mut incompatible = Connection::open_in_memory().unwrap();
        if schema != 0 {
            initialize(&incompatible).unwrap();
        }
        incompatible
            .pragma_update(None, "user_version", schema)
            .unwrap();
        let changes = incompatible.total_changes();
        let error =
            import_learning_snapshot(&mut incompatible, &snapshot, CAPTURED_AT).unwrap_err();
        assert!(
            matches!(error, StorageError::Invalid(message) if message.contains("initialized destination"))
        );
        assert_eq!(incompatible.total_changes(), changes);
        assert_eq!(
            incompatible
                .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
                .unwrap(),
            schema
        );
        assert!(incompatible.is_autocommit());
    }
}
