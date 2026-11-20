use super::*;
use lexique_storage::{find, get, initialize, read_dictionary_catalog, write_dictionary_catalog};
use std::path::PathBuf;

fn preview_csv_file_with_dictionaries(
    connection: &Connection,
    path: &Path,
    operation: &str,
    database: &crate::vocabulary::DatabasePath,
    builtin: &Path,
    registry: &crate::dictionaries::Registry,
) -> Result<NoteImportPreview, String> {
    let mut preview = preview_csv_file(connection, path, operation, &NoteImportOptions::default())?;
    enrich_preview(&mut preview, database, builtin, registry);
    Ok(preview)
}

fn save_one(
    connection: &mut Connection,
    preview: &NoteImportPreview,
) -> Result<VocabularyRecord, String> {
    save_one_with_consent(connection, preview, false)
}

fn save_one_with_consent(
    connection: &mut Connection,
    preview: &NoteImportPreview,
    allow_unlinked: bool,
) -> Result<VocabularyRecord, String> {
    let selections = preview
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| NoteImportSelection {
            index,
            action: if group.status == "different" {
                NoteImportAction::Update
            } else {
                NoteImportAction::Create
            },
            allow_unlinked,
        })
        .collect::<Vec<_>>();
    super::commit_preview(connection, preview, &selections)?
        .records
        .into_iter()
        .next()
        .ok_or("No record saved".into())
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lexique-note-enrichment-187-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        for (source, target) in [
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
            let target = root.join(target);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(repo.join(source), target).unwrap();
        }
        Self(root)
    }
    fn open(&self) -> Connection {
        let connection = Connection::open(self.0.join("lexique.sqlite3")).unwrap();
        initialize(&connection).unwrap();
        connection
    }
    fn csv(&self, headword: &str) -> PathBuf {
        let path = self.0.join("notes.csv");
        let mut writer = csv::Writer::from_path(&path).unwrap();
        writer
            .write_record(["headword", "notes", "example_fr", "example_zh"])
            .unwrap();
        writer
            .write_record([headword, "笔记", "Une phrase.", "例句"])
            .unwrap();
        writer.flush().unwrap();
        path
    }
    fn database(&self) -> crate::vocabulary::DatabasePath {
        crate::vocabulary::DatabasePath(self.0.join("lexique.sqlite3"))
    }
    fn preview(
        &self,
        connection: &Connection,
        headword: &str,
        operation: &str,
    ) -> NoteImportPreview {
        preview_csv_file_with_dictionaries(
            connection,
            &self.csv(headword),
            operation,
            &self.database(),
            &self.0.join("dictionary/dictionary.sqlite3"),
            &crate::dictionaries::Registry::default(),
        )
        .unwrap()
    }
    fn install_custom(&self) {
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../tools/mdict-probe/tests/fixtures/v2-utf8-zlib.mdx"),
        )
        .unwrap();
        crate::dictionaries::import(&self.database(), "original.mdx", &bytes).unwrap();
    }
    fn first_source(&self, connection: &Connection, id: &str) {
        let mut catalog = read_dictionary_catalog(connection).unwrap();
        let index = catalog
            .entries
            .iter()
            .position(|entry| entry.id == id)
            .unwrap();
        let entry = catalog.entries.remove(index);
        catalog.entries.insert(0, entry);
        write_dictionary_catalog(connection, &catalog).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn exact_installed_dictionary_import_persists_genuine_reference_and_digest() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let preview = preview_csv_file_with_dictionaries(
        &connection,
        &fixture.csv("œil"),
        "enrich-exact",
        &crate::vocabulary::DatabasePath(fixture.0.join("lexique.sqlite3")),
        &fixture.0.join("dictionary/dictionary.sqlite3"),
        &crate::dictionaries::Registry::default(),
    )
    .unwrap();
    let saved = save_one(&mut connection, &preview).unwrap();
    drop(connection);
    let reopened = fixture.open();
    let record = get(&reopened, saved.id).unwrap();
    assert!(
        record.reference.is_some(),
        "exact installed headword must be linked"
    );
    assert!(!record.digest.is_empty());
    assert_eq!(record.notes, "笔记");
    assert_eq!(record.example_sense, None);
}

