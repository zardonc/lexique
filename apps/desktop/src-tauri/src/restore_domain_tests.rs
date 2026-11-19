//! Independent SQLite, replay and media oracles for complete restore timelines.
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

use lexique_storage as storage;
use rusqlite::{Connection, params, types::ValueRef};
use serde_json::{Value, json};

use crate::{
    backup::{OwnedDirectory, export_backup_to_new_file},
    backup_preview::{VerifiedBackupStage, prepare_preview},
    maintenance::MaintenanceCoordinator,
    restore::{PreparedRestore, PreservationKind, StartupPhase},
    speech_cache::Inspector,
    speech_tasks,
    speech_tasks::SpeechTasks,
    vocabulary::DatabasePath,
};

const NOW: i64 = 1_794_564_000;
const BLOCKS: [&str; 23] = [
    "P", "PC", "I", "F", "C", "S", "PQP", "J", "PA", "FA", "CP", "SP", "T", "SQP", "Y", "YP", "W",
    "WP", "G", "K", "KP", "GP", "GPP",
];

struct Fixture {
    root: OwnedDirectory,
    database: DatabasePath,
}

struct Workers(SpeechTasks);

impl std::ops::Deref for Workers {
    type Target = SpeechTasks;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Workers {
    fn drop(&mut self) {
        let _ = self.0.shutdown();
    }
}

impl Fixture {
    fn new() -> Self {
        let root = OwnedDirectory::create(&std::env::temp_dir()).unwrap();
        let directory = root.0.join("Lexique-data/data");
        fs::create_dir_all(&directory).unwrap();
        let database = DatabasePath(directory.join("lexique.sqlite3"));
        storage::initialize(&Connection::open(&database.0).unwrap()).unwrap();
        Self { root, database }
    }

    fn workers(&self) -> Workers {
        Workers(SpeechTasks::start_with_notifications(
            self.database.0.clone(),
            self.root.0.join("no-model"),
            Arc::new(|| Ok(())),
        ))
    }

    fn preview(&self, archive: &Path) -> VerifiedBackupStage {
        prepare_preview(
            archive,
            self.database.0.parent().unwrap(),
            true,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap()
    }

    fn restore(&self, stage: VerifiedBackupStage, id: &str) -> PreparedRestore {
        let workers = self.workers();
        let prepared = crate::restore::prepare_normal(
            &self.database,
            &workers,
            &MaintenanceCoordinator::default(),
            stage,
            id,
            || workers.shutdown(),
            |_| Ok(()),
        )
        .unwrap();
        self.finish_restore(id);
        prepared
    }

    fn finish_restore(&self, id: &str) {
        let status = crate::startup::load(&self.database, |_| Ok(()), || Ok(()));
        assert_eq!(status.state, StartupPhase::CleanupRequired, "{status:?}");
        crate::restore::cleanup_begin(&self.database, id, |_| Ok(())).unwrap();
        crate::restore::cleanup_complete(&self.database, id, |_| Ok(())).unwrap();
        self.reopen();
    }

    fn reopen(&self) {
        let status = crate::startup::load(
            &self.database,
            |_| Ok(()),
            || {
                let connection =
                    Connection::open(&self.database.0).map_err(|error| error.to_string())?;
                storage::initialize(&connection).map_err(|error| error.to_string())?;
                connection.close().map_err(|(_, error)| error.to_string())
            },
        );
        assert_eq!(status.state, StartupPhase::Ready, "{status:?}");
        assert!(status.error.is_none(), "{status:?}");
    }
}

fn reference(headword: &str) -> storage::Reference {
    storage::Reference {
        dictionary_id: Some("mdx-6".into()),
        dictionary: "Removed MIT fixture dictionary".into(),
        lookup_key: headword.into(),
        version: "A".into(),
        attribution: Some("Original fixture text".into()),
        license: Some("MIT".into()),
        source_url: None,
        license_url: None,
        sha256: "a".repeat(64),
    }
}

fn request(operation: &str, headword: &str, notes: &str) -> storage::SaveRequest {
    storage::SaveRequest {
        category_ids: vec![],
        operation: operation.into(),
        headword: headword.into(),
        notes: notes.into(),
        example_fr: "Une école.".into(),
        example_zh: "一所学校".into(),
        example_source: Some(reference(headword)),
        example_sense: Some(0),
        more_examples: vec![
            storage::Example {
                fr: "Je vais à l'école.".into(),
                zh: "我去学校。".into(),
                source: Some(reference(headword)),
            },
            storage::Example {
                fr: "Ancien exemple sans source.".into(),
                zh: String::new(),
                source: None,
            },
        ],
        id: None,
        revision: None,
    }
}

fn save(connection: &mut Connection, request: &storage::SaveRequest) -> i64 {
    let id = storage::save(
        connection,
        request,
        Some(storage::Snapshot {
            reference: reference(&request.headword),
            digest: vec![storage::DigestEntry {
                pronunciations: vec!["ekɔl".into()],
                grammar: vec!["n.f.".into()],
                translation: Some("学校".into()),
                core: None,
            }],
        }),
    )
    .unwrap()
    .record
    .id;
    connection
        .execute(
            "UPDATE vocabulary SET created_at=?1,updated_at=?2 WHERE id=?3",
            params![1_794_564_000_123_i64, 1_794_564_001_456_i64, id],
        )
        .unwrap();
    id
}

fn rate(
    connection: &mut Connection,
    operation: &str,
    card: i64,
    rating: storage::Rating,
    now: i64,
) -> Value {
    let revision = storage::review_progress(connection, card).unwrap().revision;
    storage::rate_card(
        connection,
        &storage::RatingRequest {
            operation: operation.into(),
            card_id: card,
            expected_revision: revision,
            rating,
            now,
            timezone: "UTC".into(),
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "Rating {operation} for card {card} at {now} (revision {revision}) failed: {error:?}"
        )
    })
}

fn preferences(connection: &Connection, timeline_a: bool) {
    storage::set_review_settings(
        connection,
        &storage::ReviewSettings {
            retention_percent: if timeline_a { 87 } else { 92 },
            new_limit: if timeline_a { 2 } else { 20 },
            review_limit: if timeline_a { 3 } else { 50 },
            new_cards_ignore_review_limit: !timeline_a,
        },
    )
    .unwrap();
    storage::set_auto_enroll(connection, timeline_a).unwrap();
    storage::set_speech_preferences(
        connection,
        &storage::SpeechPreferences {
            voice: if timeline_a {
                storage::SpeechVoice::Female
            } else {
                storage::SpeechVoice::Male
            },
            wpm: if timeline_a { 100 } else { 240 },
        },
    )
    .unwrap();
    storage::set_interface_preferences(
        connection,
        &storage::InterfacePreferences {
            locale: if timeline_a {
                storage::InterfaceLocale::Fr
            } else {
                storage::InterfaceLocale::En
            },
            theme: if timeline_a {
                storage::InterfaceTheme::Sepia
            } else {
                storage::InterfaceTheme::Dark
            },
        },
    )
    .unwrap();
    storage::set_conjugation_preferences(
        connection,
        &storage::ConjugationPreferences {
            mode: if timeline_a {
                storage::ConjugationMode::Ending
            } else {
                storage::ConjugationMode::Verbal
            },
        },
    )
    .unwrap();
    let mut blocks: Vec<String> = BLOCKS.iter().map(|block| (*block).into()).collect();
    if timeline_a {
        blocks.reverse();
    }
    storage::set_conjugation_block_order(connection, &blocks.try_into().unwrap()).unwrap();
    storage::set_sort_preference(
        connection,
        if timeline_a {
            "updated-asc"
        } else {
            "headword-desc"
        },
    )
    .unwrap();
    storage::write_dictionary_catalog(
        connection,
        &storage::read_dictionary_catalog(connection).unwrap(),
    )
    .unwrap();
}

fn rows(connection: &Connection, sql: &str, json_columns: &[usize]) -> Value {
    let mut query = connection.prepare(sql).unwrap();
    let width = query.column_count();
    let mut result = query.query([]).unwrap();
    let mut rows = Vec::new();
    while let Some(row) = result.next().unwrap() {
        let values: Vec<Value> = (0..width)
            .map(|column| match row.get_ref(column).unwrap() {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(value) => json!(value),
                ValueRef::Real(value) => json!(value),
                ValueRef::Text(value) => {
                    let text = std::str::from_utf8(value).unwrap();
                    if json_columns.contains(&column) {
                        serde_json::from_str(text).unwrap()
                    } else {
                        json!(text)
                    }
                }
                ValueRef::Blob(_) => panic!("Learning contract fixture has no BLOB columns"),
            })
            .collect();
        rows.push(values);
    }
    json!(rows)
}

fn history(connection: &Connection) -> Value {
    json!({
        "records": rows(connection, "SELECT id,headword,snapshot,notes,example_fr,example_zh,example_source,example_sense,more_examples,revision,created_at,updated_at FROM vocabulary ORDER BY id", &[2,6,8]),
        "saves": rows(connection, "SELECT operation,request,outcome,record_id FROM save_operations ORDER BY operation", &[1]),
        "cards": rows(connection, "SELECT id,vocabulary_id,kind FROM review_cards ORDER BY id", &[]),
        "progress": rows(connection, "SELECT card_id,revision,state FROM review_progress ORDER BY card_id", &[2]),
        "ratings": rows(connection, "SELECT operation,card_id,request,before_state,result,reviewed_at,learning_day,quota,queue_before,undo_operation,undo_queue_rebuilt FROM review_ratings ORDER BY rowid", &[2,3,4,8]),
        "queue_context": rows(connection, "SELECT learning_day,timezone FROM review_queue_state WHERE id=1", &[]),
        "queue": rows(connection, "SELECT card_id,position FROM review_queue_entries ORDER BY position", &[]),
        "paused": rows(connection, "SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id", &[]),
        "review_preferences": rows(connection, "SELECT retention_percent,new_limit,review_limit FROM review_settings WHERE id=1", &[]),
        "portable_preferences": rows(connection, "SELECT key,CASE WHEN key='vocabulary-sort' THEN json_quote(value) ELSE value END FROM app_metadata WHERE key IN ('new-cards-ignore-review-limit','review-auto-enroll','speech-preferences','interface-preferences','conjugation-preferences','conjugation-block-order','vocabulary-sort','dictionary-settings') ORDER BY key", &[1]),
    })
}

fn healthy(connection: &Connection) {
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        19
    );
    assert_eq!(
        rows(connection, "PRAGMA integrity_check", &[]),
        json!([["ok"]])
    );
    assert_eq!(rows(connection, "PRAGMA foreign_key_check", &[]), json!([]));
}

