use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use lexique_dictionary::{Lookup, Sense};
use lexique_storage::{
    DigestEntry, Reference, SaveRequest, SaveResult, Snapshot, VocabularyRecord,
};
use rusqlite::Connection;
use tauri::{Manager, State};

pub struct DatabasePath(pub PathBuf);

#[cfg(target_os = "macos")]
// shortcut: Dock Quit bypasses this guard; cover it before macOS acceptance.
pub fn coordinate_quit(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = app.menu().ok_or("Default macOS menu unavailable")?;
    let app_menu = menu
        .items()?
        .first()
        .and_then(|item| item.as_submenu())
        .cloned()
        .ok_or("Default macOS application menu unavailable")?;
    let quit = app_menu
        .items()?
        .pop()
        .ok_or("Default macOS quit item unavailable")?;
    let label = quit
        .as_predefined_menuitem()
        .ok_or("Unexpected macOS quit item")?
        .text()?;
    let replacement =
        tauri::menu::MenuItem::with_id(app, "lexique-quit", label, true, Some("CmdOrCtrl+Q"))?;
    app_menu.remove(&quit)?;
    app_menu.append(&replacement)?;
    app.on_menu_event(|app, event| {
        if event.id() == "lexique-quit" {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(error) = window.close() {
                    eprintln!("Could not request window close: {error}");
                }
            }
        }
    });
    Ok(())
}

fn open(path: &DatabasePath) -> Result<Connection, String> {
    let connection =
        Connection::open_with_flags(&path.0, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .map_err(|error| format!("Could not open learning database: {error}"))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| error.to_string())?;
    lexique_storage::initialize(&connection).map_err(|error| error.to_string())?;
    Ok(connection)
}

fn first_translation(senses: &[Sense]) -> Option<String> {
    senses.iter().find_map(|sense| {
        sense
            .translations
            .first()
            .cloned()
            .or_else(|| first_translation(&sense.senses))
    })
}

fn snapshot(lookup: Lookup) -> Snapshot {
    let source = lookup.source;
    Snapshot {
        reference: Reference {
            dictionary: source.title,
            lookup_key: lookup.entries[0].key.clone(),
            version: source.version,
            attribution: source.attribution,
            license: source.license,
            source_url: source.source_url,
            license_url: source.license_url,
            sha256: lookup.sha256,
        },
        digest: lookup
            .entries
            .into_iter()
            .map(|entry| DigestEntry {
                pronunciations: entry.pronunciations,
                grammar: entry.grammar,
                translation: first_translation(&entry.senses),
            })
            .collect(),
    }
}

#[tauri::command]
pub async fn vocabulary_save(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    request: SaveRequest,
) -> Result<SaveResult, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::validate(&request).map_err(|error| error.to_string())?;
        let mut connection = open(&path)?;
        // A committed retry must work even if its original resource has since disappeared.
        if let Some(result) =
            lexique_storage::replay(&connection, &request).map_err(|error| error.to_string())?
        {
            return Ok(result);
        }
        let content = if request.id.is_none() {
            let resource = app
                .path()
                .resource_dir()
                .map_err(|error| error.to_string())?
                .join("dictionary/dictionary.sqlite3");
            let lookup = lexique_dictionary::lookup(
                &resource,
                &request.headword,
                env!("LEXIQUE_DICTIONARY_SHA256"),
            )
            .map_err(|error| error.to_string())?;
            if lookup.entries.is_empty() {
                return Err("Headword is not in the verified dictionary".into());
            }
            Some(snapshot(lookup))
        } else {
            None
        };
        lexique_storage::save(&mut connection, &request, content).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Vocabulary save task failed: {error}"))?
}

#[tauri::command]
pub async fn vocabulary_find(
    path: State<'_, DatabasePath>,
    headword: String,
) -> Result<Option<VocabularyRecord>, String> {
    if headword.chars().count() > 200 {
        return Err("Headword exceeds 200 characters".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::find(&open(&path)?, &headword).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Vocabulary read task failed: {error}"))?
}

#[tauri::command]
pub async fn vocabulary_list(
    path: State<'_, DatabasePath>,
    query: String,
    sort: String,
) -> Result<Vec<VocabularyRecord>, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::list(&open(&path)?, &query, &sort).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Vocabulary list task failed: {error}"))?
}

#[tauri::command]
pub async fn review_add(
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
) -> Result<Vec<lexique_storage::ReviewCard>, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::add_review(&mut open(&path)?, vocabulary_id)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Review creation task failed: {error}"))?
}

