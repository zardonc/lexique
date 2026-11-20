use super::*;
use lexique_storage::*;
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

#[path = "note_import_batch_tests.rs"]
mod batch;

fn preview_file(
    connection: &Connection,
    path: &Path,
    operation: &str,
) -> Result<NoteImportPreview, String> {
    preview_csv_file(connection, path, operation, &NoteImportOptions::default())
}
fn commit_creates(
    connection: &mut Connection,
    preview: &NoteImportPreview,
) -> Result<NoteImportResult, String> {
    let selections = preview
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| NoteImportSelection {
            index,
            action: if group.status == "identical" {
                NoteImportAction::Skip
            } else {
                NoteImportAction::Create
            },
        })
        .collect::<Vec<_>>();
    commit_preview(connection, preview, &selections)
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lexique-note-import-184-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn database(&self) -> PathBuf {
        self.0.join("lexique.sqlite3")
    }
    fn open(&self) -> Connection {
        let connection = Connection::open(self.database()).unwrap();
        initialize(&connection).unwrap();
        connection
    }
    fn csv(&self, rows: &[Vec<String>]) -> PathBuf {
        let path = self.0.join("école.csv");
        let mut writer = csv::Writer::from_path(&path).unwrap();
        writer
            .write_record(["headword", "notes", "example_fr", "example_zh"])
            .unwrap();
        for row in rows {
            writer.write_record(row).unwrap();
        }
        writer.flush().unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn row(word: &str, notes: &str, fr: &str, zh: &str) -> Vec<String> {
    [word, notes, fr, zh].map(String::from).into()
}

#[test]
fn batch_csv_groups_nonadjacent_nfc_rows_and_saves_distinct_identities() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.csv(&[
        row("école", "notes", "premier", "一"),
        row("École", "uppercase", "", ""),
        row("e\u{301}cole", "notes", "second", "二"),
    ]);
    let preview = preview_file(&connection, &path, "batch-nfc").unwrap();
    assert_eq!(
        preview.groups[0].status, "create",
        "A valid multi-word file must preview successfully"
    );
    commit_creates(&mut connection, &preview).unwrap();
    let reopened = fixture.open();
    assert_eq!(list(&reopened, "", "headword-asc").unwrap().len(), 2);
    let saved = find(&reopened, "e\u{301}cole").unwrap().unwrap();
    assert_eq!(saved.example_fr, "premier");
    assert_eq!(saved.more_examples[0].fr, "second");
}

#[test]
fn canonical_csv_saves_external_notes_and_ordered_examples_then_reopens_for_editing() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    set_auto_enroll(&connection, true).unwrap();
    let path = fixture.csv(&[
        row(
            "école",
            "中文笔记\n\"引用\",重音",
            "Je vais à l’école.",
            "我去学校。",
        ),
        row("e\u{301}cole", "中文笔记\n\"引用\",重音", "", "单侧例句"),
        row(
            "école",
            "中文笔记\n\"引用\",重音",
            "Je vais à l’école.",
            "我去学校。",
        ),
    ]);
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend(std::fs::read(&path).unwrap());
    std::fs::write(&path, bytes).unwrap();
    let preview = preview_file(&connection, &path, "csv-one").unwrap();
    assert_eq!(preview.groups[0].status, "create");
    let saved = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
    assert!(saved.reference.is_none());
    assert!(saved.digest.is_empty());
    assert!(review_cards(&connection, saved.id).unwrap().is_empty());
    let before = export_learning_snapshot(&mut connection, 1_800_000_000).unwrap();
    assert!(before.speech_tasks.is_empty());
    assert!(before.speech_owners.is_empty());
    drop(connection);
    let mut connection = fixture.open();
    let record = find(&connection, "e\u{301}cole").unwrap().unwrap();
    assert_eq!(record.import_source.as_ref().unwrap().filename, "école.csv");
    assert_eq!(record.notes, "中文笔记\n\"引用\",重音");
    assert_eq!(record.more_examples[0].zh, "单侧例句");
    assert_eq!(record.more_examples[1].fr, "Je vais à l’école.");
    let mut edit = preview.groups[0]
        .content
        .create_request("csv-edit")
        .unwrap();
    edit.id = Some(record.id);
    edit.revision = Some(record.revision);
    edit.notes = "修改后笔记".into();
    edit.example_fr = "Une nouvelle phrase.".into();
    edit.more_examples[0].fr = "Une traduction à compléter.".into();
    save(&mut connection, &edit, None).unwrap();
    drop(connection);
    let connection = fixture.open();
    let reopened = get(&connection, saved.id).unwrap();
    assert_eq!(reopened.notes, "修改后笔记");
    assert_eq!(reopened.example_fr, "Une nouvelle phrase.");
    assert!(reopened.example_sense.is_none());
    assert!(reopened.example_source.is_none());
    assert!(reopened.more_examples.iter().all(|e| e.source.is_none()));
    assert_eq!(
        list(&connection, "修改后", "headword-asc").unwrap().len(),
        1
    );
    assert!(
        get_review_records(&connection, &[saved.id])
            .unwrap()
            .contains_key(&saved.id)
    );
}

