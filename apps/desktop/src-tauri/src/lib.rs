use std::fs::create_dir_all;

use lexique_storage::initialize;
use rusqlite::Connection;
use tauri::{Emitter, Manager, State};

#[cfg(windows)]
mod audio_maintenance;
mod conjugation;
mod review_window;
use conjugation::{conjugation_lookup, conjugation_preferences};
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
async fn dictionary_lookup(
    app: tauri::AppHandle,
    key: String,
) -> Result<lexique_dictionary::Lookup, lexique_dictionary::DictionaryError> {
    let path = app
        .path()
        .resource_dir()
        .map_err(|error| lexique_dictionary::DictionaryError {
            code: "query_failed",
            message: format!("Could not resolve dictionary resources: {error}"),
        })?
        .join("dictionary/dictionary.sqlite3");
    tauri::async_runtime::spawn_blocking(move || {
        lexique_dictionary::lookup(&path, &key, env!("LEXIQUE_DICTIONARY_SHA256"))
    })
    .await
    .map_err(|error| lexique_dictionary::DictionaryError {
        code: "query_failed",
        message: format!("Dictionary query task failed: {error}"),
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
                speech_cache::report_error(
                    &window.state::<DatabasePath>().0,
                    "[native-lifecycle] Window CloseRequested",
                );
                api.prevent_close();
                if let Err(error) = window.emit("lexique-close-requested", ()) {
                    eprintln!("Could not coordinate window close: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            dictionary_lookup,
            conjugation_lookup,
            conjugation_preferences,
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
            finish_close
        ])
        .run(tauri::generate_context!())
        .expect("Lexique could not start; check native initialization errors");
}