#[tauri::command]
pub async fn review_cards(
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
) -> Result<Vec<lexique_storage::ReviewCard>, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::review_cards(&open(&path)?, vocabulary_id)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Review read task failed: {error}"))?
}

#[tauri::command]
pub async fn speech_cached(
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
) -> Result<Option<Vec<u8>>, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let cards = lexique_storage::review_cards(&open(&path)?, vocabulary_id)
            .map_err(|e| e.to_string())?;
        if !cards.iter().any(|card| card.kind == "listening") {
            return Err("Listening card does not exist".into());
        }
        crate::speech_cache::read(&path.0, vocabulary_id)
    })
    .await
    .map_err(|e| format!("Speech cache task failed: {e}"))?
}

#[tauri::command]
pub async fn speech_generate(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
    regenerate: Option<bool>,
) -> Result<Vec<u8>, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    let resources = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("speech");
    tauri::async_runtime::spawn_blocking(move || {
        static GENERATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = GENERATION.lock().map_err(|error| error.to_string())?;
        let connection = open(&path)?;
        let cards = lexique_storage::review_cards(&connection, vocabulary_id)
            .map_err(|error| error.to_string())?;
        if !cards.iter().any(|card| card.kind == "listening") {
            return Err("Listening card does not exist".into());
        }
        let replace = match crate::speech_cache::read(&path.0, vocabulary_id) {
            Ok(Some(bytes)) if !regenerate.unwrap_or(false) => return Ok(bytes),
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(error) if error == "Invalid cached speech audio" => true,
            Err(error) => return Err(error),
        };
        let record =
            lexique_storage::get(&connection, vocabulary_id).map_err(|error| error.to_string())?;
        let preferences =
            lexique_storage::speech_preferences(&connection).map_err(|error| error.to_string())?;
        let voice = match preferences.voice {
            lexique_storage::SpeechVoice::Female => "female",
            lexique_storage::SpeechVoice::Male => "male",
        };
        let request =
            serde_json::to_vec(&serde_json::json!({ "text": record.headword, "voice": voice }))
                .map_err(|error| error.to_string())?;
        let mut command = Command::new(resources.join("runtime/python.exe"));
        command
            .args([
                "-I",
                "-X",
                "utf8",
                "-c",
                include_str!("../speech_worker.py"),
            ])
            .arg(resources.join("assets"))
            .arg(resources.join("opusenc.dll"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start offline speech: {error}"))?;
        let input = child
            .stdin
            .take()
            .ok_or_else(|| "Speech input unavailable".to_string());
        let input_result = input.and_then(|mut input| {
            input
                .write_all(&request)
                .map_err(|error| format!("Could not send speech request: {error}"))
        });
        if let Err(error) = input_result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let output = child
            .wait_with_output()
            .map_err(|error| format!("Speech process failed: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "Offline speech failed: {}",
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(2048)
                    .collect::<String>()
            ));
        }
        if output.stdout.len() > 10_000_000 || !output.stdout.starts_with(b"OggS") {
            return Err("Offline speech returned invalid audio".into());
        }
        crate::speech_cache::write(&path.0, vocabulary_id, &output.stdout, replace)?;
        crate::speech_cache::read(&path.0, vocabulary_id)?
            .ok_or_else(|| "Generated speech cache disappeared".to_string())
    })
    .await
    .map_err(|error| format!("Speech task failed: {error}"))?
}

#[tauri::command]
pub async fn vocabulary_sort(
    path: State<'_, DatabasePath>,
    sort: Option<String>,
) -> Result<String, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(sort) = sort {
            lexique_storage::set_sort_preference(&connection, &sort)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::sort_preference(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Vocabulary preference task failed: {error}"))?
}

#[tauri::command]
pub async fn speech_preferences(
    path: State<'_, DatabasePath>,
    preferences: Option<lexique_storage::SpeechPreferences>,
) -> Result<lexique_storage::SpeechPreferences, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(preferences) = preferences {
            lexique_storage::set_speech_preferences(&connection, &preferences)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::speech_preferences(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Speech preference task failed: {error}"))?
}

#[tauri::command]
pub fn finish_close(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Main window unavailable")?
        .destroy()
        .map_err(|error| format!("Could not close window: {error}"))
}
