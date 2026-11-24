use super::*;
use std::io::Read;

fn archive(fixture: &Fixture, entries: &[(&str, &str)]) -> PathBuf {
    use std::io::Write;
    let path = fixture.0.join("external.zip");
    let mut zip = ::zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    for (name, text) in entries {
        zip.start_file(
            *name,
            ::zip::write::SimpleFileOptions::default()
                .compression_method(::zip::CompressionMethod::Stored),
        )
        .unwrap();
        zip.write_all(text.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    path
}
fn preview_zip(
    connection: &Connection,
    path: &Path,
    operation: &str,
) -> Result<NoteImportPreview, String> {
    let (source, bytes) = read_note_file(path, NoteFormat::Zip)?;
    crate::note_zip::preview_zip_bytes(connection, source, &bytes, operation)
}

#[test]
fn grammar_zip_coalesces_equal_identities_and_excludes_conflicting_groups() {
    let fixture = Fixture::new();
    let connection = fixture.open();
    let one = "# Lexique grammar note v1\n## exchange_id\n```text\nid-one\n```\n## title\n```text\nSame\n```\n";
    let changed = one.replace("Same", "Changed");
    let path = archive(
        &fixture,
        &[("a.md", one), ("b.md", one), ("c.md", &changed)],
    );
    let (source, bytes) = read_note_file(&path, NoteFormat::Zip).unwrap();
    let options = NoteImportOptions {
        record_kind: crate::note_export::ExportRecordKind::Grammar,
        ..Default::default()
    };
    let ExchangePreview::Grammar(preview) =
        preview_note_bytes(&connection, source, &bytes, "grammar-zip", &options).unwrap()
    else {
        panic!("grammar preview")
    };
    assert_eq!(preview.groups.len(), 1);
    assert_eq!(preview.groups[0].status, "invalid");
    assert_eq!(preview.groups[0].rows, vec![1, 2, 3]);
}

#[test]
fn grammar_zip_invalid_pair_conflicts_with_same_identity_in_either_order() {
    let fixture = Fixture::new();
    let connection = fixture.open();
    let valid = "# Lexique grammar note v1\n## exchange_id\n```text\nid-one\n```\n## title\n```text\nSame\n```\n";
    let invalid = format!("{valid}## question\n```text\nOnly question\n```\n");
    for entries in [
        vec![("good.md", valid), ("bad.md", invalid.as_str())],
        vec![("bad.md", invalid.as_str()), ("good.md", valid)],
    ] {
        let path = archive(&fixture, &entries);
        let (source, bytes) = read_note_file(&path, NoteFormat::Zip).unwrap();
        let options = NoteImportOptions {
            record_kind: crate::note_export::ExportRecordKind::Grammar,
            ..Default::default()
        };
        let ExchangePreview::Grammar(preview) =
            preview_note_bytes(&connection, source, &bytes, "pair-conflict", &options).unwrap()
        else {
            panic!("grammar preview")
        };
        assert_eq!(preview.groups.len(), 1);
        assert_eq!(preview.groups[0].status, "invalid");
        assert_eq!(preview.groups[0].files.as_ref().unwrap().len(), 2);
    }
}

#[test]
fn grammar_zip_missing_title_invalidates_matching_identity() {
    let fixture = Fixture::new();
    let connection = fixture.open();
    let valid = "# Lexique grammar note v1\n## exchange_id\n```text\nid-one\n```\n## title\n```text\nValid\n```\n";
    let missing = "# Lexique grammar note v1\n## exchange_id\n```text\nid-one\n```\n";
    let path = archive(&fixture, &[("valid.md", valid), ("missing.md", missing)]);
    let (source, bytes) = read_note_file(&path, NoteFormat::Zip).unwrap();
    let options = NoteImportOptions {
        record_kind: crate::note_export::ExportRecordKind::Grammar,
        ..Default::default()
    };
    let ExchangePreview::Grammar(preview) =
        preview_note_bytes(&connection, source, &bytes, "missing-title", &options).unwrap()
    else {
        panic!("grammar preview")
    };
    assert_eq!(preview.groups.len(), 1);
    assert_eq!(preview.groups[0].status, "invalid");
}

#[test]
fn ordinary_tool_deflated_zip_imports_through_real_file_and_sqlite() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.0.join("tool.zip");
    let status = std::process::Command::new("python").args(["-B", "-X", "utf8", "-c",
        "import zipfile,sys\nwith zipfile.ZipFile(sys.argv[1],'w',compression=zipfile.ZIP_DEFLATED) as z: z.writestr('nested/tool.md','# Lexique note v1\\n## headword\\n```text\\noutil\\n```\\n')"])
        .arg(&path).status().unwrap();
    assert!(status.success());
    let preview = preview_zip(&connection, &path, "deflated-tool").unwrap();
    commit_creates(&mut connection, &preview).unwrap();
    assert!(find(&fixture.open(), "outil").unwrap().is_some());
}

#[test]
fn corrupt_zip_container_crc_and_disagreeing_headers_are_fatal_without_writes() {
    let fixture = Fixture::new();
    let connection = fixture.open();
    let path = archive(
        &fixture,
        &[(
            "note.md",
            "# Lexique note v1\n## headword\n```text\nword\n```\n",
        )],
    );
    let bytes = std::fs::read(&path).unwrap();
    let mut crc = bytes.clone();
    let name_length = u16::from_le_bytes([crc[26], crc[27]]) as usize;
    let extra_length = u16::from_le_bytes([crc[28], crc[29]]) as usize;
    crc[30 + name_length + extra_length] ^= 1;
    let mut header = bytes.clone();
    header[30] ^= 1;
    for (variant, raw) in [
        ("truncated", bytes[..30].to_vec()),
        ("crc", crc),
        ("headers", header),
    ] {
        std::fs::write(&path, raw).unwrap();
        assert!(
            preview_zip(&connection, &path, "fatal-zip").is_err(),
            "Corrupt {variant} must fail the container, not produce selectable groups"
        );
        assert!(list(&connection, "", "headword-asc").unwrap().is_empty());
    }
}

#[test]
fn zip_identical_files_dedupe_conflicting_fields_invalidate_whole_groups_and_invalid_entries_can_be_excluded()
 {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let one = "# Lexique note v1\n## headword\n```text\nécole\n```\n";
    let equivalent = one.replace("école", "e\u{301}cole");
    let other = one.replace("école", "different");
    let conflict = format!("{other}## notes\n```text\n\n```\n");
    let path = archive(
        &fixture,
        &[
            ("../../outside.md", one),
            ("nested/same.md", &equivalent),
            ("first.md", &other),
            ("second.md", &conflict),
            ("readme.txt", "not a template"),
        ],
    );
    let preview = preview_zip(&connection, &path, "zip-grouping").unwrap();
    assert!(preview.error.is_none());
    assert_eq!(preview.groups.len(), 3);
    assert_eq!(preview.groups[0].status, "create");
    assert_eq!(preview.groups[0].rows, vec![1, 2]);
    assert_eq!(preview.groups[1].status, "invalid");
    assert!(
        preview.groups[1]
            .error
            .as_ref()
            .unwrap()
            .contains("conflicting")
    );
    assert_eq!(preview.groups[2].status, "invalid");
    assert!(
        preview.groups[2]
            .error
            .as_ref()
            .unwrap()
            .contains("readme.txt")
    );
    let selections = preview
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| NoteImportSelection {
            index,
            allow_unlinked: false,
            action: if group.status == "create" {
                NoteImportAction::Create
            } else {
                NoteImportAction::Skip
            },
        })
        .collect::<Vec<_>>();
    let result = commit_preview(&mut connection, &preview, &selections).unwrap();
    assert_eq!((result.created, result.skipped), (1, 2));
    let actual = find(&fixture.open(), "école").unwrap().unwrap();
    assert_eq!(
        actual.import_source,
        Some(ImportSource {
            format: NoteFormat::Zip,
            filename: "external.zip".into()
        })
    );
    assert!(find(&connection, "different").unwrap().is_none());
    assert!(!fixture.0.join("nested").exists());
}

