//! Isolated #24 measurement; never opened by the desktop application.
use std::{collections::HashSet, path::Path, time::Instant};

pub use lexique_storage::StorageError;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

#[path = "../../../apps/desktop/src-tauri/src/speech_cache.rs"]
#[allow(dead_code)]
mod speech_cache;

#[allow(dead_code)]
mod candidate {
    include!(concat!(env!("LEXIQUE_PERF_CANDIDATE_DIR"), "/review.rs"));
}

#[allow(dead_code)]
mod pinned_cache {
    include!(concat!(
        env!("LEXIQUE_PERF_CANDIDATE_DIR"),
        "/speech_cache.rs"
    ));
}

const NOW: i64 = 1_700_000_000;

fn open(path: &Path) -> Connection {
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
        .expect("open synthetic database");
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    lexique_storage::initialize(&connection).unwrap();
    connection
}

fn seed(path: &Path, count: i64, scenario: &str) {
    let mut connection = Connection::open(path).unwrap();
    lexique_storage::initialize(&connection).unwrap();
    let tx = connection.transaction().unwrap();
    let snapshot = json!({"reference": {"dictionary": "synthetic", "lookup_key": "synthetic", "version": "1", "attribution": "synthetic", "license": "MIT", "source_url": "https://example.org", "license_url": "https://example.org", "sha256": "a".repeat(64)}, "digest": []}).to_string();
    for id in 1..=count {
        tx.execute("INSERT INTO vocabulary(id, headword, snapshot, notes, created_at, updated_at) VALUES (?1, ?2, ?3, '', ?4, ?4)", params![id, format!("synthetic-{id}"), snapshot, NOW]).unwrap();
        if scenario == "batch" {
            continue;
        }
        for (card_id, kind) in [(2 * id - 1, "recognition"), (2 * id, "listening")] {
            tx.execute(
                "INSERT INTO review_cards(id, vocabulary_id, kind) VALUES (?1, ?2, ?3)",
                params![card_id, id, kind],
            )
            .unwrap();
            if scenario == "future" || (scenario == "sparse" && id % 10 != 0) {
                let state = json!({"phase":"review", "memory":{"stability": 10.0, "difficulty":5.0}, "last_review_at":NOW-86400, "due_at":NOW+86400}).to_string();
                tx.execute(
                    "INSERT INTO review_progress VALUES (?1, 1, ?2)",
                    params![card_id, state],
                )
                .unwrap();
            }
        }
        if scenario == "paused" && id % 2 == 0 {
            tx.execute("INSERT INTO review_paused VALUES (?1)", [id])
                .unwrap();
        }
    }
    tx.commit().unwrap();
    if scenario == "quota" {
        lexique_storage::set_review_settings(
            &connection,
            &lexique_storage::ReviewSettings {
                retention_percent: 90,
                new_limit: 0,
                review_limit: 0,
                new_cards_ignore_review_limit: false,
            },
        )
        .unwrap();
    }
}

#[derive(Default)]
struct Probes {
    calls: usize,
    bytes: usize,
    missing: usize,
    invalid: usize,
    milliseconds: f64,
}

impl Probes {
    fn inspect(
        &mut self,
        database: &Path,
        id: i64,
        audio_size: usize,
        cache: &str,
        inspector: Option<&pinned_cache::Inspector>,
    ) -> bool {
        self.calls += 1;
        let start = Instant::now();
        let result =
            inspector.map_or_else(|| speech_cache::inspect(database, id), |i| i.inspect(id));
        self.milliseconds += start.elapsed().as_secs_f64() * 1000.0;
        // The fixtures are fixed-size. This is logical payload, not physical disk traffic.
        if cache != "missing" && !(cache == "mixed" && id % 4 == 0) {
            self.bytes += audio_size;
        }
        let expected_missing = cache == "missing" || (cache == "mixed" && id % 4 == 0);
        let expected_invalid = cache == "mixed" && id % 4 == 1;
        match result {
            Ok(true) => {
                assert!(
                    !expected_missing && !expected_invalid,
                    "Unexpected valid cache for synthetic ID {id}"
                );
                true
            }
            Ok(false) => {
                assert!(
                    expected_missing,
                    "Unexpected missing cache for synthetic ID {id}"
                );
                self.missing += 1;
                false
            }
            Err(error) => {
                assert!(
                    expected_invalid && error == "Invalid cached speech audio",
                    "Unexpected cache error for synthetic ID {id}: {error}"
                );
                self.invalid += 1;
                false
            }
        }
    }
}

fn refresh(database: &Path, mode: &str, size: usize, cache: &str, skipped: &HashSet<i64>) -> Value {
    let start = Instant::now();
    let mut connection = open(database);
    let mut probes = Probes::default();
    let inspector = (mode == "pinned").then(|| pinned_cache::Inspector::new(database).unwrap());
    let queue = if mode != "candidate" {
        let inventory = lexique_storage::review_card_inventory(&connection).unwrap();
        let verified: HashSet<_> = inventory
            .iter()
            .filter(|c| c.kind == "listening" && !skipped.contains(&c.id))
            .filter(|c| probes.inspect(database, c.vocabulary_id, size, cache, inspector.as_ref()))
            .map(|c| c.id)
            .collect();
        serde_json::to_value(
            lexique_storage::review_queue_filtered(&mut connection, NOW, "UTC", |c| {
                Ok(!skipped.contains(&c.id) && (c.kind != "listening" || verified.contains(&c.id)))
            })
            .unwrap(),
        )
        .unwrap()
    } else {
        serde_json::to_value(
            candidate::review_queue_filtered(
                &mut connection,
                NOW,
                "UTC",
                |c| !skipped.contains(&c.id),
                |c| {
                    Ok(c.kind != "listening"
                        || probes.inspect(database, c.vocabulary_id, size, cache, None))
                },
            )
            .unwrap(),
        )
        .unwrap()
    };
    let record = queue["cards"].as_array().unwrap().first().map(|card| {
        serde_json::to_value(
            lexique_storage::get(&connection, card["vocabulary_id"].as_i64().unwrap()).unwrap(),
        )
        .unwrap()
    });
    let encoded = serde_json::to_vec(&json!({"queue":queue,"record":record})).unwrap();
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
    json!({"mode":mode, "total_ms":total_ms, "audio_ms":probes.milliseconds, "non_audio_ms":total_ms-probes.milliseconds,
        "audio_probes":probes.calls, "logical_audio_bytes":probes.bytes, "missing":probes.missing, "invalid":probes.invalid,
        "response_bytes":encoded.len(), "queue":queue})
}

