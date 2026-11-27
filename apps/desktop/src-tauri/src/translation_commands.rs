use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tauri::{Emitter, State};

use crate::{
    translation_store::{CredentialStore, valid_slot},
    vocabulary::{DatabasePath, open},
};

#[derive(Serialize)]
struct CredentialStates {
    deepl: &'static str,
    openai_compatible: &'static str,
}

#[derive(Serialize)]
pub(crate) struct TranslationSettings {
    preferences: lexique_storage::TranslationPreferences,
    keys: CredentialStates,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReplacementKey {
    provider: String,
    key: String,
}

#[cfg(windows)]
fn credentials(path: &DatabasePath) -> Result<Box<dyn CredentialStore>, String> {
    Ok(Box::new(
        crate::translation_credentials::WindowsCredentials::new(&path.0)?,
    ))
}

#[cfg(target_os = "macos")]
fn credentials(_: &DatabasePath) -> Result<Box<dyn CredentialStore>, String> {
    Ok(Box::new(
        crate::translation_credentials_macos::MacCredentials::new()?,
    ))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn credentials(_: &DatabasePath) -> Result<Box<dyn CredentialStore>, String> {
    Err("Credential storage is not available on this platform".into())
}

fn state(store: &dyn CredentialStore, slot: &str) -> &'static str {
    match store.read(slot) {
        Ok(Some(_)) => "configured",
        Ok(None) => "missing",
        Err(_) => "unreadable",
    }
}

#[tauri::command]
pub(crate) async fn translation_settings(
    path: State<'_, DatabasePath>,
    maintenance: State<'_, crate::maintenance::MaintenanceCoordinator>,
    locale: String,
    preferences: Option<lexique_storage::TranslationPreferences>,
    replacement: Option<ReplacementKey>,
) -> Result<TranslationSettings, String> {
    if !matches!(locale.as_str(), "zh" | "en" | "fr") {
        return Err("Invalid locale".into());
    }
    if let Some(preferences) = &preferences {
        let config = &preferences.openai_compatible;
        if config.model.len() > 256 || config.model.chars().any(char::is_control) {
            return Err("invalid_request".into());
        }
        if !config.base_url.is_empty() {
            let model = if config.model.trim().is_empty() {
                "placeholder"
            } else {
                &config.model
            };
            crate::translation_openai::endpoint(&config.base_url, model, "placeholder")?;
        }
    }
    if let Some(replacement) = &replacement {
        valid_slot(&replacement.provider)?;
        if replacement.key.is_empty()
            || replacement.key.len() > 4096
            || replacement.key.chars().any(char::is_control)
        {
            return Err("Invalid credential".into());
        }
    }
    let admission = if preferences.is_some() || replacement.is_some() {
        Some(maintenance.admit()?)
    } else {
        None
    };
    let reader = if admission.is_none() {
        Some(maintenance.admit_reader()?)
    } else {
        None
    };
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let (_admission, _reader) = (admission, reader);
        let connection = open(&path)?;
        let store = credentials(&path)?;
        if let Some(replacement) = replacement {
            store.write(&replacement.provider, &replacement.key)?;
        }
        if let Some(preferences) = preferences {
            lexique_storage::set_translation_preferences(&connection, &preferences)
                .map_err(|error| error.to_string())?;
        }
        Ok(TranslationSettings {
            preferences: lexique_storage::translation_preferences(&connection, &locale)
                .map_err(|error| error.to_string())?,
            keys: CredentialStates {
                deepl: state(store.as_ref(), "deepl"),
                openai_compatible: state(store.as_ref(), "openai_compatible"),
            },
        })
    })
    .await
    .map_err(|_| "Could not access translation settings".to_string())?
}

#[tauri::command]
pub(crate) async fn translation_clear_key(
    path: State<'_, DatabasePath>,
    maintenance: State<'_, crate::maintenance::MaintenanceCoordinator>,
    provider: String,
) -> Result<(), String> {
    valid_slot(&provider)?;
    let admission = maintenance.admit()?;
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let _admission = admission;
        credentials(&path)?.clear(&provider)
    })
    .await
    .map_err(|_| "Could not clear credential".to_string())?
}

