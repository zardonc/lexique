use lexique_core::scheduling::{Rating, Scheduled};
use lexique_storage::*;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

const NOW: i64 = 1_794_564_000;
const FEMALE_AUDIO: &[u8] = include_bytes!("../../../docs/evidence/issue-56/female-1.ogg");
const PREVIOUS_AUDIO: &[u8] = include_bytes!("../../../docs/evidence/issue-56/female-2.ogg");
const MALE_AUDIO: &[u8] = include_bytes!("../../../docs/evidence/issue-56/male-1.ogg");

fn database() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    connection
}

fn saved_word(connection: &mut Connection, operation: &str, headword: &str) -> VocabularyRecord {
    save(
        connection,
        &SaveRequest {
            operation: operation.into(),
            headword: headword.into(),
            notes: "A notes — accents é".into(),
            example_fr: "Une école.".into(),
            example_zh: "一所学校".into(),
            example_source: None,
            example_sense: Some(0),
            more_examples: vec![Example {
                fr: "Je vais à l'école.".into(),
                zh: "我去学校。".into(),
                source: None,
            }],
            id: None,
            revision: None,
        },
        Some(Snapshot {
            reference: Reference {
                dictionary_id: Some("mdx-99".into()),
                dictionary: "Removed synthetic dictionary".into(),
                lookup_key: headword.into(),
                version: "fixture-1".into(),
                attribution: Some("Synthetic fixture".into()),
                license: Some("MIT".into()),
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![DigestEntry {
                pronunciations: vec!["e.kɔl".into()],
                grammar: vec!["n".into()],
                translation: Some("学校".into()),
            }],
        }),
    )
    .unwrap()
    .record
}

fn rate(
    connection: &mut Connection,
    operation: &str,
    card_id: i64,
    rating: Rating,
    now: i64,
) -> Value {
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

fn read_value(value: &Value) -> Result<LearningSnapshot, StorageError> {
    read_learning_snapshot(serde_json::to_vec(value).unwrap().as_slice(), NOW + 10)
}

fn captured(connection: &mut Connection) -> Value {
    serde_json::to_value(export_learning_snapshot(connection, NOW + 10).unwrap()).unwrap()
}

fn review_pair(connection: &mut Connection, vocabulary_id: i64) -> [i64; 2] {
    let cards = add_review(connection, vocabulary_id).unwrap();
    ["recognition", "listening"].map(|kind| cards.iter().find(|card| card.kind == kind).unwrap().id)
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

fn task_rows(connection: &Connection) -> Vec<Value> {
    connection.prepare("SELECT id,semantic_key,priority,state,attempts,wake_at,deadline_at,cancel_requested,refresh_requested,successor_requested,interactive_requested,execution_generation,demand_class,requested_at,error FROM speech_tasks ORDER BY id").unwrap().query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?, "semantic_key": row.get::<_, String>(1)?,
            "priority": row.get::<_, i64>(2)?, "state": row.get::<_, String>(3)?,
            "attempts": row.get::<_, i64>(4)?, "wake_at": row.get::<_, i64>(5)?,
            "deadline_at": row.get::<_, Option<i64>>(6)?, "cancel_requested": row.get::<_, bool>(7)?,
            "refresh_requested": row.get::<_, bool>(8)?, "successor_requested": row.get::<_, bool>(9)?,
            "interactive_requested": row.get::<_, bool>(10)?, "execution_generation": row.get::<_, i64>(11)?,
            "demand_class": row.get::<_, i64>(12)?, "requested_at": row.get::<_, i64>(13)?,
            "error": row.get::<_, Option<String>>(14)?,
        }))
    }).unwrap().collect::<Result<_, _>>().unwrap()
}

fn prepared_word_audio(
    connection: &Connection,
    vocabulary_id: i64,
    key: Option<&str>,
    bytes: &[u8],
    generation: i64,
    now: i64,
) -> AudioVersion {
    prepare_audio_version(
        connection,
        vocabulary_id,
        key,
        AudioPublicationContent {
            hash: &content_hash(bytes),
            bytes: bytes.len() as i64,
            generated_at: key.map(|_| now),
        },
        generation,
        now,
    )
    .unwrap()
}

fn audio_fixture() -> (Connection, [i64; 5], [String; 7]) {
    let mut connection = database();
    let ecole = saved_word(&mut connection, "save-ecole", "école");
    let shared = saved_word(&mut connection, "save-shared", "partagé");
    let expired = saved_word(&mut connection, "save-expired", "autre");
    let legacy = saved_word(&mut connection, "save-legacy", "ancien");
    let initial = saved_word(&mut connection, "save-initial", "premier");
    for id in [ecole.id, shared.id, expired.id, legacy.id, initial.id] {
        review_pair(&mut connection, id);
    }
    let previous = prepared_word_audio(&connection, ecole.id, None, FEMALE_AUDIO, 0, NOW - 86_400);
    assert!(complete_audio_version(&connection, &previous.locator, NOW - 86_400).unwrap());
    assert!(
        reuse_audio_version(&connection, shared.id, &previous.locator, 0, NOW - 86_399).unwrap()
    );
    let female_key =
        register_speech(&connection, &SpeechSpec::word("école", SpeechVoice::Female)).unwrap();
    let generation = begin_audio_change(&connection, ecole.id, &female_key).unwrap();
    let current = prepared_word_audio(
        &connection,
        ecole.id,
        Some(&female_key),
        PREVIOUS_AUDIO,
        generation,
        NOW - 100,
    );
    assert!(complete_audio_version(&connection, &current.locator, NOW - 100).unwrap());

    let expired_previous = prepared_word_audio(
        &connection,
        expired.id,
        None,
        FEMALE_AUDIO,
        0,
        NOW - 2_592_000,
    );
    assert!(
        complete_audio_version(&connection, &expired_previous.locator, NOW - 2_592_000).unwrap()
    );
    let expired_key =
        register_speech(&connection, &SpeechSpec::word("autre", SpeechVoice::Male)).unwrap();
    let generation = begin_audio_change(&connection, expired.id, &expired_key).unwrap();
    let expired_current = prepared_word_audio(
        &connection,
        expired.id,
        Some(&expired_key),
        MALE_AUDIO,
        generation,
        NOW - 1_296_001,
    );
    assert!(
        complete_audio_version(&connection, &expired_current.locator, NOW - 1_296_001).unwrap()
    );

    let (pending_key, generation) = request_selected_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Male),
        ecole.id,
        NOW,
        true,
        Some("audio:pending_2"),
    )
    .unwrap();
    let pending = prepared_word_audio(
        &connection,
        ecole.id,
        Some(&pending_key),
        MALE_AUDIO,
        generation,
        NOW + 1,
    );
    connection
        .execute(
            "UPDATE word_audio_publications SET error='fixture publishing paused' WHERE locator=?1",
            [&pending.locator],
        )
        .unwrap();
    let pending_source = prepare_source_audio(
        &connection,
        &pending_key,
        &content_hash(MALE_AUDIO),
        MALE_AUDIO.len() as i64,
        NOW + 1,
    )
    .unwrap();

    let retired = prepare_source_audio(
        &connection,
        &female_key,
        &content_hash(FEMALE_AUDIO),
        FEMALE_AUDIO.len() as i64,
        NOW - 691_200,
    )
    .unwrap();
    verify_audio_content(&connection, &retired.locator, FEMALE_AUDIO.len() as i64).unwrap();
    assert!(complete_source_audio(&connection, &retired.locator).unwrap());
    assert!(
        mark_audio_gc(&connection, &retired.locator, NOW)
            .unwrap()
            .is_some()
    );

    let initial_key = attach_word_audio_at(&connection, initial.id, NOW - 10).unwrap();
    let initial_source = prepare_source_audio(
        &connection,
        &initial_key,
        &content_hash(FEMALE_AUDIO),
        FEMALE_AUDIO.len() as i64,
        NOW - 10,
    )
    .unwrap();
    verify_audio_content(
        &connection,
        &initial_source.locator,
        FEMALE_AUDIO.len() as i64,
    )
    .unwrap();
    assert!(complete_source_audio(&connection, &initial_source.locator).unwrap());
    register_legacy_speech(&connection, legacy.id, &content_hash(FEMALE_AUDIO)).unwrap();
    register_legacy_speech(&connection, shared.id, &content_hash(FEMALE_AUDIO)).unwrap();
    for (id, state, error) in [
        (ecole.id, "pending", None),
        (shared.id, "finished", None),
        (legacy.id, "failed", Some("fixture legacy import failed")),
    ] {
        connection.execute("INSERT INTO word_audio_imports(vocabulary_id,state,error) VALUES(?1,?2,?3) ON CONFLICT(vocabulary_id) DO UPDATE SET state=excluded.state,error=excluded.error", params![id, state, error]).unwrap();
    }
    let word_preview = SpeechSpec::word("temporary word", SpeechVoice::Male);
    let mut sentence_preview = SpeechSpec::word("Une phrase temporaire.", SpeechVoice::Female);
    sentence_preview.context = "sentence".into();
    for spec in [&word_preview, &sentence_preview] {
        let key = request_speech(&mut connection, spec, None, NOW, false).unwrap();
        prepare_source_audio(
            &connection,
            &key,
            &content_hash(MALE_AUDIO),
            MALE_AUDIO.len() as i64,
            NOW,
        )
        .unwrap();
    }
    set_speech_preferences(
        &connection,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            wpm: 240,
        },
    )
    .unwrap();
    (
        connection,
        [ecole.id, shared.id, expired.id, legacy.id, initial.id],
        [
            current.locator,
            previous.locator,
            expired_previous.locator,
            pending.locator,
            pending_source.locator,
            retired.locator,
            initial_source.locator,
        ],
    )
}