#[test]
fn zip_exports_all_and_current_query_in_saved_snapshot_order_with_unique_windows_names() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let items = [
        "A/B",
        "a?b",
        "CON",
        "école",
        "CON.txt",
        "con.TXT",
        "PRN.txt",
        "aUx.md",
        "NUL.data",
        "com1.txt",
        "LPT9.md",
        "lpt².txt",
    ]
    .into_iter()
    .map(|word| NoteImportItem {
        content: NoteImportContent {
            headword: word.into(),
            notes: Some("snapshot\n`````\n中文".into()),
            examples: None,
        },
        existing: None,
        source: ImportSource {
            format: NoteFormat::Csv,
            filename: "seed.csv".into(),
        },
        snapshot: None,
    })
    .collect::<Vec<_>>();
    save_note_import_batch(&mut connection, "zip-seed", &items, 0).unwrap();
    for scope in [
        NoteExportScope::All {
            sort: "headword-desc".into(),
        },
        NoteExportScope::Search {
            query: "école".into(),
            sort: "headword-desc".into(),
            category: NoteExportCategory::Named(NoteExportCategoryName::All),
        },
    ] {
        let expected = capture_note_export(&mut connection, &scope).unwrap();
        let path = fixture.0.join("notes.zip");
        crate::note_export::write_note_zip(&expected, &path).unwrap();
        let extracted = fixture.0.join("extracted");
        let extraction = std::process::Command::new("python")
            .args([
                "-B",
                "-X",
                "utf8",
                "-c",
                "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])",
            ])
            .arg(&path)
            .arg(&extracted)
            .output()
            .unwrap();
        assert!(
            extraction.status.success(),
            "Windows ZIP extraction: {}",
            String::from_utf8_lossy(&extraction.stderr)
        );
        let mut archive = ::zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
        assert_eq!(archive.len(), expected.records.len());
        let mut names = std::collections::HashSet::new();
        for (index, record) in expected.records.iter().enumerate() {
            let mut entry = archive.by_index(index).unwrap();
            let name = entry.name().unwrap().into_owned();
            assert!(names.insert(name.to_lowercase()));
            assert!(!name.chars().any(|ch| "<>:\"/\\|?*".contains(ch)));
            assert!(name.ends_with(&format!("-{}.md", record.id)));
            if record.headword.contains('.') {
                assert_eq!(name, format!("_{}-{}.md", record.headword, record.id));
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).unwrap();
            assert_eq!(std::fs::read_to_string(extracted.join(name)).unwrap(), text);
            let actual = crate::note_markdown::parse_note_markdown(&text).unwrap();
            assert_eq!(actual.headword, record.headword);
            assert_eq!(actual.notes.as_deref(), Some(record.notes.as_str()));
            assert_eq!(actual.examples, Some(vec![]));
        }
    }
}

