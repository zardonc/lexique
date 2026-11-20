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
fn default_markdown_exports_use_windows_filesystem_names_and_preserve_saved_text() {
    let path = directory();
    let mut connection = Connection::open(path.join("learning.sqlite3")).unwrap();
    initialize(&connection).unwrap();
    let contents: Vec<_> = [
        "CON.txt",
        "con.TXT",
        "PRN.md",
        "pRn.data",
        "AUX.foo",
        "aUx.bar",
        "NUL.txt",
        "nUl.TXT",
        "COM1.txt",
        "com2.txt",
        "COM9.md",
        "LPT1.txt",
        "lpt2.txt",
        "LPT9.md",
        "COM¹.txt",
        "lpt².txt",
        "LPT³.md",
        "ordinary.txt",
    ]
    .into_iter()
    .map(|word| NoteImportContent {
        headword: word.into(),
        notes: Some("  中文 `notes`\r\n终行\n".into()),
        examples: Some(vec![NoteExample {
            fr: " À l’école.\r\n".into(),
            zh: " 单侧\n".into(),
        }]),
    })
    .collect();
    let items: Vec<_> = contents
        .iter()
        .map(|content| NoteImportItem {
            content: content.clone(),
            existing: None,
            source: ImportSource {
                format: NoteFormat::Markdown,
                filename: "external.md".into(),
            },
            snapshot: None,
        })
        .collect();
    let saved = save_note_import_batch(&mut connection, "device-names", &items, 0).unwrap();
    for record in &saved.records {
        let snapshot =
            capture_note_export(&mut connection, &NoteExportScope::Single { id: record.id })
                .unwrap();
        let name = super::note_markdown_filename(&snapshot.records[0]);
        let destination = path.join(&name);
        crate::note_export::write_note_markdown(&snapshot, &destination)
            .unwrap_or_else(|error| panic!("Default filename {name}: {error}"));
        let readback = std::process::Command::new("python")
            .args(["-B", "-X", "utf8", "-c", "import pathlib,sys; sys.stdout.buffer.write(pathlib.Path(sys.argv[1]).read_bytes())"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            readback.status.success(),
            "Default filename {name}: {}",
            String::from_utf8_lossy(&readback.stderr)
        );
        let prefix = if record.headword == "ordinary.txt" {
            ""
        } else {
            "_"
        };
        assert_eq!(
            name,
            format!("{prefix}{}-{}.md", record.headword, record.id),
            "Windows-usable default filename"
        );
        let expected = contents
            .iter()
            .find(|item| item.headword == record.headword)
            .unwrap();
        assert_eq!(
            parse_note_markdown(std::str::from_utf8(&readback.stdout).unwrap()).unwrap(),
            *expected
        );
    }
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
    std::fs::write(&file, "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## notes\n\n```text\nnotes only\n```\n").unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "notes-only").unwrap();
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
    assert_eq!(
        (updated.notes.as_str(), updated.example_zh.as_str()),
        ("notes only", "译文")
    );
    std::fs::write(&file, "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## examples\n\n### 1\n\n#### fr\n\n```text\nFrench only\n```\n").unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "examples-only").unwrap();
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
    assert_eq!(updated.notes, "notes only");
    assert_eq!(
        (updated.example_fr.as_str(), updated.example_zh.as_str()),
        ("French only", "")
    );
    assert!(updated.more_examples.is_empty());
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

