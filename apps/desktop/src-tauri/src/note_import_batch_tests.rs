use super::*;

#[test]
fn incomplete_or_malformed_quoted_records_report_file_structure_errors_without_writes() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.0.join("malformed.csv");
    for text in [
        "headword,notes\nword,\"unclosed\n",
        "headword,notes\nword,\"closed\"junk\n",
    ] {
        std::fs::write(&path, text).unwrap();
        let preview = preview_file(&connection, &path, "malformed").unwrap();
        assert!(
            preview.error.is_some(),
            "Malformed quotes must be fixed before previewing content"
        );
        assert!(commit_preview(&mut connection, &preview, &[]).is_err());
        assert!(list(&connection, "", "headword-asc").unwrap().is_empty());
    }
}

#[test]
fn imported_updates_preserve_dictionary_categories_progress_history_pause_and_real_selected_audio()
{
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let category = create_category(&connection, "Category").unwrap();
    let mut snapshot = dictionary_snapshot("protected");
    snapshot.digest.push(DigestEntry {
        pronunciations: vec!["ipa".into()],
        grammar: vec!["n.f.".into()],
        translation: Some("第二义项".into()),
        core: None,
    });
    let mut request = NoteImportContent {
        headword: "protected".into(),
        notes: Some("old notes".into()),
        examples: Some(vec![NoteExample {
            fr: "old phrase".into(),
            zh: "旧例句".into(),
        }]),
    }
    .create_request("protected-seed")
    .unwrap();
    request.category_ids = vec![category.id];
    request.example_sense = Some(1);
    request.example_source = Some(snapshot.reference.clone());
    let original = save(&mut connection, &request, Some(snapshot))
        .unwrap()
        .record;
    let grammar = save_grammar(
        &connection,
        &GrammarSaveRequest {
            operation: "protected-grammar".into(),
            id: None,
            revision: None,
            title: "Règle".into(),
            body: "unchanged grammar".into(),
            category_ids: vec![category.id],
        },
        1_800_000_000,
    )
    .unwrap()
    .record;
    let cards = add_review(&mut connection, original.id).unwrap();
    let card = cards
        .iter()
        .find(|card| card.kind == "recognition")
        .unwrap();
    rate_card(
        &mut connection,
        &RatingRequest {
            operation: "protected-rate".into(),
            card_id: card.id,
            expected_revision: 0,
            rating: serde_json::from_str("\"good\"").unwrap(),
            now: 1_800_000_000,
            timezone: "UTC".into(),
        },
    )
    .unwrap();
    set_review_paused(&mut connection, original.id, true).unwrap();
    let bytes = include_bytes!("../../../../docs/evidence/issue-56/female-1.ogg");
    let inspector = crate::speech_cache::Inspector::new(&fixture.database()).unwrap();
    let key = attach_word_audio_at(&connection, original.id, 1_800_000_000).unwrap();
    let version = prepare_source_audio(
        &connection,
        &key,
        &content_hash(bytes),
        bytes.len() as i64,
        1_800_000_000,
    )
    .unwrap();
    inspector
        .publish_version(&version.physical_locator, &version.content_hash, bytes)
        .unwrap();
    verify_audio_content(&connection, &version.locator, bytes.len() as i64).unwrap();
    assert!(complete_source_audio(&connection, &version.locator).unwrap());
    assert!(
        reuse_audio_version(&connection, original.id, &version.locator, 0, 1_800_000_000).unwrap()
    );
    set_auto_enroll(&connection, true).unwrap();
    let before =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap();
    let path = fixture.csv(&[
        row("protected", "import update", "new unlinked", ""),
        row("created-with-auto-on", "", "", ""),
    ]);
    let preview = preview_file(&connection, &path, "protected-update").unwrap();
    let result = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    let updated = get(&connection, original.id).unwrap();
    assert_eq!(updated.reference, original.reference);
    assert_eq!(updated.digest, original.digest);
    assert_eq!(updated.category_ids, original.category_ids);
    assert_eq!(updated.created_at, original.created_at);
    assert_eq!(updated.import_source, original.import_source);
    assert!(updated.example_source.is_none());
    assert!(updated.example_sense.is_none());
    let created = result
        .records
        .iter()
        .find(|record| record.headword == "created-with-auto-on")
        .unwrap();
    assert!(review_cards(&connection, created.id).unwrap().is_empty());
    let after =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap();
    for field in before
        .as_object()
        .unwrap()
        .keys()
        .filter(|field| !matches!(field.as_str(), "records" | "save_operations" | "identities"))
    {
        assert_eq!(
            after[field], before[field],
            "Non-exchanged collection {field} changed"
        );
    }
    assert_eq!(after["identities"]["cards"], before["identities"]["cards"]);
    assert_eq!(
        after["identities"]["categories"],
        before["identities"]["categories"]
    );
    assert_eq!(
        get_grammar(&connection, grammar.id).unwrap().body,
        grammar.body
    );
    assert_eq!(review_cards(&connection, original.id).unwrap(), cards);
    assert!(review_paused(&connection, original.id).unwrap());
    let playback =
        crate::speech_tasks::cached_word(&connection, &inspector, original.id, 1_800_000_100)
            .unwrap()
            .unwrap();
    assert_eq!(playback.bytes, bytes);
    drop(inspector);
    drop(connection);
    let mut connection = fixture.open();
    let current = get(&connection, original.id).unwrap();
    let edit = NoteImportContent {
        headword: current.headword.clone(),
        notes: Some("edited again".into()),
        examples: Some(vec![NoteExample {
            fr: "editable imported update".into(),
            zh: "".into(),
        }]),
    }
    .save_request("protected-edit", Some(&current))
    .unwrap();
    assert_eq!(
        save(&mut connection, &edit, None).unwrap().outcome,
        "updated"
    );
}

