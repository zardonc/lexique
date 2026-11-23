use super::*;
use lexique_storage::{initialize, list_grammar, review_paused};

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
    let group = preview.group.as_ref().unwrap();
    assert_eq!(group.content.title, "  Une règle  ");
    assert_eq!(group.content.body.as_deref(), Some("été, 中文\r\n  "));
    assert_eq!(group.content.exchange_id.as_deref(), Some("external-214"));
    assert_eq!(group.row, 2);
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
    vec![crate::note_import::NoteImportSelection {
        index: 0,
        action: if preview
            .group
            .as_ref()
            .is_some_and(|group| group.status == "identical")
        {
            NoteImportAction::Skip
        } else {
            NoteImportAction::Create
        },
        allow_unlinked: false,
    }]
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
        let group = preview.group.as_ref().unwrap();
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
        "title\nOne\nTwo\n",
        "title\n\"unclosed\n",
        "title\n",
        "title,body\nRule\n",
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
fn known_identity_skips_unchanged_and_explains_changed_content_without_overwrite() {
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
    assert_eq!(same.group.as_ref().unwrap().status, "identical");
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
        assert_eq!(changed.group.as_ref().unwrap().status, "different");
        assert!(
            commit_preview(&mut connection, &changed, &decide(&changed))
                .unwrap_err()
                .contains("remain unchanged")
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
        "exchange_id,title\noccupied-214,Rule\n",
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