#[test]
fn zip_mixed_batch_revalidates_revisions_rolls_back_real_sqlite_failure_and_replays_a_lost_response()
 {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let seed = preview_file(
        &connection,
        &fixture.csv(&[row("existing", "old", "", "")]),
        "seed-existing",
    )
    .unwrap();
    let original = commit_creates(&mut connection, &seed).unwrap().records[0].clone();
    add_review(&mut connection, original.id).unwrap();
    set_review_paused(&mut connection, original.id, true).unwrap();
    let format = |word: &str, notes: &str| {
        crate::note_markdown::format_note_markdown(&NoteExportRecord {
            id: 1,
            headword: word.into(),
            notes: notes.into(),
            examples: vec![],
        })
    };
    let (update, create, fail) = (
        format("existing", "new"),
        format("created", "new"),
        format("fail-last", "new"),
    );
    let path = archive(
        &fixture,
        &[
            ("update.md", &update),
            ("new.md", &create),
            ("fail.md", &fail),
        ],
    );
    let stale = preview_zip(&connection, &path, "stale-zip").unwrap();
    let decisions = |preview: &NoteImportPreview| {
        preview
            .groups
            .iter()
            .enumerate()
            .map(|(index, group)| NoteImportSelection {
                index,
                allow_unlinked: false,
                action: if group.status == "different" {
                    NoteImportAction::Update
                } else {
                    NoteImportAction::Create
                },
            })
            .collect::<Vec<_>>()
    };
    let mut other = fixture.open();
    let edit = NoteImportContent {
        headword: "existing".into(),
        notes: Some("concurrent".into()),
        examples: None,
    }
    .save_request("concurrent-edit", Some(&original))
    .unwrap();
    save(&mut other, &edit, None).unwrap();
    assert!(
        commit_preview(&mut connection, &stale, &decisions(&stale))
            .unwrap_err()
            .contains("after preview")
    );
    assert!(find(&connection, "created").unwrap().is_none());
    let preview = preview_zip(&connection, &path, "mixed-zip").unwrap();
    let before =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_000).unwrap())
            .unwrap();
    connection.execute_batch("CREATE TRIGGER fail_zip BEFORE INSERT ON vocabulary WHEN NEW.headword='fail-last' BEGIN SELECT RAISE(ABORT,'real ZIP batch failure'); END;").unwrap();
    assert!(
        commit_preview(&mut connection, &preview, &decisions(&preview))
            .unwrap_err()
            .contains("real ZIP batch failure")
    );
    assert_eq!(
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_000).unwrap())
            .unwrap(),
        before
    );
    connection.execute_batch("DROP TRIGGER fail_zip").unwrap();
    let result = commit_preview(&mut connection, &preview, &decisions(&preview)).unwrap();
    assert_eq!((result.created, result.updated), (2, 1));
    let after =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_000).unwrap())
            .unwrap();
    assert_eq!(
        serde_json::to_value(
            commit_preview(&mut connection, &preview, &decisions(&preview)).unwrap()
        )
        .unwrap(),
        serde_json::to_value(result).unwrap()
    );
    assert_eq!(
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_000).unwrap())
            .unwrap(),
        after
    );
    for field in ["cards", "progress", "ratings", "paused", "speech_tasks"] {
        assert_eq!(after[field], before[field]);
    }
}