#[test]
fn moved_meaning_links_survive_real_backup_restore_and_saved_operation_replay() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let snapshot = dictionary_snapshot("portable");
    let mut request = NoteImportContent {
        headword: "portable".into(),
        notes: None,
        examples: Some(vec![NoteExample {
            fr: "linked".into(),
            zh: "关联".into(),
        }]),
    }
    .create_request("portable-seed")
    .unwrap();
    request.example_sense = Some(0);
    let old = save(&mut connection, &request, Some(snapshot))
        .unwrap()
        .record;
    let preview = preview_file(
        &connection,
        &fixture.csv(&[
            row("portable", "", "new", ""),
            row("portable", "", "linked", "关联"),
        ]),
        "portable-import",
    )
    .unwrap();
    commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    let before = get(&connection, old.id).unwrap();
    assert_eq!(before.more_examples[0].sense, Some(0));
    let saved = preview.groups[0]
        .content
        .save_request("portable-import-0", Some(&old))
        .unwrap();
    drop(connection);
    let archive = fixture.0.join("moved-meaning.lexique-backup");
    let cancelled = AtomicBool::new(false);
    crate::backup::export_paused(
        &crate::vocabulary::DatabasePath(fixture.database()),
        &archive,
        &cancelled,
    )
    .unwrap();
    let staging = fixture.0.join("meaning-stage");
    std::fs::create_dir(&staging).unwrap();
    let admitted = crate::backup::validate_backup(&archive, &staging, &cancelled).unwrap();
    let restore_path = fixture.0.join("meaning-restore.sqlite3");
    let mut restored = Connection::open(&restore_path).unwrap();
    initialize(&restored).unwrap();
    import_learning_snapshot(
        &mut restored,
        &admitted.snapshot,
        admitted.manifest.created_at,
    )
    .unwrap();
    drop(restored);
    let restored = Connection::open(&restore_path).unwrap();
    assert_eq!(
        get(&restored, old.id).unwrap().more_examples,
        before.more_examples
    );
    assert_eq!(
        replay(&restored, &saved).unwrap().unwrap().record.id,
        old.id
    );
}

