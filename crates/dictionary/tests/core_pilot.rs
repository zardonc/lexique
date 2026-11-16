use lexique_dictionary::core;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn resource() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/desktop-core-pilot/core.sqlite3")
}

fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "lexique-core-pilot-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn database(&self) -> PathBuf {
        self.0.join("core.sqlite3")
    }
    fn copy(&self) {
        fs::copy(resource(), self.database()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn reviewed_groups_keep_all_four_fields_frozen_pos_order_and_only_evidenced_query_bindings() {
    let path = resource();
    let expected = hash(&path);
    let curated: Value = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../resources/core-pilot-79/curated-inputs.json"),
        )
        .unwrap(),
    )
    .unwrap();
    for (query, rows, frozen) in [
        ("être", vec![1, 2, 3], "être"),
        ("avoir", vec![4, 5], "avoir"),
        ("bien", vec![70, 71], "bien"),
        ("aujourd'hui", vec![360], "aujourd'hui"),
        ("fort", vec![422, 423], "fort"),
        ("coeur", vec![483], "coeur"),
        ("côte", vec![1911], "côte"),
        ("cœur", vec![483], "coeur"),
        ("aujourd’hui", vec![360], "aujourd'hui"),
        ("e\u{0302}tre", vec![1, 2, 3], "être"),
    ] {
        let lookup = core::lookup(&path, query, &expected).unwrap();
        assert_eq!(lookup.dictionary_id.as_deref(), Some("builtin-core"));
        assert_eq!(lookup.source.version, "pilot-79-v1");
        assert_eq!(
            lookup
                .entries
                .iter()
                .map(|entry| entry.ordinal)
                .collect::<Vec<_>>(),
            rows
        );
        for entry in lookup.entries {
            assert_eq!(entry.key, frozen);
            assert!(entry.html.is_none());
            for sense in entry.core_senses {
                sense.validate().unwrap();
                let group = curated["groups"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|group| group["frozen_row"] == sense.frozen_row)
                    .unwrap();
                let selected = &group["senses"][0];
                assert_eq!(json!(sense.pos), group["lexique_cgram"]);
                assert_eq!(json!(sense.usage), selected["use"]);
                for (name, value) in [
                    ("french_definition", sense.definition_fr),
                    ("ipa", sense.ipa),
                    ("english", sense.meaning_en),
                    ("chinese", sense.meaning_zh),
                ] {
                    assert_eq!(json!(value), selected["fields"][name]["text"]);
                }
            }
        }
    }
    for query in ["cote", "côté", "coeurs", "aujourdhui"] {
        assert!(
            core::lookup(&path, query, &expected)
                .unwrap()
                .entries
                .is_empty(),
            "{query}"
        );
    }
    let connection = Connection::open(&path).unwrap();
    let version: String = connection
        .query_row(
            "SELECT value FROM metadata WHERE key='format_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "1");
}

#[test]
fn missing_corrupt_future_and_oversized_core_resources_have_explicit_errors() {
    let fixture = Fixture::new();
    let path = fixture.database();
    assert_eq!(
        core::lookup(&path, "être", &"a".repeat(64))
            .unwrap_err()
            .code,
        "missing"
    );
    fs::write(&path, b"invalid sqlite").unwrap();
    assert_eq!(
        core::lookup(&path, "être", &hash(&path)).unwrap_err().code,
        "corrupt"
    );
    fixture.copy();
    let original = hash(&path);
    Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE metadata SET value='2' WHERE key='format_version'",
            [],
        )
        .unwrap();
    assert_eq!(
        core::lookup(&path, "être", &original).unwrap_err().code,
        "corrupt"
    );
    assert_eq!(
        core::lookup(&path, "être", &hash(&path)).unwrap_err().code,
        "unsupported"
    );
    fixture.copy();
    let connection = Connection::open(&path).unwrap();
    let mut payload: Value = serde_json::from_str(
        &connection
            .query_row::<String, _, _>(
                "SELECT payload FROM senses WHERE frozen_row=1 AND sense_index=1",
                [],
                |row| row.get(0),
            )
            .unwrap(),
    )
    .unwrap();
    for index in 2..=201 {
        payload["sense_id"] = json!(format!("limit-sense-{index}"));
        connection
            .execute(
                "INSERT INTO senses VALUES(1,?1,'être',?2)",
                params![index, payload.to_string()],
            )
            .unwrap();
    }
    drop(connection);
    assert_eq!(
        core::lookup(&path, "être", &hash(&path)).unwrap_err().code,
        "limit"
    );
    for query in [String::new(), "x".repeat(513), "être\n".into()] {
        assert_eq!(
            core::lookup(&path, &query, &hash(&path)).unwrap_err().code,
            "invalid_query"
        );
    }
}
