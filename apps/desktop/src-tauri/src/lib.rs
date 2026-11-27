use tauri::{Manager, State};

#[cfg(windows)]
mod audio_maintenance;
mod cefr;
mod conjugation;
use cefr::{cefr_import, cefr_lookup, cefr_remove, cefr_settings};
mod dictionaries;
mod dictionary_links;
mod interface;
use interface::interface_preferences;
mod backup;
mod backup_operations;
mod backup_preview;
mod maintenance;
mod ocr;
mod ocr_capture;
#[cfg(test)]
mod recovery_tests;
mod restore;
mod restore_operations;
#[cfg(test)]
mod restore_tests;
mod review_window;
mod selection;
mod selection_shortcut;
use conjugation::{conjugation_block_order, conjugation_lookup, conjugation_preferences};
use dictionaries::{
    dictionary_import, dictionary_lookup, dictionary_relink, dictionary_remove, dictionary_settings,
};
mod grammar_import;
mod note_enrichment;
mod note_export;
mod note_import;
mod note_import_csv;
mod note_markdown;
mod note_zip;
mod notes;
mod speech_cache;
mod speech_tasks;
mod startup;
mod vocabulary;
#[cfg(windows)]
mod webview_diagnostics;
use vocabulary::{
    DatabasePath, finish_close, review_add, review_auto_enroll, review_availability, review_cards,
    review_pause, review_progress, review_queue, review_rate, review_scope, review_settings,
    review_undo, review_undo_target, speech_cached, speech_generate, speech_preferences,
    speech_preview, vocabulary_find, vocabulary_list, vocabulary_save, vocabulary_sort,
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
    maintenance: State<'_, maintenance::MaintenanceCoordinator>,
    _tasks: State<'_, speech_tasks::SpeechTasks>,
) -> Result<RuntimeStatus, String> {
    let reader = maintenance.admit_reader()?;
    let mut status = status.inner().clone();
    #[cfg(windows)]
    let cache = _tasks.cache_status()?;
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _reader = reader;
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
        .manage(note_import::NoteImportState::default())
        .manage(backup_operations::BackupOperations::default())
        .manage(note_export::NoteExports::default())
        .manage(restore_operations::RestoreOperations::default())
        .manage(dictionaries::Registry::default())
        .manage(cefr::CefrResource::default())
        .manage(selection::SelectionState::default())
        .manage(ocr_capture::CaptureState::default())
        .manage(startup::StartupState::new(startup::unavailable(
            "Learning initialization has not completed",
        )))
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
            let status = startup::initialize(app.handle());
            if let Err(error) = app.state::<startup::StartupState>().initialized(status) {
                eprintln!("Could not publish startup failure: {error}");
            }
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
                startup::report_lifecycle(
                    window.app_handle(),
                    "[native-lifecycle] Window CloseRequested",
                );
                api.prevent_close();
                startup::coordinate_close(window.app_handle().clone(), "window close");
            }
        })
        .invoke_handler(tauri::generate_handler![
            cefr_settings,
            cefr_import,
            cefr_remove,
            cefr_lookup,
            runtime_status,
            startup::startup_status,
            startup::startup_retry,
            startup::backup_restore_cleanup_begin,
            startup::backup_restore_cleanup_complete,
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
            note_export::note_export_snapshot,
            note_export::note_export_csv,
            note_export::note_export_markdown,
            note_export::note_export_zip,
            notes::grammar_list,
            notes::grammar_get,
            notes::grammar_save,
            notes::grammar_card_save,
            notes::grammar_delete,
            notes::grammar_sort,
            notes::learning_categories,
            notes::category_create,
            notes::category_rename,
            notes::category_delete,
            notes::category_bulk,
            vocabulary_save,
            note_import::note_import_preview,
            note_import::note_import_remap,
            note_import::note_import_commit,
            note_import::note_import_cancel,
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
            review_scope,
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
            backup_operations::backup_preview_start,
            backup_operations::backup_preview_status,
            backup_operations::backup_preview_cancel,
            restore_operations::backup_restore_start,
            restore_operations::backup_restore_status,
            restore_operations::backup_retained_directory_open,
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
                    startup::coordinate_close(app.clone(), "application exit");
                }
            }
        });
}

fn initialize_learning(app: &tauri::AppHandle, database: &DatabasePath) -> Result<(), String> {
    if app.try_state::<speech_tasks::SpeechTasks>().is_some() {
        return Err("Learning workers already exist; a controlled restart is required".into());
    }
    startup::initialize_connection(&database.0)?;
    let resources = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("speech");
    app.manage(speech_tasks::SpeechTasks::start(
        app.clone(),
        database.0.clone(),
        resources,
    ));
    app.manage(review_window::WindowValidator::start(
        app.clone(),
        database.0.clone(),
    ));
    selection_shortcut::setup(app);
    app.manage(RuntimeStatus {
        version: app.package_info().version.to_string(),
        schema_version: lexique_storage::DATABASE_SCHEMA_VERSION,
        storage: StorageKind::Sqlite,
        audio_migration: lexique_storage::AudioMigrationStatus::default(),
        audio_cache: None,
    });
    app.state::<maintenance::MaintenanceCoordinator>()
        .resume_startup()
}