#[test]
fn semicolon_bom_multiline_external_mapping_and_illegal_mappings_use_real_files() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.0.join("mapping.csv");
    for bom in ["", "\u{feff}"] {
        let word = if bom.is_empty() { "plain" } else { "bom" };
        std::fs::write(&path, format!("{bom}mot;annotation;phrase;ignored\n\"mapped-{word}\";\"多行\n引用 \"\"texte\"\"\";\"fr;comma, kept\";unused\n")).unwrap();
        let options = NoteImportOptions {
            delimiter: NoteDelimiter::Semicolon,
            mapping: None,
            format: None,
            record_kind: Default::default(),
        };
        let unmapped = preview_csv_file(&connection, &path, "map-required", &options).unwrap();
        assert!(unmapped.error.as_ref().unwrap().contains("headword"));
        assert!(commit_preview(&mut connection, &unmapped, &[]).is_err());
        let mapping = NoteImportMapping {
            headword: Some(0),
            notes: Some(1),
            example_fr: Some(2),
            example_zh: None,
        };
        for invalid in [
            NoteImportMapping {
                headword: None,
                ..mapping.clone()
            },
            NoteImportMapping {
                notes: Some(0),
                ..mapping.clone()
            },
            NoteImportMapping {
                notes: Some(9),
                ..mapping.clone()
            },
        ] {
            let preview = preview_csv_file(
                &connection,
                &path,
                "bad-map",
                &NoteImportOptions {
                    delimiter: NoteDelimiter::Semicolon,
                    mapping: Some(invalid),
                    format: None,
                    record_kind: Default::default(),
                },
            )
            .unwrap();
            assert!(preview.error.is_some());
            assert!(commit_preview(&mut connection, &preview, &[]).is_err());
        }
        let preview = preview_csv_file(
            &connection,
            &path,
            if bom.is_empty() { "no-bom" } else { "bom" },
            &NoteImportOptions {
                delimiter: NoteDelimiter::Semicolon,
                mapping: Some(mapping),
                format: None,
                record_kind: Default::default(),
            },
        )
        .unwrap();
        let saved = commit_preview(&mut connection, &preview, &decide(&preview))
            .unwrap()
            .records[0]
            .clone();
        assert_eq!(saved.notes, "多行\n引用 \"texte\"");
        assert_eq!(saved.example_fr, "fr;comma, kept");
        assert_eq!(saved.example_zh, "");
        assert_eq!(preview.groups[0].rows, [2]);
    }
    assert_eq!(list(&fixture.open(), "", "headword-asc").unwrap().len(), 2);
    std::fs::write(
        &path,
        "example_zh,notes,headword,extra\n中文,,canonical,ignored\n",
    )
    .unwrap();
    let preview = preview_file(&connection, &path, "automatic-order").unwrap();
    assert!(preview.error.is_none());
    let saved = commit_preview(&mut connection, &preview, &decide(&preview))
        .unwrap()
        .records[0]
        .clone();
    assert_eq!(
        (
            saved.notes.as_str(),
            saved.example_fr.as_str(),
            saved.example_zh.as_str()
        ),
        ("", "", "中文")
    );
}

