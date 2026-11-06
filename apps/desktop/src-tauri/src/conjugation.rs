use tauri::Manager;

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
