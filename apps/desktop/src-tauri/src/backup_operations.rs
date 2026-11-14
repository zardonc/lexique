//! Native-owned backup file selection and lifetime; the WebView receives only status.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use serde::Serialize;
use tauri::{Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::{
    backup::{BackupPhase, BackupResult, export_backup_to_new_file},
    backup_preview::{
        BackupPreviewPhase, BackupPreviewState, BackupPreviewStatus, VerifiedBackupStage,
        prepare_preview,
    },
};

const PROGRESS_EVENT: &str = "lexique-backup-progress";
const PREVIEW_PROGRESS_EVENT: &str = "lexique-backup-preview-progress";
static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupState {
    Choosing,
    Running,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackupStatus {
    pub operation: String,
    pub state: BackupState,
    pub phase: Option<BackupPhase>,
    pub cancel_requested: bool,
    pub result: Option<BackupResult>,
    pub error: Option<String>,
}

impl BackupStatus {
    fn active(&self) -> bool {
        matches!(self.state, BackupState::Choosing | BackupState::Running)
    }
}

struct Operation {
    status: BackupStatus,
    cancelled: Arc<AtomicBool>,
}

struct PreviewOperation {
    status: BackupPreviewStatus,
    cancelled: Arc<AtomicBool>,
    stage: Option<VerifiedBackupStage>,
}

impl PreviewOperation {
    fn active(&self) -> bool {
        matches!(
            self.status.state,
            BackupPreviewState::Choosing | BackupPreviewState::Running
        )
    }
}

#[derive(Default)]
struct Operations {
    current: Option<Operation>,
    preview: Option<PreviewOperation>,
    in_flight: Option<WorkerLease>,
    closing: bool,
}

struct WorkerLease {
    operation: String,
    holders: usize,
}

impl Operations {
    fn busy(&self) -> bool {
        self.in_flight.is_some()
            || self
                .current
                .as_ref()
                .is_some_and(|operation| operation.status.active())
            || self.preview.as_ref().is_some_and(PreviewOperation::active)
    }

    fn check_start(&self) -> Result<(), String> {
        if self.closing {
            return Err("Application close is already in progress".into());
        }
        if self.busy() {
            return Err("backup_busy: Another backup operation is active".into());
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct BackupOperations(Arc<Mutex<Operations>>);

struct WorkerGuard {
    operations: BackupOperations,
    operation: String,
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        let mut operations = self
            .operations
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(lease) = operations
            .in_flight
            .as_mut()
            .filter(|lease| lease.operation == self.operation)
        {
            lease.holders -= 1;
            if lease.holders == 0 {
                operations.in_flight = None;
            }
        }
    }
}

pub struct CloseGuard {
    operations: BackupOperations,
    committed: bool,
}

impl CloseGuard {
    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for CloseGuard {
    fn drop(&mut self) {
        if !self.committed {
            self.operations
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .closing = false;
        }
    }
}

impl BackupOperations {
    fn reserve(&self) -> Result<(BackupStatus, Arc<AtomicBool>, WorkerGuard), String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        operations.check_start()?;
        let status = BackupStatus {
            operation: format!(
                "backup-{}-{}-{}",
                std::process::id(),
                crate::speech_tasks::now()?,
                NEXT_OPERATION.fetch_add(1, Ordering::Relaxed)
            ),
            state: BackupState::Choosing,
            phase: None,
            cancel_requested: false,
            result: None,
            error: None,
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        operations.current = Some(Operation {
            status: status.clone(),
            cancelled: cancelled.clone(),
        });
        operations.in_flight = Some(WorkerLease {
            operation: status.operation.clone(),
            holders: 1,
        });
        let worker = WorkerGuard {
            operations: self.clone(),
            operation: status.operation.clone(),
        };
        let previous = operations.preview.take();
        drop(operations);
        drop(previous);
        Ok((status, cancelled, worker))
    }

    fn reserve_preview(
        &self,
    ) -> Result<(BackupPreviewStatus, Arc<AtomicBool>, WorkerGuard), String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        operations.check_start()?;
        let status = BackupPreviewStatus {
            operation: format!(
                "backup-preview-{}-{}-{}",
                std::process::id(),
                crate::speech_tasks::now()?,
                NEXT_OPERATION.fetch_add(1, Ordering::Relaxed)
            ),
            state: BackupPreviewState::Choosing,
            phase: None,
            cancel_requested: false,
            preview: None,
            error: None,
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let previous = operations.preview.replace(PreviewOperation {
            status: status.clone(),
            cancelled: cancelled.clone(),
            stage: None,
        });
        operations.current = None;
        operations.in_flight = Some(WorkerLease {
            operation: status.operation.clone(),
            holders: 1,
        });
        let worker = WorkerGuard {
            operations: self.clone(),
            operation: status.operation.clone(),
        };
        drop(operations);
        drop(previous);
        Ok((status, cancelled, worker))
    }

    pub fn preview_status(&self, id: &str) -> Result<BackupPreviewStatus, String> {
        let operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        operations
            .preview
            .as_ref()
            .filter(|operation| operation.status.operation == id)
            .map(|operation| operation.status.clone())
            .ok_or_else(|| "Backup preview is no longer available".into())
    }

    fn preview_phase(
        &self,
        id: &str,
        phase: BackupPreviewPhase,
    ) -> Result<BackupPreviewStatus, String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .preview
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup preview is no longer available")?;
        if operation.active() {
            operation.status.state = BackupPreviewState::Running;
            operation.status.phase = Some(phase);
        }
        Ok(operation.status.clone())
    }

    fn cancel_preview(&self, id: &str) -> Result<BackupPreviewStatus, String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .preview
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup preview is no longer available")?;
        operation.cancelled.store(true, Ordering::Release);
        operation.status = BackupPreviewStatus {
            operation: id.into(),
            state: BackupPreviewState::Cancelled,
            phase: None,
            cancel_requested: true,
            preview: None,
            error: None,
        };
        let stage = operation.stage.take();
        let status = operation.status.clone();
        let cleanup = if stage.is_some() {
            match operations.in_flight.as_mut() {
                Some(lease) => lease.holders += 1,
                None => {
                    operations.in_flight = Some(WorkerLease {
                        operation: id.into(),
                        holders: 1,
                    });
                }
            }
            Some(WorkerGuard {
                operations: self.clone(),
                operation: id.into(),
            })
        } else {
            None
        };
        drop(operations);
        drop(stage);
        drop(cleanup);
        Ok(status)
    }

    fn run_preview(
        &self,
        id: &str,
        work: impl FnOnce() -> Result<VerifiedBackupStage, String>,
    ) -> Result<BackupPreviewStatus, String> {
        let result = run_worker(work);
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .preview
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup preview is no longer available")?;
        if !operation.active() {
            let status = operation.status.clone();
            drop(operations);
            drop(result);
            return Ok(status);
        }
        operation.status.phase = None;
        match result {
            Ok(stage) => {
                operation.status.state = BackupPreviewState::PreviewReady;
                operation.status.preview = Some(stage.preview().clone());
                operation.stage = Some(stage);
            }
            Err(error) if error == "backup_cancelled" => {
                operation.status.state = BackupPreviewState::Cancelled;
            }
            Err(error) => {
                operation.status.state = BackupPreviewState::Failed;
                operation.status.error = Some(if error.trim().is_empty() {
                    "Backup preview failed without an error description".into()
                } else {
                    error
                });
            }
        }
        Ok(operation.status.clone())
    }

    pub fn consume_preview(&self, id: &str) -> Result<(VerifiedBackupStage, CloseGuard), String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        operations.check_start()?;
        let stage = operations
            .preview
            .as_mut()
            .filter(|operation| {
                operation.status.operation == id
                    && operation.status.state == BackupPreviewState::PreviewReady
            })
            .and_then(|operation| operation.stage.take())
            .ok_or("Verified backup preview is no longer available")?;
        operations.preview = None;
        operations.closing = true;
        Ok((
            stage,
            CloseGuard {
                operations: self.clone(),
                committed: false,
            },
        ))
    }

    pub fn status(&self, id: &str) -> Result<BackupStatus, String> {
        let operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        operations
            .current
            .as_ref()
            .filter(|operation| operation.status.operation == id)
            .map(|operation| operation.status.clone())
            .ok_or_else(|| "Backup operation is no longer available".into())
    }

    fn cancel(&self, id: &str) -> Result<BackupStatus, String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .current
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup operation is no longer available")?;
        if operation.status.active() {
            operation.cancelled.store(true, Ordering::Release);
            operation.status.cancel_requested = true;
        }
        Ok(operation.status.clone())
    }

    fn phase(&self, id: &str, phase: BackupPhase) -> Result<BackupStatus, String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .current
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup operation is no longer available")?;
        operation.status.state = BackupState::Running;
        operation.status.phase = Some(phase);
        Ok(operation.status.clone())
    }

    fn run(
        &self,
        id: &str,
        work: impl FnOnce() -> Result<BackupResult, String>,
    ) -> Result<BackupStatus, String> {
        let result = run_worker(work);
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        let operation = operations
            .current
            .as_mut()
            .filter(|operation| operation.status.operation == id)
            .ok_or("Backup operation is no longer available")?;
        operation.status.phase = None;
        match result {
            Ok(result) => {
                operation.status.state = BackupState::Completed;
                operation.status.result = Some(result);
            }
            Err(error)
                if error == "backup_cancelled"
                    || (operation.cancelled.load(Ordering::Acquire)
                        && error.contains("backup_cancelled")) =>
            {
                operation.status.state = BackupState::Cancelled;
            }
            Err(error) => {
                operation.status.state = BackupState::Failed;
                operation.status.error = Some(if error.is_empty() {
                    "Backup failed without an error description".into()
                } else {
                    error
                });
            }
        }
        Ok(operation.status.clone())
    }

    pub fn begin_close(&self) -> Result<CloseGuard, String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        if operations.busy() {
            return Err("backup_busy: Finish or cancel the backup before closing".into());
        }
        if operations.closing {
            return Err("Application close is already in progress".into());
        }
        operations.closing = true;
        let close = CloseGuard {
            operations: self.clone(),
            committed: false,
        };
        let previous = operations.preview.take();
        drop(operations);
        drop(previous);
        Ok(close)
    }
}

