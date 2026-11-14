//! Startup recovery precedes every learning connection and worker.
use std::sync::Mutex;

use tauri::{Emitter, Manager, State};

use crate::{
    restore::{StartupPhase, StartupStatus},
    vocabulary::DatabasePath,
};

const EVENT: &str = "lexique-startup-status";

pub(crate) fn load(
    database: &DatabasePath,
    observe: impl FnMut(&str) -> Result<(), String>,
    initialize: impl FnOnce() -> Result<(), String>,
) -> StartupStatus {
    let mut status = crate::restore::recover_startup(database, observe);
    if status.state == StartupPhase::Ready
        && let Err(error) = initialize()
    {
        status.state = StartupPhase::RecoveryRequired;
        status.error = Some(format!("Learning initialization failed: {error}"));
    }
    status
}

struct Runtime {
    status: StartupStatus,
    initialized: bool,
    needs_restart: bool,
}

pub(crate) struct StartupState(Mutex<Runtime>);

impl StartupState {
    pub(crate) fn new(status: StartupStatus) -> Self {
        let initialized = status.state == StartupPhase::Ready;
        Self(Mutex::new(Runtime {
            status,
            initialized,
            needs_restart: false,
        }))
    }

    pub(crate) fn status(&self) -> Result<StartupStatus, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Startup status unavailable")?
            .status
            .clone())
    }

    pub(crate) fn requires_restart(&self) -> Result<bool, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Startup status unavailable")?
            .needs_restart)
    }

    pub(crate) fn closed_for_restore(&self) -> Result<(), String> {
        let mut runtime = self.0.lock().map_err(|_| "Startup status unavailable")?;
        runtime.initialized = false;
        runtime.needs_restart = true;
        Ok(())
    }

    pub(crate) fn restore_handoff(&self, status: StartupStatus) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "Startup status unavailable")?
            .status = status;
        Ok(())
    }
}

pub(crate) fn learning_available(app: &tauri::AppHandle) -> bool {
    app.try_state::<StartupState>().is_some_and(|state| {
        state
            .0
            .lock()
            .map(|runtime| runtime.initialized && runtime.status.state == StartupPhase::Ready)
            .unwrap_or(false)
    })
}

pub(crate) fn emit(app: &tauri::AppHandle, status: &StartupStatus) {
    if let Err(error) = app.emit_to("main", EVENT, status) {
        eprintln!("Could not notify startup status: {error}");
    }
}

pub(crate) fn request_restart(app: &tauri::AppHandle) -> Result<(), String> {
    app.state::<crate::vocabulary::CloseAuthorization>()
        .finish(|| {
            app.request_restart();
            Ok(())
        })
}

#[tauri::command]
pub fn startup_status(
    window: tauri::WebviewWindow,
    state: State<'_, StartupState>,
) -> Result<StartupStatus, String> {
    crate::backup_operations::main_window(&window)?;
    state.status()
}

#[tauri::command]
pub async fn startup_retry(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<StartupStatus, String> {
    crate::backup_operations::main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<StartupState>();
        let mut runtime = state.0.lock().map_err(|_| "Startup recovery unavailable")?;
        if runtime.needs_restart {
            let status = runtime.status.clone();
            drop(runtime);
            request_restart(&app)?;
            return Ok(status);
        }
        if !runtime.initialized {
            let database = app.state::<DatabasePath>();
            runtime.status = load(
                &database,
                |_| Ok(()),
                || crate::initialize_learning(&app, &database),
            );
            runtime.initialized = runtime.status.state == StartupPhase::Ready;
        }
        let status = runtime.status.clone();
        drop(runtime);
        emit(&app, &status);
        Ok(status)
    })
    .await
    .map_err(|error| format!("Startup recovery task failed: {error}"))?
}

#[tauri::command]
pub async fn backup_restore_cleanup_begin(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
) -> Result<StartupStatus, String> {
    crate::backup_operations::main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<StartupState>();
        let runtime = state.0.lock().map_err(|_| "Startup cleanup unavailable")?;
        if runtime.initialized || runtime.needs_restart {
            return Err("Startup cleanup is unavailable in the old runtime".into());
        }
        crate::restore::cleanup_begin(&app.state::<DatabasePath>(), &operation, |_| Ok(()))?;
        Ok(runtime.status.clone())
    })
    .await
    .map_err(|error| format!("Startup cleanup task failed: {error}"))?
}

#[tauri::command]
pub async fn backup_restore_cleanup_complete(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    operation: String,
) -> Result<StartupStatus, String> {
    crate::backup_operations::main_window(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<StartupState>();
        let mut runtime = state.0.lock().map_err(|_| "Startup cleanup unavailable")?;
        if runtime.needs_restart {
            return Err("Startup cleanup is unavailable in the old runtime".into());
        }
        let database = app.state::<DatabasePath>();
        let mut status = match crate::restore::cleanup_complete(&database, &operation, |_| Ok(())) {
            Ok(status) => status,
            Err(error) => {
                runtime.status.state = StartupPhase::RecoveryRequired;
                runtime.status.error = Some(error.clone());
                emit(&app, &runtime.status);
                return Err(error);
            }
        };
        if !runtime.initialized
            && let Err(error) = crate::initialize_learning(&app, &database)
        {
            status.state = StartupPhase::RecoveryRequired;
            status.error = Some(format!("Restored learning initialization failed: {error}"));
        }
        runtime.initialized = status.state == StartupPhase::Ready;
        runtime.status = status.clone();
        drop(runtime);
        emit(&app, &status);
        Ok(status)
    })
    .await
    .map_err(|error| format!("Startup cleanup task failed: {error}"))?
}
