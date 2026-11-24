use crate::note_enrichment::{NoteEnrichment, enrich_new_note};
use lexique_storage::{
    ImportSource, NoteFormat, NoteImportContent, NoteImportItem, NoteImportResult, VocabularyRecord,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Mutex};
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
    pub grammar_mapping: Option<crate::grammar_import::GrammarImportMapping>,
    #[serde(default)]
    pub format: Option<NoteFormat>,
    #[serde(default)]
    pub record_kind: crate::note_export::ExportRecordKind,
}
#[derive(Clone, Debug, Serialize)]
pub struct NoteImportGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,
    pub content: NoteImportContent,
    pub rows: Vec<u64>,
    pub existing: Option<VocabularyRecord>,
    pub status: &'static str,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<NoteEnrichment>,
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
    #[serde(default)]
    pub allow_unlinked: bool,
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
    preview: Option<ExchangePreview>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ExchangePreview {
    Vocabulary(NoteImportPreview),
    Grammar(crate::grammar_import::GrammarImportPreview),
}
#[derive(Serialize)]
#[serde(untagged)]
pub enum ExchangeResult {
    Vocabulary(NoteImportResult),
    Grammar(lexique_storage::GrammarImportResult),
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
    let mut file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    Ok((source, bytes))
}

fn preview_note_bytes(
    connection: &Connection,
    source: ImportSource,
    bytes: &[u8],
    operation: &str,
    options: &NoteImportOptions,
) -> Result<ExchangePreview, String> {
    if matches!(
        options.record_kind,
        crate::note_export::ExportRecordKind::Grammar
    ) {
        if options.mapping.is_some() {
            return Err("Grammar import requires grammar CSV mapping".into());
        }
        return match source.format {
            NoteFormat::Csv => crate::grammar_import::preview_csv_with_options(
                connection, source, bytes, operation, options,
            ),
            NoteFormat::Markdown => {
                crate::grammar_import::preview_markdown_bytes(connection, source, bytes, operation)
            }
            NoteFormat::Zip => {
                crate::grammar_import::preview_zip_bytes(connection, source, bytes, operation)
            }
        }
        .map(ExchangePreview::Grammar);
    }
    match source.format {
        NoteFormat::Csv => {
            crate::note_import_csv::preview_csv_bytes(connection, source, bytes, operation, options)
        }
        NoteFormat::Markdown => {
            crate::note_markdown::preview_markdown_bytes(connection, source, bytes, operation)
        }
        NoteFormat::Zip => crate::note_zip::preview_zip_bytes(connection, source, bytes, operation),
    }
    .map(ExchangePreview::Vocabulary)
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
pub(crate) fn enrich_preview(
    preview: &mut NoteImportPreview,
    database: &crate::vocabulary::DatabasePath,
    builtin: &Path,
    registry: &crate::dictionaries::Registry,
) {
    for group in &mut preview.groups {
        group.enrichment = if preview.error.is_some() || group.error.is_some() {
            None
        } else if group.existing.is_some() {
            Some(NoteEnrichment::retained())
        } else if group.status == "create" {
            Some(enrich_new_note(
                registry,
                database,
                builtin,
                &group.content.headword,
            ))
        } else {
            None
        };
    }
}
pub(crate) fn enrich_preview_from_app(
    app: &tauri::AppHandle,
    preview: &mut NoteImportPreview,
) -> Result<(), String> {
    let builtin = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("dictionary/dictionary.sqlite3");
    enrich_preview(
        preview,
        &app.state::<crate::vocabulary::DatabasePath>(),
        &builtin,
        &app.state::<crate::dictionaries::Registry>(),
    );
    Ok(())
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
        if decision.replace(selection).is_some() {
            return Err("Duplicate import group decision".into());
        }
    }
    let mut items = Vec::new();
    let mut skipped = 0;
    for (group, decision) in preview.groups.iter().zip(decisions) {
        let selection = decision.ok_or("Decide every import group")?;
        let decision = &selection.action;
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
        if *decision == NoteImportAction::Create
            && group
                .enrichment
                .as_ref()
                .is_some_and(|value| value.status == "error")
            && !selection.allow_unlinked
        {
            return Err(format!(
                "Choose whether to continue without a dictionary link for {} before importing",
                group.content.headword
            ));
        }
        items.push(NoteImportItem {
            content: group.content.clone(),
            existing: group.existing.clone(),
            source: preview.source.clone(),
            snapshot: if *decision == NoteImportAction::Create {
                group
                    .enrichment
                    .as_ref()
                    .and_then(|value| value.snapshot.clone())
            } else {
                None
            },
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
) -> Result<Option<ExchangePreview>, String> {
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
    let cleanup_app = app.clone();
    let cleanup_operation = operation.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let format = options.format.clone().unwrap_or(NoteFormat::Csv);
        let dialog = app.dialog().file().set_parent(&window);
        let selected = match format {
            NoteFormat::Csv => dialog.add_filter("CSV", &["csv"]),
            NoteFormat::Markdown => dialog.add_filter("Markdown", &["md", "markdown"]),
            NoteFormat::Zip => dialog.add_filter("Markdown ZIP", &["zip"]),
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
        let mut preview =
            preview_note_bytes(&connection, source.clone(), &bytes, &operation, &options)?;
        if let ExchangePreview::Vocabulary(preview) = &mut preview {
            enrich_preview_from_app(&app, preview)?;
        }
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
    .map_err(|e| format!("Note preview task failed: {e}"))
    .and_then(|result| result);
    if !matches!(&result, Ok(Some(_))) {
        note_import_cancel(cleanup_app.state(), cleanup_operation)?;
    }
    result
}
#[tauri::command]
pub async fn note_import_remap(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
    options: NoteImportOptions,
) -> Result<ExchangePreview, String> {
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
        let mut preview =
            preview_note_bytes(&connection, source.clone(), bytes, &operation, &options)?;
        if let ExchangePreview::Vocabulary(preview) = &mut preview {
            enrich_preview_from_app(&app, preview)?;
        }
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
) -> Result<ExchangeResult, String> {
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
        match preview {
            ExchangePreview::Vocabulary(preview) => {
                commit_preview(&mut connection, preview, &selections)
                    .map(ExchangeResult::Vocabulary)
            }
            ExchangePreview::Grammar(preview) => {
                crate::grammar_import::commit_preview(&mut connection, preview, &selections)
                    .map(ExchangeResult::Grammar)
            }
        }
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

#[cfg(test)]
#[path = "note_enrichment_tests.rs"]
mod enrichment_tests;