fn run_worker<T>(work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(work)).unwrap_or_else(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("unknown worker panic");
        Err(format!("Backup worker stopped unexpectedly: {message}"))
    })
}

fn emit(app: &tauri::AppHandle, status: &BackupStatus) {
    if let Err(error) = app.emit_to("main", PROGRESS_EVENT, status) {
        eprintln!("Could not notify backup progress: {error}");
    }
}

fn emit_preview(app: &tauri::AppHandle, status: &BackupPreviewStatus) {
    if let Err(error) = app.emit_to("main", PREVIEW_PROGRESS_EVENT, status) {
        eprintln!("Could not notify backup preview progress: {error}");
    }
}

fn main_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Backup commands are restricted to the main window".into());
    }
    Ok(())
}

#[tauri::command]
pub fn backup_export_start(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
) -> Result<BackupStatus, String> {
    main_window(&window)?;
    let (status, cancelled, worker) = operations.reserve()?;
    let operations = operations.inner().clone();
    let id = status.operation.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = operations.run(&id, || {
            let window = app
                .get_webview_window("main")
                .ok_or("Main window unavailable")?;
            let destination = app
                .dialog()
                .file()
                .set_parent(&window)
                .add_filter("Lexique backup", &["lexique-backup"])
                .set_file_name(format!(
                    "lexique-{}.lexique-backup",
                    crate::speech_tasks::now()?
                ))
                .blocking_save_file()
                .ok_or("backup_cancelled")?
                .into_path()
                .map_err(|error| format!("Could not resolve backup destination: {error}"))?;
            if cancelled.load(Ordering::Acquire) {
                return Err("backup_cancelled".into());
            }
            export_backup_to_new_file(
                &app.state::<crate::vocabulary::DatabasePath>(),
                &app.state::<crate::speech_tasks::SpeechTasks>(),
                &app.state::<crate::maintenance::MaintenanceCoordinator>(),
                &destination,
                &id,
                &cancelled,
                |phase| match operations.phase(&id, phase) {
                    Ok(status) => emit(&app, &status),
                    Err(error) => eprintln!("Could not record backup progress: {error}"),
                },
            )
        });
        drop(worker);
        match result {
            Ok(status) => emit(&app, &status),
            Err(error) => eprintln!("Could not record backup completion: {error}"),
        }
    });
    Ok(status)
}

