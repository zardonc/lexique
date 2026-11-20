use std::{fs::File, io::Read, path::Path, sync::Mutex};

use crate::note_enrichment::{NoteEnrichment, enrich_new_note};
use lexique_storage::{
    ImportSource, NoteExample, NoteFormat, NoteImportContent, SaveResult, VocabularyRecord,
};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Clone, Debug, Serialize)]
pub struct NoteImportPreview {
    pub operation: String,
    pub source: ImportSource,
    pub content: Option<NoteImportContent>,
    pub rows: Vec<u64>,
    pub existing: Option<VocabularyRecord>,
    pub status: &'static str,
    pub error: Option<String>,
    pub enrichment: Option<NoteEnrichment>,
}

#[derive(Default)]
pub struct NoteImportState(Mutex<Option<(String, Option<NoteImportPreview>)>>);

pub fn preview_csv_file(
    connection: &Connection,
    path: &Path,
    operation: &str,
) -> Result<NoteImportPreview, String> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("CSV filename is not UTF-8")?;
    let source = ImportSource {
        format: NoteFormat::Csv,
        filename: filename.into(),
    };
    source.validate().map_err(|e| e.to_string())?;
    let file = File::open(path).map_err(|e| format!("Could not open CSV: {e}"))?;
    let mut bytes = Vec::new();
    file.take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Could not read CSV: {e}"))?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("Single-word CSV exceeds 32 MiB".into());
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    let mut reader = csv::Reader::from_reader(bytes);
    if reader.headers().map_err(|e| format!("CSV header: {e}"))?
        != ["headword", "notes", "example_fr", "example_zh"].as_slice()
    {
        return Err(
            "CSV requires canonical columns headword,notes,example_fr,example_zh in that order"
                .into(),
        );
    }
    let mut preview = NoteImportPreview {
        operation: operation.into(),
        source,
        content: None,
        rows: vec![],
        existing: None,
        status: "invalid",
        error: None,
        enrichment: None,
    };
    for row in reader.records() {
        let row = row.map_err(|e| format!("CSV record: {e}"))?;
        let line = row.position().map(|p| p.line()).unwrap_or(0);
        preview.rows.push(line);
        let headword = lexique_storage::normalize_headword(&row[0]);
        let content = preview.content.get_or_insert_with(|| NoteImportContent {
            headword: headword.clone(),
            notes: Some(row[1].into()),
            examples: Some(vec![]),
        });
        if headword != content.headword || content.notes.as_deref() != Some(&row[1]) {
            preview.error = Some(format!(
                "Row {line}: single-word CSV must use the same NFC headword and notes on every row"
            ));
            return Ok(preview);
        }
        let example = NoteExample {
            fr: row[2].into(),
            zh: row[3].into(),
        };
        if !example.fr.is_empty() || !example.zh.is_empty() {
            content.examples.as_mut().unwrap().push(example);
        }
        if let Err(error) = content.create_request(operation) {
            preview.error = Some(format!("Row {line}, {}: {error}", content.headword));
            return Ok(preview);
        }
    }
    let content = preview
        .content
        .as_ref()
        .ok_or("CSV has no vocabulary records")?;
    preview.existing =
        lexique_storage::find(connection, &content.headword).map_err(|e| e.to_string())?;
    preview.status = match &preview.existing {
        Some(record) if content.matches(record) => "identical",
        Some(_) => "different",
        None => "create",
    };
    Ok(preview)
}

pub fn commit_preview(
    connection: &mut Connection,
    preview: &NoteImportPreview,
) -> Result<SaveResult, String> {
    commit_preview_with_consent(connection, preview, false)
}

pub fn commit_preview_with_consent(
    connection: &mut Connection,
    preview: &NoteImportPreview,
    allow_unlinked: bool,
) -> Result<SaveResult, String> {
    let content = preview
        .content
        .as_ref()
        .ok_or("CSV preview has no record")?;
    let request = content
        .create_request(&preview.operation)
        .map_err(|e| e.to_string())?;
    if preview.status == "identical" {
        let expected = preview
            .existing
            .as_ref()
            .ok_or("Identical preview has no saved record")?;
        let current = lexique_storage::get(connection, expected.id).map_err(|e| e.to_string())?;
        if current.revision != expected.revision || !content.matches(&current) {
            return Err("Record changed after preview; preview again".into());
        }
        return Ok(SaveResult {
            outcome: "existing",
            record: current,
        });
    }
    if preview.status != "create" {
        return Err("Resolve CSV errors or differences before importing".into());
    }
    if preview
        .enrichment
        .as_ref()
        .is_some_and(|value| value.status == "error")
        && !allow_unlinked
    {
        return Err("Choose whether to continue without a dictionary link before importing".into());
    }
    let snapshot = preview
        .enrichment
        .as_ref()
        .and_then(|value| value.snapshot.clone());
    lexique_storage::save_imported_note(connection, &request, snapshot, preview.source.clone())
        .map_err(|e| e.to_string())
}

