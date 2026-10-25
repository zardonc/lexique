use std::path::PathBuf;

use lexique_dictionary::lookup;
use sha2::{Digest, Sha256};

fn sample() -> (PathBuf, String) {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/desktop-dictionary-sample");
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("report.json")).unwrap()).unwrap();
    (
        directory.join("dictionary.sqlite3"),
        report["sqlite_sha256"].as_str().unwrap().to_owned(),
    )
}

#[test]
fn real_sample_preserves_spelling_translations_and_separate_entries() {
    let (path, hash) = sample();
    let school = lookup(&path, "école", &hash).unwrap();
    assert_eq!(school.source.version, "2025.11.23");
    assert_eq!(school.entries[0].key, "école");
    assert!(
        school.entries[0].senses[0]
            .translations
            .contains(&"学校".to_owned())
    );
    assert_eq!(school.entries[0].pronunciations, ["e.kɔl"]);
    assert_eq!(lookup(&path, "or", &hash).unwrap().entries.len(), 3);
    for key in ["œil", "aujourd’hui", "Côte d’Ivoire", "vacances"] {
        assert!(
            !lookup(&path, key, &hash).unwrap().entries.is_empty(),
            "{key}"
        );
    }
    for key in ["ecole", "École", "aujourd'hui", "sommes", "' OR 1=1 --"] {
        assert!(
            lookup(&path, key, &hash).unwrap().entries.is_empty(),
            "{key}"
        );
    }
}

#[test]
fn failures_and_resource_replacement_preserve_dictionary_and_user_database() {
    let (source, hash) = sample();
    let directory = source
        .parent()
        .unwrap()
        .join(format!("lookup-test-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("dictionary.sqlite3");
    let user_path = directory.join("lexique.sqlite3");
    let connection = rusqlite::Connection::open(&user_path).unwrap();
    lexique_storage::initialize(&connection).unwrap();
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)",
            ["keep", "learning data"],
        )
        .unwrap();
    drop(connection);
    let user_before = std::fs::read(&user_path).unwrap();
    assert_eq!(lookup(&path, "école", &hash).unwrap_err().code, "missing");
    std::fs::create_dir(&path).unwrap();
    assert_eq!(
        lookup(&path, "école", &hash).unwrap_err().code,
        "query_failed"
    );
    std::fs::remove_dir(&path).unwrap();
    std::fs::write(&path, b"damaged resource").unwrap();
    assert_eq!(lookup(&path, "école", &hash).unwrap_err().code, "corrupt");
    let bad_hash = format!("{:x}", Sha256::digest(b"damaged resource"));
    assert_eq!(
        lookup(&path, "école", &bad_hash).unwrap_err().code,
        "corrupt"
    );
    std::fs::copy(&source, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert_eq!(lookup(&path, "or", &hash).unwrap().entries.len(), 3);
    for key in ["", "  ", "a\n", &"a".repeat(513)] {
        assert_eq!(lookup(&path, key, &hash).unwrap_err().code, "invalid_query");
    }
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read(&user_path).unwrap(), user_before);
    let files = std::fs::read_dir(&directory).unwrap().count();
    assert_eq!(
        files, 2,
        "Read-only lookup must not create SQLite sidecar files"
    );
    std::fs::remove_dir_all(directory).unwrap();
}