#[test]
fn omitted_identical_clear_skip_and_invalid_group_exclusion_have_explicit_counts() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let initial = preview_file(
        &connection,
        &fixture.csv(&[
            row("retained", "old notes", "old French", "old Chinese"),
            row("skipped", "keep", "", ""),
        ]),
        "seed-fields",
    )
    .unwrap();
    let records = commit_creates(&mut connection, &initial).unwrap().records;
    let path = fixture.0.join("fields.csv");
    std::fs::write(&path, "headword\nretained\nnew-empty\n").unwrap();
    let preview = preview_file(&connection, &path, "omitted").unwrap();
    assert_eq!(preview.groups[0].status, "identical");
    let result = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    assert_eq!((result.created, result.updated, result.skipped), (1, 0, 1));
    assert_eq!(
        get(&connection, records[0].id).unwrap().revision,
        records[0].revision
    );
    std::fs::write(
        &path,
        format!("headword,notes\nretained,\nskipped,replacement\nconflict,one\nvalid,new\nconflict,two\noversized,{}\n", "x".repeat(20_001)),
    )
    .unwrap();
    let preview = preview_file(&connection, &path, "clear-notes").unwrap();
    assert_eq!(preview.groups[2].rows, [4, 6]);
    assert!(
        preview.groups[2]
            .error
            .as_ref()
            .unwrap()
            .contains("conflicting notes")
    );
    let mut decisions = decide(&preview);
    assert_eq!(preview.groups[4].status, "invalid");
    assert_eq!(preview.groups[4].rows, [7]);
    assert!(preview.groups[4].error.as_ref().unwrap().contains("20,000"));
    decisions[4].action = NoteImportAction::Create;
    assert!(commit_preview(&mut connection, &preview, &decisions).is_err());
    assert!(find(&connection, "valid").unwrap().is_none());
    decisions[4].action = NoteImportAction::Skip;
    decisions[1].action = NoteImportAction::Skip;
    let result = commit_preview(&mut connection, &preview, &decisions).unwrap();
    assert_eq!((result.created, result.updated, result.skipped), (1, 1, 3));
    let retained = get(&connection, records[0].id).unwrap();
    assert_eq!(retained.notes, "");
    assert_eq!(retained.example_fr, "old French");
    assert_eq!(get(&connection, records[1].id).unwrap().notes, "keep");
    std::fs::write(&path, "headword,example_fr\nretained,\n").unwrap();
    let preview = preview_file(&connection, &path, "clear-examples").unwrap();
    commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    let retained = get(&fixture.open(), retained.id).unwrap();
    assert_eq!(
        (retained.example_fr.as_str(), retained.example_zh.as_str()),
        ("", "")
    );
    assert!(retained.more_examples.is_empty());
    assert!(find(&connection, "conflict").unwrap().is_none());
}

#[test]
fn nfc_groups_only_equivalent_spelling_and_preserves_interleaved_row_order() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let path = fixture.csv(&[
        row("école", "", "first", ""),
        row("Ecole", "", "", ""),
        row("ecole", "", "", ""),
        row("l'ami", "", "", ""),
        row("l’ami", "", "", ""),
        row("École", "", "", ""),
        row("e\u{301}cole", "", "second", ""),
    ]);
    let preview = preview_file(&connection, &path, "unicode-identities").unwrap();
    assert_eq!(preview.groups[0].rows, [2, 8]);
    assert_eq!(preview.groups.len(), 6);
    let result = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    assert_eq!(result.created, 6);
    let saved = find(&fixture.open(), "e\u{301}cole").unwrap().unwrap();
    assert_eq!(saved.headword, "école");
    assert_eq!(saved.example_fr, "first");
    assert_eq!(saved.more_examples[0].fr, "second");
}

#[test]
fn real_write_failure_rolls_back_mixed_creates_updates_receipts_and_all_learning_state() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let initial = preview_file(
        &connection,
        &fixture.csv(&[row("existing", "old", "", "")]),
        "rollback-seed",
    )
    .unwrap();
    let old = commit_creates(&mut connection, &initial).unwrap().records[0].clone();
    add_review(&mut connection, old.id).unwrap();
    let before =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap();
    let path = fixture.csv(&[
        row("first-create", "new", "", ""),
        row("existing", "update", "", ""),
        row("fail-last", "", "", ""),
    ]);
    let preview = preview_file(&connection, &path, "rollback-batch").unwrap();
    connection.execute_batch("CREATE TRIGGER fail_import BEFORE INSERT ON vocabulary WHEN NEW.headword='fail-last' BEGIN SELECT RAISE(ABORT,'real batch failure'); END;").unwrap();
    assert!(
        commit_preview(&mut connection, &preview, &decide(&preview))
            .unwrap_err()
            .contains("real batch failure")
    );
    let after =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap();
    assert_eq!(before, after);
    assert!(find(&fixture.open(), "first-create").unwrap().is_none());
    connection
        .execute_batch("DROP TRIGGER fail_import")
        .unwrap();
    let result = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    assert_eq!((result.created, result.updated), (2, 1));
}