const FEMALE_A: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/female-1.ogg");
const FEMALE_B: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/female-2.ogg");
const MALE_A: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/male-1.ogg");

fn source_audio(
    connection: &Connection,
    inspector: &Inspector,
    key: &str,
    bytes: &[u8],
    now: i64,
) -> storage::AudioVersion {
    let version = storage::prepare_source_audio(
        connection,
        key,
        &storage::content_hash(bytes),
        bytes.len() as i64,
        now,
    )
    .unwrap();
    inspector
        .publish_version(&version.physical_locator, &version.content_hash, bytes)
        .unwrap();
    storage::verify_audio_content(connection, &version.locator, bytes.len() as i64).unwrap();
    assert!(storage::complete_source_audio(connection, &version.locator).unwrap());
    storage::audio_version(connection, &version.locator)
        .unwrap()
        .unwrap()
}

fn selected_word(
    connection: &Connection,
    inspector: &Inspector,
    id: i64,
    bytes: &[u8],
    now: i64,
) -> storage::AudioVersion {
    let key = storage::attach_word_audio_at(connection, id, now).unwrap();
    let generation = if storage::selected_audio(connection, id).unwrap().is_some() {
        storage::begin_audio_change(connection, id, &key).unwrap()
    } else {
        0
    };
    let version = source_audio(connection, inspector, &key, bytes, now);
    assert!(
        storage::reuse_audio_version(connection, id, &version.locator, generation, now).unwrap(),
        "Word {id} must reuse its requested generation {generation}"
    );
    version
}

fn selected_operation(
    connection: &mut Connection,
    inspector: &Inspector,
    id: i64,
    voice: storage::SpeechVoice,
    operation: &str,
    bytes: &[u8],
    now: i64,
) -> (String, i64, storage::AudioVersion) {
    let spec = storage::SpeechSpec::word(&storage::get(connection, id).unwrap().headword, voice);
    let (key, generation) =
        storage::request_selected_speech(connection, &spec, id, now, true, Some(operation))
            .unwrap();
    let execution = storage::claim_speech(connection, now, 0).unwrap().unwrap_or_else(|| panic!("{operation} has no claim at {now}: {}", rows(connection, "SELECT semantic_key,priority,state,attempts,wake_at,deadline_at,execution_generation FROM speech_tasks ORDER BY id", &[])));
    assert_eq!(execution.semantic_key, key);
    let version = source_audio(connection, inspector, &key, bytes, now);
    assert!(
        storage::reuse_audio_version(connection, id, &version.locator, generation, now).unwrap()
    );
    (key, generation, version)
}

fn audio_state(connection: &Connection) -> Value {
    json!({
        "sources": rows(connection, "SELECT semantic_key,spec,context,current_version,pending_version,retain FROM speech_assets ORDER BY semantic_key", &[1]),
        "owners": rows(connection, "SELECT semantic_key,vocabulary_id FROM speech_owners ORDER BY semantic_key,vocabulary_id", &[]),
        "tasks": rows(connection, "SELECT id,semantic_key,priority,state,attempts,wake_at,deadline_at,cancel_requested,refresh_requested,successor_requested,interactive_requested,execution_generation,demand_class,requested_at,error FROM speech_tasks ORDER BY id", &[]),
        "versions": rows(connection, "SELECT locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,execution_generation,context,verification,gc_state,expires_at,orphan_until FROM speech_versions ORDER BY locator", &[]),
        "selections": rows(connection, "SELECT vocabulary_id,current_version,previous_version,previous_until,revision,desired_generation,desired_key,current_generation FROM word_audio_selections ORDER BY vocabulary_id", &[]),
        "imports": rows(connection, "SELECT vocabulary_id,state,error FROM word_audio_imports ORDER BY vocabulary_id", &[]),
        "legacy": rows(connection, "SELECT vocabulary_id,content_hash FROM legacy_speech ORDER BY vocabulary_id", &[]),
        "operations": rows(connection, "SELECT operation,vocabulary_id,semantic_key,generation,refresh FROM word_audio_operations ORDER BY operation", &[]),
        "publications": rows(connection, "SELECT locator,vocabulary_id,expected_revision,generation,execution_generation,error FROM word_audio_publications ORDER BY locator", &[]),
        "repairs": rows(connection, "SELECT vocabulary_id,failure_identity,semantic_key,generation,error FROM word_audio_repairs ORDER BY vocabulary_id,failure_identity", &[]),
        "batches": rows(connection, "SELECT operation,request,voice,created_at,cancelled FROM review_batches ORDER BY rowid", &[1,2]),
        "items": rows(connection, "SELECT operation,ordinal,vocabulary_id,semantic_key,created_cards,error FROM review_batch_items ORDER BY operation,ordinal", &[]),
    })
}

fn omit_unconfirmed(state: &mut Value, keys: &[String]) {
    // These exact fixture sources have no owner or confirmed operation. All other
    // independently queried rows must survive, including excluded version metadata.
    for (collection, key_column) in [("sources", 0), ("tasks", 1), ("versions", 1)] {
        state[collection]
            .as_array_mut()
            .unwrap()
            .retain(|row| !keys.iter().any(|key| row[key_column] == *key));
    }
}