#[test]
fn rating_operation_ids_follow_the_real_rating_boundary() {
    let mut connection = database();
    let word = saved_word(&mut connection, "save-ecole", "école");
    let cards = add_review(&mut connection, word.id).unwrap();
    let recognition = cards
        .iter()
        .find(|card| card.kind == "recognition")
        .unwrap();
    rate(
        &mut connection,
        "rating:école/first_1",
        recognition.id,
        Rating::Good,
        NOW,
    );
    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap_or_else(|error| {
        panic!("a committed rating ID accepted by rate_card must be admitted: {error}")
    });
    assert_eq!(decoded.ratings[0].operation, "rating:école/first_1");
}

#[test]
fn undo_operation_ids_follow_the_real_undo_boundary() {
    let mut connection = database();
    let word = saved_word(&mut connection, "save-ecole", "école");
    let cards = add_review(&mut connection, word.id).unwrap();
    let recognition = cards
        .iter()
        .find(|card| card.kind == "recognition")
        .unwrap();
    rate(
        &mut connection,
        "rating-first",
        recognition.id,
        Rating::Good,
        NOW,
    );
    undo_rating(&mut connection, "undo:école/first_1", "rating-first").unwrap();
    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap_or_else(|error| {
        panic!("a committed undo ID accepted by undo_rating must be admitted: {error}")
    });
    assert_eq!(
        decoded.ratings[0].undo_operation.as_deref(),
        Some("undo:école/first_1")
    );
}

#[test]
fn audio_operation_ids_follow_the_real_audio_boundary() {
    let mut connection = database();
    let word = saved_word(&mut connection, "save-ecole", "école");
    request_selected_speech(
        &mut connection,
        &SpeechSpec::word("école", SpeechVoice::Female),
        word.id,
        NOW,
        true,
        Some("audio:école/first_1"),
    )
    .unwrap();
    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap_or_else(|error| {
        panic!("a committed audio ID accepted by request_selected_speech must be admitted: {error}")
    });
    assert_eq!(decoded.audio_operations[0].operation, "audio:école/first_1");
}