#[derive(Default)]
struct OperationState {
    current: Option<(u64, String, tokio::task::AbortHandle)>,
    cancelled: VecDeque<String>,
}

#[derive(Default)]
pub(crate) struct TranslationOperation(Mutex<OperationState>, AtomicU64);

#[tauri::command]
pub(crate) async fn translation_execute(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    maintenance: State<'_, crate::maintenance::MaintenanceCoordinator>,
    operation: State<'_, TranslationOperation>,
    id: String,
    text: String,
    locale: String,
) -> Result<crate::translation_deepl::TranslationResult, String> {
    if id.is_empty() || id.len() > 128 || !matches!(locale.as_str(), "zh" | "en" | "fr") {
        return Err("invalid_request".into());
    }
    let (reader, task, generation) = {
        let mut current = operation.0.lock().map_err(|_| "network_or_timeout")?;
        if let Some(index) = current.cancelled.iter().position(|value| value == &id) {
            current.cancelled.remove(index);
            return Err("cancelled".into());
        }
        if let Some((_, _, previous)) = current.current.take() {
            previous.abort();
        }
        let reader = maintenance.admit_reader()?;
        let connection = open(&path)?;
        let preferences = lexique_storage::translation_preferences(&connection, &locale)
            .map_err(|_| "invalid_request")?;
        let store = credentials(&path)?;
        let key = match preferences.active_provider {
            lexique_storage::TranslationProvider::Disabled => return Err("unconfigured".into()),
            lexique_storage::TranslationProvider::Deepl => store
                .read("deepl")
                .map_err(|_| "credential_unreadable")?
                .ok_or("credential_missing")?,
            lexique_storage::TranslationProvider::OpenaiCompatible => store
                .read("openai_compatible")
                .map_err(|_| "credential_unreadable")?
                .unwrap_or_default(),
        };
        let source = match preferences.source_language {
            lexique_storage::TranslationSource::Auto => "auto",
            lexique_storage::TranslationSource::Fr => "fr",
            lexique_storage::TranslationSource::ZhCn => "zh-CN",
            lexique_storage::TranslationSource::En => "en",
        };
        let target = match preferences.target_language {
            lexique_storage::TranslationLanguage::Fr => "fr",
            lexique_storage::TranslationLanguage::ZhCn => "zh-CN",
            lexique_storage::TranslationLanguage::En => "en",
        };
        let event_id = id.clone();
        let task = tokio::spawn(async move {
            let _ = app.emit(
                "translation-progress",
                serde_json::json!({"id": event_id, "kind": "started"}),
            );
            let result = match preferences.active_provider {
                lexique_storage::TranslationProvider::Deepl => crate::translation_deepl::translate_deepl(&preferences.deepl.tier, &key, &text, source, target).await,
                lexique_storage::TranslationProvider::OpenaiCompatible => crate::translation_openai::translate(
                    &preferences.openai_compatible.base_url, &preferences.openai_compatible.model, &key, &text, source, target,
                    |delta, source, target| { let _ = app.emit("translation-progress", serde_json::json!({"id": event_id, "kind": "delta", "text": delta, "source": source, "target": target})); },
                ).await,
                lexique_storage::TranslationProvider::Disabled => Err("unconfigured"),
            };
            let kind = if result.is_ok() {
                "completed"
            } else {
                "failed"
            };
            let _ = app.emit(
                "translation-progress",
                serde_json::json!({"id": event_id, "kind": kind}),
            );
            result
        });
        let generation = operation.1.fetch_add(1, Ordering::Relaxed);
        current.current = Some((generation, id.clone(), task.abort_handle()));
        (reader, task, generation)
    };
    let result = task.await;
    let mut current = operation.0.lock().map_err(|_| "network_or_timeout")?;
    let active = current
        .current
        .as_ref()
        .is_some_and(|(current_id, _, _)| *current_id == generation);
    if active {
        current.current.take();
    }
    drop(reader);
    if !active {
        return Err("cancelled".into());
    }
    result
        .map_err(|_| "cancelled".to_string())?
        .map_err(str::to_string)
}

