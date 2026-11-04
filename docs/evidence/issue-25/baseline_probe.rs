//! #25 isolated probe. Generated modules are current source plus observation only.
pub use lexique_storage::StorageError;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::{CStr, c_void},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

#[allow(dead_code)]
mod speech_cache {
    include!(concat!(env!("S0_SOURCE"), "/speech_cache.rs"));
}
#[allow(dead_code)]
mod speech_tasks {
    include!(concat!(env!("S0_SOURCE"), "/speech_tasks.rs"));
}
#[allow(dead_code)]
mod review {
    include!(concat!(env!("S0_SOURCE"), "/review.rs"));
}
mod vocabulary {
    include!(concat!(env!("S0_SOURCE"), "/vocabulary.rs"));
}

#[allow(dead_code)]
mod speech_actor {
    include!(concat!(env!("S0_SOURCE"), "/speech_actor.rs"));
}
mod tauri {
    pub struct AppHandle;
    impl AppHandle {
        pub fn emit(&self, _: &str, _: ()) -> Result<(), String> {
            super::metrics::record("review_change_event", 1, 0, 0.0);
            Ok(())
        }
    }
}

mod metrics {
    use super::*;
    static VALUES: Mutex<BTreeMap<String, (u64, u64, f64)>> = Mutex::new(BTreeMap::new());
    static QUEUED: Mutex<BTreeMap<String, Instant>> = Mutex::new(BTreeMap::new());
    static PHASE_SAMPLES: Mutex<BTreeMap<String, Vec<f64>>> = Mutex::new(BTreeMap::new());

    pub fn enqueued(key: String) {
        QUEUED.lock().unwrap().insert(key, Instant::now());
    }
    pub fn claimed(key: &str) {
        if let Some(start) = QUEUED.lock().unwrap().remove(key) {
            record(
                "committed_enqueue_to_claim",
                1,
                0,
                start.elapsed().as_secs_f64() * 1000.0,
            );
        }
    }
    pub fn prepare_publication(
        connection: &Connection,
        key: &str,
        hash: &str,
        now: i64,
    ) -> Result<(), StorageError> {
        measure("publication_intent", || {
            lexique_storage::prepare_speech_publication(connection, key, hash, now)
        })
    }
    pub fn complete_publication(
        connection: &Connection,
        key: &str,
        hash: &str,
    ) -> Result<(), StorageError> {
        measure("publication_db_complete", || {
            lexique_storage::complete_speech_publication(connection, key, hash)
        })
    }