#[test]
fn second_connection_edits_or_creates_identity_stop_whole_batch_then_repreview_succeeds() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let seed = preview_file(
        &connection,
        &fixture.csv(&[row("existing", "old", "", "")]),
        "stale-seed",
    )
    .unwrap();
    let old = commit_creates(&mut connection, &seed).unwrap().records[0].clone();
    let path = fixture.csv(&[
        row("existing", "incoming", "", ""),
        row("new", "incoming", "", ""),
    ]);
    let preview = preview_file(&connection, &path, "stale-update").unwrap();
    let mut other = fixture.open();
    let mut edit = NoteImportContent {
        headword: old.headword.clone(),
        notes: Some("concurrent edit".into()),
        examples: None,
    }
    .save_request("other-edit", Some(&old))
    .unwrap();
    save(&mut other, &edit, None).unwrap();
    assert!(
        commit_preview(&mut connection, &preview, &decide(&preview))
            .unwrap_err()
            .contains("after preview")
    );
    assert!(find(&connection, "new").unwrap().is_none());
    assert_eq!(
        find(&connection, "existing").unwrap().unwrap().notes,
        "concurrent edit"
    );
    let fresh = preview_file(&connection, &path, "stale-create").unwrap();
    let other_preview = preview_file(
        &other,
        &fixture.csv(&[row("new", "other create", "", "")]),
        "other-create",
    )
    .unwrap();
    commit_creates(&mut other, &other_preview).unwrap();
    assert!(
        commit_preview(&mut connection, &fresh, &decide(&fresh))
            .unwrap_err()
            .contains("after preview")
    );
    let updated = get(&connection, old.id).unwrap();
    assert_eq!(updated.notes, "concurrent edit");
    let fresh = preview_file(&connection, &path, "fresh").unwrap();
    commit_preview(&mut connection, &fresh, &decide(&fresh)).unwrap();
    edit.operation = "replay-old-edit".into();
    assert_eq!(save(&mut other, &edit, None).unwrap().outcome, "conflict");
}

#[test]
fn lost_response_retries_same_selection_without_rewriting_and_rejects_changed_decisions() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let seed = preview_file(
        &connection,
        &fixture.csv(&[row("existing", "old", "", "")]),
        "retry-seed",
    )
    .unwrap();
    let old = commit_creates(&mut connection, &seed).unwrap().records[0].clone();
    let cards = add_review(&mut connection, old.id).unwrap();
    set_review_paused(&mut connection, old.id, true).unwrap();
    let preview = preview_file(
        &connection,
        &fixture.csv(&[
            row("existing", "new", "", ""),
            row("new", "created", "", ""),
        ]),
        "response-lost",
    )
    .unwrap();
    let decisions = decide(&preview);
    let first = commit_preview(&mut connection, &preview, &decisions).unwrap();
    drop(connection);
    let mut connection = fixture.open();
    let before =
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap();
    let replayed = commit_preview(&mut connection, &preview, &decisions).unwrap();
    assert_eq!(
        (replayed.created, replayed.updated, replayed.skipped),
        (1, 1, 0)
    );
    assert_eq!(
        replayed
            .records
            .iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        first
            .records
            .iter()
            .map(|record| record.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        serde_json::to_value(export_learning_snapshot(&mut connection, 1_800_000_100).unwrap())
            .unwrap(),
        before
    );
    assert_eq!(review_cards(&connection, old.id).unwrap(), cards);
    assert!(review_paused(&connection, old.id).unwrap());
    let mut changed = decisions.clone();
    changed[1].action = NoteImportAction::Skip;
    assert!(
        commit_preview(&mut connection, &preview, &changed)
            .unwrap_err()
            .contains("identifier reused")
    );
}

fn dictionary_snapshot(word: &str) -> Snapshot {
    Snapshot {
        reference: Reference {
            dictionary_id: Some("mdx-12".into()),
            dictionary: "Local fixture".into(),
            lookup_key: word.into(),
            version: "1".into(),
            attribution: Some("Fixture author".into()),
            license: Some("MIT".into()),
            source_url: None,
            license_url: None,
            sha256: "a".repeat(64),
        },
        digest: vec![DigestEntry {
            pronunciations: vec![],
            grammar: vec![],
            translation: Some("rich original meaning".into()),
            core: None,
        }],
    }
}