fn archive_snapshot(path: &Path) -> Value {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("snapshot.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    serde_json::from_str(&text).unwrap()
}

fn payload(snapshot: &Value, locator: &str, status: &str, reason: Option<&str>) {
    let version = snapshot["audio_versions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|version| version["locator"] == locator)
        .unwrap();
    assert_eq!(version["payload_status"], status);
    assert_eq!(version["exclusion_reason"], json!(reason));
}

fn normalize_old_fixture_receipt(state: &mut Value) {
    let receipt = state["saves"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row[0] == "legacy-save")
        .unwrap();
    receipt[1] = json!({"operation":"legacy-save","headword":"legacy-valid","notes":"Old receipt, unknown examples","example_fr":"","example_zh":"","example_sense":null,"more_examples":[],"id":null,"revision":null});
}

#[test]
fn restoring_an_earlier_backup_preserves_all_allocated_learning_identities() {
    let fixture = Fixture::new();
    let archive = fixture.root.0.join("before-notes.lexique-backup");
    let workers = fixture.workers();
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &MaintenanceCoordinator::default(),
        &archive,
        "before-notes",
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    workers.shutdown().unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    let category = storage::create_category(&connection, "Temps").unwrap();
    let request = storage::GrammarSaveRequest {
        operation: "later-note".into(),
        id: None,
        revision: None,
        title: "Passé composé".into(),
        body: "Texte\nExemple".into(),
        category_ids: vec![category.id],
    };
    let note = storage::save_grammar(&connection, &request, NOW * 1000)
        .unwrap()
        .record;
    let note = storage::save_grammar_card(
        &connection,
        &storage::GrammarCardRequest {
            operation: "later-card".into(),
            id: note.id,
            revision: note.revision,
            question: "Quand ?".into(),
            answer: "Avant.".into(),
        },
        NOW * 1000,
    )
    .unwrap()
    .record;
    let allocated = storage::learning_identities(&connection).unwrap();
    storage::delete_grammar(&connection, note.id, note.revision).unwrap();
    storage::delete_category(&connection, category.id).unwrap();
    connection.close().unwrap();
    fixture.restore(fixture.preview(&archive), "restore-before-notes");
    let connection = Connection::open(&fixture.database.0).unwrap();
    let category = storage::create_category(&connection, "Pronoms").unwrap();
    let note = storage::save_grammar(
        &connection,
        &storage::GrammarSaveRequest {
            operation: "after-restore-note".into(),
            category_ids: vec![category.id],
            ..request
        },
        NOW * 1000,
    )
    .unwrap()
    .record;
    let note = storage::save_grammar_card(
        &connection,
        &storage::GrammarCardRequest {
            operation: "after-restore-card".into(),
            id: note.id,
            revision: note.revision,
            question: "Qui ?".into(),
            answer: "Elle.".into(),
        },
        NOW * 1000,
    )
    .unwrap()
    .record;
    assert!(note.id > allocated.records);
    assert!(note.card.unwrap().id > allocated.cards);
    assert!(category.id > allocated.categories);
}

#[test]
fn ordinary_restore_keeps_history_replays_preferences_and_a_backup_that_restores_b() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original_request = request("save-ecole", "école", "A notes — accents é");
    let ecole = save(&mut connection, &original_request);
    let etre = save(&mut connection, &request("save-etre", "être", "A être"));
    let suivre = save(
        &mut connection,
        &request("save-suivre", "suivre", "A suivre"),
    );
    let double = save(
        &mut connection,
        &request("save-double", "double", "A double"),
    );
    let ecole_cards = storage::add_review(&mut connection, ecole).unwrap();
    let etre_cards = storage::add_review(&mut connection, etre).unwrap();
    let suivre_cards = storage::add_review(&mut connection, suivre).unwrap();
    let double_cards = storage::add_review(&mut connection, double).unwrap();
    preferences(&connection, true);
    rate(
        &mut connection,
        "old-review",
        suivre_cards[0].id,
        storage::Rating::Easy,
        NOW - 86_400,
    );
    let first = rate(
        &mut connection,
        "z-new",
        ecole_cards[0].id,
        storage::Rating::Good,
        NOW,
    );
    rate(
        &mut connection,
        "a-undone",
        etre_cards[0].id,
        storage::Rating::Again,
        NOW,
    );
    assert!(!storage::undo_rating(&mut connection, "undo-original", "a-undone").unwrap());
    storage::set_review_paused(&mut connection, double, true).unwrap();
    connection
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
        connection
            .execute(
                "INSERT INTO review_queue_entries(card_id,position) VALUES(?1,?2)",
                params![card, position],
            )
            .unwrap();
    }
    let latest = rate(
        &mut connection,
        "m-latest",
        ecole_cards[1].id,
        storage::Rating::Again,
        NOW,
    );
    let expected_a = history(&connection);
    assert_eq!(expected_a["records"][0][10], 1_794_564_000_123_i64);
    assert_eq!(expected_a["records"][0][11], 1_794_564_001_456_i64);
    connection.close().unwrap();
    let workers = fixture.workers();
    let archive = fixture.root.0.join("A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &MaintenanceCoordinator::default(),
        &archive,
        "export-A",
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    workers.shutdown().unwrap();
    let stage = fixture.preview(&archive);

    let mut connection = Connection::open(&fixture.database.0).unwrap();
    preferences(&connection, false);
    let mut update = request("edit-B", "école", "B notes");
    update.id = Some(ecole);
    update.revision = Some(1);
    update.example_fr = "Une phrase B.".into();
    update.example_zh = "B 例句".into();
    update.example_source = None;
    update.more_examples.clear();
    storage::save(&mut connection, &update, None).unwrap();
    storage::set_review_paused(&mut connection, double, false).unwrap();
    let extra = save(
        &mut connection,
        &request("save-B-only", "nouveau", "B only"),
    );
    let extra_cards = storage::add_review(&mut connection, extra).unwrap();
    rate(
        &mut connection,
        "B-rating",
        extra_cards[0].id,
        storage::Rating::Easy,
        NOW + 1_000,
    );
    let expected_b = history(&connection);
    assert_ne!(expected_a, expected_b);
    connection.close().unwrap();
    let prepared = fixture.restore(stage, "restore-A-domain");

    let mut connection = Connection::open(&fixture.database.0).unwrap();
    healthy(&connection);
    assert_eq!(history(&connection), expected_a);
    assert_eq!(
        storage::get(&connection, ecole).unwrap().notes,
        "A notes — accents é"
    );
    assert_eq!(
        storage::get(&connection, ecole).unwrap().more_examples[1].source,
        None
    );
    assert_eq!(
        storage::review_cards(&connection, ecole).unwrap(),
        ecole_cards
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT operation FROM review_ratings ORDER BY rowid",
            &[]
        ),
        json!([["old-review"], ["z-new"], ["a-undone"], ["m-latest"]])
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT c.id FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE p.card_id IS NULL ORDER BY c.id",
            &[]
        ),
        json!([
            [etre_cards[1].id],
            [suivre_cards[1].id],
            [double_cards[0].id],
            [double_cards[1].id]
        ])
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT result FROM review_ratings WHERE operation='z-new'",
            &[0]
        ),
        json!([[first]])
    );
    assert_eq!(
        storage::review_progress(&connection, ecole_cards[0].id)
            .unwrap()
            .card
            .due_at,
        Some(NOW + 600)
    );
    assert_eq!(
        storage::review_availability(&connection, etre_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("quota")
    );
    assert_eq!(
        storage::review_availability(&connection, double_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason
            .as_deref(),
        Some("paused")
    );
    assert_eq!(
        storage::replay(&connection, &original_request)
            .unwrap()
            .unwrap()
            .outcome,
        "replayed"
    );
    let mut changed_save = request("save-ecole", "école", "A notes — accents é");
    changed_save.notes = "same identity, different notes".into();
    assert!(storage::replay(&connection, &changed_save).is_err());
    let latest_request = storage::RatingRequest {
        operation: "m-latest".into(),
        card_id: ecole_cards[1].id,
        expected_revision: 0,
        rating: storage::Rating::Again,
        now: NOW,
        timezone: "UTC".into(),
    };
    assert_eq!(
        storage::rate_card(&mut connection, &latest_request).unwrap(),
        latest
    );
    let mut changed_rating = latest_request;
    changed_rating.rating = storage::Rating::Easy;
    assert!(storage::rate_card(&mut connection, &changed_rating).is_err());
    assert_eq!(
        storage::undo_target(&connection).unwrap().as_deref(),
        Some("m-latest")
    );
    assert!(!storage::undo_rating(&mut connection, "undo-restored", "m-latest").unwrap());
    assert_eq!(
        rows(
            &connection,
            "SELECT card_id,position FROM review_queue_entries ORDER BY position",
            &[]
        ),
        json!([
            [ecole_cards[1].id, 1_000_000],
            [suivre_cards[0].id, 2_000_000],
            [double_cards[1].id, 3_000_000]
        ])
    );
    assert_eq!(storage::undo_target(&connection).unwrap(), None);
    assert!(!storage::undo_rating(&mut connection, "undo-restored", "m-latest").unwrap());
    assert!(storage::undo_rating(&mut connection, "undo-restored", "z-new").is_err());
    assert_eq!(
        storage::review_availability(&connection, etre_cards[1].id, NOW + 1, "UTC")
            .unwrap()
            .reason,
        None
    );
    let next = rate(
        &mut connection,
        "after-restore",
        ecole_cards[1].id,
        storage::Rating::Good,
        NOW + 2,
    );
    assert_eq!(next["card"]["due_at"], NOW + 602);
    connection.close().unwrap();
    fixture.reopen();
    assert_eq!(
        storage::review_progress(
            &Connection::open(&fixture.database.0).unwrap(),
            ecole_cards[1].id
        )
        .unwrap()
        .card
        .due_at,
        Some(NOW + 602)
    );

    let retained = prepared
        .retained_copies
        .iter()
        .find(|copy| copy.kind == PreservationKind::LogicalBackup)
        .unwrap();
    assert_eq!(
        retained.byte_length,
        fs::metadata(&retained.path).unwrap().len()
    );
    assert!(retained.byte_length > 0);
    fixture.restore(
        fixture.preview(&PathBuf::from(&retained.path)),
        "restore-retained-B-domain",
    );
    let connection = Connection::open(&fixture.database.0).unwrap();
    healthy(&connection);
    assert_eq!(history(&connection), expected_b);
    assert_eq!(storage::get(&connection, extra).unwrap().notes, "B only");
}

#[test]
fn ordinary_restore_pairs_audio_bytes_with_original_confirmed_demand_and_retained_b() {
    let fixture = Fixture::new();
    let workers = fixture.workers();
    let cancelled = AtomicBool::new(false);
    let pause = workers.pause(&cancelled).unwrap();
    let time = speech_tasks::now().unwrap();
    let inspector = Inspector::new(&fixture.database.0).unwrap();
    let audio_directory =
        crate::speech_cache::managed_audio_directory(&fixture.database.0).unwrap();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    preferences(&connection, true);
    // Author the old executions explicitly; auto-enrollment would create new
    // demands at SQLite's live clock before these historical requests.
    storage::set_auto_enroll(&connection, false).unwrap();
    let played = save(
        &mut connection,
        &request("save-played", "école", "A audio notes"),
    );
    let cards = storage::add_review(&mut connection, played).unwrap();
    rate(
        &mut connection,
        "audio-A-rating",
        cards[0].id,
        storage::Rating::Good,
        NOW,
    );
    let (_, old_generation, not_selected) = selected_operation(
        &mut connection,
        &inspector,
        played,
        storage::SpeechVoice::Female,
        "audio-old",
        FEMALE_A,
        time - 120,
    );
    let (_, _, previous) = selected_operation(
        &mut connection,
        &inspector,
        played,
        storage::SpeechVoice::Female,
        "audio-middle",
        FEMALE_A,
        time - 90,
    );
    let (played_key, current_generation, current) = selected_operation(
        &mut connection,
        &inspector,
        played,
        storage::SpeechVoice::Female,
        "audio-current",
        FEMALE_B,
        time - 60,
    );
    connection
        .execute(
            "UPDATE word_audio_selections SET previous_until=?1 WHERE vocabulary_id=?2",
            params![time + 3_600, played],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO legacy_speech VALUES(?1,?2)",
            params![played, storage::content_hash(FEMALE_A)],
        )
        .unwrap();
    inspector
        .publish_legacy(played, &storage::content_hash(FEMALE_A), FEMALE_A)
        .unwrap();
    connection.execute("INSERT INTO word_audio_imports(vocabulary_id,state) VALUES(?1,'finished') ON CONFLICT(vocabulary_id) DO UPDATE SET state='finished'", [played]).unwrap();

    let shared_a = save(
        &mut connection,
        &request("save-shared-a", "partage-a", "A shared legacy selection"),
    );
    let shared_b = save(
        &mut connection,
        &request("save-shared-b", "partage-b", "A other shared owner"),
    );
    speech_tasks::publish_selection(
        &connection,
        &inspector,
        shared_a,
        None,
        None,
        MALE_A,
        0,
        time - 50,
    )
    .unwrap();
    let shared = storage::selected_audio(&connection, shared_a)
        .unwrap()
        .unwrap();
    assert!(
        storage::reuse_audio_version(&connection, shared_b, &shared.locator, 0, time - 40).unwrap()
    );

    let expired_id = save(
        &mut connection,
        &request("save-expired", "ancien", "A original previous deadline"),
    );
    let expired = selected_word(
        &connection,
        &inspector,
        expired_id,
        FEMALE_A,
        time - 2_000_000,
    );
    let expired_current = selected_word(
        &connection,
        &inspector,
        expired_id,
        FEMALE_B,
        time - 1_500_000,
    );
    connection
        .execute(
            "UPDATE word_audio_selections SET previous_until=?1 WHERE vocabulary_id=?2",
            params![time - 100, expired_id],
        )
        .unwrap();
    // Protect only the old fixture file during the worker handoff. Its expired
    // selection deadline still independently excludes its payload from the backup.
    connection
        .execute(
            "UPDATE speech_versions SET orphan_until=?1 WHERE locator=?2",
            params![time + 604_800, expired.locator],
        )
        .unwrap();

    let missing_id = save(
        &mut connection,
        &request("save-missing", "manquant", "A missing current"),
    );
    let missing = selected_word(&connection, &inspector, missing_id, FEMALE_B, time - 30);
    let missing_cards = storage::add_review(&mut connection, missing_id).unwrap();
    rate(
        &mut connection,
        "missing-audio-listening-rating",
        missing_cards[1].id,
        storage::Rating::Good,
        NOW + 2,
    );
    fs::remove_file(audio_directory.join(format!(
        "s-{}-{}.ogg",
        missing.physical_locator, missing.content_hash
    )))
    .unwrap();
    connection
        .execute(
            "UPDATE speech_versions SET verification='missing' WHERE locator=?1",
            [&missing.locator],
        )
        .unwrap();
    let invalid_id = save(
        &mut connection,
        &request("save-invalid", "invalide", "A invalid current"),
    );
    let invalid = selected_word(&connection, &inspector, invalid_id, FEMALE_B, time - 30);
    fs::write(
        audio_directory.join(format!(
            "s-{}-{}.ogg",
            invalid.physical_locator, invalid.content_hash
        )),
        FEMALE_A,
    )
    .unwrap();
    connection
        .execute(
            "UPDATE speech_versions SET verification='invalid' WHERE locator=?1",
            [&invalid.locator],
        )
        .unwrap();

    let mut old_request = request(
        "legacy-save",
        "legacy-valid",
        "Old receipt, unknown examples",
    );
    old_request.example_fr.clear();
    old_request.example_zh.clear();
    old_request.example_source = None;
    old_request.example_sense = None;
    old_request.more_examples.clear();
    let legacy_valid = save(&mut connection, &old_request);
    connection.execute("UPDATE save_operations SET request=?1 WHERE operation='legacy-save'", [json!({"operation":"legacy-save","headword":"legacy-valid","notes":"Old receipt, unknown examples","id":null,"revision":null}).to_string()]).unwrap();
    let legacy_missing = save(
        &mut connection,
        &request("save-legacy-missing", "legacy-missing", "A absent legacy"),
    );
    let legacy_invalid = save(
        &mut connection,
        &request("save-legacy-invalid", "legacy-invalid", "A invalid legacy"),
    );
    for (id, bytes, state, error) in [
        (legacy_valid, FEMALE_A, "pending", None),
        (
            legacy_missing,
            FEMALE_B,
            "failed",
            Some("Original absent legacy"),
        ),
        (
            legacy_invalid,
            FEMALE_A,
            "failed",
            Some("Original unrepairable legacy"),
        ),
    ] {
        connection
            .execute(
                "INSERT INTO legacy_speech VALUES(?1,?2)",
                params![id, storage::content_hash(bytes)],
            )
            .unwrap();
        connection.execute("INSERT INTO word_audio_imports(vocabulary_id,state,error) VALUES(?1,?2,?3) ON CONFLICT(vocabulary_id) DO UPDATE SET state=excluded.state,error=excluded.error", params![id,state,error]).unwrap();
    }
    inspector
        .publish_legacy(legacy_valid, &storage::content_hash(FEMALE_A), FEMALE_A)
        .unwrap();
    inspector
        .publish_legacy(legacy_invalid, &storage::content_hash(FEMALE_A), FEMALE_A)
        .unwrap();
    fs::write(
        audio_directory.join(format!("{legacy_invalid}.ogg")),
        FEMALE_B,
    )
    .unwrap();

    let mut demands = Vec::new();
    for (name, state, attempts, cancel, successor, deadline, class, refresh, interactive, error) in [
        (
            "demand-running",
            "running",
            2,
            0,
            0,
            time + 444,
            1,
            0,
            1,
            None,
        ),
        (
            "demand-running-cancel",
            "running",
            1,
            1,
            0,
            time + 333,
            2,
            0,
            0,
            None,
        ),
        (
            "demand-successor",
            "running",
            4,
            1,
            1,
            time - 1,
            3,
            0,
            1,
            None,
        ),
        (
            "demand-exhausted",
            "running",
            4,
            0,
            0,
            time + 222,
            2,
            1,
            0,
            None,
        ),
        (
            "demand-expired",
            "running",
            2,
            0,
            0,
            time - 1,
            2,
            0,
            0,
            None,
        ),
        (
            "demand-retry",
            "retry_wait",
            3,
            0,
            0,
            time + 500,
            2,
            1,
            0,
            Some("Original retry"),
        ),
        (
            "demand-failed",
            "failed",
            4,
            0,
            0,
            time - 111,
            1,
            1,
            1,
            Some("Original exhausted terminal"),
        ),
        (
            "demand-cancelled",
            "cancelled",
            2,
            1,
            0,
            time + 777,
            3,
            0,
            0,
            Some("Explicit cancellation"),
        ),
        (
            "demand-queued-expired",
            "queued",
            3,
            0,
            0,
            time - 1,
            2,
            0,
            0,
            None,
        ),
    ] {
        let id = save(
            &mut connection,
            &request(&format!("save-{name}"), name, "A confirmed intent"),
        );
        let spec = storage::SpeechSpec::word(name, storage::SpeechVoice::Female);
        let (key, _) = storage::request_selected_speech(
            &mut connection,
            &spec,
            id,
            time - 40,
            false,
            Some(name),
        )
        .unwrap();
        connection.execute("UPDATE speech_tasks SET priority=1,state=?1,attempts=?2,wake_at=?3,deadline_at=?4,cancel_requested=?5,refresh_requested=?6,successor_requested=?7,interactive_requested=?8,execution_generation=7,demand_class=?9,requested_at=?10,error=?11 WHERE semantic_key=?12", params![state,attempts,if state=="retry_wait" {time+300} else {time-5},deadline,cancel,refresh,successor,interactive,class,time-40,error,key]).unwrap();
        let task_id: i64 = connection
            .query_row(
                "SELECT id FROM speech_tasks WHERE semantic_key=?1",
                [&key],
                |row| row.get(0),
            )
            .unwrap();
        demands.push((name, id, key, task_id));
    }
    let tail = save(
        &mut connection,
        &request("save-tail", "batch-tail", "A not-yet-committed batch item"),
    );
    let batch_cancelled = save(
        &mut connection,
        &request("save-batch-cancel", "batch-cancel", "A cancelled batch"),
    );
    let batch_request = [demands[0].1, demands[5].1, demands[0].1, 999_999, tail];
    storage::add_review_batch(&mut connection, "z-batch", &batch_request, time).unwrap();
    connection
        .execute(
            "DELETE FROM review_batch_items WHERE operation='z-batch' AND vocabulary_id=?1",
            [tail],
        )
        .unwrap();
    storage::add_review_batch(&mut connection, "a-batch", &[batch_cancelled], time).unwrap();
    storage::cancel_batch(&connection, "a-batch").unwrap();

    let mut pending_versions = Vec::new();
    for (name, word_publication) in [("source-publisher", false), ("word-publisher", true)] {
        let id = save(
            &mut connection,
            &request(
                &format!("save-{name}"),
                name,
                "A sealed but unpublished bytes",
            ),
        );
        let (key, generation) = storage::request_selected_speech(
            &mut connection,
            &storage::SpeechSpec::word(name, storage::SpeechVoice::Female),
            id,
            time - 20,
            true,
            Some(name),
        )
        .unwrap();
        connection.execute("UPDATE speech_tasks SET priority=1,state='running',attempts=2,deadline_at=?1,execution_generation=7 WHERE semantic_key=?2", params![time+500,key]).unwrap();
        let version = if word_publication {
            storage::prepare_audio_version(
                &connection,
                id,
                Some(&key),
                storage::AudioPublicationContent {
                    hash: &storage::content_hash(FEMALE_A),
                    bytes: FEMALE_A.len() as i64,
                    generated_at: Some(time - 10),
                },
                generation,
                time - 10,
            )
            .unwrap()
        } else {
            storage::prepare_source_audio(
                &connection,
                &key,
                &storage::content_hash(FEMALE_A),
                FEMALE_A.len() as i64,
                time - 10,
            )
            .unwrap()
        };
        inspector
            .publish_version(&version.physical_locator, &version.content_hash, FEMALE_A)
            .unwrap();
        storage::verify_audio_content(&connection, &version.locator, FEMALE_A.len() as i64)
            .unwrap();
        if word_publication {
            connection.execute("UPDATE word_audio_publications SET error='Original interrupted publication' WHERE locator=?1", [&version.locator]).unwrap();
        }
        pending_versions.push((id, version));
    }
    for id in [missing_id, legacy_invalid] {
        assert!(
            storage::demand_word_repair(
                &mut connection,
                &storage::WordDemand {
                    vocabulary_id: id,
                    class: 1
                },
                time
            )
            .unwrap()
        );
    }
    connection.execute("UPDATE speech_tasks SET state='failed',attempts=4,error='Repair budget exhausted',deadline_at=?1 WHERE semantic_key=(SELECT semantic_key FROM word_audio_repairs WHERE vocabulary_id=?2)", params![time-1,missing_id]).unwrap();
    let repairs_a = rows(
        &connection,
        "SELECT vocabulary_id,failure_identity,semantic_key,generation,error FROM word_audio_repairs ORDER BY vocabulary_id,failure_identity",
        &[],
    );
    assert!(!repairs_a[0][2].is_null());
    assert_eq!(repairs_a[0][3], 1);
    assert!(repairs_a[1][2].is_null());
    assert!(repairs_a[1][3].is_null());

    let mut unconfirmed = Vec::new();
    for (text, context) in [
        ("Temporary word preview", "word"),
        ("Une phrase temporaire.", "sentence"),
    ] {
        let mut spec = storage::SpeechSpec::word(text, storage::SpeechVoice::Female);
        spec.context = context.into();
        let key = storage::register_speech(&connection, &spec).unwrap();
        source_audio(&connection, &inspector, &key, FEMALE_A, time - 10);
        unconfirmed.push(key);
    }
    storage::set_auto_enroll(&connection, true).unwrap();
    let mut expected_a = audio_state(&connection);
    omit_unconfirmed(&mut expected_a, &unconfirmed);
    let mut history_a = history(&connection);
    normalize_old_fixture_receipt(&mut history_a);
    let archive = fixture.root.0.join("audio-A.lexique-backup");
    crate::backup::export_paused(&fixture.database, &archive, &cancelled).unwrap();
    let snapshot = archive_snapshot(&archive);
    for (version, status, reason) in [
        (&current, "included", None),
        (&previous, "included", None),
        (&shared, "included", None),
        (&expired_current, "included", None),
        (&expired, "excluded", Some("expired")),
        (&not_selected, "excluded", Some("not_selected")),
        (&missing, "source_missing", None),
        (&invalid, "source_invalid", None),
    ] {
        payload(&snapshot, &version.locator, status, reason);
    }
    for (_, version) in &pending_versions {
        payload(&snapshot, &version.locator, "excluded", Some("unpublished"));
    }
    for (id, status, reason) in [
        (played, "excluded", Some("not_selected")),
        (legacy_valid, "included", None),
        (legacy_missing, "source_missing", None),
        (legacy_invalid, "source_invalid", None),
    ] {
        let legacy = snapshot["legacy_audio"]
            .as_array()
            .unwrap()
            .iter()
            .find(|audio| audio["vocabulary_id"] == id)
            .unwrap();
        assert_eq!(legacy["payload_status"], status);
        assert_eq!(legacy["exclusion_reason"], json!(reason));
    }
    let stage = fixture.preview(&archive);
    assert_eq!(stage.manifest().counts.audio_files, 3);
    assert_eq!(stage.manifest().counts.included_audio_versions, 4);
    assert_eq!(stage.manifest().counts.included_legacy_audio, 1);
    assert_eq!(stage.manifest().counts.source_missing_audio, 2);
    assert_eq!(stage.manifest().counts.source_invalid_audio, 2);

    // B is deliberately quiescent. Its retention backup must preserve B, rather
    // than a state changed by task dispatch during the coordinator handoff.
    connection.execute("UPDATE speech_tasks SET state='failed',error='B closed task' WHERE state IN ('queued','running','retry_wait')", []).unwrap();
    preferences(&connection, false);
    let mut edit_b = request("audio-B-edit", "école", "B paired male audio notes");
    edit_b.id = Some(played);
    edit_b.revision = Some(1);
    storage::save(&mut connection, &edit_b, None).unwrap();
    let (_, _, b_current) = selected_operation(
        &mut connection,
        &inspector,
        played,
        storage::SpeechVoice::Male,
        "audio-B-current",
        MALE_A,
        time,
    );
    rate(
        &mut connection,
        "audio-B-rating",
        cards[1].id,
        storage::Rating::Easy,
        NOW + 3,
    );
    let mut expected_b = audio_state(&connection);
    omit_unconfirmed(&mut expected_b, &unconfirmed);
    let mut history_b = history(&connection);
    normalize_old_fixture_receipt(&mut history_b);
    connection.close().unwrap();
    drop(inspector);
    drop(pause);
    let prepared = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &MaintenanceCoordinator::default(),
        stage,
        "restore-A-audio-domain",
        || workers.shutdown(),
        |_| Ok(()),
    )
    .unwrap();
    fixture.finish_restore("restore-A-audio-domain");

    let mut connection = Connection::open(&fixture.database.0).unwrap();
    healthy(&connection);
    assert_eq!(audio_state(&connection), expected_a);
    assert_eq!(history(&connection), history_a);
    assert_eq!(
        rows(
            &connection,
            "SELECT a.semantic_key FROM speech_assets a LEFT JOIN speech_versions c ON c.locator=a.current_version LEFT JOIN speech_versions p ON p.locator=a.pending_version WHERE a.content_hash IS NOT c.content_hash OR a.generated_at IS NOT c.generated_at OR a.expires_at IS NOT c.expires_at OR a.pending_hash IS NOT p.content_hash OR a.pending_generated_at IS NOT p.generated_at OR a.pending_execution IS NOT p.execution_generation ORDER BY a.semantic_key",
            &[]
        ),
        json!([])
    );
    assert_eq!(
        storage::replay(&connection, &old_request)
            .unwrap()
            .unwrap()
            .outcome,
        "replayed"
    );
    let inspector = Inspector::new(&fixture.database.0).unwrap();
    let heard = speech_tasks::cached_word(&connection, &inspector, played, time + 1)
        .unwrap()
        .unwrap();
    assert_eq!(heard.bytes, FEMALE_B);
    assert_eq!(heard.voice, Some(storage::SpeechVoice::Female));
    let restored_previous = storage::audio_version(&connection, &previous.locator)
        .unwrap()
        .unwrap();
    assert_eq!(
        speech_tasks::read_selected(&inspector, &restored_previous)
            .unwrap()
            .unwrap(),
        FEMALE_A
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT previous_until FROM word_audio_selections WHERE vocabulary_id=(SELECT id FROM vocabulary WHERE headword='école')",
            &[]
        ),
        json!([[time + 3_600]])
    );
    for id in [shared_a, shared_b] {
        assert_eq!(
            storage::selected_audio(&connection, id)
                .unwrap()
                .unwrap()
                .locator,
            shared.locator
        );
        assert_eq!(
            speech_tasks::cached_word(&connection, &inspector, id, time + 1)
                .unwrap()
                .unwrap()
                .bytes,
            MALE_A
        );
    }
    for version in [
        &expired,
        &not_selected,
        &missing,
        &invalid,
        &pending_versions[0].1,
        &pending_versions[1].1,
    ] {
        let restored = storage::audio_version(&connection, &version.locator)
            .unwrap()
            .unwrap();
        assert!(
            speech_tasks::read_selected(&inspector, &restored)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        rows(
            &connection,
            "SELECT previous_until FROM word_audio_selections WHERE vocabulary_id=(SELECT id FROM vocabulary WHERE headword='ancien')",
            &[]
        ),
        json!([[time - 100]])
    );
    for id in [
        missing_id,
        invalid_id,
        pending_versions[0].0,
        pending_versions[1].0,
    ] {
        assert!(
            speech_tasks::cached_word(&connection, &inspector, id, time + 1)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(inspector.legacy(legacy_valid).unwrap().unwrap(), FEMALE_A);
    for id in [played, legacy_missing, legacy_invalid] {
        assert!(inspector.legacy(id).unwrap().is_none());
    }
    assert_eq!(
        speech_tasks::cached_word(&connection, &inspector, legacy_valid, time + 1)
            .unwrap()
            .unwrap()
            .bytes,
        FEMALE_A
    );
    for id in [missing_id, legacy_invalid] {
        assert!(
            !storage::demand_word_repair(
                &mut connection,
                &storage::WordDemand {
                    vocabulary_id: id,
                    class: 1
                },
                time + 1
            )
            .unwrap()
        );
    }
    assert_eq!(
        rows(
            &connection,
            "SELECT vocabulary_id,failure_identity,semantic_key,generation,error FROM word_audio_repairs ORDER BY vocabulary_id,failure_identity",
            &[]
        ),
        repairs_a
    );
    assert_eq!(
        storage::audio_demand_diagnostics(&connection, &[missing_id])
            .unwrap()
            .error
            .as_deref(),
        Some("Repair budget exhausted")
    );
    assert_eq!(
        storage::audio_demand_diagnostics(&connection, &[legacy_invalid])
            .unwrap()
            .error
            .as_deref(),
        Some("Original unrepairable legacy")
    );
    assert_eq!(
        storage::review_progress(&connection, missing_cards[1].id)
            .unwrap()
            .card
            .due_at,
        Some(NOW + 602)
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT card_id,revision,state FROM review_progress ORDER BY card_id",
            &[2]
        ),
        history_a["progress"]
    );
    assert_eq!(
        storage::selected_speech_operation(&connection, "audio-old", played, true).unwrap(),
        Some((played_key.clone(), old_generation))
    );
    assert_eq!(
        storage::selected_speech_operation(&connection, "audio-current", played, true).unwrap(),
        Some((played_key.clone(), current_generation))
    );
    assert!(
        storage::selected_speech_operation(&connection, "audio-current", shared_a, true).is_err()
    );
    assert!(
        storage::selected_speech_operation(&connection, "audio-current", played, false).is_err()
    );
    assert!(
        !storage::reuse_audio_version(
            &connection,
            played,
            &not_selected.locator,
            old_generation,
            time + 1
        )
        .unwrap()
    );
    assert_eq!(
        storage::selected_audio(&connection, played)
            .unwrap()
            .unwrap()
            .locator,
        current.locator
    );
    storage::set_speech_preferences(
        &connection,
        &storage::SpeechPreferences {
            voice: storage::SpeechVoice::Male,
            wpm: 240,
        },
    )
    .unwrap();
    let original_spec = storage::SpeechSpec::word("école", storage::SpeechVoice::Female);
    assert_eq!(
        storage::request_selected_speech(
            &mut connection,
            &original_spec,
            played,
            time + 1,
            true,
            Some("audio-old")
        )
        .unwrap(),
        (played_key.clone(), old_generation)
    );
    assert_eq!(
        storage::request_selected_speech(
            &mut connection,
            &original_spec,
            played,
            time + 1,
            true,
            Some("audio-current")
        )
        .unwrap(),
        (played_key.clone(), current_generation)
    );
    assert!(
        storage::request_selected_speech(
            &mut connection,
            &storage::SpeechSpec::word("école", storage::SpeechVoice::Male),
            played,
            time + 1,
            true,
            Some("audio-current")
        )
        .is_err()
    );
    assert_eq!(
        speech_tasks::cached_word(&connection, &inspector, played, time + 1)
            .unwrap()
            .unwrap()
            .voice,
        Some(storage::SpeechVoice::Female)
    );
    assert_eq!(
        storage::latest_batch(&connection)
            .unwrap()
            .unwrap()
            .operation,
        "a-batch"
    );
    assert!(
        storage::batch_status(&connection, "a-batch")
            .unwrap()
            .cancelled
    );
    let partial = storage::batch_status(&connection, "z-batch").unwrap();
    assert_eq!(
        partial
            .items
            .iter()
            .map(|item| item.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![demands[0].1, demands[5].1, 999_999]
    );
    assert!(partial.items[2].error.is_some());
    let completed =
        storage::add_review_batch(&mut connection, "z-batch", &batch_request, time + 1).unwrap();
    assert_eq!(
        completed
            .items
            .iter()
            .map(|item| item.vocabulary_id)
            .collect::<Vec<_>>(),
        vec![demands[0].1, demands[5].1, 999_999, tail]
    );
    assert_eq!(completed.items[3].created_cards, 0);
    let tail_key: String = connection.query_row("SELECT semantic_key FROM review_batch_items WHERE operation='z-batch' AND vocabulary_id=?1", [tail], |row| row.get(0)).unwrap();
    assert_eq!(
        tail_key,
        storage::SpeechSpec::word("batch-tail", storage::SpeechVoice::Female)
            .key()
            .unwrap()
    );
    assert!(storage::add_review_batch(&mut connection, "z-batch", &[tail], time + 1).is_err());

    // Ordinary startup has been observed before any new actor starts. These are
    // the same deterministic storage recovery calls used by the real actor.
    for (_, _, _, task_id) in &demands {
        storage::resume_speech(&connection, *task_id, time + 10).unwrap();
    }
    assert_eq!(
        rows(
            &connection,
            "SELECT w.headword,t.priority,t.state,t.attempts,t.wake_at,t.deadline_at,t.cancel_requested,t.refresh_requested,t.successor_requested,t.interactive_requested,t.execution_generation,t.demand_class,t.requested_at,t.error FROM speech_tasks t JOIN speech_owners o USING(semantic_key) JOIN vocabulary w ON w.id=o.vocabulary_id WHERE w.headword LIKE 'demand-%' ORDER BY w.headword",
            &[]
        ),
        json!([
            [
                "demand-cancelled",
                1,
                "cancelled",
                2,
                time - 5,
                time + 777,
                1,
                0,
                0,
                0,
                7,
                3,
                time - 40,
                "Explicit cancellation"
            ],
            [
                "demand-exhausted",
                1,
                "failed",
                4,
                time + 10,
                time + 222,
                0,
                1,
                0,
                0,
                7,
                2,
                time - 40,
                "Speech worker interrupted"
            ],
            [
                "demand-expired",
                1,
                "failed",
                2,
                time + 10,
                time - 1,
                0,
                0,
                0,
                0,
                7,
                2,
                time - 40,
                "Speech worker interrupted"
            ],
            [
                "demand-failed",
                1,
                "failed",
                4,
                time - 5,
                time - 111,
                0,
                1,
                0,
                1,
                7,
                1,
                time - 40,
                "Original exhausted terminal"
            ],
            [
                "demand-queued-expired",
                1,
                "queued",
                3,
                time - 5,
                time - 1,
                0,
                0,
                0,
                0,
                7,
                2,
                time - 40,
                null
            ],
            [
                "demand-retry",
                1,
                "retry_wait",
                3,
                time + 300,
                time + 500,
                0,
                1,
                0,
                0,
                7,
                2,
                time - 40,
                "Original retry"
            ],
            [
                "demand-running",
                1,
                "queued",
                2,
                time + 10,
                time + 444,
                0,
                0,
                0,
                1,
                7,
                1,
                time - 40,
                "Speech worker interrupted"
            ],
            [
                "demand-running-cancel",
                1,
                "cancelled",
                1,
                time + 10,
                time + 333,
                1,
                0,
                0,
                0,
                7,
                2,
                time - 40,
                "Speech worker interrupted"
            ],
            [
                "demand-successor",
                0,
                "queued",
                0,
                time + 10,
                null,
                1,
                1,
                0,
                1,
                7,
                3,
                time - 40,
                "Speech worker interrupted"
            ],
        ])
    );
    let successor = storage::claim_speech(&mut connection, time + 600, 0)
        .unwrap()
        .unwrap();
    assert_eq!(successor.semantic_key, demands[2].2);
    assert_eq!(successor.attempts, 1);
    assert_eq!(successor.execution_generation, 8);
    assert_eq!(successor.deadline_at, time + 720);
    assert_eq!(
        rows(
            &connection,
            "SELECT attempts,deadline_at,error FROM speech_tasks WHERE semantic_key=(SELECT semantic_key FROM speech_owners WHERE vocabulary_id=(SELECT id FROM vocabulary WHERE headword='demand-queued-expired'))",
            &[]
        ),
        json!([[3, time - 1, "Speech task budget exhausted"]])
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT attempts,deadline_at,error FROM speech_tasks WHERE semantic_key=(SELECT semantic_key FROM speech_owners WHERE vocabulary_id=(SELECT id FROM vocabulary WHERE headword='demand-retry'))",
            &[]
        ),
        json!([[3, time + 500, "Speech task budget exhausted"]])
    );
    connection.close().unwrap();
    drop(inspector);
    fixture.reopen();

    let retained = prepared
        .retained_copies
        .iter()
        .find(|copy| copy.kind == PreservationKind::LogicalBackup)
        .unwrap();
    assert_eq!(
        retained.byte_length,
        fs::metadata(&retained.path).unwrap().len()
    );
    fixture.restore(
        fixture.preview(&PathBuf::from(&retained.path)),
        "restore-retained-B-audio-domain",
    );
    let connection = Connection::open(&fixture.database.0).unwrap();
    healthy(&connection);
    assert_eq!(audio_state(&connection), expected_b);
    assert_eq!(history(&connection), history_b);
    let heard_b = speech_tasks::cached_word(
        &connection,
        &Inspector::new(&fixture.database.0).unwrap(),
        played,
        time + 601,
    )
    .unwrap()
    .unwrap();
    assert_eq!(heard_b.bytes, MALE_A);
    assert_eq!(heard_b.voice, Some(storage::SpeechVoice::Male));
    assert_eq!(
        storage::selected_audio(&connection, played)
            .unwrap()
            .unwrap()
            .locator,
        b_current.locator
    );
    assert_eq!(
        storage::get(&connection, played).unwrap().notes,
        "B paired male audio notes"
    );
}

#[test]
fn ordinary_restore_binds_exact_device_mdx_keeps_shortcuts_and_frozen_sources() {
    const VERBS: &[u8] = include_bytes!("../../e2e/fixtures/mdx/verbs.mdx");
    const UPDATED: &[u8] = include_bytes!("../../e2e/fixtures/mdx/verbs-updated.mdx");
    const OTHER: &[u8] = include_bytes!("../../e2e/fixtures/mdx/other.mdx");
    let verbs_hash = storage::content_hash(VERBS);
    let updated_hash = storage::content_hash(UPDATED);
    let other_hash = storage::content_hash(OTHER);
    let source = Fixture::new();
    let target = Fixture::new();
    assert_ne!(source.database.0, target.database.0);
    super::import(&source.database, "verbs.mdx", VERBS).unwrap();
    super::import(&source.database, "other.mdx", OTHER).unwrap();
    let mut source_connection = Connection::open(&source.database.0).unwrap();
    let mut catalog_a = storage::read_dictionary_catalog(&source_connection).unwrap();
    assert_eq!(
        catalog_a
            .entries
            .iter()
            .find(|entry| entry.id == "mdx-1")
            .unwrap()
            .content_hash
            .as_deref(),
        Some(verbs_hash.as_str())
    );
    assert_eq!(
        catalog_a
            .entries
            .iter()
            .find(|entry| entry.id == "mdx-2")
            .unwrap()
            .content_hash
            .as_deref(),
        Some(other_hash.as_str())
    );
    catalog_a.next_id = 5;
    catalog_a.entries.extend([
        storage::DictionaryCatalogEntry {
            id: "mdx-3".into(),
            title: "missing-updated.mdx".into(),
            content_hash: Some(updated_hash.clone()),
        },
        storage::DictionaryCatalogEntry {
            id: "mdx-4".into(),
            title: "unknown.mdx".into(),
            content_hash: None,
        },
    ]);
    storage::write_dictionary_catalog(&source_connection, &catalog_a).unwrap();
    let order: Vec<String> = [
        "mdx-4",
        "builtin-fra-eng",
        "mdx-2",
        "mdx-1",
        "mdx-3",
        "builtin",
        "builtin-core",
    ]
    .map(String::from)
    .into();
    super::settings(&source.database, Some(order.clone())).unwrap();
    catalog_a = storage::read_dictionary_catalog(&source_connection).unwrap();
    let mut frozen_reference = reference("être");
    frozen_reference.dictionary_id = Some("mdx-1".into());
    frozen_reference.dictionary = "verbs.mdx".into();
    frozen_reference.sha256 = verbs_hash.clone();
    let mut original_request = request(
        "save-frozen-mdx",
        "être",
        "A frozen note and original digest",
    );
    original_request.example_source = Some(frozen_reference.clone());
    let mut missing_reference = reference("être");
    missing_reference.dictionary_id = Some("mdx-3".into());
    missing_reference.sha256 = updated_hash.clone();
    original_request.more_examples[0].source = Some(missing_reference);
    let original_record = storage::save(
        &mut source_connection,
        &original_request,
        Some(storage::Snapshot {
            reference: frozen_reference,
            digest: vec![storage::DigestEntry {
                pronunciations: vec!["ɛtʁ".into()],
                grammar: vec!["v.".into()],
                translation: Some("Original to be".into()),
                core: None,
            }],
        }),
    )
    .unwrap()
    .record;
    save(
        &mut source_connection,
        &request(
            "save-removed-mdx",
            "ancien",
            "A retained removed mdx-6 identity",
        ),
    );
    storage::add_review(&mut source_connection, original_record.id).unwrap();
    preferences(&source_connection, true);
    for (key, value) in [
        ("selection-shortcut", "Ctrl+Alt+Shift+F"),
        ("ocr-shortcut", "Ctrl+Alt+Shift+G"),
    ] {
        source_connection
            .execute(
                "INSERT INTO app_metadata(key,value) VALUES(?1,?2)",
                params![key, json!(value).to_string()],
            )
            .unwrap();
    }
    let expected_a = history(&source_connection);
    source_connection.close().unwrap();
    let archive = source.root.0.join("mdx-A.lexique-backup");
    let source_workers = source.workers();
    export_backup_to_new_file(
        &source.database,
        &source_workers,
        &MaintenanceCoordinator::default(),
        &archive,
        "export-A-mdx-domain",
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    source_workers.shutdown().unwrap();

    // Target ID/title 1 is wrong; ID 4 is the exact original file. ID 3 has stale
    // verified metadata but changed real bytes. Neither ID nor title is a match.
    super::import(&target.database, "verbs.mdx", OTHER).unwrap();
    let mut target_connection = Connection::open(&target.database.0).unwrap();
    let mut target_catalog = storage::read_dictionary_catalog(&target_connection).unwrap();
    target_catalog.next_id = 3;
    storage::write_dictionary_catalog(&target_connection, &target_catalog).unwrap();
    super::import(&target.database, "stale-updated.mdx", UPDATED).unwrap();
    super::import(&target.database, "unknown.mdx", VERBS).unwrap();
    fs::write(
        super::custom_path(&target.database, "mdx-3").unwrap(),
        OTHER,
    )
    .unwrap();
    fs::write(
        super::custom_path(&target.database, "mdx-9").unwrap(),
        OTHER,
    )
    .unwrap();
    let mut files = super::read_files(&target_connection).unwrap();
    files.files.push(super::ManagedFile {
        id: "mdx-9".into(),
        file_id: 9,
        content_hash: Some(other_hash.clone()),
    });
    files.bindings.insert("mdx-9".into(), 9);
    super::write_files(&target_connection, &files).unwrap();
    target_catalog = storage::read_dictionary_catalog(&target_connection).unwrap();
    target_catalog.next_id = 10;
    target_catalog
        .entries
        .push(storage::DictionaryCatalogEntry {
            id: "mdx-9".into(),
            title: "retained.mdx".into(),
            content_hash: Some(other_hash.clone()),
        });
    storage::write_dictionary_catalog(&target_connection, &target_catalog).unwrap();
    save(
        &mut target_connection,
        &request("save-target-only", "target-only", "B target learning"),
    );
    preferences(&target_connection, false);
    for (key, value) in [
        ("selection-shortcut", "Ctrl+Alt+Shift+J"),
        ("ocr-shortcut", "Ctrl+Alt+Shift+K"),
    ] {
        target_connection
            .execute(
                "INSERT INTO app_metadata(key,value) VALUES(?1,?2)",
                params![key, json!(value).to_string()],
            )
            .unwrap();
    }
    let target_files =
        serde_json::to_value(super::read_files(&target_connection).unwrap().files).unwrap();
    let retained_physical: Vec<_> = [
        ("mdx-1", OTHER),
        ("mdx-3", OTHER),
        ("mdx-4", VERBS),
        ("mdx-9", OTHER),
    ]
    .into_iter()
    .map(|(id, bytes)| (super::custom_path(&target.database, id).unwrap(), bytes))
    .collect();
    target_connection.close().unwrap();
    target.restore(target.preview(&archive), "restore-A-mdx-domain");

    let connection = Connection::open(&target.database.0).unwrap();
    healthy(&connection);
    assert_eq!(history(&connection), expected_a);
    assert_eq!(
        storage::read_dictionary_catalog(&connection).unwrap(),
        catalog_a
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT key,value FROM app_metadata WHERE key IN ('selection-shortcut','ocr-shortcut') ORDER BY key",
            &[1]
        ),
        json!([
            ["ocr-shortcut", "Ctrl+Alt+Shift+K"],
            ["selection-shortcut", "Ctrl+Alt+Shift+J"]
        ])
    );
    assert_eq!(
        rows(
            &connection,
            "SELECT id FROM vocabulary WHERE headword='target-only'",
            &[]
        ),
        json!([])
    );
    let restored_files = super::read_files(&connection).unwrap();
    assert_eq!(
        serde_json::to_value(restored_files.files).unwrap(),
        target_files
    );
    assert_eq!(
        serde_json::to_value(restored_files.bindings).unwrap(),
        json!({"mdx-1":4,"mdx-2":1})
    );
    let status = super::settings(&target.database, None).unwrap();
    assert_eq!(
        status
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>(),
        order
    );
    for entry in &status {
        assert_eq!(
            entry.availability,
            if matches!(entry.id.as_str(), "mdx-3" | "mdx-4") {
                "missing"
            } else {
                "available"
            }
        );
    }
    let registry = super::Registry::default();
    let builtin = target.root.0.join("absent-builtin.sqlite3");
    let read = |id: &str, html: &str, hash: &str| {
        let lookup = registry
            .lookup(&target.database, &builtin, "être", Some(id))
            .unwrap();
        assert_eq!(lookup.dictionary_id.as_deref(), Some(id));
        assert_eq!(lookup.sha256, hash);
        assert_eq!(lookup.entries[0].html.as_deref(), Some(html));
    };
    read("mdx-1", "<p>to be</p>", &verbs_hash);
    read("mdx-2", "<p>another dictionary</p>", &other_hash);
    for id in ["mdx-3", "mdx-4"] {
        assert_eq!(
            registry
                .lookup(&target.database, &builtin, "être", Some(id))
                .unwrap_err()
                .code,
            "missing"
        );
    }
    let imported = super::import(&target.database, "fresh.mdx", OTHER).unwrap();
    assert_eq!(imported.last().unwrap().id, "mdx-10");
    assert_eq!(
        storage::read_dictionary_catalog(&connection)
            .unwrap()
            .next_id,
        11
    );
    assert!(matches!(
        super::relink(
            &target.database,
            "mdx-3",
            "same-original.mdx",
            UPDATED,
            None
        )
        .unwrap(),
        super::RelinkResult::Linked { .. }
    ));
    read("mdx-3", "<p>to be, revised</p>", &updated_hash);
    match super::relink(
        &target.database,
        "mdx-1",
        "changed-version.mdx",
        UPDATED,
        None,
    )
    .unwrap()
    {
        super::RelinkResult::ConfirmationRequired {
            dictionary_id,
            expected_hash,
            content_hash,
        } => {
            assert_eq!(dictionary_id, "mdx-1");
            assert_eq!(expected_hash, Some(verbs_hash.clone()));
            assert_eq!(content_hash, updated_hash);
        }
        _ => panic!("Changed content must await explicit confirmation"),
    }
    read("mdx-1", "<p>to be</p>", &verbs_hash);
    assert!(matches!(
        super::relink(
            &target.database,
            "mdx-1",
            "changed-version.mdx",
            UPDATED,
            Some(&updated_hash)
        )
        .unwrap(),
        super::RelinkResult::Linked { .. }
    ));
    read("mdx-1", "<p>to be, revised</p>", &updated_hash);
    match super::relink(
        &target.database,
        "mdx-4",
        "unknown-original.mdx",
        VERBS,
        None,
    )
    .unwrap()
    {
        super::RelinkResult::ConfirmationRequired {
            dictionary_id,
            expected_hash,
            content_hash,
        } => {
            assert_eq!(dictionary_id, "mdx-4");
            assert_eq!(expected_hash, None);
            assert_eq!(content_hash, verbs_hash);
        }
        _ => panic!("Unknown content must await explicit confirmation"),
    }
    assert_eq!(
        registry
            .lookup(&target.database, &builtin, "être", Some("mdx-4"))
            .unwrap_err()
            .code,
        "missing"
    );
    assert!(matches!(
        super::relink(
            &target.database,
            "mdx-4",
            "unknown-original.mdx",
            VERBS,
            Some(&verbs_hash)
        )
        .unwrap(),
        super::RelinkResult::Linked { .. }
    ));
    read("mdx-4", "<p>to be</p>", &verbs_hash);
    assert_eq!(history(&connection)["records"], expected_a["records"]);
    assert_eq!(history(&connection)["saves"], expected_a["saves"]);
    assert_eq!(
        serde_json::to_value(storage::get(&connection, original_record.id).unwrap()).unwrap(),
        serde_json::to_value(original_record).unwrap()
    );
    assert_eq!(
        storage::replay(&connection, &original_request)
            .unwrap()
            .unwrap()
            .outcome,
        "replayed"
    );
    for (path, bytes) in retained_physical {
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    healthy(&connection);
}
