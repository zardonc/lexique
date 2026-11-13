use crate::vocabulary::{DatabasePath, open};
use tauri::State;

#[tauri::command]
pub async fn interface_preferences(
    path: State<'_, DatabasePath>,
    preferences: Option<lexique_storage::InterfacePreferences>,
) -> Result<lexique_storage::InterfacePreferences, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(preferences) = preferences {
            lexique_storage::set_interface_preferences(&connection, &preferences)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::interface_preferences(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Interface preference task failed: {error}"))?
}
