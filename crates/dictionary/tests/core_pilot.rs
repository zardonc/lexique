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
        ("je", vec![6], "je"),
        ("de", vec![7], "de"),
        ("ne", vec![8], "ne"),
        ("pas", vec![9, 10], "pas"),
        ("le", vec![11, 12], "le"),
        ("la", vec![13, 14], "la"),
        ("tu", vec![15], "tu"),
        ("vous", vec![16], "vous"),
        ("il", vec![17], "il"),
        ("et", vec![18], "et"),
        ("un", vec![19, 20, 21], "un"),
        ("à", vec![22], "à"),
        ("l'", vec![23, 24], "l'"),
        ("qui", vec![25, 26], "qui"),
        ("aller", vec![27, 28], "aller"),
        ("les", vec![29, 30], "les"),
        ("en", vec![31, 32, 33], "en"),
        ("ça", vec![34, 35], "ça"),
        ("faire", vec![36], "faire"),
        ("tout", vec![37, 38, 39, 40, 41], "tout"),
        ("on", vec![42], "on"),
        ("que", vec![43, 44], "que"),
        ("une", vec![45, 46, 47], "une"),
        ("ce", vec![48, 49], "ce"),
        ("d'", vec![50], "d'"),
        ("pour", vec![51, 52], "pour"),
        ("des", vec![53], "des"),
        ("dire", vec![54, 55], "dire"),
        ("pouvoir", vec![56, 57], "pouvoir"),
        ("vouloir", vec![58], "vouloir"),
        ("mais", vec![59], "mais"),
        ("me", vec![60], "me"),
        ("nous", vec![61], "nous"),
        ("dans", vec![62], "dans"),
        ("savoir", vec![63, 64], "savoir"),
        ("elle", vec![65], "elle"),
        ("du", vec![66], "du"),
        ("y", vec![67, 68], "y"),
        ("bien", vec![69, 70, 71], "bien"),
        ("où", vec![72, 73], "où"),
        ("t'", vec![74], "t'"),
        ("voir", vec![75], "voir"),
        ("plus", vec![76, 77], "plus"),
        ("non", vec![78], "non"),
        ("te", vec![79], "te"),
        ("mon", vec![80], "mon"),
        ("au", vec![81], "au"),
        ("avec", vec![82], "avec"),
        ("moi", vec![83, 84], "moi"),
        ("si", vec![85, 86], "si"),
        ("quoi", vec![88, 89], "quoi"),
        ("devoir", vec![90, 91], "devoir"),
        ("s'", vec![92, 93], "s'"),
        ("oui", vec![94], "oui"),
        ("ils", vec![95], "ils"),
        ("comme", vec![96, 97], "comme"),
        ("se", vec![98], "se"),
        ("venir", vec![99], "venir"),
        ("sur", vec![100, 101], "sur"),
        ("toi", vec![102], "toi"),
        ("ici", vec![103], "ici"),
        ("rien", vec![104], "rien"),
        ("ma", vec![105], "ma"),
        ("lui", vec![106], "lui"),
        ("bon", vec![107, 108, 109, 110], "bon"),
        ("là", vec![111, 112], "là"),
        ("suivre", vec![113], "suivre"),
        ("pourquoi", vec![114, 115], "pourquoi"),
        ("parler", vec![116, 117], "parler"),
        ("prendre", vec![118], "prendre"),
        ("cette", vec![119], "cette"),
        ("votre", vec![120], "votre"),
        ("quand", vec![121], "quand"),
        ("son", vec![122, 123], "son"),
        ("alors", vec![124], "alors"),
        ("ton", vec![125, 126], "ton"),
        ("chose", vec![127], "chose"),
        ("par", vec![128, 129], "par"),
        ("croire", vec![130], "croire"),
        ("aimer", vec![131], "aimer"),
        ("falloir", vec![132], "falloir"),
        ("comment", vec![133, 134, 135], "comment"),
        ("très", vec![136], "très"),
        ("ou", vec![137], "ou"),
        ("passer", vec![138], "passer"),
        ("penser", vec![139], "penser"),
        ("aussi", vec![140], "aussi"),
        ("jamais", vec![141], "jamais"),
        ("même", vec![142, 143, 144], "même"),
        ("attendre", vec![145], "attendre"),
        ("petit", vec![146, 147, 148], "petit"),
        ("trouver", vec![149], "trouver"),
        ("laisser", vec![150], "laisser"),
        ("merci", vec![151, 152], "merci"),
        ("quel", vec![243], "quel"),
        ("leur", vec![260, 261], "leur"),
        ("dont", vec![306], "dont"),
        ("oeil", vec![328], "oeil"),
        ("ok", vec![356, 357], "ok"),
        ("côté", vec![418], "côté"),
        ("aujourd'hui", vec![360], "aujourd'hui"),
        ("fort", vec![422, 423, 424], "fort"),
        ("coeur", vec![483], "coeur"),
        ("côte", vec![1911], "côte"),
        ("cœur", vec![483], "coeur"),
        ("œil", vec![328], "oeil"),
        ("OK", vec![356, 357], "ok"),
        ("aujourd’hui", vec![360], "aujourd'hui"),
        ("l’", vec![23, 24], "l'"),
        ("d’", vec![50], "d'"),
        ("t’", vec![74], "t'"),
        ("s’", vec![92, 93], "s'"),
        ("e\u{0302}tre", vec![1, 2, 3], "être"),
    ] {
        let lookup = core::lookup(&path, query, &expected).unwrap();
        assert_eq!(lookup.dictionary_id.as_deref(), Some("builtin-core"));
        assert_eq!(json!(lookup.source.version), curated["content_version"]);
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
    for query in ["cote", "côtés", "coeurs", "aujourdhui", "l", "d"] {
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
fn complete_core_fields_at_the_character_budget_are_retained_and_oversized_content_fails() {
    let fixture = Fixture::new();
    fixture.copy();
    let path = fixture.database();
    let mut connection = Connection::open(&path).unwrap();
    connection
        .execute_batch("DELETE FROM senses WHERE headword <> 'être'; VACUUM;")
        .unwrap();
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
    for (field, character) in [
        ("definition_fr", "é"),
        ("meaning_en", "e"),
        ("meaning_zh", "字"),
        ("usage", "u"),
    ] {
        payload[field] = json!(character.repeat(20_000));
    }
    payload["field_sources"]["definition_fr"]["span"] = json!("s".repeat(20_000));
    connection
        .execute(
            "UPDATE senses SET payload=?1 WHERE frozen_row=1 AND sense_index=1",
            [payload.to_string()],
        )
        .unwrap();
    let lookup = core::lookup(&path, "être", &hash(&path)).unwrap();
    assert_eq!(lookup.entries.len(), 3);
    assert_eq!(
        serde_json::to_value(&lookup.entries[0].core_senses[0]).unwrap(),
        payload
    );
    payload["definition_fr"] = json!("é".repeat(20_001));
    connection
        .execute(
            "UPDATE senses SET payload=?1 WHERE frozen_row=1 AND sense_index=1",
            [payload.to_string()],
        )
        .unwrap();
    let error = core::lookup(&path, "être", &hash(&path)).unwrap_err();
    assert_eq!(error.code, "corrupt");
    assert!(error.message.contains("French definition"));
    assert!(error.message.contains("20000"));
    payload["definition_fr"] = json!("é".repeat(20_000));
    let transaction = connection.transaction().unwrap();
    for index in 1..=40 {
        payload["sense_id"] = json!(format!("byte-budget-{index}"));
        transaction
            .execute(
                "INSERT OR REPLACE INTO senses VALUES(1,?1,'être',?2)",
                params![index, payload.to_string()],
            )
            .unwrap();
    }
    transaction.commit().unwrap();
    assert!(fs::metadata(&path).unwrap().len() < core::MAX_RESOURCE_BYTES);
    let retained = fs::read(&path).unwrap();
    let error = core::lookup(&path, "être", &hash(&path)).unwrap_err();
    assert_eq!(error.code, "limit");
    assert!(error.message.contains("4 MiB"));
    assert_eq!(fs::read(&path).unwrap(), retained);
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
    let mut connection = Connection::open(&path).unwrap();
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
    let transaction = connection.transaction().unwrap();
    transaction
        .execute(
            "DELETE FROM senses WHERE frozen_row != 1 OR sense_index != 1",
            [],
        )
        .unwrap();
    for index in 2..=201 {
        payload["sense_id"] = json!(format!("limit-sense-{index}"));
        transaction
            .execute(
                "INSERT INTO senses VALUES(1,?1,'être',?2)",
                params![index, payload.to_string()],
            )
            .unwrap();
    }
    transaction.commit().unwrap();
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