#[test]
fn invalid_template_files_report_filename_and_reason_without_writing() {
    let path = directory();
    let mut connection = Connection::open(path.join("learning.sqlite3")).unwrap();
    initialize(&connection).unwrap();
    let file = path.join("invalid.md");
    let head = "# Lexique note v1\n\n## headword\n\n```text\nmot\n```\n";
    for (text, reason) in [
        ("# Free Markdown\n".into(), "identifier"),
        ("# Lexique note v2\n".into(), "identifier"),
        ("# Lexique note v1\n## notes\n".into(), "headword"),
        (format!("{head}\n## headword\n"), "Duplicate section"),
        (
            format!("{head}\n## notes\n\n## notes\n"),
            "Duplicate section",
        ),
        (
            format!("{head}\n## examples\n\n## examples\n"),
            "Duplicate section",
        ),
        (format!("{head}\n## summary\n"), "Unknown template"),
        (format!("{head}\n## notes\n```text\nno close"), "Unclosed"),
        (
            format!("{head}\n## notes\n```markdown\ntext\n```\n"),
            "text fence",
        ),
        (format!("{head}\n## examples\n### 2\n"), "number"),
        (format!("{head}\n## examples\n### 0\n"), "number"),
        (
            format!("{head}\n## examples\n### 1\n#### fr\n```text\na\n```\n#### fr\n"),
            "Duplicate example",
        ),
        (
            format!("{head}\n## examples\n### 1\n#### en\n"),
            "Unknown example",
        ),
        (format!("{head}\n## examples\n### 1\n"), "requires fr or zh"),
        (
            format!("{head}\n## examples\n### 1\n#### fr\n```text\na\n```\n### 1\n"),
            "number",
        ),
        (
            format!("{head}\n## examples\n### 1\n#### fr\n```text\na\n```\n### 3\n"),
            "number",
        ),
        (
            format!("{head}\n## notes\n```text\n{}\n```\n", "中".repeat(20_001)),
            "20,000",
        ),
    ] {
        std::fs::write(&file, text).unwrap();
        let preview = super::preview_markdown_file(&connection, &file, "invalid-file").unwrap();
        let error = preview.error.as_ref().unwrap();
        assert!(
            error.contains("invalid.md") && error.contains(reason),
            "{error}"
        );
        assert!(crate::note_import::commit_preview(&mut connection, &preview, &[]).is_err());
    }
    std::fs::write(&file, [0xff, 0xfe]).unwrap();
    assert!(
        super::preview_markdown_file(&connection, &file, "encoding")
            .unwrap()
            .error
            .unwrap()
            .contains("invalid.md: Markdown must be UTF-8")
    );
    assert!(
        lexique_storage::list(&connection, "", "headword-asc")
            .unwrap()
            .is_empty()
    );
    assert!(
        super::preview_markdown_file(&connection, &path.join("missing.md"), "missing")
            .unwrap_err()
            .contains("missing.md")
    );
}

#[test]
fn crlf_template_and_unicode_field_and_example_count_limits_use_storage_validation() {
    let head = "# Lexique note v1\n\n## headword\n\n```text\nmot\n```\n";
    let crlf = parse_note_markdown(
        "\u{feff}# Lexique note v1\r\n\r\n## headword\r\n\r\n```text\récole\r\n```\r\n",
    );
    assert!(crlf.is_err());
    let crlf = parse_note_markdown("\u{feff}# Lexique note v1\r\n\r\n## headword\r\n\r\n```text\r\nécole\r\n```\r\n\r\n## notes\r\n\r\n```text\r\n\r\n  学校\r\n\r\n```\r\n").unwrap();
    assert_eq!(crlf.notes.as_deref(), Some("\r\n  学校\r\n"));
    assert_eq!(crlf.examples, None);
    for (section, value, limit) in [
        ("## headword", "é", 200),
        ("## notes", "中", 20_000),
        ("## examples\n\n### 1\n\n#### fr", "中", 20_000),
    ] {
        let prefix = if section == "## headword" {
            "# Lexique note v1\n\n"
        } else {
            head
        };
        assert!(
            parse_note_markdown(&format!(
                "{prefix}{section}\n\n```text\n{}\n```\n",
                value.repeat(limit)
            ))
            .is_ok()
        );
        assert!(
            parse_note_markdown(&format!(
                "{prefix}{section}\n\n```text\n{}\n```\n",
                value.repeat(limit + 1)
            ))
            .is_err()
        );
    }
    let mut text = format!("{head}\n## examples\n\n");
    for number in 1..=101 {
        text.push_str(&format!(
            "### {number}\n\n#### zh\n\n```text\n例句\n```\n\n"
        ));
    }
    assert_eq!(
        parse_note_markdown(&text).unwrap().examples.unwrap().len(),
        101
    );
    text.push_str("### 102\n\n#### fr\n\n```text\nextra\n```\n");
    assert!(parse_note_markdown(&text).unwrap_err().contains("101"));
}

