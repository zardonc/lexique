//! Single-use verified preview ownership and controlled restore restart.
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Emitter, Manager, State};

use crate::{
    backup_operations::{BackupOperations, CloseGuard},
    backup_preview::{BackupPreview, VerifiedBackupStage},
    restore::{RetainedCopy, StartupPhase, StartupStatus},
};

const EVENT: &str = "lexique-backup-restore-progress";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreState {
    Running,
    RestartRequested,
    Failed,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestorePhase {
    Preserving,
    Preparing,
    Closing,
}

#[derive(Clone, Debug, Serialize)]
pub struct RestoreStatus {
    pub operation: String,
    pub state: RestoreState,
    pub phase: Option<RestorePhase>,
    pub preview: BackupPreview,
    pub retained_copies: Vec<RetainedCopy>,
    pub error: Option<String>,
}

#[derive(Clone, Default)]
pub struct RestoreOperations(Arc<Mutex<Option<RestoreStatus>>>);

type RestoreStart = (RestoreStatus, Option<(VerifiedBackupStage, CloseGuard)>);

impl RestoreOperations {
    pub(crate) fn begin(
        &self,
        backups: &BackupOperations,
        id: &str,
        learning_available: bool,
    ) -> Result<RestoreStart, String> {
        let mut current = self.0.lock().map_err(|_| "Restore status unavailable")?;
        if let Some(status) = current.as_ref() {
            if status.operation == id {
                return Ok((status.clone(), None));
            }
            if status.state != RestoreState::Failed {
                return Err("backup_busy: Another restore is active".into());
            }
        }
        if !learning_available {
            return Err("Learning startup must finish before normal restore".into());
        }
        let (stage, close) = backups.consume_preview(id)?;
        let status = RestoreStatus {
            operation: id.into(),
            state: RestoreState::Running,
            phase: Some(RestorePhase::Preserving),
            preview: stage.preview().clone(),
            retained_copies: Vec::new(),
            error: None,
        };
        *current = Some(status.clone());
        Ok((status, Some((stage, close))))
    }

    fn change(
        &self,
        id: &str,
        update: impl FnOnce(&mut RestoreStatus),
    ) -> Result<RestoreStatus, String> {
        let mut current = self.0.lock().map_err(|_| "Restore status unavailable")?;
        let status = current
            .as_mut()
            .filter(|status| status.operation == id)
            .ok_or("Restore identity is no longer available")?;
        update(status);
        Ok(status.clone())
    }

    pub(crate) fn fail(
        &self,
        id: &str,
        error: &str,
        copies: &[RetainedCopy],
        close: CloseGuard,
    ) -> Result<RestoreStatus, String> {
        let status = self.change(id, |status| {
            status.state = RestoreState::Failed;
            status.phase = None;
            status.error = Some(error.into());
            status.retained_copies = copies.to_vec();
        });
        drop(close);
        status
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> bool {
        self.0
            .lock()
            .map(|current| {
                current
                    .as_ref()
                    .is_some_and(|status| status.state != RestoreState::Failed)
            })
            .unwrap_or(true)
    }

    fn status(&self, id: &str) -> Result<RestoreStatus, String> {
        self.0
            .lock()
            .map_err(|_| "Restore status unavailable")?
            .as_ref()
            .filter(|status| status.operation == id)
            .cloned()
            .ok_or_else(|| "Restore identity is no longer available".into())
    }
}

fn emit(app: &tauri::AppHandle, status: &RestoreStatus) {
    if let Err(error) = app.emit_to("main", EVENT, status) {
        eprintln!("Could not notify restore progress: {error}");
    }
}

#[tauri::command]
pub fn backup_restore_start(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operations: State<'_, RestoreOperations>,
    backups: State<'_, BackupOperations>,
    operation: String,
) -> Result<RestoreStatus, String> {
    crate::backup_operations::main_window(&window)?;
    let (status, work) = operations.begin(
        &backups,
        &operation,
        crate::startup::learning_available(&app),
    )?;
    let Some((stage, close)) = work else {
        return Ok(status);
    };
    let operations = operations.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let database = app.state::<crate::vocabulary::DatabasePath>();
        let startup = app.state::<crate::startup::StartupState>();
        let result = crate::backup_operations::run_worker(|| {
            crate::restore::prepare_normal(
                &database,
                &app.state::<crate::speech_tasks::SpeechTasks>(),
                &app.state::<crate::maintenance::MaintenanceCoordinator>(),
                stage,
                &operation,
                || {
                    startup.closed_for_restore()?;
                    crate::selection::selection_cancel(app.clone())?;
                    app.state::<crate::speech_tasks::SpeechTasks>().shutdown()?;
                    if let Some(validator) =
                        app.try_state::<Arc<crate::review_window::WindowValidator>>()
                    {
                        validator.stop();
                    }
                    Ok(())
                },
                |point| {
                    let phase = match point {
                        "preserving" => Some(RestorePhase::Preserving),
                        "preparing" => Some(RestorePhase::Preparing),
                        "closing" => Some(RestorePhase::Closing),
                        _ => return Ok(()),
                    };
                    let status = operations.change(&operation, |status| status.phase = phase)?;
                    emit(&app, &status);
                    Ok(())
                },
            )
        });
        match result {
            Ok(prepared) => {
                close.commit();
                let status = operations.change(&operation, |status| {
                    status.state = RestoreState::RestartRequested;
                    status.phase = None;
                    status.retained_copies = prepared.retained_copies.clone();
                });
                if let Ok(status) = &status {
                    emit(&app, status);
                }
                let handoff = StartupStatus {
                    state: StartupPhase::RecoveryRequired,
                    operation: Some(operation.clone()),
                    restored: None,
                    retained_copies: prepared.retained_copies,
                    error: Some(
                        "A controlled restart was requested to finish restoring the backup".into(),
                    ),
                };
                if let Err(error) = startup.restore_handoff(handoff.clone()) {
                    eprintln!("{error}");
                }
                crate::startup::emit(&app, &handoff);
                if let Err(error) = crate::startup::request_restart(&app) {
                    eprintln!("Could not request restore restart: {error}");
                }
            }
            Err(mut error) => {
                let copies = match crate::restore::retained_copies(&database) {
                    Ok(copies) => copies,
                    Err(retained_error) => {
                        error = format!("{error}; retained copies unavailable: {retained_error}");
                        Vec::new()
                    }
                };
                let closed = startup.requires_restart().unwrap_or(true);
                let status = operations.fail(&operation, &error, &copies, close);
                if let Ok(status) = &status {
                    emit(&app, status);
                }
                if closed {
                    if let Err(suspend_error) = app
                        .state::<crate::maintenance::MaintenanceCoordinator>()
                        .suspend_startup()
                    {
                        error = format!("{error}; learning closure: {suspend_error}");
                    }
                    let handoff = StartupStatus {
                        state: StartupPhase::RecoveryRequired,
                        operation: Some(operation),
                        restored: None,
                        retained_copies: copies,
                        error: Some(error),
                    };
                    if let Err(error) = startup.restore_handoff(handoff.clone()) {
                        eprintln!("{error}");
                    }
                    crate::startup::emit(&app, &handoff);
                }
            }
        }
    });
    Ok(status)
}

#[tauri::command]
pub fn backup_restore_status(
    window: tauri::WebviewWindow,
    operations: State<'_, RestoreOperations>,
    operation: String,
) -> Result<RestoreStatus, String> {
    crate::backup_operations::main_window(&window)?;
    operations.status(&operation)
}

#[cfg(windows)]
fn open_directory(path: &std::path::Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("explorer.exe")
        .arg(path)
        .creation_flags(0x0800_0000)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open retained directory: {error}"))
}

#[cfg(target_os = "macos")]
fn open_directory(path: &std::path::Path) -> Result<(), String> {
    let status = std::process::Command::new("open")
        .arg(path)
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Retained directory launcher exited with {status}"))
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn open_directory(_path: &std::path::Path) -> Result<(), String> {
    Err("Opening retained directories is unavailable on this platform".into())
}

#[tauri::command]
pub async fn backup_retained_directory_open(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    copy: String,
) -> Result<(), String> {
    crate::backup_operations::main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        open_directory(&crate::restore::retained_directory(
            &app.state::<crate::vocabulary::DatabasePath>(),
            &copy,
        )?)
    })
    .await
    .map_err(|error| format!("Retained directory task failed: {error}"))?
}
