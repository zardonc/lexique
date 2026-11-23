use super::*;
use lexique_storage::{initialize, list_grammar, review_paused};

#[test]
fn batch_csv_previews_multiple_independent_rows() {
    let connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let batch = preview(&connection, "title\nSame\nSame\n", "batch-red");
    assert!(batch.error.is_none(), "{:?}", batch.error);
    assert_eq!(batch.groups.len(), 2);
}

#[test]
fn receipt_cannot_reassign_a_selected_identifier_free_row_to_an_equal_independent_row() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    let batch = preview(
        &db,
        "title\nIndependent\nIndependent\n",
        "selected-row-replay",
    );
    let mut first = decide(&batch);
    first[1].action = NoteImportAction::Skip;
    let saved = commit_preview(&mut db, &batch, &first).unwrap();
    assert_eq!(saved.created, 1);
    assert_eq!(
        commit_preview(&mut db, &batch, &first).unwrap().records,
        saved.records
    );
    let mut changed = first;
    changed[0].action = NoteImportAction::Skip;
    changed[1].action = NoteImportAction::Create;
    assert!(
        commit_preview(&mut db, &batch, &changed).is_err(),
        "independent rows must retain distinct submission identity even when their text is equal"
    );
    assert_eq!(list_grammar(&db, "", "created-asc").unwrap().len(), 1);
}

#[test]
fn mapped_bom_variants_and_delimiters_preserve_raw_fields_and_validate_mapping() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (index, (bom, delimiter)) in [
        ("", NoteDelimiter::Comma),
        ("\u{feff}", NoteDelimiter::Comma),
        ("", NoteDelimiter::Semicolon),
        ("\u{feff}", NoteDelimiter::Semicolon),
    ]
    .into_iter()
    .enumerate()
    {
        let sep = if matches!(delimiter, NoteDelimiter::Comma) {
            ","
        } else {
            ";"
        };
        let csv = format!(
            "{bom}ID{sep}Heading{sep}Text{sep}Q{sep}A\r\nexternal-{index}{sep}  Été 中文  {sep}\"raw,; \'\'\r\ntext\"{sep}Q{sep}A\r\n"
        );
        let source = ImportSource {
            filename: "external.csv".into(),
            format: lexique_storage::NoteFormat::Csv,
        };
        let mut options = NoteImportOptions {
            delimiter,
            ..Default::default()
        };
        let automatic = preview_csv_with_options(
            &connection,
            source.clone(),
            csv.as_bytes(),
            "mapping",
            &options,
        )
        .unwrap();
        assert!(automatic.error.as_ref().unwrap().contains("title"));
        assert_eq!(automatic.columns.len(), 5);
        let mapping = GrammarImportMapping {
            exchange_id: Some(0),
            title: Some(1),
            body: Some(2),
            question: Some(3),
            answer: Some(4),
        };
        for invalid in [
            GrammarImportMapping {
                title: None,
                ..mapping.clone()
            },
            GrammarImportMapping {
                title: Some(5),
                ..mapping.clone()
            },
            GrammarImportMapping {
                body: Some(1),
                ..mapping.clone()
            },
        ] {
            options.grammar_mapping = Some(invalid);
            let invalid = preview_csv_with_options(
                &connection,
                source.clone(),
                csv.as_bytes(),
                "bad-mapping",
                &options,
            )
            .unwrap();
            assert!(invalid.error.is_some());
            assert!(commit_preview(&mut connection, &invalid, &[]).is_err());
        }
        options.grammar_mapping = Some(mapping);
        let mapped = preview_csv_with_options(
            &connection,
            source,
            csv.as_bytes(),
            &format!("mapped-{index}"),
            &options,
        )
        .unwrap();
        assert!(mapped.error.is_none(), "{:?}", mapped.error);
        assert_eq!(mapped.groups[0].rows, vec![2]);
        let saved = commit_preview(&mut connection, &mapped, &decide(&mapped)).unwrap();
        assert_eq!(saved.records[0].title, "Été 中文");
        assert_eq!(saved.records[0].body, "raw,; ''\r\ntext");
        assert_eq!(saved.records[0].card.as_ref().unwrap().answer, "A");
    }
}