#[test]
fn reordered_duplicate_examples_keep_occurrence_provenance_and_moved_primary_meaning() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let snapshot = dictionary_snapshot("provenance");
    let mut request = NoteImportContent {
        headword: "provenance".into(),
        notes: Some("keep".into()),
        examples: Some(vec![
            NoteExample {
                fr: "duplicate".into(),
                zh: "重复".into(),
            },
            NoteExample {
                fr: "other".into(),
                zh: "其他".into(),
            },
            NoteExample {
                fr: "duplicate".into(),
                zh: "重复".into(),
            },
        ]),
    }
    .create_request("linked-seed")
    .unwrap();
    request.example_sense = Some(0);
    request.example_source = Some(snapshot.reference.clone());
    let mut second_source = snapshot.reference.clone();
    second_source.dictionary = "Second occurrence".into();
    request.more_examples[1].source = Some(second_source.clone());
    let original = save(&mut connection, &request, Some(snapshot))
        .unwrap()
        .record;
    let path = fixture.csv(&[
        row("provenance", "keep", "other", "其他"),
        row("provenance", "keep", "duplicate", "重复"),
        row("provenance", "keep", "duplicate", "重复"),
        row("provenance", "keep", "new French", "new Chinese"),
    ]);
    let preview = preview_file(&connection, &path, "reorder").unwrap();
    commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    let saved = get(&fixture.open(), original.id).unwrap();
    assert!(saved.example_sense.is_none());
    assert_eq!(saved.more_examples[0].source, original.example_source);
    assert_eq!(saved.more_examples[1].source, Some(second_source));
    assert!(saved.more_examples[2].source.is_none());
    assert_eq!(
        serde_json::to_value(&saved.more_examples[0]).unwrap()["sense"],
        0,
        "Moving an unchanged primary example must retain its meaning link"
    );
    assert!(
        serde_json::to_value(&saved.more_examples[2])
            .unwrap()
            .get("sense")
            .is_none()
    );
}

fn decide(preview: &NoteImportPreview) -> Vec<NoteImportSelection> {
    preview
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| NoteImportSelection {
            index,
            allow_unlinked: false,
            action: match group.status {
                "create" => NoteImportAction::Create,
                "different" => NoteImportAction::Update,
                _ => NoteImportAction::Skip,
            },
        })
        .collect()
}

#[test]
fn updates_apply_only_supplied_fields_and_reopen_for_unlinked_example_edits() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let initial = preview_file(
        &connection,
        &fixture.csv(&[row("ancien", "keep notes", "old French", "old Chinese")]),
        "seed",
    )
    .unwrap();
    let original = commit_creates(&mut connection, &initial).unwrap().records[0].clone();
    let path = fixture.0.join("partial.csv");
    std::fs::write(&path, "headword,example_fr\nancien,new French\nnouveau,\n").unwrap();
    let preview = preview_file(&connection, &path, "partial-update").unwrap();
    assert_eq!(preview.groups[0].status, "different");
    let saved = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
    assert_eq!((saved.created, saved.updated, saved.skipped), (1, 1, 0));
    let updated = get(&connection, original.id).unwrap();
    assert_eq!(updated.notes, "keep notes");
    assert_eq!(updated.example_fr, "new French");
    assert_eq!(updated.example_zh, "");
    assert!(updated.more_examples.is_empty());
    assert_eq!(updated.created_at, original.created_at);
    assert_eq!(updated.import_source, original.import_source);
    assert_eq!(find(&connection, "nouveau").unwrap().unwrap().notes, "");
    drop(connection);
    let mut connection = fixture.open();
    let current = get(&connection, original.id).unwrap();
    let mut edit = NoteImportContent {
        headword: current.headword.clone(),
        notes: Some("edited after restart".into()),
        examples: Some(vec![NoteExample {
            fr: "editable unlinked".into(),
            zh: "".into(),
        }]),
    }
    .create_request("after-import-edit")
    .unwrap();
    edit.id = Some(current.id);
    edit.revision = Some(current.revision);
    assert_eq!(
        save(&mut connection, &edit, None).unwrap().outcome,
        "updated"
    );
}