#[test]
fn installed_core_builtin_bundled_and_custom_sources_keep_real_snapshots_and_order() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    fixture.install_custom();
    for (index, id, headword) in [
        (1, "builtin-core", "être"),
        (2, "builtin", "œil"),
        (3, "builtin-fra-eng", "école"),
        (4, "mdx-1", "double"),
    ] {
        fixture.first_source(&connection, id);
        let preview = fixture.preview(&connection, headword, &format!("source-{index}"));
        let enrichment = preview.groups[0].enrichment.as_ref().unwrap();
        assert_eq!(enrichment.status, "linked", "{id}");
        let snapshot = enrichment.snapshot.as_ref().unwrap();
        assert_eq!(snapshot.reference.dictionary_id.as_deref(), Some(id));
        let saved = save_one(&mut connection, &preview).unwrap();
        assert_eq!(
            get(&fixture.open(), saved.id).unwrap().reference.as_ref(),
            Some(&snapshot.reference)
        );
        assert_eq!(saved.digest, snapshot.digest);
        assert_eq!(saved.headword, headword);
        assert_eq!(saved.import_source.as_ref().unwrap().filename, "notes.csv");
        assert_eq!(saved.example_sense, None);
        assert!(saved.example_source.is_none());
        if id == "builtin-core" {
            assert!(saved.digest[0].core.is_some());
        }
        if id == "mdx-1" {
            assert_eq!(saved.digest.len(), 2);
            assert!(saved.digest[0].translation.is_none());
        }
    }
}

#[test]
fn nfc_matches_but_case_accents_wordforms_and_aliases_do_not_change_import_identity() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    fixture.install_custom();
    let preview = fixture.preview(&connection, "e\u{301}cole", "canonical");
    let saved = save_one(&mut connection, &preview).unwrap();
    assert_eq!(saved.headword, "école");
    assert_eq!(saved.reference.as_ref().unwrap().lookup_key, "école");
    let exact_spelling = fixture.preview(&connection, "oeil", "separate-spelling");
    let spelling = save_one(&mut connection, &exact_spelling).unwrap();
    assert_eq!(spelling.headword, "oeil");
    assert_eq!(spelling.reference.as_ref().unwrap().lookup_key, "oeil");
    for (index, word) in ["ÊTRE", "ECOLE", "suis", "mangeons", "inconnu-187"]
        .iter()
        .enumerate()
    {
        let preview = fixture.preview(&connection, word, &format!("absence-{index}"));
        assert_eq!(
            preview.groups[0].enrichment.as_ref().unwrap().status,
            "not_found",
            "{word}"
        );
        let saved = save_one(&mut connection, &preview).unwrap();
        assert_eq!(saved.headword, *word);
        assert!(saved.reference.is_none());
        assert!(saved.digest.is_empty());
    }
}

#[test]
fn missing_corrupt_and_read_failures_require_consent_and_preserve_actual_file_context() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    std::fs::remove_file(fixture.0.join("dictionary/dictionary.sqlite3")).unwrap();
    std::fs::write(
        fixture.0.join("dictionary/core-pilot-79/core.sqlite3"),
        b"broken",
    )
    .unwrap();
    let bundled = fixture
        .0
        .join("dictionary/freedict-fra-eng-mdx/freedict-fra-eng-0.4.1.mdx");
    std::fs::remove_file(&bundled).unwrap();
    std::fs::write(&bundled, b"broken mdx").unwrap();
    let preview = fixture.preview(&connection, "inconnu-187", "resource-errors");
    let enrichment = preview.groups[0].enrichment.as_ref().unwrap();
    assert_eq!(enrichment.status, "error");
    assert_eq!(enrichment.warnings.len(), 3);
    assert!(
        enrichment
            .warnings
            .iter()
            .any(|warning| warning.code == "missing")
    );
    assert!(
        enrichment
            .warnings
            .iter()
            .any(|warning| warning.code == "corrupt")
    );
    assert!(
        save_one(&mut connection, &preview)
            .unwrap_err()
            .contains("continue")
    );
    assert!(find(&fixture.open(), "inconnu-187").unwrap().is_none());
    let saved = save_one_with_consent(&mut connection, &preview, true).unwrap();
    let reopened = get(&fixture.open(), saved.id).unwrap();
    assert!(reopened.reference.is_none());
    assert!(reopened.digest.is_empty());
    assert_eq!(
        reopened.import_source.as_ref().unwrap().filename,
        "notes.csv"
    );
}

#[test]
fn unreadable_resource_is_a_read_error_and_not_an_absent_headword() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let builtin = fixture.0.join("dictionary/dictionary.sqlite3");
    std::fs::remove_file(&builtin).unwrap();
    std::fs::create_dir(&builtin).unwrap();
    let preview = fixture.preview(&connection, "inconnu-187", "read-error");
    let enrichment = preview.groups[0].enrichment.as_ref().unwrap();
    assert_eq!(enrichment.status, "error");
    assert_eq!(enrichment.warnings[0].code, "query_failed");
    assert!(
        enrichment.warnings[0]
            .message
            .contains("dictionary.sqlite3")
    );
    assert!(save_one_with_consent(&mut connection, &preview, false).is_err());
    assert!(find(&fixture.open(), "inconnu-187").unwrap().is_none());
    assert!(
        save_one_with_consent(&mut connection, &preview, true)
            .unwrap()
            .reference
            .is_none()
    );
}

