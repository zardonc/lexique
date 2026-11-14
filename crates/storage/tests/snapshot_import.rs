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
