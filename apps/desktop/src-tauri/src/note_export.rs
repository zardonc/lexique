use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use lexique_storage::{
    GrammarExportSnapshot, NoteExportScope, NoteExportSnapshot, capture_grammar_export,
    capture_note_export,
};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::vocabulary::{DatabasePath, open};

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
#[path = "note_export_tests.rs"]
mod tests;

#[derive(Default)]
pub struct NoteExports(Mutex<Option<(String, Arc<ExportSnapshot>)>>);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportRecordKind {
    #[default]
    Vocabulary,
    Grammar,
}

enum ExportSnapshot {
    Vocabulary(Arc<NoteExportSnapshot>),
    Grammar(GrammarExportSnapshot),
}

impl ExportSnapshot {
    fn save_markdown(&self, destination: Option<&Path>) -> Result<ExportResult, String> {
        let (records, rows) = self.counts();
        if records != 1 {
            return Err("Single Markdown export requires one saved record".into());
        }
        let state = if let Some(path) = destination {
            match self {
                Self::Vocabulary(snapshot) => write_note_markdown(snapshot, path)?,
                Self::Grammar(snapshot) => {
                    std::fs::write(
                        path,
                        crate::note_markdown::format_grammar_markdown(&snapshot.records[0]),
                    )
                    .map_err(|error| {
                        format!("Could not write Markdown file {}: {error}", path.display())
                    })?;
                }
            }
            "saved"
        } else {
            "cancelled"
        };
        Ok(ExportResult {
            state,
            records,
            rows,
        })
    }

    fn counts(&self) -> (usize, usize) {
        match self {
            Self::Vocabulary(snapshot) => (snapshot.records.len(), snapshot.row_count()),
            Self::Grammar(snapshot) => (snapshot.records.len(), snapshot.records.len()),
        }
    }

    fn save_csv(&self, destination: Option<&Path>) -> Result<ExportResult, String> {
        let (records, rows) = self.counts();
        if records == 0 {
            return Err("No saved records in this export scope".into());
        }
        let state = if let Some(path) = destination {
            match self {
                Self::Vocabulary(snapshot) => write_note_csv(snapshot, path)?,
                Self::Grammar(snapshot) => write_grammar_csv(snapshot, path)?,
            }
            "saved"
        } else {
            "cancelled"
        };
        Ok(ExportResult {
            state,
            records,
            rows,
        })
    }
}

impl NoteExports {
    pub(crate) fn retained(&self, token: &str) -> Result<Arc<NoteExportSnapshot>, String> {
        match self.retained_snapshot(token)?.as_ref() {
            ExportSnapshot::Vocabulary(snapshot) => Ok(snapshot.clone()),
            ExportSnapshot::Grammar(_) => Err("Grammar export supports CSV".into()),
        }
    }

    fn retained_snapshot(&self, token: &str) -> Result<Arc<ExportSnapshot>, String> {
        self.0
            .lock()
            .map_err(|_| "Note export snapshot unavailable")?
            .as_ref()
            .filter(|(identity, _)| identity == token)
            .map(|(_, snapshot)| snapshot.clone())
            .ok_or_else(|| "Note export snapshot changed; select the scope again".into())
    }
}

pub fn write_grammar_csv(
    snapshot: &GrammarExportSnapshot,
    destination: &Path,
) -> Result<(), String> {
    if snapshot.records.is_empty() {
        return Err("No saved grammar records in this export scope".into());
    }
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(vec![0xef, 0xbb, 0xbf]);
    writer
        .write_record(["exchange_id", "title", "body", "question", "answer"])
        .map_err(|error| format!("Could not serialize CSV header: {error}"))?;
    for record in &snapshot.records {
        writer
            .write_record([
                &record.exchange_id,
                &record.title,
                &record.body,
                &record.question,
                &record.answer,
            ])
            .map_err(|error| format!("Could not serialize grammar CSV record: {error}"))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("Could not finish CSV: {error}"))?;
    std::fs::write(destination, bytes).map_err(|error| format!("Could not write CSV file: {error}"))
}

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

pub fn write_note_markdown(
    snapshot: &NoteExportSnapshot,
    destination: &Path,
) -> Result<(), String> {
    let [record] = snapshot.records.as_slice() else {
        return Err("Single Markdown export requires one saved vocabulary record".into());
    };
    std::fs::write(
        destination,
        crate::note_markdown::format_note_markdown(record),
    )
    .map_err(|error| {
        format!(
            "Could not write Markdown file {}: {error}",
            destination.display()
        )
    })
}

pub fn write_note_zip(snapshot: &NoteExportSnapshot, destination: &Path) -> Result<(), String> {
    use std::io::Write;
    if snapshot.records.is_empty() {
        return Err("No saved vocabulary in this export scope".into());
    }
    let file = std::fs::File::create(destination)
        .map_err(|error| format!("Could not create ZIP {}: {error}", destination.display()))?;
    let mut archive = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for record in &snapshot.records {
        let name = crate::note_markdown::note_markdown_filename(record);
        archive
            .start_file(&name, options)
            .map_err(|error| format!("Could not write ZIP entry {name}: {error}"))?;
        archive
            .write_all(crate::note_markdown::format_note_markdown(record).as_bytes())
            .map_err(|error| format!("Could not write ZIP entry {name}: {error}"))?;
    }
    archive
        .finish()
        .and_then(|file| {
            file.sync_all()?;
            Ok(file)
        })
        .map_err(|error| format!("Could not finish ZIP {}: {error}", destination.display()))?;
    Ok(())
}

