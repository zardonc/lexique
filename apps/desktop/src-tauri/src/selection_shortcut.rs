use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

use crate::vocabulary::{DatabasePath, open};

const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+Shift+D";

#[derive(Clone, Serialize)]
pub struct ShortcutStatus {
    shortcut: String,
    registered: bool,
    error: Option<String>,
}

pub struct ShortcutSettings(Mutex<ShortcutStatus>);

fn parse(value: &str) -> Result<Shortcut, String> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err("Enter a shortcut of 1–128 bytes without control characters".into());
    }
    let shortcut: Shortcut = value
        .parse()
        .map_err(|error| format!("Invalid shortcut: {error}"))?;
    if !shortcut
        .mods
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER)
    {
        return Err("A global shortcut must include Control, Alt or Super".into());
    }
    Ok(shortcut)
}

fn load(connection: &Connection) -> Result<String, String> {
    let raw: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'selection-shortcut'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read shortcut: {error}"))?;
    let value = match raw {
        Some(raw) => serde_json::from_str(&raw)
            .map_err(|error| format!("Invalid saved shortcut: {error}"))?,
        None => DEFAULT_SHORTCUT.to_owned(),
    };
    parse(&value)?;
    Ok(value)
}

fn save(connection: &Connection, value: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO app_metadata (key, value) VALUES ('selection-shortcut', ?1)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [serde_json::to_string(value).map_err(|error| error.to_string())?],
        )
        .map_err(|error| format!("Could not save shortcut: {error}"))?;
    Ok(())
}

pub fn setup(app: &tauri::AppHandle) {
    let mut status = ShortcutStatus {
        shortcut: DEFAULT_SHORTCUT.into(),
        registered: false,
        error: None,
    };
    let result = open(&app.state::<DatabasePath>())
        .and_then(|connection| load(&connection))
        .and_then(|value| {
            status.shortcut = value;
            app.global_shortcut()
                .register(parse(&status.shortcut)?)
                .map_err(|error| format!("Could not register shortcut: {error}"))
        });
    match result {
        Ok(()) => status.registered = true,
        Err(error) => status.error = Some(error),
    }
    app.manage(ShortcutSettings(Mutex::new(status)));
}

fn replace(app: &tauri::AppHandle, value: String) -> Result<ShortcutStatus, String> {
    let next = parse(&value)?;
    let state = app.state::<ShortcutSettings>();
    let mut current = state
        .0
        .lock()
        .map_err(|error| format!("Shortcut state unavailable: {error}"))?;
    let previous = parse(&current.shortcut)?;
    let changed = !current.registered || previous != next;
    let connection = open(&app.state::<DatabasePath>())?;
    if changed {
        app.global_shortcut()
            .register(next)
            .map_err(|error| format!("Could not register shortcut; choose another: {error}"))?;
        if current.registered {
            if let Err(error) = app.global_shortcut().unregister(previous) {
                let rollback = app.global_shortcut().unregister(next);
                return Err(format!(
                    "Could not release previous shortcut: {error}; rollback: {rollback:?}"
                ));
            }
        }
    }
    if let Err(error) = save(&connection, &value) {
        if changed {
            let restore = if current.registered {
                app.global_shortcut().register(previous)
            } else {
                Ok(())
            };
            let release = app.global_shortcut().unregister(next);
            current.registered = app.global_shortcut().is_registered(previous);
            current.error = Some(format!(
                "{error}; restore: {restore:?}; release: {release:?}"
            ));
            return Err(current.error.clone().unwrap_or(error));
        }
        return Err(error);
    }
    *current = ShortcutStatus {
        shortcut: value,
        registered: true,
        error: None,
    };
    Ok(current.clone())
}

#[tauri::command]
pub async fn selection_shortcut(
    app: tauri::AppHandle,
    shortcut: Option<String>,
) -> Result<ShortcutStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(shortcut) = shortcut {
            return replace(&app, shortcut);
        }
        app.state::<ShortcutSettings>()
            .0
            .lock()
            .map(|value| value.clone())
            .map_err(|error| format!("Shortcut state unavailable: {error}"))
    })
    .await
    .map_err(|error| format!("Shortcut preference task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_requires_modifier_and_retains_saved_value() {
        assert!(parse("Ctrl+Alt+Shift+D").is_ok());
        assert!(parse("Ctrl+Alt+D").is_ok());
        assert!(parse("Super+Shift+D").is_ok());
        for value in ["", "D", "Shift+D", "Ctrl+\nD", "Ctrl+Unknown"] {
            assert!(parse(value).is_err(), "{value}");
        }
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE app_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
            .unwrap();
        assert_eq!(load(&connection).unwrap(), "Ctrl+Alt+Shift+D");
        save(&connection, "Ctrl+Alt+K").unwrap();
        assert_eq!(load(&connection).unwrap(), "Ctrl+Alt+K");
        connection
            .execute("UPDATE app_metadata SET value = 'broken'", [])
            .unwrap();
        assert!(load(&connection).is_err());
        assert_eq!(
            connection
                .query_row("SELECT value FROM app_metadata", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            "broken"
        );
    }
}