    pub fn record(name: &str, count: u64, bytes: u64, ms: f64) {
        if name.starts_with("publication_")
            || name.starts_with("worker_")
            || name == "committed_enqueue_to_claim"
        {
            PHASE_SAMPLES
                .lock()
                .unwrap()
                .entry(name.to_owned())
                .or_default()
                .push(ms);
        }
        let mut values = VALUES.lock().expect("measurement lock");
        let value = values.entry(name.to_owned()).or_default();
        value.0 += count;
        value.1 += bytes;
        value.2 += ms;
    }
    pub fn measure<T>(name: &str, action: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let result = action();
        record(name, 1, 0, start.elapsed().as_secs_f64() * 1000.0);
        result
    }
    pub fn parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
        measure("json_parse", || serde_json::from_str(text))
    }
    fn category(sql: &str) -> String {
        let verb = sql
            .split_whitespace()
            .next()
            .unwrap_or("other")
            .to_ascii_uppercase();
        let table = [
            "review_queue_entries",
            "review_progress",
            "review_ratings",
            "speech_assets",
            "word_audio",
            "review_cards",
            "review_settings",
            "review_queue_state",
            "app_metadata",
            "vocabulary",
        ]
        .into_iter()
        .find(|name| sql.contains(name))
        .unwrap_or("other");
        format!("sql_{verb}_{table}")
    }
    // SQLite invokes synchronously while the Connection and statement are alive.
    // No bound values or SQL text enter the report; only fixed category counters.
    unsafe extern "C" fn trace(
        event: u32,
        _: *mut c_void,
        statement: *mut c_void,
        extra: *mut c_void,
    ) -> i32 {
        let pointer = unsafe { rusqlite::ffi::sqlite3_sql(statement.cast()) };
        if pointer.is_null() {
            return 0;
        }
        let sql = unsafe { CStr::from_ptr(pointer) }.to_string_lossy();
        let name = category(&sql);
        let (count, rows, ms) = match event {
            1 => (1, 0, 0.0),
            4 => (0, 1, 0.0),
            2 if !extra.is_null() => (0, 0, unsafe { *extra.cast::<u64>() } as f64 / 1_000_000.0),
            _ => return 0,
        };
        record(&name, count, rows, ms);
        0
    }
    pub fn attach(connection: &Connection) {
        let code = unsafe {
            rusqlite::ffi::sqlite3_trace_v2(
                connection.handle(),
                1 | 2 | 4,
                Some(trace),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(code, rusqlite::ffi::SQLITE_OK);
    }
    pub fn take() -> Value {
        let values = std::mem::take(&mut *VALUES.lock().expect("measurement lock"));
        let samples = std::mem::take(&mut *PHASE_SAMPLES.lock().unwrap());
        json!(
            values
                .into_iter()
                .map(|(name, (calls, units, cumulative_ms))| (
                    name.clone(),
                    json!({"calls":calls,"units":units,"cumulative_ms":cumulative_ms,"phase_samples_ms":samples.get(&name)})
                ))
                .collect::<BTreeMap<_, _>>()
        )
    }
}

fn owned(root: &Path) -> PathBuf {
    let root = root.canonicalize().expect("existing isolated fixture");
    let owner = Path::new(env!("S0_REPO"))
        .join("target/issue-25")
        .canonicalize()
        .unwrap();
    assert!(
        root.starts_with(&owner) && root != owner && root.join(".s0-owned").is_file(),
        "Only S0 fixtures may be opened"
    );
    root.join("Lexique-data/data/lexique.sqlite3")
}

fn main() {
    let request: Value = serde_json::from_reader(std::io::stdin()).unwrap();
    let root = Path::new(request["root"].as_str().unwrap());
    let path = owned(root);
    let mode = request["mode"].as_str().unwrap();
    if mode == "init" {
        assert!(!path.exists(), "Initialization must not overwrite data");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let connection = Connection::open(path).unwrap();
        let schema = lexique_storage::initialize(&connection).unwrap();
        println!(
            "{}",
            json!({"schema":schema,"sqlite_version":connection.query_row("SELECT sqlite_version()",[],|r|r.get::<_,String>(0)).unwrap(),
            "sqlite_source_id":connection.query_row("SELECT sqlite_source_id()",[],|r|r.get::<_,String>(0)).unwrap(),
            "journal_mode":connection.query_row("PRAGMA journal_mode",[],|r|r.get::<_,String>(0)).unwrap(),
            "synchronous":connection.query_row("PRAGMA synchronous",[],|r|r.get::<_,i64>(0)).unwrap()})
        );
        return;
    }
    let now = request["now"].as_i64().unwrap();
    let zone = request["timezone"].as_str().unwrap();
    let start = Instant::now();
    let database = vocabulary::DatabasePath(path.clone());
    let result: Result<Value, String> = match mode {
        "worker" => {
            let connection = vocabulary::open(&database).unwrap();
            for (index, text) in [
                "bonjour",
                "merci",
                "maison",
                "école",
                "apprendre",
                "ensemble",
            ]
            .into_iter()
            .enumerate()
            {
                let spec =
                    lexique_storage::SpeechSpec::word(text, lexique_storage::SpeechVoice::Female);
                lexique_storage::demand_speech(&connection, &spec, u8::from(index != 0), now, true)
                    .unwrap();
                metrics::enqueued(spec.key().unwrap());
            }
            let tasks = speech_actor::SpeechTasks::start(
                tauri::AppHandle,
                path.clone(),
                PathBuf::from(request["resources"].as_str().unwrap()),
            );
            let waiting = Instant::now();
            loop {
                let ready: i64 = connection
                    .query_row(
                        "SELECT COUNT(*) FROM speech_tasks WHERE state='ready'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                let failed: i64 = connection
                    .query_row(
                        "SELECT COUNT(*) FROM speech_tasks WHERE state='failed'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap();
                if ready == 6 || failed > 0 || waiting.elapsed().as_secs() >= 45 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            tasks.shutdown().unwrap();
            let ready: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM speech_tasks WHERE state='ready'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            if ready == 6 {
                Ok(json!({"ready":ready,"normal_shutdown":true}))
            } else {
                Err(format!("Only {ready}/6 current Actor tasks completed"))
            }
        }
        "queue" => vocabulary::queue(
            &database,
            zone.into(),
            serde_json::from_value(request["skipped"].clone()).unwrap_or_default(),
            now,
        )
        .map(|q| metrics::measure("json_to_value", || serde_json::to_value(q).unwrap())),
        "rate" => {
            let mut connection = vocabulary::open(&database).unwrap();
            let rate = lexique_storage::RatingRequest {
                operation: request["operation"].as_str().unwrap().into(),
                card_id: request["card_id"].as_i64().unwrap(),
                expected_revision: request["revision"].as_i64().unwrap(),
                rating: lexique_core::scheduling::Rating::Good,
                now,
                timezone: zone.into(),
            };
            lexique_storage::rate_card(&mut connection, &rate)
                .map(|r| serde_json::to_value(r).unwrap())
                .map_err(|e| e.to_string())
        }
        "undo" => lexique_storage::undo_rating(
            &mut vocabulary::open(&database).unwrap(),
            request["operation"].as_str().unwrap(),
            request["rating_operation"].as_str().unwrap(),
        )
        .map(|r| json!(r))
        .map_err(|e| e.to_string()),
        "batch" => {
            let ids: Vec<i64> = serde_json::from_value(request["ids"].clone()).unwrap();
            lexique_storage::add_review_batch(
                &mut vocabulary::open(&database).unwrap(),
                request["operation"].as_str().unwrap(),
                &ids,
                now,
            )
            .map(|r| serde_json::to_value(r).unwrap())
            .map_err(|e| e.to_string())
        }
        "lock" => {
            let blocker = Connection::open(&path).unwrap();
            blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
            let result = vocabulary::queue(&database, zone.into(), vec![], now)
                .map(|q| serde_json::to_value(q).unwrap());
            blocker.execute_batch("ROLLBACK").unwrap();
            result
        }
        "pause" => lexique_storage::set_review_paused(
            &mut vocabulary::open(&database).unwrap(),
            request["word"].as_i64().unwrap(),
            request["paused"].as_bool().unwrap(),
        )
        .map(|_| json!(true))
        .map_err(|e| e.to_string()),
        "read" => {
            let connection = vocabulary::open(&database).unwrap();
            speech_tasks::cached_word(
                &connection,
                &speech_cache::Inspector::new(&path).unwrap(),
                request["word"].as_i64().unwrap(),
                now,
            )
            .map(|bytes| json!({"bytes":bytes.as_ref().map(Vec::len)}))
        }
        _ => panic!("Unknown isolated probe mode"),
    };
    let encoded = metrics::measure("json_encode", || {
        serde_json::to_vec(&result.as_ref().ok()).unwrap()
    });
    let stages = metrics::take();
    println!(
        "{}",
        json!({"status":if result.is_ok(){"passed"}else{"failed"},"result":result.as_ref().ok(),"error":result.as_ref().err(),"ms":start.elapsed().as_secs_f64()*1000.0,"response_bytes":encoded.len(),"stages":stages,
        "boundary":"current desktop blocking body and storage/cache; no IPC/UI or physical cold claim"})
    );
}
