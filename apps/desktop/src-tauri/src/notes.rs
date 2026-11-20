use lexique_storage::{self as storage, StorageError};
use rusqlite::Connection;
use tauri::Manager;

use crate::vocabulary::{DatabasePath, open};

fn now_millis(connection: &Connection) -> Result<i64, StorageError> {
    Ok(connection.query_row(
        "SELECT CAST(unixepoch('subsec') * 1000 AS INTEGER)",
        [],
        |row| row.get(0),
    )?)
}

async fn read<T: Send + 'static>(
    app: tauri::AppHandle,
    action: impl FnOnce(&Connection) -> Result<T, StorageError> + Send + 'static,
) -> Result<T, String> {
    let reader = crate::maintenance::admit_reader_app(&app)?;
    let path = DatabasePath(app.state::<DatabasePath>().0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _reader = reader;
        action(&open(&path)?).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Notebook read task failed: {error}"))?
}

async fn write<T: Send + 'static>(
    app: tauri::AppHandle,
    action: impl FnOnce(&Connection) -> Result<T, StorageError> + Send + 'static,
) -> Result<T, String> {
    let admission = crate::maintenance::admit_app(&app)?;
    let path = DatabasePath(app.state::<DatabasePath>().0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        let result = action(&open(&path)?).map_err(|error| error.to_string())?;
        crate::speech_tasks::changed(&app);
        Ok(result)
    })
    .await
    .map_err(|error| format!("Notebook write task failed: {error}"))?
}

#[tauri::command]
pub async fn grammar_list(
    app: tauri::AppHandle,
    query: String,
    sort: String,
) -> Result<Vec<storage::GrammarRecord>, String> {
    read(app, move |connection| {
        storage::list_grammar(connection, &query, &sort)
    })
    .await
}

#[tauri::command]
pub async fn grammar_get(
    app: tauri::AppHandle,
    id: i64,
) -> Result<Option<storage::GrammarRecord>, String> {
    read(app, move |connection| {
        match storage::get_grammar(connection, id) {
            Ok(record) => Ok(Some(record)),
            Err(StorageError::Database(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(error) => Err(error),
        }
    })
    .await
}

#[tauri::command]
pub async fn grammar_save(
    app: tauri::AppHandle,
    request: storage::GrammarSaveRequest,
) -> Result<storage::GrammarSaveResult, String> {
    write(app, move |connection| {
        storage::save_grammar(connection, &request, now_millis(connection)?)
    })
    .await
}

#[tauri::command]
pub async fn grammar_card_save(
    app: tauri::AppHandle,
    request: storage::GrammarCardRequest,
) -> Result<storage::GrammarSaveResult, String> {
    write(app, move |connection| {
        storage::save_grammar_card(connection, &request, now_millis(connection)?)
    })
    .await
}

#[tauri::command]
pub async fn grammar_delete(app: tauri::AppHandle, id: i64, revision: i64) -> Result<(), String> {
    write(app, move |connection| {
        storage::delete_grammar(connection, id, revision)
    })
    .await
}

#[tauri::command]
pub async fn grammar_sort(app: tauri::AppHandle, sort: Option<String>) -> Result<String, String> {
    if let Some(sort) = sort {
        write(app, move |connection| {
            storage::set_grammar_sort_preference(connection, &sort)?;
            storage::grammar_sort_preference(connection)
        })
        .await
    } else {
        read(app, storage::grammar_sort_preference).await
    }
}

#[tauri::command]
pub async fn learning_categories(
    app: tauri::AppHandle,
) -> Result<Vec<storage::LearningCategory>, String> {
    read(app, storage::learning_categories).await
}

#[tauri::command]
pub async fn category_create(
    app: tauri::AppHandle,
    name: String,
) -> Result<storage::LearningCategory, String> {
    write(app, move |connection| {
        storage::create_category(connection, &name)
    })
    .await
}

#[tauri::command]
pub async fn category_rename(
    app: tauri::AppHandle,
    id: i64,
    name: String,
) -> Result<storage::LearningCategory, String> {
    write(app, move |connection| {
        storage::rename_category(connection, id, &name)
    })
    .await
}

#[tauri::command]
pub async fn category_delete(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    write(app, move |connection| {
        storage::delete_category(connection, id)
    })
    .await
}

#[tauri::command]
pub async fn category_bulk(
    app: tauri::AppHandle,
    request: storage::CategoryBulkRequest,
) -> Result<Vec<storage::CategoryBulkResult>, String> {
    write(app, move |connection| {
        storage::bulk_classify(connection, &request)
    })
    .await
}