#[test]
fn version_one_records_require_their_content_fields() {
    let mut connection = database();
    saved_word(&mut connection, "save-ecole", "école");
    let original = captured(&mut connection);
    read_value(&original).unwrap();
    let mut admitted = Vec::new();
    for field in [
        "id",
        "headword",
        "reference",
        "digest",
        "notes",
        "example_fr",
        "example_zh",
        "example_sense",
        "more_examples",
        "revision",
        "created_at",
        "updated_at",
    ] {
        let mut malformed = original.clone();
        malformed["records"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        if read_value(&malformed).is_ok() {
            admitted.push(field);
        }
    }
    assert!(
        admitted.is_empty(),
        "record fields admitted when absent: {admitted:?}"
    );
}

#[test]
fn equal_time_ratings_keep_submission_order_undo_queue_and_independent_progress() {
    let mut connection = database();
    let ecole = saved_word(&mut connection, "save-ecole", "école");
    let etre = saved_word(&mut connection, "save-etre", "être");
    let suivre = saved_word(&mut connection, "save-suivre", "suivre");
    let double = saved_word(&mut connection, "save-double", "double");
    let [ecole_recognition, ecole_listening] = review_pair(&mut connection, ecole.id);
    let [etre_recognition, etre_listening] = review_pair(&mut connection, etre.id);
    let [suivre_recognition, suivre_listening] = review_pair(&mut connection, suivre.id);
    let [double_recognition, double_listening] = review_pair(&mut connection, double.id);
    set_review_settings(
        &connection,
        &ReviewSettings {
            retention_percent: 87,
            new_limit: 2,
            review_limit: 3,
            new_cards_ignore_review_limit: false,
        },
    )
    .unwrap();
    let old = rate(
        &mut connection,
        "old-review",
        suivre_recognition,
        Rating::Easy,
        NOW - 86_400,
    );
    let first = rate(
        &mut connection,
        "z-new",
        ecole_recognition,
        Rating::Good,
        NOW,
    );
    let undone = rate(
        &mut connection,
        "a-undone",
        etre_recognition,
        Rating::Again,
        NOW,
    );
    assert!(!undo_rating(&mut connection, "undo-A", "a-undone").unwrap());
    set_review_paused(&mut connection, double.id, true).unwrap();

    connection
        .execute(
            "INSERT INTO review_queue_state(id,learning_day,timezone) VALUES(1,'2026-11-13','UTC')",
            [],
        )
        .unwrap();
    for (card_id, position) in [
        (ecole_listening, 1_000_000),
        (suivre_recognition, 2_000_000),
        (double_listening, 3_000_000),
    ] {
        connection
            .execute(
                "INSERT INTO review_queue_entries(card_id,position) VALUES(?1,?2)",
                params![card_id, position],
            )
            .unwrap();
    }
    let latest = rate(
        &mut connection,
        "m-latest",
        ecole_listening,
        Rating::Again,
        NOW,
    );
    let expected_queue = vec![
        (suivre_recognition, 2_000_000),
        (double_listening, 3_000_000),
    ];
    assert_eq!(queue_rows(&connection), expected_queue);
    let absent: Vec<i64> = connection.prepare(
        "SELECT c.id FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE p.card_id IS NULL ORDER BY c.id",
    ).unwrap().query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(
        absent,
        vec![
            etre_listening,
            suivre_listening,
            double_recognition,
            double_listening
        ]
    );

    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap();
    assert_eq!(queue_rows(&connection), expected_queue);
    assert_eq!(
        decoded
            .ratings
            .iter()
            .map(|rating| (
                rating.sequence,
                rating.operation.as_str(),
                rating.reviewed_at,
                rating.learning_day.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, "old-review", NOW - 86_400, "2026-11-12"),
            (2, "z-new", NOW, "2026-11-13"),
            (3, "a-undone", NOW, "2026-11-13"),
            (4, "m-latest", NOW, "2026-11-13"),
        ]
    );
    for (index, committed_result) in [old, first, undone, latest].iter().enumerate() {
        let committed: Scheduled = serde_json::from_value(committed_result.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&decoded.ratings[index].result).unwrap(),
            serde_json::to_value(committed).unwrap(),
        );
    }
    assert_eq!(value["ratings"][1]["result"]["card"]["due_at"], NOW + 600);
    assert_eq!(value["ratings"][3]["result"]["card"]["due_at"], NOW + 60);
    assert_eq!(decoded.ratings[2].undo_operation.as_deref(), Some("undo-A"));
    assert!(!decoded.ratings[2].undo_queue_rebuilt);
    assert_eq!(
        value["ratings"][3]["before_state"],
        json!({"phase":"new","memory":null,"last_review_at":null,"due_at":null})
    );
    assert_eq!(
        value["ratings"][3]["queue_before"],
        json!({
            "card_id": ecole_listening,
            "position": 1_000_000,
            "ordered_cards": [ecole_listening, suivre_recognition, double_listening],
            "learning_day": "2026-11-13",
            "timezone": "UTC",
        })
    );
    assert_eq!(decoded.cards.len(), 8);
    assert_eq!(decoded.progress.len(), 4);
    assert!(
        decoded
            .progress
            .iter()
            .all(|progress| !absent.contains(&progress.card_id))
    );
    assert_eq!(decoded.paused_vocabulary_ids, vec![double.id]);
    assert_eq!(
        value["queue"],
        json!({"context":{"learning_day":"2026-11-13","timezone":"UTC"},"entries":[
            {"card_id":suivre_recognition,"position":2_000_000},
            {"card_id":double_listening,"position":3_000_000},
        ]})
    );
    assert_eq!(
        review_availability(&connection, etre_listening, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert_eq!(
        review_availability(&connection, double_listening, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("paused")
    );
    assert_eq!(
        undo_target(&connection).unwrap().as_deref(),
        Some("m-latest")
    );
    assert!(!undo_rating(&mut connection, "undo-latest-A", "m-latest").unwrap());
    assert_eq!(
        queue_rows(&connection),
        vec![
            (ecole_listening, 1_000_000),
            (suivre_recognition, 2_000_000),
            (double_listening, 3_000_000)
        ]
    );
    assert_eq!(undo_target(&connection).unwrap(), None);
    assert!(!undo_rating(&mut connection, "undo-latest-A", "m-latest").unwrap());
    assert!(undo_rating(&mut connection, "undo-latest-A", "z-new").is_err());
    assert_eq!(
        review_availability(&connection, etre_listening, NOW + 1, "UTC")
            .unwrap()
            .reason,
        None
    );
    let restored = review_progress(&connection, ecole_listening).unwrap();
    assert_eq!(restored.revision, 2);
    assert_eq!(
        serde_json::to_value(restored.card).unwrap(),
        json!({"phase":"new","memory":null,"last_review_at":null,"due_at":null})
    );
}

#[test]
fn save_histories_keep_original_outcomes_and_current_record_retry_results() {
    let mut connection = database();
    let original = saved_word(&mut connection, "save-original", "école");
    let stored: String = connection
        .query_row(
            "SELECT request FROM save_operations WHERE operation='save-original'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let original_request: SaveRequest = serde_json::from_str(&stored).unwrap();
    let mut update_request: SaveRequest = serde_json::from_str(&stored).unwrap();
    update_request.operation = "save-edited".into();
    update_request.id = Some(original.id);
    update_request.revision = Some(original.revision);
    update_request.notes = "A edited after original save".into();
    let updated = save(&mut connection, &update_request, None).unwrap();
    assert_eq!(updated.outcome, "updated");
    let mut existing_request: SaveRequest = serde_json::from_str(&stored).unwrap();
    existing_request.operation = "save-existing".into();
    existing_request.notes = "Attempted duplicate notes".into();
    assert_eq!(
        save(&mut connection, &existing_request, None)
            .unwrap()
            .outcome,
        "existing"
    );
    let [recognition, listening] = review_pair(&mut connection, original.id);

    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap();
    assert_eq!(
        decoded
            .save_operations
            .iter()
            .map(|operation| (
                operation.operation.as_str(),
                operation.outcome.as_str(),
                operation.record_id
            ))
            .collect::<Vec<_>>(),
        vec![
            ("save-original", "created", original.id),
            ("save-edited", "updated", original.id),
            ("save-existing", "existing", original.id),
        ]
    );
    assert_eq!(
        decoded.save_operations[0].request.notes,
        "A notes — accents é"
    );
    assert_eq!(
        decoded.save_operations[2].request.notes,
        "Attempted duplicate notes"
    );
    assert_eq!(decoded.records[0].notes, "A edited after original save");
    assert_eq!(decoded.records[0].revision, 2);
    assert_eq!(decoded.records[0].created_at, original.created_at);
    assert_eq!(decoded.records[0].updated_at, updated.record.updated_at);
    assert_eq!(
        decoded.records[0].reference.dictionary_id.as_deref(),
        Some("mdx-99")
    );
    assert_eq!(decoded.records[0].digest[0].pronunciations, vec!["e.kɔl"]);
    assert_eq!(decoded.records[0].example_fr, "Une école.");
    assert_eq!(decoded.records[0].example_source, None);
    assert_eq!(decoded.records[0].more_examples[0].source, None);
    assert!(decoded.progress.is_empty());
    for card_id in [recognition, listening] {
        let progress = review_progress(&connection, card_id).unwrap();
        assert_eq!(progress.revision, 0);
        assert_eq!(
            serde_json::to_value(progress.card).unwrap(),
            json!({"phase":"new","memory":null,"last_review_at":null,"due_at":null})
        );
    }
    for (request, outcome) in [
        (&original_request, "replayed"),
        (&update_request, "replayed"),
        (&existing_request, "existing"),
    ] {
        let replay = save(&mut connection, request, None).unwrap();
        assert_eq!(replay.outcome, outcome);
        assert_eq!(replay.record.id, original.id);
        assert_eq!(replay.record.revision, 2);
        assert_eq!(replay.record.notes, "A edited after original save");
    }
    let mut conflicting = original_request;
    conflicting.notes = "B conflicting retry".into();
    assert!(save(&mut connection, &conflicting, None).is_err());
    let counts: (i64, i64, i64) = connection.query_row(
        "SELECT (SELECT COUNT(*) FROM vocabulary),(SELECT COUNT(*) FROM save_operations),(SELECT COUNT(*) FROM review_progress)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap();
    assert_eq!(counts, (1, 3, 0));
    assert_eq!(get(&connection, original.id).unwrap().revision, 2);
}

#[test]
fn equal_time_partial_and_cancelled_batches_keep_requests_results_and_captured_voice() {
    let mut connection = database();
    let ecole = saved_word(&mut connection, "save-ecole", "école");
    let etre = saved_word(&mut connection, "save-etre", "être");
    let suivre = saved_word(&mut connection, "save-suivre", "suivre");
    let double = saved_word(&mut connection, "save-double", "double");
    let requested = [ecole.id, ecole.id, 999_999, etre.id, suivre.id];
    add_review_batch(&mut connection, "z-batch", &requested, NOW).unwrap();
    connection
        .execute(
            "DELETE FROM review_batch_items WHERE operation='z-batch' AND ordinal>=2",
            [],
        )
        .unwrap();
    connection.execute("UPDATE review_batch_items SET error='fixture missing vocabulary 999999' WHERE operation='z-batch' AND ordinal=1", []).unwrap();
    add_review_batch(&mut connection, "a-batch", &[double.id], NOW).unwrap();
    cancel_batch(&connection, "a-batch").unwrap();
    set_speech_preferences(
        &connection,
        &SpeechPreferences {
            voice: SpeechVoice::Male,
            wpm: 240,
        },
    )
    .unwrap();

    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap();
    assert_eq!(
        decoded
            .batches
            .iter()
            .map(|batch| (
                batch.sequence,
                batch.operation.as_str(),
                batch.created_at,
                batch.voice,
                batch.cancelled
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, "z-batch", NOW, SpeechVoice::Female, false),
            (2, "a-batch", NOW, SpeechVoice::Female, true),
        ]
    );
    assert_eq!(decoded.batches[0].requested_vocabulary_ids, requested);
    assert_eq!(decoded.batches[1].requested_vocabulary_ids, vec![double.id]);
    assert_eq!(
        decoded.batches[0]
            .items
            .iter()
            .map(|item| (
                item.ordinal,
                item.vocabulary_id,
                item.created_cards,
                item.error.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, ecole.id, 2, None),
            (1, 999_999, 0, Some("fixture missing vocabulary 999999")),
        ]
    );
    assert_eq!(decoded.batches[0].items[1].semantic_key, None);
    let original_key = decoded.batches[0].items[0].semantic_key.as_deref().unwrap();
    assert!(
        decoded
            .speech_sources
            .iter()
            .any(|source| source.semantic_key == original_key
                && source.spec.voice == "female"
                && source.spec.text == "école")
    );
    assert_eq!(decoded.batches[1].items.len(), 1);
    let cancelled_key = decoded.batches[1].items[0].semantic_key.as_deref().unwrap();
    assert_eq!(
        speech_task_state(&connection, cancelled_key)
            .unwrap()
            .unwrap()
            .0,
        "cancelled"
    );
    assert_eq!(
        latest_batch(&connection).unwrap().unwrap().operation,
        "a-batch"
    );
    let partial = batch_status(&connection, "z-batch").unwrap();
    assert_eq!(
        partial
            .items
            .iter()
            .map(|item| item.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![ecole.id, 999_999]
    );
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM review_batch_items WHERE operation='z-batch'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);

    let resumed = add_review_batch(&mut connection, "z-batch", &requested, NOW + 20).unwrap();
    assert_eq!(
        resumed
            .items
            .iter()
            .map(|item| item.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![ecole.id, 999_999, etre.id, suivre.id]
    );
    assert_eq!(
        resumed.items[1].error.as_deref(),
        Some("fixture missing vocabulary 999999")
    );
    let specs: Vec<String> = connection.prepare(
        "SELECT a.spec FROM review_batch_items i JOIN speech_assets a USING(semantic_key) WHERE i.operation='z-batch' ORDER BY i.ordinal",
    ).unwrap().query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(
        specs
            .into_iter()
            .map(|spec| serde_json::from_str::<SpeechSpec>(&spec).unwrap().voice)
            .collect::<Vec<_>>(),
        vec!["female", "female", "female"]
    );
    assert!(add_review_batch(&mut connection, "z-batch", &[ecole.id], NOW + 21).is_err());
    assert_eq!(
        latest_batch(&connection).unwrap().unwrap().operation,
        "a-batch"
    );
    assert!(batch_status(&connection, "a-batch").unwrap().cancelled);
}

#[test]
fn audio_capture_preserves_shared_protection_pending_and_retired_metadata() {
    let (
        mut connection,
        [ecole, shared, expired, legacy, initial],
        [
            current,
            previous,
            expired_previous,
            pending,
            pending_source,
            retired,
            initial_source,
        ],
    ) = audio_fixture();
    assert_eq!(
        content_hash(FEMALE_AUDIO),
        "7b6b039db39a3a444a466d31f37f58cf831b9a0cfd88d5b26c82bf9afeda109a"
    );
    assert_eq!(
        content_hash(PREVIOUS_AUDIO),
        "069995106a8d286bd2cf845bd319221e24ea4b62d91be274063de43f19e31072"
    );
    assert_eq!(
        content_hash(MALE_AUDIO),
        "86d9016c4fa838fcc4096fe93bbd448a6afc501de1f501ce736e92ec7622825a"
    );
    assert_eq!(
        [FEMALE_AUDIO.len(), PREVIOUS_AUDIO.len(), MALE_AUDIO.len()],
        [36_938, 44_843, 37_256]
    );
    let value = captured(&mut connection);
    let decoded = read_value(&value).unwrap();
    assert_eq!(decoded.audio_versions.len(), 8);
    assert_eq!(decoded.speech_sources.len(), 7);
    assert_eq!(decoded.speech_tasks.len(), 1);
    let version = |locator: &str| {
        decoded
            .audio_versions
            .iter()
            .find(|version| version.locator == locator)
            .unwrap()
    };
    assert_eq!(
        version(&current).payload_status,
        AudioPayloadStatus::Included
    );
    assert_eq!(
        version(&previous).payload_status,
        AudioPayloadStatus::Included
    );
    assert_eq!(version(&previous).generation, 0);
    assert_eq!(version(&previous).semantic_key, None);
    assert_eq!(version(&previous).generated_at, None);
    assert_eq!(version(&previous).execution_generation, None);
    assert_eq!(version(&previous).expires_at, None);
    assert_eq!(version(&previous).byte_length, Some(36_938));
    assert_eq!(
        version(&expired_previous).payload_status,
        AudioPayloadStatus::Excluded
    );
    assert_eq!(
        version(&expired_previous).exclusion_reason,
        Some(AudioExclusionReason::Expired)
    );
    assert_eq!(version(&retired).gc_state, "marked");
    assert_eq!(version(&retired).expires_at, Some(NOW - 86_400));
    assert_eq!(
        version(&retired).payload_status,
        AudioPayloadStatus::Excluded
    );
    for locator in [&pending, &pending_source] {
        assert_eq!(
            version(locator).payload_status,
            AudioPayloadStatus::Excluded
        );
        assert_eq!(
            version(locator).exclusion_reason,
            Some(AudioExclusionReason::Unpublished)
        );
        assert!(!version(locator).sealed);
        assert_eq!(version(locator).execution_generation, Some(0));
    }
    assert_eq!(
        version(&initial_source).payload_status,
        AudioPayloadStatus::Excluded
    );
    assert_eq!(
        version(&initial_source).exclusion_reason,
        Some(AudioExclusionReason::NotSelected)
    );
    let selected = |id| {
        decoded
            .audio_selections
            .iter()
            .find(|selection| selection.vocabulary_id == id)
            .unwrap()
    };
    assert_eq!(
        selected(ecole).current_version.as_deref(),
        Some(current.as_str())
    );
    assert_eq!(
        selected(ecole).previous_version.as_deref(),
        Some(previous.as_str())
    );
    assert_eq!(selected(ecole).previous_until, Some(NOW - 100 + 1_209_600));
    assert_eq!(selected(ecole).current_generation, 1);
    assert_eq!(selected(ecole).desired_generation, 2);
    assert_eq!(
        selected(shared).current_version.as_deref(),
        Some(previous.as_str())
    );
    assert_eq!(selected(expired).previous_until, Some(NOW - 86_401));
    assert!(
        decoded
            .audio_selections
            .iter()
            .all(|selection| selection.vocabulary_id != initial)
    );
    assert_eq!(decoded.audio_publications.len(), 1);
    assert_eq!(decoded.audio_publications[0].locator, pending);
    assert_eq!(decoded.audio_publications[0].vocabulary_id, ecole);
    assert_eq!(decoded.audio_publications[0].expected_revision, 2);
    assert_eq!(decoded.audio_publications[0].generation, 2);
    assert_eq!(decoded.audio_publications[0].execution_generation, Some(0));
    assert_eq!(
        decoded.audio_publications[0].error.as_deref(),
        Some("fixture publishing paused")
    );
    assert_eq!(
        decoded
            .audio_imports
            .iter()
            .map(|import| (
                import.vocabulary_id,
                import.state.as_str(),
                import.error.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            (ecole, "pending", None),
            (shared, "finished", None),
            (legacy, "failed", Some("fixture legacy import failed")),
        ]
    );
    assert_eq!(
        decoded
            .legacy_audio
            .iter()
            .map(|audio| audio.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![shared, legacy]
    );
    let permanent: (bool, String) = connection
        .query_row(
            "SELECT owned,gc_state FROM audio_version_protection WHERE locator=?1",
            [&previous],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(permanent, (true, "live".into()));
    assert!(
        decoded
            .speech_sources
            .iter()
            .all(|source| !source.spec.text.contains("tempor"))
    );
    for version in value["audio_versions"].as_array().unwrap() {
        for field in [
            "physical_locator",
            "import_locator",
            "last_used",
            "gc_generation",
            "gc_error",
        ] {
            assert!(!version.as_object().unwrap().contains_key(field), "{field}");
        }
    }
}

fn strict_fixture() -> Value {
    let (mut connection, [ecole, _, _, legacy, _], _) = audio_fixture();
    let [card, _] = review_pair(&mut connection, ecole);
    rate(
        &mut connection,
        "required-first",
        card,
        Rating::Good,
        NOW - 86_400,
    );
    rate(&mut connection, "required-rating", card, Rating::Good, NOW);
    undo_rating(&mut connection, "required-undo", "required-rating").unwrap();
    let batch_word = saved_word(&mut connection, "required-batch-word", "nécessaire");
    add_review_batch(&mut connection, "required-batch", &[batch_word.id], NOW).unwrap();
    assert!(
        demand_word_repair(
            &mut connection,
            &WordDemand {
                vocabulary_id: legacy,
                class: 1
            },
            NOW
        )
        .unwrap()
    );
    set_review_paused(&mut connection, legacy, true).unwrap();
    review_queue(&mut connection, NOW, "UTC").unwrap();
    let baseline = captured(&mut connection);
    read_value(&baseline).unwrap();
    baseline
}

#[test]
fn nullable_snapshot_fields_must_be_present_in_version_one() {
    let baseline = strict_fixture();
    assert_eq!(baseline["save_operations"][0]["request"]["id"], Value::Null);
    assert_eq!(baseline["audio_repairs"][0]["semantic_key"], Value::Null);
    assert_eq!(baseline["audio_repairs"][0]["generation"], Value::Null);
    let paths = [
        "/save_operations/0/request/id",
        "/save_operations/0/request/revision",
        "/records/0/digest/0/translation",
        "/ratings/0/queue_before",
        "/ratings/0/queue_before/position",
        "/ratings/0/queue_before/ordered_cards",
        "/ratings/0/queue_before/learning_day",
        "/ratings/0/queue_before/timezone",
        "/ratings/0/undo_operation",
        "/ratings/0/before_state/memory",
        "/ratings/0/before_state/last_review_at",
        "/ratings/0/before_state/due_at",
        "/ratings/0/result/card/memory",
        "/ratings/0/result/card/last_review_at",
        "/ratings/0/result/card/due_at",
        "/ratings/0/result/step_seconds",
        "/ratings/0/result/interval_days",
        "/progress/0/card/memory",
        "/progress/0/card/last_review_at",
        "/progress/0/card/due_at",
        "/queue/context",
        "/batches/0/items/0/semantic_key",
        "/batches/0/items/0/error",
        "/speech_sources/0/current_version",
        "/speech_sources/0/pending_version",
        "/speech_tasks/0/deadline_at",
        "/speech_tasks/0/error",
        "/audio_versions/0/semantic_key",
        "/audio_versions/0/byte_length",
        "/audio_versions/0/generated_at",
        "/audio_versions/0/origin_vocabulary_id",
        "/audio_versions/0/execution_generation",
        "/audio_versions/0/expires_at",
        "/audio_versions/0/orphan_until",
        "/audio_versions/0/exclusion_reason",
        "/audio_selections/0/current_version",
        "/audio_selections/0/previous_version",
        "/audio_selections/0/previous_until",
        "/audio_selections/0/desired_key",
        "/audio_imports/0/error",
        "/legacy_audio/0/exclusion_reason",
        "/audio_publications/0/execution_generation",
        "/audio_publications/0/error",
        "/audio_repairs/0/semantic_key",
        "/audio_repairs/0/generation",
        "/audio_repairs/0/error",
        "/dictionary_catalog/entries/0/content_hash",
    ];
    let mut admitted = Vec::new();
    for path in paths {
        let mut omitted = baseline.clone();
        let (parent, field) = path.rsplit_once('/').unwrap();
        assert!(
            omitted
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field)
                .is_some(),
            "fixture field {path}"
        );
        if read_value(&omitted).is_ok() {
            admitted.push(path);
        }
    }
    assert!(
        admitted.is_empty(),
        "omitted mandatory fields were admitted: {admitted:?}"
    );
}

#[test]
fn audio_admission_rejects_inconsistent_live_relationships() {
    let (
        mut connection,
        [ecole, _, expired, _, _],
        [current, _, _, pending, pending_source, _, initial_source],
    ) = audio_fixture();
    let baseline = captured(&mut connection);
    read_value(&baseline).unwrap();
    let selected_index = |id| {
        baseline["audio_selections"]
            .as_array()
            .unwrap()
            .iter()
            .position(|selection| selection["vocabulary_id"] == id)
            .unwrap()
    };
    let version_index = |locator: &str| {
        baseline["audio_versions"]
            .as_array()
            .unwrap()
            .iter()
            .position(|version| version["locator"] == locator)
            .unwrap()
    };
    let source_index = |locator: &str, field: &str| {
        baseline["speech_sources"]
            .as_array()
            .unwrap()
            .iter()
            .position(|source| source[field] == locator)
            .unwrap()
    };
    let selected = selected_index(ecole);
    let other_selection = &baseline["audio_selections"][selected_index(expired)];
    let current_version = version_index(&current);
    let initial_version = version_index(&initial_source);
    let pending_version = version_index(&pending);
    let initial_asset = source_index(&initial_source, "current_version");
    let pending_asset = source_index(&pending_source, "pending_version");
    let cases = [
        (
            "current generation exceeds desired generation",
            format!("/audio_selections/{selected}/current_generation"),
            json!(3),
        ),
        (
            "desired specification belongs to another word",
            format!("/audio_selections/{selected}/desired_key"),
            other_selection["desired_key"].clone(),
        ),
        (
            "selected known specification belongs to another word",
            format!("/audio_selections/{selected}/current_version"),
            other_selection["current_version"].clone(),
        ),
        (
            "current source is unsealed",
            format!("/audio_versions/{initial_version}/sealed"),
            json!(false),
        ),
        (
            "source current points to its pending version",
            format!("/speech_sources/{pending_asset}/current_version"),
            json!(pending_source),
        ),
        (
            "source pending points to its current version",
            format!("/speech_sources/{initial_asset}/pending_version"),
            json!(initial_source),
        ),
        (
            "current source points to sentence content",
            format!("/audio_versions/{initial_version}/context"),
            json!("sentence"),
        ),
        (
            "included audio carries missing verification",
            format!("/audio_versions/{current_version}/verification"),
            json!("missing"),
        ),
        (
            "missing audio carries verified status",
            format!("/audio_versions/{current_version}/payload_status"),
            json!("source_missing"),
        ),
        (
            "invalid audio carries verified status",
            format!("/audio_versions/{current_version}/payload_status"),
            json!("source_invalid"),
        ),
        (
            "publication origin belongs to another word",
            format!("/audio_versions/{pending_version}/origin_vocabulary_id"),
            json!(expired),
        ),
        (
            "publication target belongs to another word",
            "/audio_publications/0/vocabulary_id".into(),
            json!(expired),
        ),
        (
            "publication revision disagrees with version",
            "/audio_publications/0/expected_revision".into(),
            json!(3),
        ),
        (
            "publication generation disagrees with version",
            "/audio_publications/0/generation".into(),
            json!(3),
        ),
        (
            "publication execution disagrees with version",
            "/audio_publications/0/execution_generation".into(),
            json!(1),
        ),
    ];
    let mut admitted = Vec::new();
    for (name, path, replacement) in cases {
        let mut malformed = baseline.clone();
        *malformed.pointer_mut(&path).unwrap() = replacement;
        if read_value(&malformed).is_ok() {
            admitted.push(name);
        }
    }
    assert!(
        admitted.is_empty(),
        "inconsistent audio relationships were admitted: {admitted:?}"
    );
}

#[test]
fn verified_pending_bytes_stay_unpublished_until_completion() {
    let (mut connection, _, [_, _, _, pending, pending_source, _, _]) = audio_fixture();
    let before = captured(&mut connection);
    for locator in [&pending, &pending_source] {
        verify_audio_content(&connection, locator, MALE_AUDIO.len() as i64).unwrap();
    }
    let after = captured(&mut connection);
    read_value(&after)
        .expect("verification may seal pending bytes before pointer publication commits");
    assert_eq!(after["audio_publications"], before["audio_publications"]);
    assert_eq!(after["speech_sources"], before["speech_sources"]);
    assert_eq!(after["speech_tasks"], before["speech_tasks"]);
    assert_eq!(after["audio_selections"], before["audio_selections"]);
    for locator in [&pending, &pending_source] {
        let version = after["audio_versions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|version| version["locator"] == *locator)
            .unwrap();
        assert_eq!(version["sealed"], true);
        assert_eq!(version["verification"], "verified");
        assert_eq!(version["payload_status"], "excluded");
        assert_eq!(version["exclusion_reason"], "unpublished");
    }
}

#[test]
fn initial_audio_has_one_sealed_word_winner_and_selected_specs_match_the_word() {
    let (mut connection, [ecole, _, expired, _, initial], [current, _, _, _, _, _, initial_source]) =
        audio_fixture();
    let male_key = attach_word_audio_at(&connection, initial, NOW).unwrap();
    let male_source = prepare_source_audio(
        &connection,
        &male_key,
        &content_hash(MALE_AUDIO),
        MALE_AUDIO.len() as i64,
        NOW,
    )
    .unwrap();
    verify_audio_content(&connection, &male_source.locator, MALE_AUDIO.len() as i64).unwrap();
    assert!(complete_source_audio(&connection, &male_source.locator).unwrap());
    register_legacy_speech(&connection, initial, &content_hash(FEMALE_AUDIO)).unwrap();
    let mut baseline = captured(&mut connection);
    let initial_index = baseline["audio_versions"]
        .as_array()
        .unwrap()
        .iter()
        .position(|version| version["locator"] == initial_source)
        .unwrap();
    let male_index = baseline["audio_versions"]
        .as_array()
        .unwrap()
        .iter()
        .position(|version| version["locator"] == male_source.locator)
        .unwrap();
    let legacy_index = baseline["legacy_audio"]
        .as_array()
        .unwrap()
        .iter()
        .position(|audio| audio["vocabulary_id"] == initial)
        .unwrap();
    baseline["audio_versions"][initial_index]["payload_status"] = json!("included");
    baseline["audio_versions"][initial_index]["exclusion_reason"] = Value::Null;
    baseline["legacy_audio"][legacy_index]["payload_status"] = json!("excluded");
    baseline["legacy_audio"][legacy_index]["exclusion_reason"] = json!("not_selected");
    read_value(&baseline).expect(
        "a single verified and sealed initial source is a supported native inspection result",
    );
    let mut admitted = Vec::new();
    let mut unsealed = baseline.clone();
    unsealed["audio_versions"][initial_index]["sealed"] = json!(false);
    if read_value(&unsealed).is_ok() {
        admitted.push("included initial source is unsealed");
    }
    let mut both_sources = baseline.clone();
    both_sources["audio_versions"][male_index]["payload_status"] = json!("included");
    both_sources["audio_versions"][male_index]["exclusion_reason"] = Value::Null;
    if read_value(&both_sources).is_ok() {
        admitted.push("two initial source winners");
    }
    let mut source_and_legacy = baseline.clone();
    source_and_legacy["legacy_audio"][legacy_index]["payload_status"] = json!("included");
    source_and_legacy["legacy_audio"][legacy_index]["exclusion_reason"] = Value::Null;
    if read_value(&source_and_legacy).is_ok() {
        admitted.push("initial source and legacy both win");
    }
    let mut wrong_selection = baseline.clone();
    let selection_index = wrong_selection["audio_selections"]
        .as_array()
        .unwrap()
        .iter()
        .position(|selection| selection["vocabulary_id"] == ecole)
        .unwrap();
    let other_current = wrong_selection["audio_selections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|selection| selection["vocabulary_id"] == expired)
        .unwrap()["current_version"]
        .clone();
    wrong_selection["audio_selections"][selection_index]["current_version"] = other_current;
    // Keep the original current protected, so this fails for its known wrong-word spec.
    wrong_selection["audio_selections"][selection_index]["previous_version"] = json!(current);
    if read_value(&wrong_selection).is_ok() {
        admitted.push("selected known specification belongs to another word");
    }
    assert!(
        admitted.is_empty(),
        "invalid initial or selected audio was admitted: {admitted:?}"
    );
}

#[test]
fn confirmed_tasks_keep_all_states_budgets_flags_and_repair_intents() {
    let mut connection = database();
    let states = [
        (
            "queued",
            1,
            0,
            NOW + 33,
            None,
            false,
            true,
            false,
            false,
            0,
            2,
            None,
        ),
        (
            "running",
            0,
            2,
            NOW - 1,
            Some(NOW + 100),
            true,
            true,
            true,
            true,
            5,
            3,
            Some("cancel current for committed successor"),
        ),
        (
            "retry_wait",
            1,
            3,
            NOW + 8,
            Some(NOW + 81),
            false,
            true,
            false,
            false,
            3,
            1,
            Some("transient fixture failure"),
        ),
        (
            "ready",
            1,
            1,
            NOW - 100,
            Some(NOW - 80),
            false,
            false,
            false,
            false,
            1,
            2,
            None,
        ),
        (
            "failed",
            0,
            4,
            NOW + 20,
            Some(NOW - 1),
            false,
            false,
            false,
            true,
            4,
            3,
            Some("attempt budget exhausted"),
        ),
        (
            "cancelled",
            1,
            2,
            NOW + 8,
            Some(NOW + 90),
            true,
            true,
            false,
            false,
            2,
            2,
            Some("batch cancelled"),
        ),
    ];
    let mut expected = Vec::new();
    let mut repair = None;
    for (
        index,
        (
            state,
            priority,
            attempts,
            wake,
            deadline,
            cancel,
            refresh,
            successor,
            interactive,
            execution,
            class,
            error,
        ),
    ) in states.into_iter().enumerate()
    {
        let headword = format!("état {state}");
        let word = saved_word(&mut connection, &format!("task-word-{index}"), &headword);
        let requested = NOW - 200 + index as i64;
        let key = request_speech(
            &mut connection,
            &SpeechSpec::word(&headword, SpeechVoice::Female),
            Some(word.id),
            requested,
            false,
        )
        .unwrap();
        if state == "queued" {
            assert!(
                demand_word_repair(
                    &mut connection,
                    &WordDemand {
                        vocabulary_id: word.id,
                        class: 2
                    },
                    NOW
                )
                .unwrap()
            );
            assert!(
                !demand_word_repair(
                    &mut connection,
                    &WordDemand {
                        vocabulary_id: word.id,
                        class: 2
                    },
                    NOW + 1
                )
                .unwrap()
            );
            repair = Some(
                json!({"vocabulary_id":word.id,"failure_identity":"absent:missing","semantic_key":key,"generation":1,"error":null}),
            );
        }
        connection.execute("UPDATE speech_tasks SET priority=?2,state=?3,attempts=?4,wake_at=?5,deadline_at=?6,cancel_requested=?7,refresh_requested=?8,successor_requested=?9,interactive_requested=?10,execution_generation=?11,demand_class=?12,error=?13 WHERE semantic_key=?1",params![key,priority,state,attempts,wake,deadline,cancel,refresh,successor,interactive,execution,class,error]).unwrap();
        let id: i64 = connection
            .query_row(
                "SELECT id FROM speech_tasks WHERE semantic_key=?1",
                [&key],
                |row| row.get(0),
            )
            .unwrap();
        expected.push(json!({"id":id,"semantic_key":key,"priority":priority,"state":state,"attempts":attempts,"wake_at":wake,"deadline_at":deadline,"cancel_requested":cancel,"refresh_requested":refresh,"successor_requested":successor,"interactive_requested":interactive,"execution_generation":execution,"demand_class":class,"requested_at":requested,"error":error}));
    }
    request_speech(
        &mut connection,
        &SpeechSpec::word("temporary task", SpeechVoice::Female),
        None,
        NOW,
        false,
    )
    .unwrap();
    let original = task_rows(&connection);
    assert_eq!(&original[..6], expected.as_slice());
    assert_eq!(original.len(), 7);
    let snapshot = captured(&mut connection);
    read_value(&snapshot).unwrap();
    assert_eq!(snapshot["speech_tasks"], json!(expected));
    assert_eq!(snapshot["audio_repairs"], json!([repair.unwrap()]));
    assert_eq!(
        task_rows(&connection),
        original,
        "capture must not resume work or refresh execution budgets"
    );
}

#[test]
fn interrupted_task_recovery_keeps_original_budgets_except_a_committed_successor() {
    for (name, attempts, deadline, cancel, successor, resulting_state) in [
        ("normal", 2, NOW + 100, false, false, "queued"),
        ("exhausted", 4, NOW + 100, false, false, "failed"),
        ("expired", 2, NOW - 1, false, false, "failed"),
        ("cancelled", 2, NOW + 100, true, false, "cancelled"),
        ("successor", 4, NOW - 1, true, true, "queued"),
    ] {
        let mut connection = database();
        let word = saved_word(&mut connection, "interrupted-word", name);
        let key = request_speech(
            &mut connection,
            &SpeechSpec::word(name, SpeechVoice::Female),
            Some(word.id),
            NOW - 900,
            false,
        )
        .unwrap();
        connection.execute("UPDATE speech_tasks SET state='running',priority=1,attempts=?2,wake_at=?3,deadline_at=?4,cancel_requested=?5,refresh_requested=0,successor_requested=?6,interactive_requested=?6,execution_generation=7,demand_class=2,error='prior execution diagnostic' WHERE semantic_key=?1",params![key,attempts,NOW-30,deadline,cancel,successor]).unwrap();
        let original = task_rows(&connection)[0].clone();
        let before = captured(&mut connection);
        read_value(&before).unwrap();
        assert_eq!(before["speech_tasks"], json!([original]));
        resume_speech(&connection, original["id"].as_i64().unwrap(), NOW).unwrap();
        let mut expected = original.clone();
        expected["state"] = json!(resulting_state);
        expected["wake_at"] = json!(NOW);
        expected["error"] = json!("Speech worker interrupted");
        expected["successor_requested"] = json!(false);
        if successor {
            expected["priority"] = json!(0);
            expected["attempts"] = json!(0);
            expected["deadline_at"] = Value::Null;
            expected["refresh_requested"] = json!(true);
        }
        assert_eq!(
            task_rows(&connection),
            vec![expected.clone()],
            "startup transformation for {name}"
        );
        let after = captured(&mut connection);
        read_value(&after).unwrap();
        assert_eq!(
            after["speech_tasks"],
            json!([expected]),
            "post-recovery snapshot for {name}"
        );
    }
}

#[test]
fn portable_preferences_and_ordered_catalog_exclude_private_device_metadata() {
    let mut connection = database();
    saved_word(&mut connection, "preferences-word", "préférence");
    set_review_settings(
        &connection,
        &ReviewSettings {
            retention_percent: 95,
            new_limit: 3,
            review_limit: 7,
            new_cards_ignore_review_limit: true,
        },
    )
    .unwrap();
    set_auto_enroll(&connection, true).unwrap();
    set_interface_preferences(
        &connection,
        &InterfacePreferences {
            locale: InterfaceLocale::Fr,
            theme: InterfaceTheme::Dark,
        },
    )
    .unwrap();
    set_conjugation_preferences(
        &connection,
        &ConjugationPreferences {
            mode: ConjugationMode::Main,
        },
    )
    .unwrap();
    let order: Vec<String> = [
        "GPP", "GP", "KP", "K", "G", "WP", "W", "YP", "Y", "SQP", "T", "SP", "CP", "FA", "PA", "J",
        "PQP", "S", "C", "F", "I", "PC", "P",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    set_conjugation_block_order(
        &connection,
        &ConjugationBlockOrder::try_from(order.clone()).unwrap(),
    )
    .unwrap();
    set_sort_preference(&connection, "headword-asc").unwrap();
    let legacy_speech = r#"{"voice":"tom","speed":"slow"}"#;
    connection
        .execute(
            "INSERT INTO app_metadata(key,value) VALUES('speech-preferences',?1)",
            [legacy_speech],
        )
        .unwrap();
    let catalog = json!({"next_id":100,"entries":[
        {"id":"mdx-7","title":"Second custom — é","content_hash":"b".repeat(64)},
        {"id":"builtin-fra-eng","title":"English builtin","content_hash":null},
        {"id":"mdx-2","title":"Unknown custom","content_hash":null},
        {"id":"builtin","title":"Chinese builtin","content_hash":null},
    ]});
    write_dictionary_catalog(
        &connection,
        &serde_json::from_value(catalog.clone()).unwrap(),
    )
    .unwrap();
    for key in [
        "device-path",
        "oauth-token",
        "api-key",
        "translation-settings",
        "model-installation",
        "mdx-physical-associations",
        "webview-profile",
        "speech-worker-class1-streak",
        "worker-lease",
        "restore-operation",
    ] {
        connection
            .execute(
                "INSERT INTO app_metadata(key,value) VALUES(?1,?2)",
                params![key, "private synthetic metadata excluded from backups"],
            )
            .unwrap();
    }
    let snapshot = captured(&mut connection);
    read_value(&snapshot).unwrap();
    assert_eq!(
        snapshot["preferences"],
        json!({
            "review":{"retention_percent":95,"new_limit":3,"review_limit":7,"new_cards_ignore_review_limit":true},
            "auto_enroll":true,"speech":{"voice":"male","wpm":100},"interface":{"locale":"fr","theme":"dark"},
            "conjugation":{"mode":"main","block_order":order},"vocabulary_sort":"headword-asc",
        })
    );
    assert_eq!(snapshot["dictionary_catalog"], catalog);
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("private synthetic metadata")
    );
    let original: String = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key='speech-preferences'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        original, legacy_speech,
        "capture must preserve the saved legacy preference rather than rewriting it"
    );
}

#[test]
fn corrupt_saved_preferences_fail_instead_of_becoming_defaults() {
    for (key, value) in [
        ("review-auto-enroll", "TRUE"),
        ("new-cards-ignore-review-limit", "maybe"),
        ("speech-preferences", r#"{"voice":"female","wpm":90}"#),
        (
            "interface-preferences",
            r#"{"locale":"xx","theme":"white"}"#,
        ),
        ("conjugation-preferences", r#"{"mode":"unknown"}"#),
        ("conjugation-block-order", r#"["P","P"]"#),
        ("vocabulary-sort", "unknown"),
        ("dictionary-settings", "{"),
    ] {
        let mut connection = database();
        connection
            .execute(
                "INSERT INTO app_metadata(key,value) VALUES(?1,?2)",
                params![key, value],
            )
            .unwrap();
        assert!(
            export_learning_snapshot(&mut connection, NOW).is_err(),
            "corrupt saved {key} must remain a visible export failure"
        );
    }
}

#[test]
fn legacy_batch_voices_export_canonically_and_archive_aliases_are_rejected() {
    for (legacy, voice, portable) in [
        ("siwis", SpeechVoice::Female, "female"),
        ("tom", SpeechVoice::Male, "male"),
    ] {
        let mut connection = database();
        let word = saved_word(&mut connection, "legacy-voice-word", "voix");
        set_speech_preferences(&connection, &SpeechPreferences { voice, wpm: 180 }).unwrap();
        add_review_batch(&mut connection, "legacy-voice-batch", &[word.id], NOW).unwrap();
        connection
            .execute(
                "UPDATE review_batches SET voice=?1 WHERE operation='legacy-voice-batch'",
                [serde_json::to_string(legacy).unwrap()],
            )
            .unwrap();
        let mut snapshot = captured(&mut connection);
        assert_eq!(snapshot["batches"][0]["voice"], portable);
        read_value(&snapshot).unwrap();
        snapshot["batches"][0]["voice"] = json!(legacy);
        assert!(
            read_value(&snapshot).is_err(),
            "version-one archive must reject nonportable batch voice {legacy}"
        );
    }
}

#[test]
fn every_snapshot_object_rejects_unknown_and_duplicate_json_fields() {
    let baseline = strict_fixture();
    let encoded = serde_json::to_string(&baseline).unwrap();
    let objects = [
        "",
        "/records/0",
        "/records/0/reference",
        "/records/0/digest/0",
        "/records/0/more_examples/0",
        "/save_operations/0",
        "/save_operations/0/request",
        "/save_operations/0/request/more_examples/0",
        "/cards/0",
        "/progress/0",
        "/progress/0/card",
        "/progress/0/card/memory",
        "/ratings/0",
        "/ratings/0/request",
        "/ratings/0/before_state",
        "/ratings/0/result",
        "/ratings/0/result/card",
        "/ratings/0/result/card/memory",
        "/ratings/0/result/model",
        "/ratings/1/before_state/memory",
        "/ratings/0/queue_before",
        "/queue",
        "/queue/context",
        "/queue/entries/0",
        "/batches/0",
        "/batches/0/items/0",
        "/speech_sources/0",
        "/speech_sources/0/spec",
        "/speech_owners/0",
        "/speech_tasks/0",
        "/audio_versions/0",
        "/audio_selections/0",
        "/audio_imports/0",
        "/legacy_audio/0",
        "/audio_operations/0",
        "/audio_publications/0",
        "/audio_repairs/0",
        "/preferences",
        "/preferences/review",
        "/preferences/speech",
        "/preferences/interface",
        "/preferences/conjugation",
        "/dictionary_catalog",
        "/dictionary_catalog/entries/0",
    ];
    for path in objects {
        let mut unknown = baseline.clone();
        unknown
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected_fixture_field".into(), json!(true));
        assert!(read_value(&unknown).is_err(), "unknown field at {path}");
        let object = baseline.pointer(path).unwrap();
        let (field, value) = object.as_object().unwrap().iter().next().unwrap();
        let fragment = serde_json::to_string(object).unwrap();
        let duplicate = format!(
            "{},{}:{}}}",
            &fragment[..fragment.len() - 1],
            serde_json::to_string(field).unwrap(),
            serde_json::to_string(value).unwrap()
        );
        let duplicate_document = encoded.replacen(&fragment, &duplicate, 1);
        assert_ne!(duplicate_document, encoded, "duplicate fixture at {path}");
        assert!(
            read_learning_snapshot(duplicate_document.as_bytes(), NOW + 10).is_err(),
            "duplicate JSON field at {path}"
        );
    }
    for suffix in [" false", " {}", " trailing"] {
        assert!(
            read_learning_snapshot(format!("{encoded}{suffix}").as_bytes(), NOW + 10).is_err(),
            "trailing JSON at {suffix}"
        );
    }
}

#[test]
fn snapshot_types_domains_collections_and_identities_are_strict() {
    let baseline = strict_fixture();
    for (path, value) in [
        ("/snapshot_version", json!("1")),
        ("/snapshot_version", json!(2)),
        ("/records/0/id", json!(1.0)),
        ("/records/0/created_at", json!(9_007_199_254_740_992_i64)),
        ("/records/0/reference/sha256", json!("A".repeat(64))),
        ("/cards/0/kind", json!("production")),
        ("/cards/0/vocabulary_id", json!(999_999)),
        ("/progress/0/revision", json!("1")),
        ("/progress/0/card_id", json!(999_999)),
        ("/ratings/0/sequence", json!(0)),
        ("/ratings/0/request/operation", json!("another-operation")),
        ("/ratings/0/request/expected_revision", json!("0")),
        ("/ratings/0/reviewed_at", json!(NOW)),
        ("/batches/0/sequence", json!(0)),
        ("/batches/0/items/0/ordinal", json!(1)),
        ("/speech_sources/0/semantic_key", json!("a".repeat(64))),
        ("/speech_sources/0/spec/steps", json!("8")),
        ("/speech_owners/0/vocabulary_id", json!(999_999)),
        ("/speech_tasks/0/semantic_key", json!("a".repeat(64))),
        ("/speech_tasks/0/priority", json!(2)),
        ("/speech_tasks/0/attempts", json!(5)),
        ("/speech_tasks/0/demand_class", json!(0)),
        ("/speech_tasks/0/execution_generation", json!(-1)),
        ("/speech_tasks/0/state", json!("paused")),
        ("/speech_tasks/0/wake_at", json!(9_007_199_254_740_992_i64)),
        ("/speech_tasks/0/cancel_requested", json!(1)),
        ("/audio_versions/0/byte_length", json!("36938")),
        ("/audio_versions/0/byte_length", json!(3)),
        ("/audio_versions/0/content_hash", json!("A".repeat(64))),
        ("/audio_versions/0/gc_state", json!("lost")),
        ("/audio_versions/0/payload_status", json!("unknown")),
        ("/audio_selections/0/current_version", json!("a".repeat(64))),
        ("/audio_operations/0/vocabulary_id", json!(999_999)),
        ("/audio_publications/0/locator", json!("a".repeat(64))),
        ("/audio_repairs/0/generation", json!(-1)),
        ("/preferences/auto_enroll", json!("true")),
        ("/preferences/review/retention_percent", json!(100)),
        ("/preferences/speech/wpm", json!(90)),
        ("/preferences/speech/voice", json!("siwis")),
        ("/preferences/conjugation/block_order", json!(["P", "P"])),
        ("/dictionary_catalog/next_id", json!(0)),
        (
            "/dictionary_catalog/entries/0/content_hash",
            json!("A".repeat(64)),
        ),
    ] {
        let mut malformed = baseline.clone();
        *malformed.pointer_mut(path).unwrap() = value;
        assert!(
            read_value(&malformed).is_err(),
            "invalid type, identity or domain at {path}"
        );
    }
    for field in [
        "records",
        "save_operations",
        "cards",
        "progress",
        "ratings",
        "paused_vocabulary_ids",
        "batches",
        "speech_sources",
        "speech_owners",
        "speech_tasks",
        "audio_versions",
        "audio_selections",
        "audio_imports",
        "legacy_audio",
        "audio_operations",
        "audio_publications",
        "audio_repairs",
    ] {
        let mut duplicate = baseline.clone();
        let entries = duplicate[field].as_array_mut().unwrap();
        entries.push(entries[0].clone());
        assert!(
            read_value(&duplicate).is_err(),
            "duplicate identity in {field}"
        );
        let mut null = baseline.clone();
        null[field] = Value::Null;
        assert!(
            read_value(&null).is_err(),
            "required array {field} cannot be null"
        );
        let mut omitted = baseline.clone();
        omitted.as_object_mut().unwrap().remove(field);
        assert!(
            read_value(&omitted).is_err(),
            "required array {field} cannot be omitted"
        );
    }
}
