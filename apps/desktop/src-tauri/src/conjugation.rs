use crate::vocabulary::{DatabasePath, open};
use tauri::{Manager, State};

#[tauri::command]
pub async fn conjugation_lookup(
    app: tauri::AppHandle,
    key: String,
) -> Result<lexique_dictionary::conjugation::Lookup, lexique_dictionary::DictionaryError> {
    let path = app
        .path()
        .resource_dir()
        .map_err(|error| lexique_dictionary::DictionaryError {
            code: "query_failed",
            message: format!("Could not resolve conjugation resources: {error}"),
        })?
        .join("conjugation/conjugation.sqlite3");
    tauri::async_runtime::spawn_blocking(move || {
        lexique_dictionary::conjugation::lookup(&path, &key, env!("LEXIQUE_CONJUGATION_SHA256"))
    })
    .await
    .map_err(|error| lexique_dictionary::DictionaryError {
        code: "query_failed",
        message: format!("Conjugation query task failed: {error}"),
    })?
}

#[tauri::command]
pub async fn conjugation_preferences(
    path: State<'_, DatabasePath>,
    preferences: Option<lexique_storage::ConjugationPreferences>,
) -> Result<lexique_storage::ConjugationPreferences, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(preferences) = preferences {
            lexique_storage::set_conjugation_preferences(&connection, &preferences)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::conjugation_preferences(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Conjugation preference task failed: {error}"))?
}

#[tauri::command]
pub async fn conjugation_block_order(
    path: State<'_, DatabasePath>,
    order: Option<lexique_storage::ConjugationBlockOrder>,
) -> Result<lexique_storage::ConjugationBlockOrder, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(order) = order {
            lexique_storage::set_conjugation_block_order(&connection, &order)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::conjugation_block_order(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Conjugation block order task failed: {error}"))?
}