#[test]
fn missing_earlier_source_does_not_hide_exact_custom_match_or_its_warnings() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    fixture.install_custom();
    std::fs::remove_file(fixture.0.join("dictionary/core-pilot-79/core.sqlite3")).unwrap();
    let preview = fixture.preview(&connection, "redirect", "later-source");
    let enrichment = preview.groups[0].enrichment.as_ref().unwrap();
    assert_eq!(enrichment.status, "linked");
    assert_eq!(enrichment.warnings[0].dictionary_id, "builtin-core");
    let saved = save_one(&mut connection, &preview).unwrap();
    assert_eq!(
        saved.reference.unwrap().dictionary_id.as_deref(),
        Some("mdx-1")
    );
}

#[test]
fn existing_preview_and_successful_replay_retain_snapshot_without_resources() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let preview = fixture.preview(&connection, "être", "saved-snapshot");
    let saved = save_one(&mut connection, &preview).unwrap();
    assert!(saved.digest[0].core.is_some());
    std::fs::remove_dir_all(fixture.0.join("dictionary")).unwrap();
    let replay = save_one(&mut connection, &preview).unwrap();
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    let existing = fixture.preview(&connection, "être", "existing-snapshot");
    assert_eq!(
        existing.groups[0].enrichment.as_ref().unwrap().status,
        "retained"
    );
    let skipped = super::commit_preview(
        &mut connection,
        &existing,
        &[NoteImportSelection {
            index: 0,
            action: NoteImportAction::Skip,
            allow_unlinked: false,
        }],
    )
    .unwrap();
    assert_eq!(
        (skipped.created, skipped.updated, skipped.skipped),
        (0, 0, 1)
    );
    let unchanged = get(&fixture.open(), saved.id).unwrap();
    assert_eq!(unchanged.reference, saved.reference);
    assert_eq!(unchanged.digest, saved.digest);
}

#[test]
fn updates_retain_rich_and_empty_snapshots_when_resources_are_missing_or_corrupt() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let mut saved = Vec::new();
    for (index, word) in ["être", "inconnu-187"].iter().enumerate() {
        let preview = fixture.preview(&connection, word, &format!("before-update-{index}"));
        saved.push(save_one(&mut connection, &preview).unwrap());
    }
    assert!(saved[0].digest[0].core.is_some());
    assert!(saved[1].reference.is_none());
    std::fs::remove_file(fixture.0.join("dictionary/core-pilot-79/core.sqlite3")).unwrap();
    std::fs::write(fixture.0.join("dictionary/dictionary.sqlite3"), b"corrupt").unwrap();
    for (index, old) in saved.iter().enumerate() {
        let path = fixture.csv(&old.headword);
        let mut writer = csv::Writer::from_path(&path).unwrap();
        writer
            .write_record(["headword", "notes", "example_fr", "example_zh"])
            .unwrap();
        writer
            .write_record([
                old.headword.as_str(),
                "修改后的笔记",
                "Nouvel exemple.",
                "新例句",
            ])
            .unwrap();
        writer.flush().unwrap();
        let preview = preview_csv_file_with_dictionaries(
            &connection,
            &path,
            &format!("update-snapshot-{index}"),
            &fixture.database(),
            &fixture.0.join("dictionary/dictionary.sqlite3"),
            &crate::dictionaries::Registry::default(),
        )
        .unwrap();
        assert_eq!(
            preview.groups[0].enrichment.as_ref().unwrap().status,
            "retained"
        );
        let updated = save_one(&mut connection, &preview).unwrap();
        let reopened = get(&fixture.open(), updated.id).unwrap();
        assert_eq!(reopened.id, old.id);
        assert_eq!(reopened.reference, old.reference);
        assert_eq!(reopened.digest, old.digest);
        assert_eq!(reopened.notes, "修改后的笔记");
        assert_eq!(reopened.example_fr, "Nouvel exemple.");
        assert_eq!(reopened.example_sense, None);
    }
}