pub fn write_grammar_zip(
    snapshot: &GrammarExportSnapshot,
    destination: &Path,
) -> Result<(), String> {
    use std::io::Write;
    if snapshot.records.is_empty() {
        return Err("No saved grammar records in this export scope".into());
    }
    let file = std::fs::File::create(destination)
        .map_err(|error| format!("Could not create ZIP {}: {error}", destination.display()))?;
    let mut archive = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for record in &snapshot.records {
        let name = crate::note_markdown::grammar_markdown_filename(record);
        archive
            .start_file(&name, options)
            .map_err(|error| format!("Could not write ZIP entry {name}: {error}"))?;
        archive
            .write_all(crate::note_markdown::format_grammar_markdown(record).as_bytes())
            .map_err(|error| format!("Could not write ZIP entry {name}: {error}"))?;
    }
    archive
        .finish()
        .and_then(|file| {
            file.sync_all()?;
            Ok(file)
        })
        .map_err(|error| format!("Could not finish ZIP {}: {error}", destination.display()))?;
    Ok(())
}

#[tauri::command]
pub async fn note_export_snapshot(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    path: State<'_, DatabasePath>,
    maintenance: State<'_, crate::maintenance::MaintenanceCoordinator>,
    scope: NoteExportScope,
    record_kind: Option<ExportRecordKind>,
) -> Result<ExportSummary, String> {
    crate::backup_operations::main_window(&window)?;
    let reader = maintenance.admit_reader()?;
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _reader = reader;
        let mut connection = open(&path)?;
        let snapshot = match record_kind.unwrap_or_default() {
            ExportRecordKind::Vocabulary => ExportSnapshot::Vocabulary(Arc::new(
                capture_note_export(&mut connection, &scope).map_err(|error| error.to_string())?,
            )),
            ExportRecordKind::Grammar => ExportSnapshot::Grammar(
                capture_grammar_export(&mut connection, &scope)
                    .map_err(|error| error.to_string())?,
            ),
        };
        let token = format!(
            "note-export-{}",
            NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed)
        );
        let (records, rows) = snapshot.counts();
        let summary = ExportSummary {
            token: token.clone(),
            records,
            rows,
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
    let snapshot = exports.retained_snapshot(&token)?;
    if snapshot.counts().0 == 0 {
        return Err("No saved records in this export scope".into());
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
            .set_file_name(match snapshot.as_ref() {
                ExportSnapshot::Grammar(_) => "lexique-grammar.csv",
                _ => "lexique-notes.csv",
            })
            .blocking_save_file();
        let path = destination
            .map(|destination| {
                destination
                    .into_path()
                    .map_err(|error| format!("Could not resolve CSV destination: {error}"))
            })
            .transpose()?;
        snapshot.save_csv(path.as_deref())
    })
    .await
    .map_err(|error| format!("CSV export task failed: {error}"))?
}

#[tauri::command]
pub async fn note_export_markdown(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    exports: State<'_, NoteExports>,
    token: Option<String>,
) -> Result<ExportResult, String> {
    crate::backup_operations::main_window(&window)?;
    let snapshot = token
        .as_deref()
        .map(|token| exports.retained_snapshot(token))
        .transpose()?;
    let (filename, rows) = if let Some(snapshot) = &snapshot {
        if snapshot.counts().0 != 1 {
            return Err("Single Markdown export requires one saved record".into());
        }
        let filename = match snapshot.as_ref() {
            ExportSnapshot::Vocabulary(snapshot) => {
                crate::note_markdown::note_markdown_filename(&snapshot.records[0])
            }
            ExportSnapshot::Grammar(_) => "lexique-grammar.md".into(),
        };
        (filename, snapshot.counts().1)
    } else {
        ("lexique-note-v1.md".into(), 1)
    };
    tauri::async_runtime::spawn_blocking(move || {
        let destination = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("Markdown", &["md"])
            .set_file_name(&filename)
            .blocking_save_file();
        let path = destination
            .map(|destination| {
                destination
                    .into_path()
                    .map_err(|error| format!("Could not resolve Markdown destination: {error}"))
            })
            .transpose()?;
        if let Some(snapshot) = &snapshot {
            return snapshot.save_markdown(path.as_deref());
        }
        let state = if let Some(path) = path {
            std::fs::write(
                &path,
                include_str!("../../../../docs/examples/lexique-note-v1.md"),
            )
            .map_err(|error| {
                format!("Could not write Markdown file {}: {error}", path.display())
            })?;
            "saved"
        } else {
            "cancelled"
        };
        Ok(ExportResult {
            state,
            records: 1,
            rows,
        })
    })
    .await
    .map_err(|error| format!("Markdown export task failed: {error}"))?
}

#[tauri::command]
pub async fn note_export_zip(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    exports: State<'_, NoteExports>,
    token: String,
) -> Result<ExportResult, String> {
    crate::backup_operations::main_window(&window)?;
    let snapshot = exports.retained_snapshot(&token)?;
    if snapshot.counts().0 == 0 {
        return Err("No saved records in this export scope".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let destination = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("Markdown ZIP", &["zip"])
            .set_file_name(match snapshot.as_ref() {
                ExportSnapshot::Grammar(_) => "lexique-grammar.zip",
                _ => "lexique-notes.zip",
            })
            .blocking_save_file();
        let state = if let Some(destination) = destination {
            let path = destination
                .into_path()
                .map_err(|error| format!("Could not resolve ZIP destination: {error}"))?;
            match snapshot.as_ref() {
                ExportSnapshot::Grammar(snapshot) => write_grammar_zip(snapshot, &path)?,
                ExportSnapshot::Vocabulary(snapshot) => write_note_zip(snapshot, &path)?,
            };
            "saved"
        } else {
            "cancelled"
        };
        Ok(ExportResult {
            state,
            records: snapshot.counts().0,
            rows: snapshot.counts().1,
        })
    })
    .await
    .map_err(|error| format!("Markdown ZIP export task failed: {error}"))?
}
