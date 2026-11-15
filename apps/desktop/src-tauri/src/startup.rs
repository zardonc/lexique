//! Startup recovery precedes every learning connection and worker.
use std::sync::Mutex;

use tauri::{Emitter, Manager, State};

use crate::{
    restore::{StartupPhase, StartupStatus},
    vocabulary::DatabasePath,
};

const EVENT: &str = "lexique-startup-status";

pub(crate) fn initialize(app: &tauri::AppHandle) -> StartupStatus {
    let result: Result<StartupStatus, String> = (|| {
        app.state::<crate::maintenance::MaintenanceCoordinator>()
            .suspend_startup()?;
        #[cfg(target_os = "macos")]
        crate::vocabulary::coordinate_quit(app)
            .map_err(|error| format!("Could not configure application quit: {error}"))?;
        let database = if let Some(path) = app.try_state::<DatabasePath>() {
            DatabasePath(path.0.clone())
        } else {
            let directory = app.path().app_data_dir().map_err(|error| {
                format!("Could not resolve the intended learning directory: {error}")
            })?;
            let database = DatabasePath(directory.join(crate::DATABASE_FILENAME));
            app.manage(DatabasePath(database.0.clone()));
            database
        };
        #[cfg(windows)]
        crate::webview_diagnostics::attach(app, database.0.clone())
            .map_err(|error| format!("Could not attach native diagnostics: {error}"))?;
        Ok(load(
            &database,
            |_| Ok(()),
            || crate::initialize_learning(app, &database),
        ))
    })();
    result.unwrap_or_else(unavailable)
}

pub(crate) fn database(app: &tauri::AppHandle) -> Result<DatabasePath, String> {
    app.try_state::<DatabasePath>().map(|path| DatabasePath(path.0.clone()))
        .ok_or_else(|| "The intended learning data path was not resolved during startup; retry startup before accessing retained files".into())
}

pub(crate) fn report_lifecycle(app: &tauri::AppHandle, message: &str) {
    if let Some(database) = app.try_state::<DatabasePath>() {
        crate::speech_cache::report_error(&database.0, message);
    } else {
        eprintln!("{message}; learning data path unavailable");
    }
}

pub(crate) fn load(
    database: &DatabasePath,
    observe: impl FnMut(&str) -> Result<(), String>,
    initialize: impl FnOnce() -> Result<(), String>,
) -> StartupStatus {
    if let Err(error) = prepare_directory(database) {
        let mut status = unavailable(error);
        match crate::restore::retained_copies(database) {
            Ok(copies) => status.retained_copies = copies,
            Err(error) => {
                status.error = Some(format!(
                    "{}; retained copies unavailable: {error}",
                    status.error.as_deref().unwrap_or_default()
                ))
            }
        }
        return status;
    }
    let mut status = crate::restore::recover_startup(database, observe);
    if status.state == StartupPhase::Ready
        && let Err(error) = validate_database_copy(database).and_then(|()| initialize())
    {
        status.state = StartupPhase::RecoveryRequired;
        status.error = Some(format!("Learning initialization failed: {error}"));
    }
    status
}

pub(crate) fn unavailable(error: impl Into<String>) -> StartupStatus {
    StartupStatus {
        state: StartupPhase::RecoveryRequired,
        operation: None,
        restored: None,
        retained_copies: Vec::new(),
        error: Some(error.into()),
    }
}

fn prepare_directory(database: &DatabasePath) -> Result<(), String> {
    let directory = database
        .0
        .parent()
        .ok_or("Learning database has no application directory")?;
    std::fs::create_dir_all(directory).map_err(|error| {
        format!(
            "Could not prepare learning directory {}: {error}",
            directory.display()
        )
    })?;
    #[cfg(windows)]
    let directory = {
        let audio = crate::speech_cache::managed_audio_directory(&database.0)?;
        let temporary = audio
            .parent()
            .ok_or("Managed audio has no portable temp directory")?
            .to_owned();
        std::fs::create_dir_all(&temporary).map_err(|error| {
            format!(
                "Could not prepare portable temp directory {}: {error}",
                temporary.display()
            )
        })?;
        temporary
    };
    let probe = crate::backup::OwnedDirectory::create(&directory)?;
    std::fs::File::create(probe.0.join("startup-write-probe"))
        .and_then(|file| file.sync_all())
        .map_err(|error| {
            format!(
                "Could not write and sync startup directory {}: {error}",
                directory.display()
            )
        })
}

