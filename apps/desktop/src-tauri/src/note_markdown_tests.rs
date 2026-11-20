use super::parse_note_markdown;
use lexique_storage::{
    ImportSource, NoteExample, NoteExportScope, NoteFormat, NoteImportContent, NoteImportItem,
    capture_note_export, get, initialize, save_note_import_batch,
};
use rusqlite::Connection;

fn directory() -> std::path::PathBuf {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/issue-188/files")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn saved_single_markdown_file_roundtrips_unicode_backticks_and_every_newline_ending() {
    let path = directory();
    let mut connection = Connection::open(path.join("learning.sqlite3")).unwrap();
    initialize(&connection).unwrap();
    let source = ImportSource {
        format: NoteFormat::Markdown,
        filename: "original.md".into(),
    };
    let content = NoteImportContent {
        headword: "école".into(),
        notes: Some("  **学校**\n\n```````\r\n终行\r".into()),
        examples: Some(vec![
            NoteExample {
                fr: "\n À l’école.\r\n".into(),
                zh: "\n".into(),
            },
            NoteExample {
                fr: "".into(),
                zh: " 单侧\r".into(),
            },
        ]),
    };
    let result = save_note_import_batch(
        &mut connection,
        "original",
        &[NoteImportItem {
            content: content.clone(),
            existing: None,
            source: source.clone(),
            snapshot: None,
        }],
        0,
    )
    .unwrap();
    let id = result.records[0].id;
    let snapshot = capture_note_export(&mut connection, &NoteExportScope::Single { id }).unwrap();
    let existing = get(&connection, id).unwrap();
    save_note_import_batch(
        &mut connection,
        "later",
        &[NoteImportItem {
            content: NoteImportContent {
                notes: Some("later save".into()),
                examples: None,
                headword: "école".into(),
            },
            existing: Some(existing),
            source,
            snapshot: None,
        }],
        0,
    )
    .unwrap();
    let destination = path.join("saved.md");
    crate::note_export::write_note_markdown(&snapshot, &destination).unwrap();
    let text = std::fs::read_to_string(&destination).unwrap();
    assert!(text.contains("````````text\n"));
    assert_eq!(parse_note_markdown(&text).unwrap(), content);
    assert!(!text.contains("original.md"));
    assert_eq!(get(&connection, id).unwrap().notes, "later save");
}

#[test]
fn template_preserves_plain_text_and_newline_boundaries() {
    let content = parse_note_markdown(
        "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## notes\n\n`````text\n\n  **学校** `x`\r\n````\n尾行\r\n\n`````\n\n## examples\n\n### 1\n\n#### fr\n\n```text\n\n À l’école. \r\n```\n\n#### zh\n\n```text\n\n```\n",
    ).unwrap();
    assert_eq!(content.headword, "école");
    assert_eq!(
        content.notes.as_deref(),
        Some("\n  **学校** `x`\r\n````\n尾行\r\n")
    );
    let examples = content.examples.unwrap();
    assert_eq!(examples[0].fr, "\n À l’école. \r");
    assert_eq!(examples[0].zh, "");
}

#[test]
fn markdown_filenames_remain_readable_and_unique_after_windows_sanitization() {
    let record = |id, word: &str| lexique_storage::NoteExportRecord {
        id,
        headword: word.into(),
        notes: String::new(),
        examples: vec![],
    };
    assert_eq!(
        super::note_markdown_filename(&record(12, "école:<A>/B\\?*|\"")),
        "école__A__B_____-12.md"
    );
    assert_eq!(super::note_markdown_filename(&record(3, "CON")), "CON-3.md");
    assert_eq!(
        super::note_markdown_filename(&record(4, "  ...  ")),
        "note-4.md"
    );
    assert_ne!(
        super::note_markdown_filename(&record(1, "a:b")),
        super::note_markdown_filename(&record(2, "a/b"))
    );
    assert!(
        super::note_markdown_filename(&record(9, &"𐐷".repeat(200)))
            .encode_utf16()
            .count()
            < 200
    );
}

#[test]
fn real_markdown_preview_commits_reopens_skips_and_clears_only_supplied_fields() {
    use crate::note_import::{NoteImportAction, NoteImportSelection, commit_preview};
    let path = directory();
    let database = path.join("learning.sqlite3");
    let mut connection = Connection::open(&database).unwrap();
    initialize(&connection).unwrap();
    let file = path.join("external.md");
    let initial = "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## notes\n\n```text\n keep \r\n\n```\n\n## examples\n\n### 1\n\n#### zh\n\n```text\n译文\n```\n";
    std::fs::write(&file, initial).unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "create-file").unwrap();
    assert!(preview.error.is_none());
    let saved = commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Create,
            allow_unlinked: false,
        }],
    )
    .unwrap()
    .records[0]
        .clone();
    assert_eq!(saved.notes, " keep \r\n");
    assert_eq!(
        (saved.example_fr.as_str(), saved.example_zh.as_str()),
        ("", "译文")
    );
    assert_eq!(
        saved.import_source.as_ref().unwrap().filename,
        "external.md"
    );
    drop(connection);
    let mut connection = Connection::open(&database).unwrap();
    std::fs::write(
        &file,
        "# Lexique note v1\n\n## headword\n\n```text\ne\u{301}cole\n```\n",
    )
    .unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "missing-fields").unwrap();
    assert_eq!(preview.groups[0].status, "identical");
    let result = commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Skip,
            allow_unlinked: false,
        }],
    )
    .unwrap();
    assert_eq!(result.skipped, 1);
    assert_eq!(get(&connection, saved.id).unwrap().revision, saved.revision);
    std::fs::write(
        &file,
        "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## notes\n\n## examples\n",
    )
    .unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "clear-fields").unwrap();
    assert_eq!(preview.groups[0].status, "different");
    let updated = commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Update,
            allow_unlinked: false,
        }],
    )
    .unwrap()
    .records[0]
        .clone();
    assert_eq!(updated.id, saved.id);
    assert_eq!(
        (
            updated.notes.as_str(),
            updated.example_fr.as_str(),
            updated.example_zh.as_str()
        ),
        ("", "", "")
    );
    assert_eq!(updated.import_source, saved.import_source);
}
