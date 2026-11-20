use lexique_storage::{
    ImportSource, NoteFormat, NoteImportContent, NoteImportItem, NoteImportResult, VocabularyRecord,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteImportMapping {
    pub headword: Option<usize>,
    pub notes: Option<usize>,
    pub example_fr: Option<usize>,
    pub example_zh: Option<usize>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteDelimiter {
    #[default]
    Comma,
    Semicolon,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteImportOptions {
    pub delimiter: NoteDelimiter,
    pub mapping: Option<NoteImportMapping>,
    #[serde(default)]
    pub format: Option<NoteFormat>,
}
#[derive(Clone, Debug, Serialize)]
pub struct NoteImportGroup {
    pub content: NoteImportContent,
    pub rows: Vec<u64>,
    pub existing: Option<VocabularyRecord>,
    pub status: &'static str,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct NoteImportPreview {
    pub operation: String,
    pub source: ImportSource,
    pub columns: Vec<String>,
    pub mapping: NoteImportMapping,
    pub groups: Vec<NoteImportGroup>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteImportSelection {
    pub index: usize,
    pub action: NoteImportAction,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NoteImportAction {
    Create,
    Update,
    Skip,
}
struct Session {
    operation: String,
    file: Option<(ImportSource, Vec<u8>)>,
    preview: Option<NoteImportPreview>,
}
#[derive(Default)]
pub struct NoteImportState(Mutex<Option<Session>>);

pub fn read_note_file(path: &Path, format: NoteFormat) -> Result<(ImportSource, Vec<u8>), String> {
    use std::{fs::File, io::Read};
    let source = ImportSource {
        format,
        filename: path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Import filename is not UTF-8: {}", path.display()))?
            .into(),
    };
    source
        .validate()
        .map_err(|error| format!("{}: {error}", source.filename))?;
    let file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(format!("{}: note file exceeds 32 MiB", source.filename));
    }
    Ok((source, bytes))
}

fn preview_note_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
    options: &NoteImportOptions,
) -> Result<NoteImportPreview, String> {
    match source.format {
        NoteFormat::Csv => {
            crate::note_import_csv::preview_csv_bytes(connection, source, bytes, operation, options)
        }
        NoteFormat::Markdown => {
            crate::note_markdown::preview_markdown_bytes(connection, source, bytes, operation)
        }
    }
}

#[cfg(test)]
pub fn preview_csv_file(
    connection: &Connection,
    path: &Path,
    operation: &str,
    options: &NoteImportOptions,
) -> Result<NoteImportPreview, String> {
    let (source, bytes) = crate::note_import_csv::read_csv_file(path)?;
    crate::note_import_csv::preview_csv_bytes(connection, source, &bytes, operation, options)
}
pub fn commit_preview(
    connection: &mut Connection,
    preview: &NoteImportPreview,
    selections: &[NoteImportSelection],
) -> Result<NoteImportResult, String> {
    if preview.error.is_some() || selections.len() != preview.groups.len() {
        return Err("Resolve file structure and mapping, then decide every group".into());
    }
    let mut decisions = vec![None; preview.groups.len()];
    for selection in selections {
        let decision = decisions
            .get_mut(selection.index)
            .ok_or("Invalid import group index")?;
        if decision.replace(&selection.action).is_some() {
            return Err("Duplicate import group decision".into());
        }
    }
    let mut items = Vec::new();
    let mut skipped = 0;
    for (group, decision) in preview.groups.iter().zip(decisions) {
        let decision = decision.ok_or("Decide every import group")?;
        if *decision == NoteImportAction::Skip {
            skipped += 1;
            continue;
        }
        if !matches!(
            (group.status, decision),
            ("create", NoteImportAction::Create) | ("different", NoteImportAction::Update)
        ) || group.error.is_some()
        {
            return Err("Resolve errors and choose create, update or skip for each group".into());
        }
        items.push(NoteImportItem {
            content: group.content.clone(),
            existing: group.existing.clone(),
            source: preview.source.clone(),
            snapshot: None,
        });
    }
    lexique_storage::save_note_import_batch(connection, &preview.operation, &items, skipped)
        .map_err(|e| e.to_string())
}
fn check_operation(operation: &str) -> Result<(), String> {
    NoteImportContent {
        headword: "validate".into(),
        notes: None,
        examples: None,
    }
    .create_request(operation)
    .map_err(|e| e.to_string())?;
    // Reserve space for each record's save-operation suffix.
    if operation.len() > 110 {
        return Err("Import operation exceeds 110 characters".into());
    }
    Ok(())
}
#[tauri::command]
pub async fn note_import_preview(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
    options: NoteImportOptions,
) -> Result<Option<NoteImportPreview>, String> {
    if window.label() != "main" {
        return Err("Note import requires the main window".into());
    }
    check_operation(&operation)?;
    let admission = crate::maintenance::admit_app(&app)?;
    *app.state::<NoteImportState>()
        .0
        .lock()
        .map_err(|_| "Note import state unavailable")? = Some(Session {
        operation: operation.clone(),
        file: None,
        preview: None,
    });
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let format = options.format.clone().unwrap_or(NoteFormat::Csv);
        let dialog = app.dialog().file().set_parent(&window);
        let selected = match format {
            NoteFormat::Csv => dialog.add_filter("CSV", &["csv"]),
            NoteFormat::Markdown => dialog.add_filter("Markdown", &["md", "markdown"]),
        }
        .blocking_pick_file();
        let Some(selected) = selected else {
            return Ok(None);
        };
        let path = selected
            .into_path()
            .map_err(|e| format!("Could not resolve note file: {e}"))?;
        let (source, bytes) = read_note_file(&path, format)?;
        let connection =
            crate::vocabulary::open(app.state::<crate::vocabulary::DatabasePath>().inner())?;
        let preview =
            preview_note_bytes(&connection, source.clone(), &bytes, &operation, &options)?;
        let state = app.state::<NoteImportState>();
        let mut state = state
            .0
            .lock()
            .map_err(|_| "Note import state unavailable")?;
        let session = state
            .as_mut()
            .filter(|session| session.operation == operation)
            .ok_or("Note selection was replaced or cancelled")?;
        session.file = Some((source, bytes));
        session.preview = Some(preview.clone());
        Ok(Some(preview))
    })
    .await
    .map_err(|e| format!("Note preview task failed: {e}"))?
}
#[tauri::command]
pub async fn note_import_remap(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
    options: NoteImportOptions,
) -> Result<NoteImportPreview, String> {
    if window.label() != "main" {
        return Err("Note import requires the main window".into());
    }
    let admission = crate::maintenance::admit_app(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let state = app.state::<NoteImportState>();
        let mut state = state
            .0
            .lock()
            .map_err(|_| "Note import state unavailable")?;
        let session = state
            .as_mut()
            .filter(|session| session.operation == operation)
            .ok_or("Note preview no longer available; preview again")?;
        session.preview = None;
        let (source, bytes) = session.file.as_ref().ok_or("Note selection is not ready")?;
        let connection =
            crate::vocabulary::open(app.state::<crate::vocabulary::DatabasePath>().inner())?;
        let preview = preview_note_bytes(&connection, source.clone(), bytes, &operation, &options)?;
        session.preview = Some(preview.clone());
        Ok(preview)
    })
    .await
    .map_err(|e| format!("Note preview task failed: {e}"))?
}
#[tauri::command]
pub async fn note_import_commit(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
    selections: Vec<NoteImportSelection>,
) -> Result<NoteImportResult, String> {
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
            .filter(|session| session.operation == operation)
            .and_then(|session| session.preview.as_ref())
            .ok_or("Note preview no longer available; preview again")?;
        let mut connection =
            crate::vocabulary::open(app.state::<crate::vocabulary::DatabasePath>().inner())?;
        commit_preview(&mut connection, preview, &selections)
    })
    .await
    .map_err(|e| format!("Note import task failed: {e}"))?
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
        .is_some_and(|session| session.operation == operation)
    {
        *state = None;
    }
    Ok(())
}
#[cfg(test)]
#[path = "note_import_tests.rs"]
mod tests;
