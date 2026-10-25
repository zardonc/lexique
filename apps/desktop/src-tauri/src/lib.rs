use std::fs::create_dir_all;

use lexique_storage::initialize;
use rusqlite::Connection;
use tauri::{Manager, State};

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
            let directory = app.path().app_data_dir()?;
            create_dir_all(&directory)?;
            let connection = Connection::open(directory.join(DATABASE_FILENAME))?;
            let schema_version = initialize(&connection)?;
            app.manage(RuntimeStatus {
                version: app.package_info().version.to_string(),
                schema_version,
                storage: StorageKind::Sqlite,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![runtime_status, dictionary_lookup])
        .run(tauri::generate_context!())
        .expect("Lexique could not start; check native initialization errors");
}
