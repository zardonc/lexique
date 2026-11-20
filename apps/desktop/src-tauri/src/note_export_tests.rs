use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use lexique_storage::{NoteExportScope, capture_note_export, get, initialize, save};
use rusqlite::Connection;

use super::write_note_csv;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);
static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/ticket-185/files")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&directory).unwrap();
        Self(directory)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn save_fixture(
    connection: &mut Connection,
    word: &str,
    notes: &str,
    examples: serde_json::Value,
) -> i64 {
    let request = serde_json::from_value(serde_json::json!({
        "operation": format!("fixture-{}", NEXT_OPERATION.fetch_add(1, Ordering::Relaxed)), "headword": word, "notes": notes,
        "example_fr": "", "example_zh": "", "example_sense": null,
        "more_examples": examples, "id": null, "revision": null
    }))
    .unwrap();
    let snapshot = serde_json::from_value(serde_json::json!({
        "reference": {"dictionary": "Private dictionary", "lookup_key": word, "version": "1",
            "attribution": null, "license": null, "source_url": null, "license_url": null,
            "sha256": "a".repeat(64)},
        "digest": [{"pronunciations": ["ipa"], "grammar": ["private grammar"], "translation": "private summary"}]
    })).unwrap();
    save(connection, &request, Some(snapshot))
        .unwrap()
        .record
        .id
}

fn read_rows(path: &std::path::Path) -> Vec<Vec<String>> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(&bytes[..3], &[0xef, 0xbb, 0xbf]);
    let mut reader = csv::Reader::from_reader(bytes.as_slice());
    assert_eq!(
        reader.headers().unwrap().iter().collect::<Vec<_>>(),
        ["headword", "notes", "example_fr", "example_zh"]
    );
    reader
        .records()
        .map(|record| record.unwrap().iter().map(String::from).collect())
        .collect()
}

#[test]
fn csv_files_preserve_saved_unicode_whitespace_and_all_ordered_example_pairs() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(fixture.0.join("learning.sqlite3")).unwrap();
    initialize(&connection).unwrap();
    let id = save_fixture(
        &mut connection,
        "école",
        "  学校, \"notes\"\r\n最后一行\n  ",
        serde_json::json!([]),
    );
    let mut request: lexique_storage::SaveRequest = serde_json::from_value(serde_json::json!({
        "operation": "examples", "headword": "école", "notes": "  学校, \"notes\"\r\n最后一行\n  ",
        "example_fr": "  À l'école, \"bonjour\".\nEncore.  ", "example_zh": "  中文\n译文。  ", "example_sense": 0,
        "more_examples": [{"fr":"œuf", "zh":""}, {"fr":"", "zh":"单侧"},
            {"fr":"répété", "zh":"重复"}, {"fr":"répété", "zh":"重复"}],
        "id": id, "revision": 1
    })).unwrap();
    save(&mut connection, &request, None).unwrap();
    save_fixture(&mut connection, "bonjour", "无例句", serde_json::json!([]));
    let snapshot = capture_note_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    request.operation = "after-snapshot".into();
    request.revision = Some(2);
    request.notes = "new text".into();
    save(&mut connection, &request, None).unwrap();
    let destination = fixture.0.join("single.csv");
    write_note_csv(&snapshot, &destination).unwrap();
    let rows = read_rows(&destination);
    assert_eq!(snapshot.row_count(), 5);
    assert_eq!(
        rows,
        vec![
            vec![
                "école",
                "  学校, \"notes\"\r\n最后一行\n  ",
                "  À l'école, \"bonjour\".\nEncore.  ",
                "  中文\n译文。  "
            ],
            vec!["école", "  学校, \"notes\"\r\n最后一行\n  ", "œuf", ""],
            vec!["école", "  学校, \"notes\"\r\n最后一行\n  ", "", "单侧"],
            vec![
                "école",
                "  学校, \"notes\"\r\n最后一行\n  ",
                "répété",
                "重复"
            ],
            vec![
                "école",
                "  学校, \"notes\"\r\n最后一行\n  ",
                "répété",
                "重复"
            ],
        ]
    );
    assert_eq!(get(&connection, id).unwrap().notes, "new text");
    let all = capture_note_export(
        &mut connection,
        &NoteExportScope::All {
            sort: "headword-asc".into(),
        },
    )
    .unwrap();
    write_note_csv(&all, &fixture.0.join("all.csv")).unwrap();
    let rows = read_rows(&fixture.0.join("all.csv"));
    assert_eq!(all.records.len(), 2);
    assert_eq!(all.row_count(), 6);
    assert_eq!(rows[0], ["bonjour", "无例句", "", ""]);
    let search = serde_json::from_value(serde_json::json!({"kind":"search", "query":"new text", "sort":"headword-desc", "category":"all"})).unwrap();
    let snapshot = capture_note_export(&mut connection, &search).unwrap();
    write_note_csv(&snapshot, &fixture.0.join("search.csv")).unwrap();
    assert_eq!(snapshot.records.len(), 1);
    assert_eq!(
        read_rows(&fixture.0.join("search.csv"))[0],
        [
            "école",
            "new text",
            "  À l'école, \"bonjour\".\nEncore.  ",
            "  中文\n译文。  "
        ]
    );
}

#[test]
fn failed_csv_write_keeps_learning_data_usable_and_empty_scope_creates_no_file() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(fixture.0.join("learning.sqlite3")).unwrap();
    initialize(&connection).unwrap();
    let empty = capture_note_export(
        &mut connection,
        &NoteExportScope::All {
            sort: "created-asc".into(),
        },
    )
    .unwrap();
    assert!(
        write_note_csv(&empty, &fixture.0.join("empty.csv"))
            .unwrap_err()
            .contains("No saved vocabulary")
    );
    assert!(!fixture.0.join("empty.csv").exists());
    let id = save_fixture(
        &mut connection,
        "école",
        "unchanged",
        serde_json::json!([
            {"fr":"", "zh":""}, {"fr":"", "zh":""}
        ]),
    );
    let snapshot = capture_note_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    assert!(snapshot.records[0].examples.is_empty());
    assert_eq!(snapshot.row_count(), 1);
    assert!(
        write_note_csv(&snapshot, &fixture.0)
            .unwrap_err()
            .contains("Could not write CSV file")
    );
    drop(connection);
    let connection = Connection::open(fixture.0.join("learning.sqlite3")).unwrap();
    assert_eq!(get(&connection, id).unwrap().notes, "unchanged");
}