fn queue_run(
    root: &Path,
    count: i64,
    scenario: &str,
    cache: &str,
    size: usize,
    iteration: usize,
    pin: bool,
) {
    let database = root.join("Lexique-data/data/baseline.sqlite3");
    let alternative = database.with_file_name("candidate.sqlite3");
    let speech = root.join("Lexique-data/temp/speech");
    if iteration == 0 {
        assert!(
            !root.exists(),
            "measurement must use a new isolated directory"
        );
        std::fs::create_dir_all(database.parent().unwrap()).unwrap();
        seed(&database, count, scenario);
        std::fs::copy(&database, &alternative).unwrap();
        std::fs::create_dir_all(&speech).unwrap();
        let mut audio = vec![b'x'; size];
        audio[..4].copy_from_slice(b"OggS");
        for id in 1..=count {
            if cache == "missing" || (cache == "mixed" && id % 4 == 0) {
                continue;
            }
            if cache == "mixed" && id % 4 == 1 {
                audio[..4].copy_from_slice(b"BAD!");
            } else {
                audio[..4].copy_from_slice(b"OggS");
            }
            std::fs::write(speech.join(format!("{id}.ogg")), &audio).unwrap();
        }
    }
    let skipped = if scenario == "skipped" {
        (1..=count)
            .filter(|id| id % 2 == 0)
            .map(|id| 2 * id)
            .collect()
    } else {
        HashSet::new()
    };
    {
        let other = if pin { "pinned" } else { "candidate" };
        let modes = if iteration % 2 == 0 {
            ["baseline", other]
        } else {
            [other, "baseline"]
        };
        let mut results = Vec::new();
        for mode in modes {
            let path = if mode == "baseline" {
                &database
            } else {
                &alternative
            };
            let mut result = refresh(path, mode, size, cache, &skipped);
            result["iteration"] = json!(iteration);
            if cache == "valid"
                || scenario == "due"
                || scenario == "skipped"
                || scenario == "paused"
            {
                results.push(json!({"queue":result["queue"], "missing":result["missing"], "invalid":result["invalid"]}));
            }
            if mode == "candidate" && (scenario == "future" || scenario == "quota") {
                assert_eq!(
                    result["audio_probes"], 0,
                    "ineligible cards must not read audio"
                );
            }
            println!("{result}");
        }
        if results.len() == 2 {
            assert_eq!(results[0], results[1], "queue semantics differ");
        }
    }
}

fn batch_run(root: &Path, count: i64, chunk: usize) {
    std::fs::create_dir_all(root).unwrap();
    {
        let database = root.join(format!("batch-{chunk}.sqlite3"));
        seed(&database, count, "batch");
        println!(
            "{}",
            json!({"event":"started", "count":count,"chunk":chunk})
        );
        let start = Instant::now();
        let mut commits = 0;
        if chunk == 1 {
            for id in 1..=count {
                lexique_storage::add_review(&mut open(&database), id).unwrap();
                commits += 1;
            }
        } else {
            let mut connection = open(&database);
            for first in (1..=count).step_by(chunk) {
                let tx = connection
                    .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                    .unwrap();
                for id in first..=(first + chunk as i64 - 1).min(count) {
                    candidate::ensure_review_cards(&tx, id).unwrap();
                    candidate::review_cards(&tx, id).unwrap();
                }
                tx.commit().unwrap();
                commits += 1;
            }
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let mut connection = open(&database);
        let inventory = lexique_storage::review_card_inventory(&connection).unwrap();
        assert_eq!(inventory.len(), 2 * count as usize);
        for id in [1, count] {
            let before = lexique_storage::review_cards(&connection, id).unwrap();
            assert_eq!(
                before,
                lexique_storage::add_review(&mut connection, id).unwrap()
            );
        }
        println!(
            "{}",
            json!({"chunk":chunk,"count":count,"total_ms":elapsed,"commits":commits,"cards":inventory.len(),"journal_mode":connection.pragma_query_value(None,"journal_mode",|r|r.get::<_,String>(0)).unwrap(),"synchronous":connection.pragma_query_value(None,"synchronous",|r|r.get::<_,i64>(0)).unwrap()})
        );
    }
}

fn main() {
    assert!(cfg!(windows), "this measures the Windows cache adapter");
    let args: Vec<_> = std::env::args().collect();
    let root = Path::new(&args[1]);
    let count: i64 = args[2].parse().unwrap();
    assert!((1..=100_000).contains(&count));
    if args[3] == "batch" {
        assert!(
            !root.exists(),
            "measurement must use a new isolated directory"
        );
        batch_run(root, count, args.get(4).map_or(1, |s| s.parse().unwrap()));
    } else {
        queue_run(
            root,
            count,
            &args[3],
            &args[4],
            args[5].parse().unwrap(),
            args.get(6).map_or(0, |s| s.parse().unwrap()),
            args.get(7).is_some_and(|s| s == "pin"),
        );
    }
}