#[test]
fn markdown_update_retains_metadata_and_unchanged_example_provenance_and_rejects_stale_preview() {
    use crate::note_import::{NoteImportAction, NoteImportSelection, commit_preview};
    use lexique_storage::{
        Snapshot, add_review, create_category, review_cards, review_paused, save, set_review_paused,
    };
    let path = directory();
    let database = path.join("learning.sqlite3");
    let mut connection = Connection::open(&database).unwrap();
    initialize(&connection).unwrap();
    let snapshot: Snapshot = serde_json::from_value(serde_json::json!({
        "reference": { "dictionary": "Original dictionary", "dictionary_id": "mdx-12", "lookup_key": "école", "version": "1", "sha256": "a".repeat(64) },
        "digest": [{ "pronunciations": ["ipa"], "grammar": ["n.f."], "translation": "original meaning" }]
    })).unwrap();
    let mut request = NoteImportContent {
        headword: "école".into(),
        notes: Some("original".into()),
        examples: Some(vec![NoteExample {
            fr: "unchanged".into(),
            zh: "原句".into(),
        }]),
    }
    .create_request("rich-fixture")
    .unwrap();
    request.category_ids = vec![create_category(&connection, "Category").unwrap().id];
    request.example_source = Some(snapshot.reference.clone());
    request.example_sense = Some(0);
    let original = save(&mut connection, &request, Some(snapshot))
        .unwrap()
        .record;
    add_review(&mut connection, original.id).unwrap();
    set_review_paused(&mut connection, original.id, true).unwrap();
    let cards = serde_json::to_value(review_cards(&connection, original.id).unwrap()).unwrap();
    let file = path.join("update.md");
    let mut export = capture_note_export(
        &mut connection,
        &NoteExportScope::Single { id: original.id },
    )
    .unwrap();
    export.records[0].notes = "file update".into();
    export.records[0]
        .examples
        .push(lexique_storage::NoteExportExample {
            fr: "changed".into(),
            zh: "新句".into(),
        });
    crate::note_export::write_note_markdown(&export, &file).unwrap();
    let preview = super::preview_markdown_file(&connection, &file, "rich-update").unwrap();
    commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Update,
            allow_unlinked: false,
        }],
    )
    .unwrap();
    let updated = get(&connection, original.id).unwrap();
    assert_eq!(updated.reference, original.reference);
    assert_eq!(updated.digest, original.digest);
    assert_eq!(updated.category_ids, original.category_ids);
    assert_eq!(updated.import_source, original.import_source);
    assert_eq!(updated.created_at, original.created_at);
    assert_eq!(updated.example_sense, Some(0));
    assert_eq!(updated.example_source, original.example_source);
    assert_eq!(updated.more_examples[0].sense, None);
    assert_eq!(updated.more_examples[0].source, None);
    assert_eq!(
        serde_json::to_value(review_cards(&connection, original.id).unwrap()).unwrap(),
        cards
    );
    assert!(review_paused(&connection, original.id).unwrap());
    export.records[0].notes = "second file update".into();
    crate::note_export::write_note_markdown(&export, &file).unwrap();
    let stale = super::preview_markdown_file(&connection, &file, "stale-update").unwrap();
    request.id = Some(updated.id);
    request.revision = Some(updated.revision);
    request.operation = "concurrent-edit".into();
    request.notes = "concurrent".into();
    let mut other = Connection::open(&database).unwrap();
    save(&mut other, &request, None).unwrap();
    assert!(
        commit_preview(
            &mut connection,
            &stale,
            &[NoteImportSelection {
                index: 0,
                action: NoteImportAction::Update,
                allow_unlinked: false
            }]
        )
        .unwrap_err()
        .contains("preview")
    );
    assert_eq!(get(&connection, original.id).unwrap().notes, "concurrent");
}