#[test]
fn identical_identity_coalesces_conflicts_invalidate_whole_group_and_id_free_rows_stay_independent()
{
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    for (index, (first, second)) in [("Same", "Changed"), ("Changed", "Same"), (" Same", "Same")]
        .into_iter()
        .enumerate()
    {
        let batch = preview(
            &db,
            &format!(
                "exchange_id,title\nconflict,{first}\nconflict,{second}\nidentical,Identical\nidentical,Identical\n,Independent\n,Independent\nbad, \n"
            ),
            &format!("groups-{index}"),
        );
        assert!(batch.error.is_none());
        assert_eq!(batch.groups.len(), 5);
        assert_eq!(batch.groups[0].status, "invalid");
        assert_eq!(batch.groups[0].rows, vec![2, 3]);
        assert!(
            batch.groups[0]
                .error
                .as_ref()
                .unwrap()
                .contains("conflicting")
        );
        assert_eq!(batch.groups[1].rows, vec![4, 5]);
        assert_eq!(
            batch.groups[1].status,
            if index == 0 { "create" } else { "identical" }
        );
        let mut decisions = decide(&batch);
        decisions[0].action = NoteImportAction::Skip;
        decisions[4].action = NoteImportAction::Skip;
        let saved = commit_preview(&mut db, &batch, &decisions).unwrap();
        assert_eq!(saved.created, if index == 0 { 3 } else { 2 });
        assert!(
            lexique_storage::find_grammar_exchange(&db, "conflict")
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(list_grammar(&db, "", "created-asc").unwrap().len(), 7);
}

#[test]
fn malformed_rows_are_located_and_excludable_and_presence_is_not_coalesced_with_empty() {
    let mut db = Connection::open_in_memory().unwrap();
    initialize(&db).unwrap();
    for (index, csv) in [
        "exchange_id,title,body\nsame,Same\nsame,Same,\n,Valid,Body\n",
        "exchange_id,title,body\nsame,Same,\nsame,Same\n,Valid,Body\n",
    ]
    .into_iter()
    .enumerate()
    {
        let batch = preview(&db, csv, &format!("presence-{index}"));
        assert!(batch.error.is_none());
        assert_eq!(batch.groups[0].status, "invalid");
        assert_eq!(batch.groups[0].rows, vec![2, 3]);
        let mut selections = decide(&batch);
        selections[0].action = NoteImportAction::Skip;
        commit_preview(&mut db, &batch, &selections).unwrap();
    }
    assert_eq!(list_grammar(&db, "", "created-asc").unwrap().len(), 2);
}

#[test]
fn canonical_csv_previews_real_file_and_commits_paused_pair_with_identity() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../target/ticket-214/csv");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("paired.csv");
    std::fs::write(&path, "\u{feff}exchange_id,title,body,question,answer\r\nexternal-214,  Une règle  ,\"été, 中文\r\n  \",Pourquoi ?,Réponse\r\n").unwrap();
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let (source, bytes) =
        crate::note_import::read_note_file(&path, lexique_storage::NoteFormat::Csv).unwrap();
    let preview = preview_csv_bytes(&connection, source, &bytes, "csv-paired").unwrap();
    assert!(preview.error.is_none(), "{:?}", preview.error);
    let group = preview.groups.first().unwrap();
    assert_eq!(group.content.title, "  Une règle  ");
    assert_eq!(group.content.body.as_deref(), Some("été, 中文\r\n  "));
    assert_eq!(group.content.exchange_id.as_deref(), Some("external-214"));
    assert_eq!(group.rows, vec![2]);
    assert_eq!(group.status, "create");
    assert!(
        list_grammar(&connection, "", "created-asc")
            .unwrap()
            .is_empty()
    );
    let decision = crate::note_import::NoteImportSelection {
        index: 0,
        action: crate::note_import::NoteImportAction::Create,
        allow_unlinked: false,
    };
    let result = commit_preview(&mut connection, &preview, &[decision]).unwrap();
    assert_eq!(result.created, 1);
    assert!(review_paused(&connection, result.records[0].id).unwrap());
}

fn preview(connection: &Connection, text: &str, operation: &str) -> GrammarImportPreview {
    preview_csv_bytes(
        connection,
        lexique_storage::ImportSource {
            filename: "grammar.csv".into(),
            format: lexique_storage::NoteFormat::Csv,
        },
        text.as_bytes(),
        operation,
    )
    .unwrap()
}

fn decide(preview: &GrammarImportPreview) -> Vec<crate::note_import::NoteImportSelection> {
    preview
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| crate::note_import::NoteImportSelection {
            index,
            action: match group.status {
                "identical" => NoteImportAction::Skip,
                "different" => NoteImportAction::Update,
                _ => NoteImportAction::Create,
            },
            allow_unlinked: false,
        })
        .collect()
}

#[test]
fn canonical_csv_missing_fields_and_blank_pair_follow_new_note_rules() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for (index, text) in [
        "title\nOnly title\n",
        "exchange_id,title,body,question,answer\n,Only title,,  ,\t\n",
        "title,body\nOnly title,\n",
    ]
    .iter()
    .enumerate()
    {
        let preview = preview(&connection, text, &format!("missing-{index}"));
        assert!(preview.error.is_none());
        let saved = commit_preview(&mut connection, &preview, &decide(&preview)).unwrap();
        assert_eq!(saved.records[0].body, "");
        assert_eq!(saved.records[0].card, None);
    }
    assert_eq!(
        list_grammar(&connection, "", "created-asc").unwrap().len(),
        3
    );
}