#[tauri::command]
pub fn backup_operation_status(
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
    operation: String,
) -> Result<BackupStatus, String> {
    main_window(&window)?;
    operations.status(&operation)
}

#[tauri::command]
pub fn backup_operation_cancel(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
    operation: String,
) -> Result<BackupStatus, String> {
    main_window(&window)?;
    let status = operations.cancel(&operation)?;
    emit(&app, &status);
    Ok(status)
}

#[tauri::command]
pub fn backup_preview_start(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
) -> Result<BackupPreviewStatus, String> {
    main_window(&window)?;
    let (status, cancelled, worker) = operations.reserve_preview()?;
    let operations = operations.inner().clone();
    let id = status.operation.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = operations.run_preview(&id, || {
            let window = app
                .get_webview_window("main")
                .ok_or("Main window unavailable")?;
            let source = app
                .dialog()
                .file()
                .set_parent(&window)
                .add_filter("Lexique backup", &["lexique-backup"])
                .blocking_pick_file()
                .ok_or("backup_cancelled")?
                .into_path()
                .map_err(|error| format!("Could not resolve selected backup: {error}"))?;
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("Could not resolve backup staging directory: {error}"))?;
            prepare_preview(&source, &app_data, &cancelled, |phase| {
                match operations.preview_phase(&id, phase) {
                    Ok(status) => emit_preview(&app, &status),
                    Err(error) => eprintln!("Could not record backup preview progress: {error}"),
                }
            })
        });
        drop(worker);
        match result {
            Ok(status) => emit_preview(&app, &status),
            Err(error) => eprintln!("Could not record backup preview completion: {error}"),
        }
    });
    Ok(status)
}

