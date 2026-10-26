use std::fs::create_dir_all;

use lexique_storage::initialize;
use rusqlite::Connection;
use tauri::{Emitter, Manager, State};

mod vocabulary;
use vocabulary::{
    DatabasePath, finish_close, speech_preferences, vocabulary_find, vocabulary_list,
    vocabulary_save, vocabulary_sort,
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
fn runtime_status(status: State<'_, RuntimeStatus>) -> RuntimeStatus {
    status.inner().clone()
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
            app.manage(DatabasePath(database_path));
            app.manage(RuntimeStatus {
                version: app.package_info().version.to_string(),
                schema_version,
                storage: StorageKind::Sqlite,
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window.emit("lexique-close-requested", ()) {
                    eprintln!("Could not coordinate window close: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            runtime_status,
            dictionary_lookup,
            vocabulary_find,
            vocabulary_list,
            vocabulary_save,
            vocabulary_sort,
            speech_preferences,
            finish_close
        ])
        .run(tauri::generate_context!())
        .expect("Lexique could not start; check native initialization errors");
}
