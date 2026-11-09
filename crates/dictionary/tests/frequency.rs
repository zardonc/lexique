use std::path::PathBuf;

use lexique_dictionary::frequency;

fn sample() -> (PathBuf, String) {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/desktop-frequency-v1");
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("report.json")).unwrap()).unwrap();
    (
        directory.join("frequency.sqlite3"),
        report["sqlite_sha256"].as_str().unwrap().to_owned(),
    )
}

#[test]
fn notebook_batch_uses_each_lemma_pos_once_and_preserves_accents() {
    let (path, hash) = sample();
    let keys = ["a priori", "abaisser", "cote", "côte", "CÔTÉ"].map(str::to_owned);
    let result = frequency::lookup(&path, &keys, &hash).unwrap();
    assert_eq!(result.source.version, "3.83");
    assert_eq!(result.metric, "freqlemfilms2");
    assert_eq!(result.entries.len(), keys.len());
    for (entry, (key, value)) in result
        .entries
        .iter()
        .zip(keys.iter().zip([1.45, 4.93, 8.48, 35.16, 285.99]))
    {
        assert_eq!(&entry.key, key);
        assert_eq!(entry.per_million, Some(value));
        assert_eq!(entry.reason, None);
    }
    assert_eq!(result.entries[0].parts.len(), 2);
}

#[test]
fn forms_require_one_lemma_and_missing_pos_retains_its_source_value() {
    let (path, hash) = sample();
    let keys = [
        "suis",
        "étaient",
        "a",
        "o",
        "a contrario",
        "mot-non-enregistré-lexique57",
    ]
    .map(str::to_owned);
    let result = frequency::lookup(&path, &keys, &hash).unwrap();
    let entries = result.entries;
    assert_eq!(entries[0].reason, Some(frequency::Reason::Ambiguous));
    assert_eq!(entries[0].per_million, None);
    assert_eq!(entries[0].candidates, ["suivre", "être"]);
    assert_eq!(entries[1].lemma.as_deref(), Some("être"));
    assert_eq!(entries[1].per_million, Some(40411.43));
    assert_eq!(entries[2].lemma.as_deref(), Some("a"));
    assert_eq!(entries[2].per_million, Some(81.36));
    assert_eq!(entries[3].reason, Some(frequency::Reason::MissingPos));
    assert_eq!(entries[3].per_million, None);
    let missing = entries[3]
        .parts
        .iter()
        .find(|part| part.part_of_speech.is_none())
        .unwrap();
    assert_eq!(missing.per_million, Some(0.01));
    assert_eq!(missing.reason, Some(frequency::Reason::MissingPos));
    assert_eq!(entries[4].reason, None);
    assert_eq!(entries[4].per_million, Some(0.0));
    assert_eq!(entries[5].reason, Some(frequency::Reason::NotFound));
    assert_eq!(entries[5].per_million, None);
}

#[test]
fn resource_failures_and_invalid_batches_preserve_the_read_only_source() {
    let (source, hash) = sample();
    let directory = source
        .parent()
        .unwrap()
        .join(format!("frequency-test-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("frequency.sqlite3");
    let keys = ["école".to_owned()];
    assert_eq!(
        frequency::lookup(&path, &keys, &hash).unwrap_err().code,
        "missing"
    );
    std::fs::write(&path, b"damaged resource").unwrap();
    assert_eq!(
        frequency::lookup(&path, &keys, &hash).unwrap_err().code,
        "corrupt"
    );
    std::fs::copy(&source, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    for keys in [
        vec![],
        vec!["a".to_owned(); 101],
        vec![" ".to_owned()],
        vec!["a\n".to_owned()],
        vec!["a".repeat(513)],
    ] {
        assert_eq!(
            frequency::lookup(&path, &keys, &hash).unwrap_err().code,
            "invalid_query"
        );
    }
    assert_eq!(
        frequency::lookup(&path, &vec!["école".to_owned(); 100], &hash)
            .unwrap()
            .entries
            .len(),
        100
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(
        std::fs::read_dir(&directory).unwrap().count(),
        1,
        "Lookup must not create SQLite sidecars"
    );
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