fn validate_database_copy(database: &DatabasePath) -> Result<(), String> {
    if database.0.try_exists().map_err(|error| error.to_string())? {
        // Even a read-only SQLite WAL connection can rewrite SHM; probe an owned byte copy first.
        let root = crate::backup::OwnedDirectory::create(
            database
                .0
                .parent()
                .ok_or("Learning database has no application directory")?,
        )?;
        let probe = root.0.join("learning.sqlite3");
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let source = std::path::PathBuf::from(format!("{}{suffix}", database.0.display()));
            if source.try_exists().map_err(|error| error.to_string())? {
                std::fs::copy(&source, root.0.join(format!("learning.sqlite3{suffix}"))).map_err(
                    |error| format!("Could not read learning file {}: {error}", source.display()),
                )?;
            }
        }
        initialize_connection(&probe)?;
    }
    Ok(())
}

pub(crate) fn initialize_connection(path: &std::path::Path) -> Result<(), String> {
    let connection = rusqlite::Connection::open(path).map_err(|error| error.to_string())?;
    lexique_storage::initialize(&connection).map_err(|error| error.to_string())?;
    lexique_storage::validate_learning_database(&connection).map_err(|error| error.to_string())?;
    connection.close().map_err(|(_, error)| error.to_string())
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

    pub(crate) fn initialized(&self, status: StartupStatus) -> Result<(), String> {
        let mut runtime = self.0.lock().map_err(|_| "Startup status unavailable")?;
        runtime.initialized = status.state == StartupPhase::Ready;
        runtime.status = status;
        Ok(())
    }

    pub(crate) fn requires_restart(&self) -> Result<bool, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Startup status unavailable")?
            .needs_restart)
    }

    pub(crate) fn raw_restore(&self, raw_requests: bool) -> Result<bool, String> {
        let runtime = self.0.lock().map_err(|_| "Startup recovery unavailable")?;
        if runtime.initialized
            && runtime.status.state == StartupPhase::Ready
            && !runtime.needs_restart
        {
            return if raw_requests {
                Err("Raw-file preservation is only available from startup recovery".into())
            } else {
                Ok(false)
            };
        }
        if !runtime.initialized && runtime.status.state == StartupPhase::RecoveryRequired {
            return if raw_requests {
                Ok(true)
            } else {
                Err("Startup recovery requires preserving the raw pending request strings".into())
            };
        }
        Err(
            "Finish the current startup recovery and cleanup before restoring another backup"
                .into(),
        )
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

pub(crate) fn coordinate_close(app: tauri::AppHandle, source: &'static str) {
    tauri::async_runtime::spawn_blocking(move || {
        let result = if learning_available(&app) {
            crate::vocabulary::request_close(&app)
        } else {
            crate::vocabulary::finish_close(app)
        };
        if let Err(error) = result {
            eprintln!("Could not coordinate {source}: {error}");
        }
    });
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

#[tauri::command(async)]
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
            let _retry = app
                .state::<crate::backup_operations::BackupOperations>()
                .begin_close()?;
            runtime.status = initialize(&app);
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
        crate::restore::cleanup_begin(&database(&app)?, &operation, |_| Ok(()))?;
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
        let _cleanup = app
            .state::<crate::backup_operations::BackupOperations>()
            .begin_close()?;
        let state = app.state::<StartupState>();
        let mut runtime = state.0.lock().map_err(|_| "Startup cleanup unavailable")?;
        if runtime.needs_restart {
            return Err("Startup cleanup is unavailable in the old runtime".into());
        }
        let database = database(&app)?;
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
            && let Err(error) = validate_database_copy(&database)
                .and_then(|()| crate::initialize_learning(&app, &database))
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
