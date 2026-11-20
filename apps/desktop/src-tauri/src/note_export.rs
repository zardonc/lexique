use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use lexique_storage::{NoteExportScope, NoteExportSnapshot, capture_note_export};
use serde::Serialize;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::vocabulary::{DatabasePath, open};

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
#[path = "note_export_tests.rs"]
mod tests;

#[derive(Default)]
pub struct NoteExports(Mutex<Option<(String, Arc<NoteExportSnapshot>)>>);

#[derive(Serialize)]
pub struct ExportSummary {
    token: String,
    records: usize,
    rows: usize,
}

#[derive(Serialize)]
pub struct ExportResult {
    state: &'static str,
    records: usize,
    rows: usize,
}

pub fn write_note_csv(snapshot: &NoteExportSnapshot, destination: &Path) -> Result<(), String> {
    if snapshot.records.is_empty() {
        return Err("No saved vocabulary in this export scope".into());
    }
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(vec![0xef, 0xbb, 0xbf]);
    writer
        .write_record(["headword", "notes", "example_fr", "example_zh"])
        .map_err(|error| format!("Could not serialize CSV header: {error}"))?;
    for record in &snapshot.records {
        if record.examples.is_empty() {
            writer
                .write_record([record.headword.as_str(), record.notes.as_str(), "", ""])
                .map_err(|error| format!("Could not serialize CSV record: {error}"))?;
        } else {
            for example in &record.examples {
                writer
                    .write_record([&record.headword, &record.notes, &example.fr, &example.zh])
                    .map_err(|error| format!("Could not serialize CSV example: {error}"))?;
            }
        }
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("Could not finish CSV: {error}"))?;
    std::fs::write(destination, bytes).map_err(|error| format!("Could not write CSV file: {error}"))
}

#[tauri::command]
pub async fn note_export_snapshot(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    path: State<'_, DatabasePath>,
    maintenance: State<'_, crate::maintenance::MaintenanceCoordinator>,
    scope: NoteExportScope,
) -> Result<ExportSummary, String> {
    crate::backup_operations::main_window(&window)?;
    let reader = maintenance.admit_reader()?;
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _reader = reader;
        let snapshot =
            capture_note_export(&mut open(&path)?, &scope).map_err(|error| error.to_string())?;
        let token = format!(
            "note-export-{}",
            NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed)
        );
        let summary = ExportSummary {
            token: token.clone(),
            records: snapshot.records.len(),
            rows: snapshot.row_count(),
        };
        *app.state::<NoteExports>()
            .0
            .lock()
            .map_err(|_| "Note export snapshot unavailable")? = Some((token, Arc::new(snapshot)));
        Ok(summary)
    })
    .await
    .map_err(|error| format!("Note export snapshot task failed: {error}"))?
}

#[tauri::command]
pub async fn note_export_csv(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    exports: State<'_, NoteExports>,
    token: String,
) -> Result<ExportResult, String> {
    crate::backup_operations::main_window(&window)?;
    let snapshot = exports
        .0
        .lock()
        .map_err(|_| "Note export snapshot unavailable")?
        .as_ref()
        .filter(|(identity, _)| *identity == token)
        .map(|(_, snapshot)| snapshot.clone())
        .ok_or("Note export snapshot changed; select the scope again")?;
    if snapshot.records.is_empty() {
        return Err("No saved vocabulary in this export scope".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let window = app
            .get_webview_window("main")
            .ok_or("Main window unavailable")?;
        let destination = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("CSV", &["csv"])
            .set_file_name("lexique-notes.csv")
            .blocking_save_file();
        let state = if let Some(destination) = destination {
            let path = destination
                .into_path()
                .map_err(|error| format!("Could not resolve CSV destination: {error}"))?;
            write_note_csv(&snapshot, &path)?;
            "saved"
        } else {
            "cancelled"
        };
        Ok(ExportResult {
            state,
            records: snapshot.records.len(),
            rows: snapshot.row_count(),
        })
    })
    .await
    .map_err(|error| format!("CSV export task failed: {error}"))?
}