#[test]
fn csv_zip_csv_preserves_101_ordered_examples_exact_text_identity_and_progress_and_rejects_102() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.csv(
        &(0..101)
            .map(|index| {
                row(
                    "école",
                    "\n 中文\r\n```````\n",
                    &format!(" {index}\r\n"),
                    " 中文 ",
                )
            })
            .collect::<Vec<_>>(),
    );
    let preview = preview_file(&connection, &path, "csv-start").unwrap();
    let original = commit_creates(&mut connection, &preview).unwrap().records[0].clone();
    add_review(&mut connection, original.id).unwrap();
    set_review_paused(&mut connection, original.id, true).unwrap();
    let expected = capture_note_export(
        &mut connection,
        &NoteExportScope::Single { id: original.id },
    )
    .unwrap();
    let zip = fixture.0.join("roundtrip.zip");
    crate::note_export::write_note_zip(&expected, &zip).unwrap();
    let preview = preview_zip(&connection, &zip, "zip-roundtrip").unwrap();
    assert_eq!(preview.groups[0].status, "identical");
    commit_preview(
        &mut connection,
        &preview,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Skip,
            allow_unlinked: false,
        }],
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(get(&connection, original.id).unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let mut destination = Connection::open(fixture.0.join("destination.sqlite3")).unwrap();
    initialize(&destination).unwrap();
    let preview = preview_zip(&destination, &zip, "zip-new").unwrap();
    commit_creates(&mut destination, &preview).unwrap();
    let actual = capture_note_export(
        &mut destination,
        &NoteExportScope::Single { id: original.id },
    )
    .unwrap();
    assert_eq!(actual.records, expected.records);
    let (first, last) = (fixture.0.join("first.csv"), fixture.0.join("last.csv"));
    crate::note_export::write_note_csv(&expected, &first).unwrap();
    crate::note_export::write_note_csv(&actual, &last).unwrap();
    assert_eq!(std::fs::read(first).unwrap(), std::fs::read(last).unwrap());
    let mut invalid = expected.records[0].clone();
    invalid.examples.push(NoteExportExample {
        fr: "102".into(),
        zh: "".into(),
    });
    let path = archive(
        &fixture,
        &[(
            "too-many.md",
            &crate::note_markdown::format_note_markdown(&invalid),
        )],
    );
    let preview = preview_zip(&connection, &path, "too-many").unwrap();
    assert_eq!(preview.groups[0].status, "invalid");
    assert!(
        preview.groups[0]
            .error
            .as_ref()
            .unwrap()
            .contains("too-many.md")
    );
}
