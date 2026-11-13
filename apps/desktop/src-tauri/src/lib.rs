use std::fs::create_dir_all;

use lexique_storage::initialize;
use rusqlite::Connection;
use tauri::{Manager, State};

#[cfg(windows)]
mod audio_maintenance;
mod conjugation;
mod dictionaries;
mod dictionary_links;
mod interface;
use interface::interface_preferences;
mod backup;
mod backup_operations;
mod maintenance;
mod ocr;
mod ocr_capture;
mod review_window;
mod selection;
mod selection_shortcut;
use conjugation::{conjugation_block_order, conjugation_lookup, conjugation_preferences};
use dictionaries::{
    dictionary_import, dictionary_lookup, dictionary_relink, dictionary_remove, dictionary_settings,
};
mod speech_cache;
mod speech_tasks;
mod vocabulary;
#[cfg(windows)]
mod webview_diagnostics;
use vocabulary::{
    DatabasePath, finish_close, review_add, review_auto_enroll, review_availability, review_cards,
    review_pause, review_progress, review_queue, review_rate, review_settings, review_undo,
    review_undo_target, speech_cached, speech_generate, speech_preferences, speech_preview,
    vocabulary_find, vocabulary_list, vocabulary_save, vocabulary_sort,
};

mod status;
use status::{RuntimeStatus, StorageKind};

const DATABASE_FILENAME: &str = "lexique.sqlite3";

#[tauri::command]
async fn frequency_lookup(
    app: tauri::AppHandle,
    keys: Vec<String>,
) -> Result<lexique_dictionary::frequency::Lookup, lexique_dictionary::DictionaryError> {
    let path = app
        .path()
        .resource_dir()
        .map_err(|error| lexique_dictionary::DictionaryError {
            code: "query_failed",
            message: format!("Could not resolve frequency resources: {error}"),
        })?
        .join("frequency/frequency.sqlite3");
    tauri::async_runtime::spawn_blocking(move || {
        lexique_dictionary::frequency::lookup(&path, &keys, env!("LEXIQUE_FREQUENCY_SHA256"))
    })
    .await
    .map_err(|error| lexique_dictionary::DictionaryError {
        code: "query_failed",
        message: format!("Frequency query task failed: {error}"),
    })?
}

#[tauri::command]
async fn runtime_status(
    status: State<'_, RuntimeStatus>,
    path: State<'_, DatabasePath>,
    _tasks: State<'_, speech_tasks::SpeechTasks>,
) -> Result<RuntimeStatus, String> {
    let mut status = status.inner().clone();
    #[cfg(windows)]
    let cache = _tasks.cache_status()?;
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = vocabulary::open(&path)?;
        status.audio_migration =
            lexique_storage::audio_migration_status(&connection).map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            status.audio_cache = Some(status::AudioCacheStatus {
                files: cache,
                usage: lexique_storage::audio_usage(&connection, speech_tasks::now()?)
                    .map_err(|e| e.to_string())?,
            });
        }
        Ok(status)
    })
    .await
    .map_err(|e| format!("Audio migration status failed: {e}"))?
}

pub fn run() {
    tauri::Builder::default()
        .manage(vocabulary::CloseAuthorization::default())
        .manage(maintenance::MaintenanceCoordinator::default())
        .manage(backup_operations::BackupOperations::default())
        .manage(dictionaries::Registry::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let app = app.clone();
                        let shortcut = *shortcut;
                        tauri::async_runtime::spawn_blocking(move || {
                            if let Err(error) = selection_shortcut::dispatch(&app, &shortcut) {
                                eprintln!("{error}");
                            }
                        });
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            vocabulary::coordinate_quit(app)?;
            let directory = app.path().app_data_dir()?;
            create_dir_all(&directory)?;
            let database_path = directory.join(DATABASE_FILENAME);
            let connection = Connection::open(&database_path)?;
            let schema_version = initialize(&connection)?;
            drop(connection);
            #[cfg(windows)]
            webview_diagnostics::attach(app, database_path.clone())?;
            let resources = app.path().resource_dir()?.join("speech");
            app.manage(speech_tasks::SpeechTasks::start(
                app.handle().clone(),
                database_path.clone(),
                resources,
            ));
            app.manage(review_window::WindowValidator::start(
                app.handle().clone(),
                database_path.clone(),
            ));
            app.manage(DatabasePath(database_path));
            app.manage(selection::SelectionState::default());
            app.manage(ocr_capture::CaptureState::default());
            selection_shortcut::setup(app.handle());
            app.manage(RuntimeStatus {
                version: app.package_info().version.to_string(),
                schema_version,
                storage: StorageKind::Sqlite,
                audio_migration: lexique_storage::AudioMigrationStatus::default(),
                audio_cache: None,
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(request) = window
                    .label()
                    .strip_prefix("ocr-")
                    .and_then(|label| label.split('-').next())
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    api.prevent_close();
                    if let Err(error) = ocr_capture::abort(window.app_handle(), request) {
                        eprintln!("{error}");
                    }
                    return;
                }
                speech_cache::report_error(
                    &window.state::<DatabasePath>().0,
                    "[native-lifecycle] Window CloseRequested",
                );
                api.prevent_close();
                if let Err(error) = vocabulary::request_close(window.app_handle()) {
                    eprintln!("Could not coordinate window close: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            dictionary_lookup,
            dictionary_settings,
            dictionary_import,
            dictionary_relink,
            dictionary_remove,
            dictionary_links::dictionary_resource_open,
            frequency_lookup,
            selection::selection_capture,
            selection::selection_cancel,
            selection_shortcut::selection_shortcut,
            selection_shortcut::ocr_shortcut,
            ocr::ocr_image,
            ocr_capture::ocr_capture,
            ocr_capture::ocr_locale,
            ocr_capture::ocr_screen,
            ocr_capture::ocr_region,
            ocr_capture::ocr_abort,
            conjugation_lookup,
            conjugation_preferences,
            interface_preferences,
            conjugation_block_order,
            vocabulary_find,
            vocabulary_list,
            vocabulary_save,
            vocabulary_sort,
            review_add,
            vocabulary::review_add_batch,
            vocabulary::review_batch_status,
            vocabulary::review_cancel_batch,
            vocabulary::speech_retry_missing,
            review_pause,
            review_auto_enroll,
            review_cards,
            review_availability,
            review_progress,
            review_rate,
            review_undo_target,
            review_undo,
            review_queue,
            review_settings,
            speech_cached,
            speech_generate,
            speech_preview,
            speech_preferences,
            backup_operations::backup_export_start,
            backup_operations::backup_operation_status,
            backup_operations::backup_operation_cancel,
            finish_close
        ])
        .build(tauri::generate_context!())
        .expect("Lexique could not start; check native initialization errors")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if app.get_webview_window("main").is_some()
                    && !app.state::<vocabulary::CloseAuthorization>().permits_exit()
                {
                    api.prevent_exit();
                    if let Err(error) = vocabulary::request_close(app) {
                        eprintln!("Could not coordinate application exit: {error}");
                    }
                }
            }
        });
}
