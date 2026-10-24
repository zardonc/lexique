use std::fs::create_dir_all;

use lexique_storage::initialize;
use rusqlite::Connection;
use tauri::{Manager, State};

mod status;
use status::{RuntimeStatus, StorageKind};

const DATABASE_FILENAME: &str = "lexique.sqlite3";

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
        .invoke_handler(tauri::generate_handler![runtime_status])
        .run(tauri::generate_context!())
        .expect("Lexique could not start; check native initialization errors");
}
