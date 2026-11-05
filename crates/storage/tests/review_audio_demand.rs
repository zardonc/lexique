use lexique_storage::*;
use rusqlite::{Connection, params};

fn fixture() -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let snapshot = serde_json::json!({"reference":{"dictionary":"synthetic","lookup_key":"école","version":"1","attribution":"fixture","license":"MIT","source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},"digest":[]}).to_string();
    for (id, word) in [(1, "école"), (2, "maison")] {
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?1,?2,?3,'keep',1,1)",params![id,word,snapshot]).unwrap();
        add_review(&mut connection, id).unwrap();
    }
    connection
}

#[test]
fn automatic_repair_preserves_generation_deadline_attempts_and_terminal_failure() {
    let mut connection = fixture();
    let demand = WordDemand {
        vocabulary_id: 1,
        class: 1,
    };
    assert!(demand_word_repair(&mut connection, &demand, 1000).unwrap());
    let generation: i64 = connection
        .query_row(
            "SELECT desired_generation FROM word_audio_selections WHERE vocabulary_id=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let task = claim_speech(&mut connection, 1000, 1).unwrap().unwrap();
    assert_eq!(task.priority, 1);
    fail_speech(&connection, task.id, 1001, "busy", false).unwrap();
    for now in [1002, 1003, 1004] {
        assert!(!demand_word_repair(&mut connection, &demand, now).unwrap());
    }
    let state: (i64, i64, String) = connection
        .query_row(
            "SELECT attempts,deadline_at,state FROM speech_tasks WHERE id=?1",
            [task.id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(state, (1, 1120, "retry_wait".into()));
    let next = claim_speech(&mut connection, 1003, 1).unwrap().unwrap();
    fail_speech(&connection, next.id, 1004, "model unavailable", true).unwrap();
    assert!(!demand_word_repair(&mut connection, &demand, 1005).unwrap());
    assert!(claim_speech(&mut connection, 1100, 1).unwrap().is_none());
    assert_eq!(
        audio_demand_diagnostics(&connection, &[1])
            .unwrap()
            .error
            .as_deref(),
        Some("model unavailable")
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT desired_generation FROM word_audio_selections WHERE vocabulary_id=1",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        generation
    );
    request_selected_speech(
        &mut connection,
        &task.spec,
        1,
        1101,
        true,
        Some("explicit-retry"),
    )
    .unwrap();
    assert_eq!(
        claim_speech(&mut connection, 1101, 1)
            .unwrap()
            .unwrap()
            .attempts,
        1
    );
}

#[test]
fn background_fairness_preserves_explicit_batches_and_the_interaction_slot() {
    let mut connection = fixture();
    let batch = SpeechSpec::word("lointain", SpeechVoice::Female);
    let batch_key = batch.key().unwrap();
    demand_speech(&connection, &batch, 1, 1000, false).unwrap();
    for index in 0..5 {
        let spec = SpeechSpec::word(&format!("fenêtre{index}"), SpeechVoice::Female);
        let key = spec.key().unwrap();
        demand_speech(&connection, &spec, 1, 1001, false).unwrap();
        connection
            .execute(
                "UPDATE speech_tasks SET demand_class=1 WHERE semantic_key=?1",
                [key],
            )
            .unwrap();
    }
    for _ in 0..4 {
        let task = claim_speech(&mut connection, 1002, 1).unwrap().unwrap();
        assert_ne!(task.semantic_key, batch_key);
        fail_speech(&connection, task.id, 1002, "fixture terminal", true).unwrap();
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT value FROM app_metadata WHERE key='speech-window-streak'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "4"
    );
    let batch_task = claim_speech(&mut connection, 1002, 1).unwrap().unwrap();
    assert_eq!(batch_task.semantic_key, batch_key);
    let second = claim_speech(&mut connection, 1002, 2).unwrap().unwrap();
    assert_eq!(second.priority, 1);
    let click = SpeechSpec::word("clic", SpeechVoice::Female);
    demand_speech(&connection, &click, 0, 1002, false).unwrap();
    assert_eq!(
        claim_speech(&mut connection, 1002, 2)
            .unwrap()
            .unwrap()
            .priority,
        0
    );
    assert!(claim_speech(&mut connection, 1002, 2).unwrap().is_none());
}

#[test]
fn today_demand_is_read_only_handles_zero_quota_learning_and_excludes_future_cards() {
    let mut connection = fixture();
    let now = 1_700_000_000;
    let cards = review_cards(&connection, 1).unwrap();
    for card in cards {
        connection.execute("INSERT INTO review_progress VALUES(?1,1,?2)",params![card.id,serde_json::json!({"phase":"learning_first","memory":null,"last_review_at":now-120,"due_at":now-1}).to_string()]).unwrap();
    }
    let future = review_cards(&connection, 2).unwrap();
    for card in future {
        connection.execute("INSERT INTO review_progress VALUES(?1,1,?2)",params![card.id,serde_json::json!({"phase":"learning_first","memory":null,"last_review_at":now-120,"due_at":now+1_000_000}).to_string()]).unwrap();
    }
    let mut settings = review_settings(&connection).unwrap();
    settings.new_limit = 0;
    settings.review_limit = 0;
    set_review_settings(&connection, &settings).unwrap();
    let before = connection.total_changes();
    let revision = queue_revision(&connection).unwrap();
    let mut cache = QueueCache::default();
    let candidates =
        review_audio_candidates(&mut connection, now, "UTC", &Default::default(), &mut cache)
            .unwrap();
    assert_eq!(candidates.cards.len(), 2);
    assert!(candidates.cards.iter().all(|card| card.vocabulary_id == 1));
    assert_eq!(connection.total_changes(), before);
    assert_eq!(queue_revision(&connection).unwrap(), revision);
    assert_eq!(
        plan_review_words(&candidates.cards, &candidates.cards, &Default::default()),
        vec![WordDemand {
            vocabulary_id: 1,
            class: 1
        }]
    );
}

#[test]
fn automatic_repair_keeps_known_voice_and_reports_unsupported_specs() {
    let mut connection = fixture();
    let spec = SpeechSpec::word("école", SpeechVoice::Female);
    let key = register_speech(&connection, &spec).unwrap();
    let version = prepare_audio_version(
        &connection,
        1,
        Some(&key),
        &content_hash(b"OggSvalid"),
        9,
        Some(1000),
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &version.locator, 1000).unwrap();
    mark_audio_unavailable(&connection, &version.locator, "missing").unwrap();
    set_speech_preferences(
        &connection,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            speed: SpeechSpeed::Normal,
        },
    )
    .unwrap();
    assert_eq!(word_repair_spec(&connection, 1).unwrap().voice, "female");
    let mut unsupported = SpeechSpec::word("maison", SpeechVoice::Female);
    unsupported.engine = "older-engine".into();
    let key = register_speech(&connection, &unsupported).unwrap();
    let version = prepare_audio_version(
        &connection,
        2,
        Some(&key),
        &content_hash(b"OggSvalid"),
        9,
        Some(1000),
        0,
        1000,
    )
    .unwrap();
    complete_audio_version(&connection, &version.locator, 1000).unwrap();
    mark_audio_unavailable(&connection, &version.locator, "invalid").unwrap();
    assert!(
        demand_word_repair(
            &mut connection,
            &WordDemand {
                vocabulary_id: 2,
                class: 2
            },
            1001
        )
        .unwrap()
    );
    assert!(
        audio_demand_diagnostics(&connection, &[2])
            .unwrap()
            .error
            .unwrap()
            .contains("unsupported")
    );
    assert!(
        !demand_word_repair(
            &mut connection,
            &WordDemand {
                vocabulary_id: 2,
                class: 2
            },
            1002
        )
        .unwrap()
    );
    assert_eq!(
        connection
            .query_row("SELECT COUNT(*) FROM speech_tasks", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