#[test]
fn canonical_csv_invalid_fields_are_located_and_cannot_commit() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let texts = [
        "title\n \n".into(),
        format!("title\n{}\n", "é".repeat(201)),
        "title\n\"two\nlines\"\n".into(),
        format!("title,body\nRule,{}\n", "é".repeat(20001)),
        "title,question,answer\nRule,Question,\n".into(),
        "title,question,answer\nRule, ,Answer\n".into(),
        format!("title,question,answer\nRule,{},Answer\n", "é".repeat(20001)),
        format!(
            "title,question,answer\nRule,Question,{}\n",
            "é".repeat(20001)
        ),
        "exchange_id,title\ninvalid/id,Rule\n".into(),
    ];
    for text in texts {
        let preview = preview(&connection, &text, "invalid");
        let group = preview.groups.first().unwrap();
        assert_eq!(group.status, "invalid");
        assert!(group.error.as_ref().unwrap().contains("grammar.csv row 2"));
        assert!(commit_preview(&mut connection, &preview, &decide(&preview)).is_err());
    }
    assert!(
        list_grammar(&connection, "", "created-asc")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn malformed_wrong_template_duplicate_header_and_multi_record_csv_never_commit() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    for text in [
        "headword,notes\nword,note\n",
        "title,title\nOne,Two\n",
        "title\n\"unclosed\n",
        "title\n",
    ] {
        let preview = preview(&connection, text, "structure");
        assert!(preview.error.as_ref().unwrap().contains("grammar.csv"));
        assert!(commit_preview(&mut connection, &preview, &[]).is_err());
    }
    assert!(
        list_grammar(&connection, "", "created-asc")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn known_identity_skips_unchanged_and_allows_explicit_skip_of_differences() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = preview(
        &connection,
        "exchange_id,title,body,question,answer\nknown-214,Rule,Body,Question,Answer\n",
        "first",
    );
    let original = commit_preview(&mut connection, &first, &decide(&first))
        .unwrap()
        .records
        .remove(0);
    let same = preview(&connection, "exchange_id,title\nknown-214,Rule\n", "same");
    assert_eq!(same.groups.first().unwrap().status, "identical");
    assert_eq!(
        commit_preview(&mut connection, &same, &decide(&same))
            .unwrap()
            .skipped,
        1
    );
    for text in [
        "exchange_id,title\nknown-214,Renamed\n",
        "exchange_id,title,body\nknown-214,Rule,\n",
        "exchange_id,title,question,answer\nknown-214,Rule,New question,New answer\n",
    ] {
        let changed = preview(&connection, text, "changed");
        assert_eq!(changed.groups.first().unwrap().status, "different");
        let mut selections = decide(&changed);
        selections[0].action = NoteImportAction::Skip;
        assert_eq!(
            commit_preview(&mut connection, &changed, &selections)
                .unwrap()
                .skipped,
            1
        );
    }
    assert_eq!(
        lexique_storage::get_grammar(&connection, original.id).unwrap(),
        original
    );
    assert!(review_paused(&connection, original.id).unwrap());
}

#[test]
fn native_preview_commit_replays_and_rejects_newly_occupied_identity_or_revised_match() {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let first = preview(
        &connection,
        "exchange_id,title\noccupied-214,Rule\n",
        "first",
    );
    let stale = preview(
        &connection,
        "exchange_id,title\noccupied-214,Rule\n",
        "stale",
    );
    let result = commit_preview(&mut connection, &first, &decide(&first)).unwrap();
    let retry = commit_preview(&mut connection, &first, &decide(&first)).unwrap();
    assert_eq!(retry.records, result.records);
    assert!(
        commit_preview(&mut connection, &stale, &decide(&stale))
            .unwrap_err()
            .contains("changed after preview")
    );
    let known = preview(
        &connection,
        "exchange_id,title\noccupied-214,Imported rename\n",
        "known",
    );
    let saved = &result.records[0];
    lexique_storage::save_grammar(
        &connection,
        &lexique_storage::GrammarSaveRequest {
            operation: "edit-local".into(),
            id: Some(saved.id),
            revision: Some(saved.revision),
            title: "Local edit".into(),
            body: "Local body".into(),
            category_ids: vec![],
        },
        1000,
    )
    .unwrap();
    assert!(
        commit_preview(&mut connection, &known, &decide(&known))
            .unwrap_err()
            .contains("changed after preview")
    );
    assert_eq!(
        list_grammar(&connection, "", "created-asc").unwrap().len(),
        1
    );
}