#[tauri::command]
pub(crate) fn translation_cancel(
    operation: State<'_, TranslationOperation>,
    id: String,
) -> Result<(), String> {
    let mut current = operation
        .0
        .lock()
        .map_err(|_| "Could not cancel translation")?;
    if id.is_empty() || id.len() > 128 {
        return Err("invalid_request".into());
    }
    if current
        .current
        .as_ref()
        .is_some_and(|(_, current_id, _)| current_id == &id)
    {
        if let Some((_, _, task)) = current.current.take() {
            task.abort();
        }
    } else if !current.cancelled.contains(&id) {
        // shortcut: retains 128 pending cancellations, increase if the UI permits more concurrent requests.
        if current.cancelled.len() == 128 {
            current.cancelled.pop_front();
        }
        current.cancelled.push_back(id);
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn translation_test_connection(
    path: State<'_, DatabasePath>,
    preferences: lexique_storage::TranslationPreferences,
    replacement: Option<ReplacementKey>,
) -> Result<(), String> {
    if replacement
        .as_ref()
        .is_some_and(|value| value.key.len() > 4096 || value.key.chars().any(char::is_control))
    {
        return Err("invalid_request".into());
    }
    let slot = match preferences.active_provider {
        lexique_storage::TranslationProvider::Deepl => "deepl",
        lexique_storage::TranslationProvider::OpenaiCompatible => "openai_compatible",
        lexique_storage::TranslationProvider::Disabled => return Err("unconfigured".into()),
    };
    let key = match replacement {
        Some(value) if value.provider == slot && !value.key.is_empty() => value.key,
        Some(_) => return Err("invalid_request".into()),
        None => credentials(&path)?
            .read(slot)
            .map_err(|_| "credential_unreadable")?
            .unwrap_or_default(),
    };
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        match preferences.active_provider {
            lexique_storage::TranslationProvider::Deepl if key.is_empty() => {
                Err("credential_missing")
            }
            lexique_storage::TranslationProvider::Deepl => {
                crate::translation_deepl::translate_deepl(
                    &preferences.deepl.tier,
                    &key,
                    "Bonjour",
                    "fr",
                    "zh-CN",
                )
                .await
            }
            lexique_storage::TranslationProvider::OpenaiCompatible => {
                crate::translation_openai::translate(
                    &preferences.openai_compatible.base_url,
                    &preferences.openai_compatible.model,
                    &key,
                    "Bonjour",
                    "fr",
                    "zh-CN",
                    |_, _, _| {},
                )
                .await
            }
            lexique_storage::TranslationProvider::Disabled => Err("unconfigured"),
        }
    })
    .await
    .map_err(|_| "network_or_timeout")?
    .map(|_| ())
    .map_err(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture;
    impl CredentialStore for Fixture {
        fn read(&self, slot: &str) -> Result<Option<String>, String> {
            match slot {
                "deepl" => Ok(Some("secret-that-must-not-be-returned".into())),
                "openai_compatible" => Err("unreadable".into()),
                _ => Ok(None),
            }
        }
        fn write(&self, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn clear(&self, _: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn settings_response_exposes_only_credential_states() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        lexique_storage::initialize(&connection).unwrap();
        let result = TranslationSettings {
            preferences: lexique_storage::translation_preferences(&connection, "zh").unwrap(),
            keys: CredentialStates {
                deepl: state(&Fixture, "deepl"),
                openai_compatible: state(&Fixture, "openai_compatible"),
            },
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["keys"]["deepl"], "configured");
        assert_eq!(json["keys"]["openai_compatible"], "unreadable");
        assert!(
            !json
                .to_string()
                .contains("secret-that-must-not-be-returned")
        );
        assert_eq!(state(&Fixture, "missing"), "missing");
    }
}