pub fn preview_csv_file_with_dictionaries(
    connection: &Connection,
    file: &Path,
    operation: &str,
    database: &crate::vocabulary::DatabasePath,
    builtin: &Path,
    registry: &crate::dictionaries::Registry,
) -> Result<NoteImportPreview, String> {
    let mut preview = preview_csv_file(connection, file, operation)?;
    if preview.status == "create" {
        preview.enrichment = Some(enrich_new_note(
            registry,
            database,
            builtin,
            &preview.content.as_ref().unwrap().headword,
        ));
    } else if preview.existing.is_some() {
        preview.enrichment = Some(NoteEnrichment::retained());
    }
    Ok(preview)
}

#[tauri::command]
pub async fn note_import_preview(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
) -> Result<Option<NoteImportPreview>, String> {
    if window.label() != "main" {
        return Err("Note import requires the main window".into());
    }
    lexique_storage::NoteImportContent {
        headword: "validate".into(),
        notes: None,
        examples: None,
    }
    .create_request(&operation)
    .map_err(|e| e.to_string())?;
    let admission = crate::maintenance::admit_app(&app)?;
    *app.state::<NoteImportState>()
        .0
        .lock()
        .map_err(|_| "Note import state unavailable")? = Some((operation.clone(), None));
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let selected = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("CSV", &["csv"])
            .blocking_pick_file();
        let Some(selected) = selected else {
            return Ok(None);
        };
        let path = selected
            .into_path()
            .map_err(|e| format!("Could not resolve CSV file: {e}"))?;
        let connection =
            crate::vocabulary::open(app.state::<crate::vocabulary::DatabasePath>().inner())?;
        let builtin = app
            .path()
            .resource_dir()
            .map_err(|e| e.to_string())?
            .join("dictionary/dictionary.sqlite3");
        let preview = preview_csv_file_with_dictionaries(
            &connection,
            &path,
            &operation,
            &app.state::<crate::vocabulary::DatabasePath>(),
            &builtin,
            &app.state::<crate::dictionaries::Registry>(),
        )?;
        let state = app.state::<NoteImportState>();
        let mut state = state
            .0
            .lock()
            .map_err(|_| "Note import state unavailable")?;
        if state
            .as_ref()
            .is_none_or(|(active, _)| active != &operation)
        {
            return Err("CSV selection was replaced or cancelled".into());
        }
        *state = Some((operation, Some(preview.clone())));
        Ok(Some(preview))
    })
    .await
    .map_err(|e| format!("CSV preview task failed: {e}"))?
}

#[tauri::command]
pub async fn note_import_commit(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
    allow_unlinked: Option<bool>,
) -> Result<SaveResult, String> {
    if window.label() != "main" {
        return Err("Note import requires the main window".into());
    }
    let admission = crate::maintenance::admit_app(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let state = app.state::<NoteImportState>();
        let state = state
            .0
            .lock()
            .map_err(|_| "Note import state unavailable")?;
        let preview = state
            .as_ref()
            .filter(|(active, _)| active == &operation)
            .and_then(|(_, preview)| preview.as_ref())
            .ok_or("CSV preview no longer available; preview again")?;
        let mut connection =
            crate::vocabulary::open(app.state::<crate::vocabulary::DatabasePath>().inner())?;
        commit_preview_with_consent(&mut connection, preview, allow_unlinked.unwrap_or(false))
    })
    .await
    .map_err(|e| format!("CSV import task failed: {e}"))?
}

#[tauri::command]
pub fn note_import_cancel(
    state: State<'_, NoteImportState>,
    operation: String,
) -> Result<(), String> {
    let mut state = state
        .0
        .lock()
        .map_err(|_| "Note import state unavailable")?;
    if state
        .as_ref()
        .is_some_and(|(active, _)| active == &operation)
    {
        *state = None;
    }
    Ok(())
}

#[cfg(test)]
#[path = "note_import_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "note_enrichment_tests.rs"]
mod enrichment_tests;