#[test]
fn invalid_export_scope_destination_and_replaced_snapshot_preserve_real_results() {
    use lexique_storage::{NoteExportRecord, NoteExportSnapshot};
    let path = directory();
    let empty = NoteExportSnapshot { records: vec![] };
    assert!(crate::note_export::write_note_markdown(&empty, &path.join("empty.md")).is_err());
    assert!(!path.join("empty.md").exists());
    let record = NoteExportRecord {
        id: 1,
        headword: "école".into(),
        notes: "原文".into(),
        examples: vec![],
    };
    let multiple = NoteExportSnapshot {
        records: vec![record.clone(), record.clone()],
    };
    assert!(crate::note_export::write_note_markdown(&multiple, &path.join("multiple.md")).is_err());
    assert!(
        crate::note_export::write_note_markdown(
            &NoteExportSnapshot {
                records: vec![record]
            },
            &path
        )
        .unwrap_err()
        .contains("Could not write Markdown file")
    );
    assert!(
        crate::note_export::NoteExports::default()
            .retained("expired")
            .unwrap_err()
            .contains("snapshot changed")
    );
}

#[test]
fn markdown_new_notes_use_shared_real_dictionary_enrichment_and_error_consent() {
    use crate::note_import::{
        NoteImportAction, NoteImportSelection, commit_preview, enrich_preview,
    };
    let path = directory();
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for (source, destination) in [
        (
            "target/desktop-dictionary-sample/dictionary.sqlite3",
            "dictionary/dictionary.sqlite3",
        ),
        (
            "target/desktop-core-pilot/core.sqlite3",
            "dictionary/core-pilot-79/core.sqlite3",
        ),
        (
            "resources/freedict-fra-eng-mdx-0.4.1/freedict-fra-eng-0.4.1.mdx",
            "dictionary/freedict-fra-eng-mdx/freedict-fra-eng-0.4.1.mdx",
        ),
    ] {
        let destination = path.join(destination);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(repo.join(source), destination).unwrap();
    }
    let database = crate::vocabulary::DatabasePath(path.join("learning.sqlite3"));
    let mut connection = Connection::open(&database.0).unwrap();
    initialize(&connection).unwrap();
    let file = path.join("dictionary.md");
    std::fs::write(&file, "# Lexique note v1\n\n## headword\n\n```text\nécole\n```\n\n## notes\n\n```text\n  imported text \r\n\n```\n").unwrap();
    let mut preview =
        super::preview_markdown_file(&connection, &file, "dictionary-markdown").unwrap();
    let registry = crate::dictionaries::Registry::default();
    let builtin = path.join("dictionary/dictionary.sqlite3");
    enrich_preview(&mut preview, &database, &builtin, &registry);
    assert_eq!(
        preview.groups[0].enrichment.as_ref().unwrap().status,
        "linked"
    );
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
    assert!(saved.reference.is_some() && !saved.digest.is_empty());
    assert_eq!(saved.notes, "  imported text \r\n");
    assert_eq!(saved.import_source.unwrap().format, NoteFormat::Markdown);
    std::fs::write(
        &file,
        "# Lexique note v1\n\n## headword\n\n```text\nnew-external-word\n```\n",
    )
    .unwrap();
    std::fs::rename(&builtin, path.join("dictionary/unavailable.sqlite3")).unwrap();
    let mut preview = super::preview_markdown_file(&connection, &file, "dictionary-error").unwrap();
    enrich_preview(&mut preview, &database, &builtin, &registry);
    assert_eq!(
        preview.groups[0].enrichment.as_ref().unwrap().status,
        "error"
    );
    assert!(
        commit_preview(
            &mut connection,
            &preview,
            &[NoteImportSelection {
                index: 0,
                action: NoteImportAction::Create,
                allow_unlinked: false
            }]
        )
        .is_err()
    );
    assert!(
        lexique_storage::find(&connection, "new-external-word")
            .unwrap()
            .is_none()
    );
    let saved = commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Create,
            allow_unlinked: true,
        }],
    )
    .unwrap()
    .records[0]
        .clone();
    assert!(saved.reference.is_none() && saved.digest.is_empty());
}