#[test]
fn csv_entry_accepts_unicode_limits_and_reports_oversized_or_illegal_fields_without_writing() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    for (field, allowed, rejected) in [
        (0, 200, 201),
        (1, 20_000, 20_001),
        (2, 20_000, 20_001),
        (3, 20_000, 20_001),
    ] {
        for (count, status) in [(allowed, "create"), (rejected, "invalid")] {
            let mut fields = row(&format!("limite-{field}"), "", "", "");
            fields[field] = "🦉".repeat(count);
            let preview = preview_file(
                &connection,
                &fixture.csv(&[fields]),
                &format!("limit-{field}-{count}"),
            )
            .unwrap();
            assert_eq!(preview.groups[0].status, status, "field {field}, {count}");
            if status == "create" {
                let record = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
                let actual = [
                    &record.headword,
                    &record.notes,
                    &record.example_fr,
                    &record.example_zh,
                ][field];
                assert_eq!(actual.chars().count(), allowed);
            } else {
                assert!(preview.groups[0].error.as_ref().unwrap().contains("Row 2"));
                assert!(commit_creates(&mut connection, &preview).is_err());
            }
        }
    }
    for word in ["", "  ", "bad\nword", "bad\u{7f}word"] {
        let preview = preview_file(
            &connection,
            &fixture.csv(&[row(word, "", "", "")]),
            "bad-headword",
        )
        .unwrap();
        assert_eq!(preview.groups[0].status, "invalid");
        assert!(commit_creates(&mut connection, &preview).is_err());
    }
    for count in [101, 102] {
        let rows = (0..count)
            .map(|i| row(&format!("examples-{count}"), "", &format!("Phrase {i}"), ""))
            .collect::<Vec<_>>();
        let preview =
            preview_file(&connection, &fixture.csv(&rows), &format!("pairs-{count}")).unwrap();
        if count == 101 {
            let record = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
            assert_eq!(record.more_examples.len(), 100);
            assert_eq!(record.more_examples[99].fr, "Phrase 100");
        } else {
            assert_eq!(preview.groups[0].status, "invalid");
            assert!(commit_creates(&mut connection, &preview).is_err());
        }
    }
}

#[test]
fn same_content_skips_differences_block_and_creation_after_preview_never_overwrites() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.csv(&[row("identité", "original", "", "")]);
    let preview = preview_file(&connection, &path, "first").unwrap();
    let saved = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
    assert_eq!(
        commit_creates(&mut connection, &preview).unwrap().created,
        1
    );
    let identical = preview_file(&connection, &path, "identical").unwrap();
    assert_eq!(identical.groups[0].status, "identical");
    assert_eq!(
        commit_creates(&mut connection, &identical).unwrap().skipped,
        1
    );
    assert_eq!(get(&connection, saved.id).unwrap().revision, 1);
    let different = preview_file(
        &connection,
        &fixture.csv(&[row("identité", "different", "", "")]),
        "different",
    )
    .unwrap();
    assert_eq!(different.groups[0].status, "different");
    assert!(commit_creates(&mut connection, &different).is_err());
    let path = fixture.csv(&[row("concurrent", "file", "", "")]);
    let preview = preview_file(&connection, &path, "stale").unwrap();
    let mut other = fixture.open();
    let other_preview = preview_file(&other, &path, "other-save").unwrap();
    let other_record = commit_creates(&mut other, &other_preview).unwrap().records[0].clone();
    assert!(
        commit_creates(&mut connection, &preview)
            .unwrap_err()
            .contains("after preview")
    );
    assert_eq!(get(&connection, other_record.id).unwrap().notes, "file");
}