#[tauri::command]
pub fn backup_preview_status(
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
    operation: String,
) -> Result<BackupPreviewStatus, String> {
    main_window(&window)?;
    operations.preview_status(&operation)
}

#[tauri::command]
pub fn backup_preview_cancel(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operations: State<'_, BackupOperations>,
    operation: String,
) -> Result<BackupPreviewStatus, String> {
    main_window(&window)?;
    let status = operations.cancel_preview(&operation)?;
    emit_preview(&app, &status);
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::{BackupCounts, BackupManifest, OwnedDirectory};
    use std::{fs, io::Write, path::PathBuf};

    fn stage_fixture() -> (OwnedDirectory, PathBuf, VerifiedBackupStage) {
        let directory = OwnedDirectory::create(&std::env::temp_dir()).unwrap();
        let mut connection =
            rusqlite::Connection::open(directory.0.join("source.sqlite3")).unwrap();
        lexique_storage::initialize(&connection).unwrap();
        let snapshot =
            lexique_storage::export_learning_snapshot(&mut connection, 1_794_564_000).unwrap();
        connection.close().unwrap();
        let manifest = BackupManifest {
            kind: "lexique-backup".into(),
            format_version: 1,
            snapshot_version: 1,
            created_at: 1_794_564_000,
            application_version: "0.1.0".into(),
            counts: BackupCounts::from_snapshot(&snapshot),
        };
        let source = directory.0.join("selected.lexique-backup");
        let mut archive = zip::ZipWriter::new(fs::File::create(&source).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .large_file(true)
            .unix_permissions(0o600);
        for (name, bytes) in [
            ("manifest.json", serde_json::to_vec(&manifest).unwrap()),
            ("snapshot.json", serde_json::to_vec(&snapshot).unwrap()),
        ] {
            archive.start_file(name, options).unwrap();
            archive.write_all(&bytes).unwrap();
        }
        archive.finish().unwrap().sync_all().unwrap();
        let app_data = directory.0.join("app-data");
        fs::create_dir(&app_data).unwrap();
        let stage = crate::backup_preview::prepare_preview(
            &source,
            &app_data,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        (directory, source, stage)
    }

    #[test]
    fn ready_preview_survives_source_replacement_and_consumes_once_with_close_reservation() {
        let (directory, source, stage) = stage_fixture();
        let staged_bytes = fs::read(stage.database()).unwrap();
        let root = stage.root().to_path_buf();
        let operations = BackupOperations::default();
        let (status, _, worker) = operations.reserve_preview().unwrap();
        let ready = operations
            .run_preview(&status.operation, || Ok(stage))
            .unwrap();
        assert_eq!(ready.state, BackupPreviewState::PreviewReady);
        assert!(operations.consume_preview(&status.operation).is_err());
        assert!(operations.begin_close().is_err());
        drop(worker);
        fs::write(&source, b"corrupt replacement after preview").unwrap();
        fs::remove_file(&source).unwrap();
        assert!(operations.consume_preview("unrelated-id").is_err());
        let (stage, close) = operations.consume_preview(&status.operation).unwrap();
        assert_eq!(fs::read(stage.database()).unwrap(), staged_bytes);
        assert_eq!(stage.preview().filename, "selected.lexique-backup");
        assert_eq!(stage.manifest().created_at, 1_794_564_000);
        let serialized = serde_json::to_string(&ready).unwrap();
        let native_path = serde_json::to_string(&directory.0.to_string_lossy()).unwrap();
        assert!(!serialized.contains(native_path.trim_matches('"')));
        assert!(operations.consume_preview(&status.operation).is_err());
        assert!(operations.preview_status(&status.operation).is_err());
        assert!(operations.reserve().is_err());
        assert!(operations.reserve_preview().is_err());
        assert!(operations.begin_close().is_err());
        assert!(root.is_dir());
        drop(stage);
        assert!(!root.exists());
        assert!(operations.reserve_preview().is_err());
        drop(close);
        assert!(operations.reserve_preview().is_ok());
    }

    #[test]
    fn consumed_stage_transfers_root_ownership_without_process_exit() {
        let (directory, _, stage) = stage_fixture();
        let root = stage.root().to_path_buf();
        let database = stage.database().to_path_buf();
        let operations = BackupOperations::default();
        let (status, _, worker) = operations.reserve_preview().unwrap();
        operations
            .run_preview(&status.operation, || Ok(stage))
            .unwrap();
        drop(worker);
        let (stage, close) = operations.consume_preview(&status.operation).unwrap();
        let retained = stage.keep();
        assert_eq!(retained, root);
        close.commit();
        assert!(operations.reserve_preview().is_err());
        drop(operations);
        assert!(root.is_dir() && database.is_file());
        let readback = rusqlite::Connection::open_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let integrity: String = readback
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
        readback.close().unwrap();
        drop(directory);
        assert!(!root.exists());
    }

    #[test]
    fn cancelled_preview_retains_status_and_blocks_export_and_close_until_worker_cleanup() {
        let operations = BackupOperations::default();
        let (status, cancelled, worker) = operations.reserve_preview().unwrap();
        assert_eq!(status.state, BackupPreviewState::Choosing);
        assert!(operations.reserve().is_err());
        assert!(operations.reserve_preview().is_err());
        assert!(operations.begin_close().is_err());
        let status = operations.cancel_preview(&status.operation).unwrap();
        assert_eq!(status.state, BackupPreviewState::Cancelled);
        assert!(status.cancel_requested && cancelled.load(Ordering::Acquire));
        assert!(status.phase.is_none() && status.preview.is_none() && status.error.is_none());
        assert_eq!(
            operations.preview_status(&status.operation).unwrap().state,
            BackupPreviewState::Cancelled
        );
        assert!(operations.reserve().is_err());
        assert!(operations.reserve_preview().is_err());
        assert!(operations.begin_close().is_err());
        let status = operations
            .run_preview(&status.operation, || Err("backup_cancelled".into()))
            .unwrap();
        assert_eq!(status.state, BackupPreviewState::Cancelled);
        assert!(operations.reserve().is_err());
        assert!(operations.begin_close().is_err());
        drop(worker);
        let (next, _, next_worker) = operations.reserve_preview().unwrap();
        assert_ne!(next.operation, status.operation);
        assert!(operations.preview_status(&status.operation).is_err());
        drop(next_worker);
    }

    #[test]
    fn preview_progress_and_late_results_cannot_revive_cancelled_or_superseded_ids() {
        let (_directory, _, stage) = stage_fixture();
        let root = stage.root().to_path_buf();
        assert!(root.is_dir());
        let operations = BackupOperations::default();
        let (first, _, worker) = operations.reserve_preview().unwrap();
        let running = operations
            .preview_phase(
                &first.operation,
                crate::backup_preview::BackupPreviewPhase::Staging,
            )
            .unwrap();
        assert_eq!(running.state, BackupPreviewState::Running);
        let cancelled = operations.cancel_preview(&first.operation).unwrap();
        let late_phase = operations
            .preview_phase(
                &first.operation,
                crate::backup_preview::BackupPreviewPhase::Validating,
            )
            .unwrap();
        assert_eq!(late_phase.state, BackupPreviewState::Cancelled);
        assert!(late_phase.cancel_requested && late_phase.phase.is_none());
        let late_ready = operations
            .run_preview(&first.operation, || Ok(stage))
            .unwrap();
        assert_eq!(late_ready.state, BackupPreviewState::Cancelled);
        assert!(late_ready.preview.is_none() && !root.exists());
        assert!(operations.consume_preview(&first.operation).is_err());
        assert!(operations.reserve_preview().is_err());
        drop(worker);
        let (next, _, next_worker) = operations.reserve_preview().unwrap();
        assert!(operations.cancel_preview(&cancelled.operation).is_err());
        assert!(
            operations
                .preview_phase(
                    &first.operation,
                    crate::backup_preview::BackupPreviewPhase::Staging,
                )
                .is_err()
        );
        assert!(
            operations
                .run_preview(&first.operation, || panic!("superseded worker panic"))
                .is_err()
        );
        assert_eq!(
            operations.preview_status(&next.operation).unwrap().state,
            BackupPreviewState::Choosing
        );
        let failed = operations
            .run_preview(&next.operation, || panic!("preview worker panic"))
            .unwrap();
        assert_eq!(failed.state, BackupPreviewState::Failed);
        assert!(failed.phase.is_none() && failed.preview.is_none());
        assert!(failed.error.unwrap().contains("preview worker panic"));
        assert!(operations.begin_close().is_err());
        drop(next_worker);
        assert!(operations.begin_close().is_ok());
    }

    #[test]
    fn ready_staging_is_discarded_by_cancel_reselection_export_and_close() {
        for action in ["cancel", "reselect", "export", "close"] {
            let (_directory, _, stage) = stage_fixture();
            let root = stage.root().to_path_buf();
            let operations = BackupOperations::default();
            let (status, _, worker) = operations.reserve_preview().unwrap();
            operations
                .run_preview(&status.operation, || Ok(stage))
                .unwrap();
            drop(worker);
            match action {
                "cancel" => {
                    let cancelled = operations.cancel_preview(&status.operation).unwrap();
                    assert_eq!(cancelled.state, BackupPreviewState::Cancelled);
                    assert!(cancelled.cancel_requested);
                    assert!(cancelled.phase.is_none() && cancelled.preview.is_none());
                    assert!(cancelled.error.is_none());
                    assert_eq!(
                        operations.preview_status(&status.operation).unwrap().state,
                        BackupPreviewState::Cancelled
                    );
                }
                "reselect" => {
                    let (next, _, next_worker) = operations.reserve_preview().unwrap();
                    assert_ne!(next.operation, status.operation);
                    assert!(operations.preview_status(&status.operation).is_err());
                    drop(next_worker);
                }
                "export" => {
                    let (_, _, export_worker) = operations.reserve().unwrap();
                    assert!(operations.preview_status(&status.operation).is_err());
                    drop(export_worker);
                }
                "close" => {
                    let close = operations.begin_close().unwrap();
                    assert!(operations.preview_status(&status.operation).is_err());
                    assert!(operations.reserve_preview().is_err());
                    drop(close);
                }
                _ => unreachable!(),
            }
            assert!(!root.exists(), "Ready stage survived {action}");
            assert!(operations.consume_preview(&status.operation).is_err());
        }
    }

    #[test]
    fn picker_cancellation_and_real_file_failure_retain_status_until_reselection() {
        let (_directory, source, stage) = stage_fixture();
        let app_data = stage.root().parent().unwrap().to_path_buf();
        drop(stage);
        let operations = BackupOperations::default();
        let (status, _, worker) = operations.reserve_preview().unwrap();
        let cancelled = operations
            .run_preview(&status.operation, || Err("backup_cancelled".into()))
            .unwrap();
        assert_eq!(cancelled.state, BackupPreviewState::Cancelled);
        assert!(!cancelled.cancel_requested);
        assert!(cancelled.phase.is_none() && cancelled.preview.is_none());
        assert!(cancelled.error.is_none());
        assert!(operations.reserve_preview().is_err() && operations.begin_close().is_err());
        drop(worker);
        assert_eq!(
            operations.preview_status(&status.operation).unwrap().state,
            BackupPreviewState::Cancelled
        );
        let (next, cancelled, next_worker) = operations.reserve_preview().unwrap();
        assert!(operations.preview_status(&status.operation).is_err());
        let failed = operations
            .run_preview(&next.operation, || {
                prepare_preview(
                    &source.with_file_name("missing.lexique-backup"),
                    &app_data,
                    &cancelled,
                    |_| {},
                )
            })
            .unwrap();
        assert_eq!(failed.state, BackupPreviewState::Failed);
        assert!(failed.phase.is_none() && failed.preview.is_none());
        assert!(failed.error.unwrap().starts_with("Could not read backup:"));
        assert!(fs::read_dir(&app_data).unwrap().next().is_none());
        assert!(source.is_file());
        assert!(operations.begin_close().is_err());
        drop(next_worker);
        assert_eq!(
            operations.preview_status(&next.operation).unwrap().state,
            BackupPreviewState::Failed
        );
        assert!(operations.begin_close().is_ok());
    }

    #[test]
    fn active_export_blocks_close_and_duplicate_start_until_cleanup() {
        let operations = BackupOperations::default();
        let (status, _, worker) = operations.reserve().unwrap();
        assert!(operations.reserve().is_err());
        assert!(operations.reserve_preview().is_err());
        assert!(operations.begin_close().is_err());
        operations
            .phase(&status.operation, BackupPhase::Packaging)
            .unwrap();
        assert!(operations.begin_close().is_err());
        let status = operations
            .run(&status.operation, || Err("synthetic disk failure".into()))
            .unwrap();
        assert_eq!(status.state, BackupState::Failed);
        assert!(status.phase.is_none());
        assert!(operations.begin_close().is_err());
        drop(worker);
        let close = operations.begin_close().unwrap();
        assert!(operations.reserve().is_err());
        drop(close);
        assert!(operations.reserve().is_ok());
    }

    #[test]
    fn cancellation_and_panics_always_release_the_active_operation() {
        let operations = BackupOperations::default();
        let (status, cancelled, worker) = operations.reserve().unwrap();
        assert!(operations.cancel("unrelated-id").is_err());
        let status = operations.cancel(&status.operation).unwrap();
        assert!(cancelled.load(Ordering::Acquire));
        assert!(status.cancel_requested);
        let status = operations
            .run(&status.operation, || Err("backup_cancelled".into()))
            .unwrap();
        assert_eq!(status.state, BackupState::Cancelled);
        assert!(status.phase.is_none() && status.result.is_none() && status.error.is_none());
        drop(worker);
        let (status, _, worker) = operations.reserve().unwrap();
        let status = operations
            .run(&status.operation, || panic!("synthetic worker panic"))
            .unwrap();
        assert_eq!(status.state, BackupState::Failed);
        assert!(status.error.unwrap().contains("synthetic worker panic"));
        drop(worker);
        assert!(operations.reserve().is_ok());
    }
}
