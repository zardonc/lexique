//! Native-owned file selection and export lifetime; the WebView receives only status.
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

use crate::backup::{BackupPhase, BackupResult, export_backup_to_new_file};

const PROGRESS_EVENT: &str = "lexique-backup-progress";
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

#[derive(Default)]
struct Operations {
    current: Option<Operation>,
    closing: bool,
}

#[derive(Clone, Default)]
pub struct BackupOperations(Arc<Mutex<Operations>>);

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
    fn reserve(&self) -> Result<(BackupStatus, Arc<AtomicBool>), String> {
        let mut operations = self.0.lock().map_err(|_| "Backup status unavailable")?;
        if operations.closing {
            return Err("Application close is already in progress".into());
        }
        if operations
            .current
            .as_ref()
            .is_some_and(|operation| operation.status.active())
        {
            return Err("backup_busy: Another backup operation is active".into());
        }
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
        Ok((status, cancelled))
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
        let result = catch_unwind(AssertUnwindSafe(work)).unwrap_or_else(|panic| {
            let message = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("unknown worker panic");
            Err(format!("Backup worker stopped unexpectedly: {message}"))
        });
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
        if operations
            .current
            .as_ref()
            .is_some_and(|operation| operation.status.active())
        {
            return Err("backup_busy: Finish or cancel the backup before closing".into());
        }
        if operations.closing {
            return Err("Application close is already in progress".into());
        }
        operations.closing = true;
        Ok(CloseGuard {
            operations: self.clone(),
            committed: false,
        })
    }
}

fn emit(app: &tauri::AppHandle, status: &BackupStatus) {
    if let Err(error) = app.emit_to("main", PROGRESS_EVENT, status) {
        eprintln!("Could not notify backup progress: {error}");
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
    let (status, cancelled) = operations.reserve()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_export_blocks_close_and_duplicate_start_until_cleanup() {
        let operations = BackupOperations::default();
        let (status, _) = operations.reserve().unwrap();
        assert!(operations.reserve().is_err());
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
        let close = operations.begin_close().unwrap();
        assert!(operations.reserve().is_err());
        drop(close);
        assert!(operations.reserve().is_ok());
    }

    #[test]
    fn cancellation_and_panics_always_release_the_active_operation() {
        let operations = BackupOperations::default();
        let (status, cancelled) = operations.reserve().unwrap();
        assert!(operations.cancel("unrelated-id").is_err());
        let status = operations.cancel(&status.operation).unwrap();
        assert!(cancelled.load(Ordering::Acquire));
        assert!(status.cancel_requested);
        let status = operations
            .run(&status.operation, || Err("backup_cancelled".into()))
            .unwrap();
        assert_eq!(status.state, BackupState::Cancelled);
        assert!(status.phase.is_none() && status.result.is_none() && status.error.is_none());
        let (status, _) = operations.reserve().unwrap();
        let status = operations
            .run(&status.operation, || panic!("synthetic worker panic"))
            .unwrap();
        assert_eq!(status.state, BackupState::Failed);
        assert!(status.error.unwrap().contains("synthetic worker panic"));
        assert!(operations.reserve().is_ok());
    }
}