#[test]
fn unreadable_invalid_csv_and_failed_sqlite_writes_keep_existing_learning_data() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.csv(&[row("protected", "keep", "", "")]);
    let preview = preview_file(&connection, &path, "protected").unwrap();
    commit_creates(&mut connection, &preview).unwrap();
    assert!(preview_file(&connection, &fixture.0.join("missing.csv"), "missing").is_err());
    for bytes in [
        b"headword,notes,example_fr,example_zh\nword,too,few\n".as_slice(),
        b"headword,notes,example_fr,example_zh\n\xff,,,\n",
        b"unknown,notes,extra\nword,notes,extra\n",
    ] {
        std::fs::write(&path, bytes).unwrap();
        assert!(
            preview_file(&connection, &path, "bad-file")
                .unwrap()
                .error
                .is_some()
        );
    }
    let preview = preview_file(
        &connection,
        &fixture.csv(&[row("blocked", "", "", "")]),
        "blocked",
    )
    .unwrap();
    connection.execute_batch("CREATE TRIGGER fail_note_import BEFORE INSERT ON vocabulary BEGIN SELECT RAISE(ABORT,'fixture write failure'); END;").unwrap();
    assert!(
        commit_creates(&mut connection, &preview)
            .unwrap_err()
            .contains("fixture write failure")
    );
    assert!(find(&connection, "blocked").unwrap().is_none());
    assert_eq!(
        list(&connection, "", "headword-asc").unwrap()[0].notes,
        "keep"
    );
}

#[test]
fn external_manual_double_cards_keep_progress_and_pause_and_mixed_real_backup_restores_every_record()
 {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let preview = preview_file(
        &connection,
        &fixture.csv(&[row("externe", "imported", "Une phrase.", "")]),
        "external",
    )
    .unwrap();
    let external = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
    let cards = add_review(&mut connection, external.id).unwrap();
    assert_eq!(cards.len(), 2);
    let recognition = cards.iter().find(|c| c.kind == "recognition").unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "external-rate".into(),
            card_id: recognition.id,
            expected_revision: 0,
            rating: serde_json::from_str("\"good\"").unwrap(),
            now: 1_800_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    let progress =
        serde_json::to_value(review_progress(&connection, recognition.id).unwrap()).unwrap();
    set_review_paused(&mut connection, external.id, true).unwrap();
    assert_eq!(add_review(&mut connection, external.id).unwrap(), cards);
    assert!(review_paused(&connection, external.id).unwrap());
    assert_eq!(
        serde_json::to_value(review_progress(&connection, recognition.id).unwrap()).unwrap(),
        progress
    );
    let category = create_category(&connection, "Shared category").unwrap();
    let grammar = save_grammar(
        &connection,
        &GrammarSaveRequest {
            operation: "grammar".into(),
            id: None,
            revision: None,
            title: "Grammaire".into(),
            body: "Règle".into(),
            category_ids: vec![category.id],
        },
        1_800_000_000,
    )
    .unwrap()
    .record;
    let mut request = NoteImportContent {
        headword: "dictionary-word".into(),
        notes: Some("retained".into()),
        examples: None,
    }
    .create_request("dictionary-word")
    .unwrap();
    request.category_ids = vec![category.id];
    let word = save(
        &mut connection,
        &request,
        Some(Snapshot {
            reference: Reference {
                dictionary_id: None,
                dictionary: "Fixture".into(),
                lookup_key: "dictionary-word".into(),
                version: "1".into(),
                attribution: None,
                license: Some("MIT".into()),
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![DigestEntry {
                pronunciations: vec![],
                grammar: vec![],
                translation: Some("original".into()),
                core: None,
            }],
        }),
    )
    .unwrap()
    .record;
    drop(connection);
    let cancelled = AtomicBool::new(false);
    let archive = fixture.0.join("mixed.lexique-backup");
    crate::backup::export_paused(
        &crate::vocabulary::DatabasePath(fixture.database()),
        &archive,
        &cancelled,
    )
    .unwrap();
    let staging = fixture.0.join("preview");
    std::fs::create_dir(&staging).unwrap();
    let admitted = crate::backup::validate_backup(&archive, &staging, &cancelled).unwrap();
    assert_eq!(admitted.manifest.snapshot_version, 4);
    let restore_path = fixture.0.join("restored.sqlite3");
    let mut restored = Connection::open(&restore_path).unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(
        &mut restored,
        &admitted.snapshot,
        admitted.manifest.created_at,
    )
    .unwrap();
    drop(restored);
    let restored = Connection::open(restore_path).unwrap();
    assert_eq!(
        get(&restored, external.id).unwrap().import_source,
        external.import_source
    );
    assert!(get(&restored, external.id).unwrap().reference.is_none());
    assert_eq!(
        get_grammar(&restored, grammar.id).unwrap().category_ids,
        vec![category.id]
    );
    assert_eq!(
        get(&restored, word.id).unwrap().category_ids,
        vec![category.id]
    );
    assert_eq!(get(&restored, word.id).unwrap().reference, word.reference);
    assert_eq!(
        serde_json::to_value(review_progress(&restored, recognition.id).unwrap()).unwrap(),
        progress
    );
    assert!(review_paused(&restored, external.id).unwrap());
}