#[test]
fn batch_resource_consent_gates_all_writes_and_replays_verified_snapshots_without_resources() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    fixture.install_custom();
    std::fs::remove_file(fixture.0.join("dictionary/dictionary.sqlite3")).unwrap();
    std::fs::write(
        fixture.0.join("dictionary/core-pilot-79/core.sqlite3"),
        b"broken",
    )
    .unwrap();
    std::fs::write(
        fixture
            .0
            .join("dictionary/freedict-fra-eng-mdx/freedict-fra-eng-0.4.1.mdx"),
        b"broken",
    )
    .unwrap();
    let path = fixture.0.join("batch.csv");
    std::fs::write(
        &path,
        "headword,notes\ndouble,première note\ninconnu-batch-187,note externe\n",
    )
    .unwrap();
    let preview = preview_csv_file_with_dictionaries(
        &connection,
        &path,
        "consent-batch",
        &fixture.database(),
        &fixture.0.join("dictionary/dictionary.sqlite3"),
        &crate::dictionaries::Registry::default(),
    )
    .unwrap();
    assert_eq!(
        preview.groups[0].enrichment.as_ref().unwrap().status,
        "linked"
    );
    assert_eq!(
        preview.groups[1].enrichment.as_ref().unwrap().status,
        "error"
    );
    let mut selections = vec![
        NoteImportSelection {
            index: 0,
            action: NoteImportAction::Create,
            allow_unlinked: false,
        },
        NoteImportSelection {
            index: 1,
            action: NoteImportAction::Create,
            allow_unlinked: false,
        },
    ];
    assert!(
        super::commit_preview(&mut connection, &preview, &selections).is_err(),
        "an unchecked affected creation must block the entire selected batch"
    );
    assert!(find(&fixture.open(), "double").unwrap().is_none());
    assert!(
        find(&fixture.open(), "inconnu-batch-187")
            .unwrap()
            .is_none()
    );

    std::fs::write(
        &path,
        "headword,notes\nredirect,note liée\ninconnu-skip-187,à exclure\n",
    )
    .unwrap();
    let skipped_preview = preview_csv_file_with_dictionaries(
        &connection,
        &path,
        "skip-batch",
        &fixture.database(),
        &fixture.0.join("dictionary/dictionary.sqlite3"),
        &crate::dictionaries::Registry::default(),
    )
    .unwrap();
    let skipped = super::commit_preview(
        &mut connection,
        &skipped_preview,
        &[
            NoteImportSelection {
                index: 0,
                action: NoteImportAction::Create,
                allow_unlinked: false,
            },
            NoteImportSelection {
                index: 1,
                action: NoteImportAction::Skip,
                allow_unlinked: false,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        (skipped.created, skipped.updated, skipped.skipped),
        (1, 0, 1)
    );
    assert!(skipped.records[0].reference.is_some());
    assert!(find(&fixture.open(), "inconnu-skip-187").unwrap().is_none());

    selections[1].allow_unlinked = true;
    let saved = super::commit_preview(&mut connection, &preview, &selections).unwrap();
    assert_eq!((saved.created, saved.updated, saved.skipped), (2, 0, 0));
    assert_eq!(
        saved.records[0]
            .reference
            .as_ref()
            .unwrap()
            .dictionary_id
            .as_deref(),
        Some("mdx-1")
    );
    assert_eq!(saved.records[0].digest.len(), 2);
    assert!(saved.records[1].reference.is_none());
    assert!(saved.records[1].digest.is_empty());
    for record in &saved.records {
        let reopened = get(&fixture.open(), record.id).unwrap();
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(record).unwrap()
        );
        assert_eq!(
            reopened.import_source.as_ref().unwrap().filename,
            "batch.csv"
        );
    }
    std::fs::remove_dir_all(fixture.0.join("dictionary")).unwrap();
    let replay = super::commit_preview(&mut connection, &preview, &selections).unwrap();
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    selections[1].action = NoteImportAction::Skip;
    assert!(super::commit_preview(&mut connection, &preview, &selections).is_err());
}

#[test]
fn valid_long_import_headword_reports_query_limit_without_claiming_resource_damage() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let headword = "词".repeat(200);
    let preview = fixture.preview(&connection, &headword, "long-headword");
    assert_eq!(preview.groups[0].status, "create");
    let enrichment = preview.groups[0].enrichment.as_ref().unwrap();
    assert_eq!(enrichment.status, "error");
    assert!(
        enrichment
            .warnings
            .iter()
            .all(|warning| warning.code == "invalid_query")
    );
    assert!(save_one(&mut connection, &preview).is_err());
    let record = save_one_with_consent(&mut connection, &preview, true).unwrap();
    assert_eq!(record.headword, headword);
    assert!(record.reference.is_none());
}

#[test]
fn decomposed_custom_dictionary_headword_links_by_exact_nfc_identity() {
    let fixture = Fixture::new();
    let mut connection = fixture.open();
    let bytes = include_bytes!("../fixtures/nfd-headword.mdx");
    crate::dictionaries::import(&fixture.database(), "nfd.mdx", bytes).unwrap();
    fixture.first_source(&connection, "mdx-1");
    let preview = fixture.preview(&connection, "élève", "decomposed-source");
    let record = save_one(&mut connection, &preview).unwrap();
    assert_eq!(
        record.reference.as_ref().unwrap().dictionary_id.as_deref(),
        Some("mdx-1")
    );
    assert_eq!(
        record.reference.as_ref().unwrap().lookup_key,
        "e\u{301}le\u{300}ve"
    );
    assert_eq!(record.headword, "élève");
}
