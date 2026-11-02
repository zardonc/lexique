use std::{path::PathBuf, time::Duration};

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

pub(crate) fn open(path: &DatabasePath) -> Result<Connection, String> {
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
        let result = lexique_storage::save(&mut connection, &request, content)
            .map_err(|error| error.to_string())?;
        app.state::<crate::speech_tasks::SpeechTasks>().wake()?;
        crate::speech_tasks::changed(&app);
        Ok(result)
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
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
) -> Result<Vec<lexique_storage::ReviewCard>, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = open(&path)?;
        let operation = format!(
            "single-{}-{}",
            vocabulary_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos()
        );
        let batch = lexique_storage::add_review_batch(
            &mut connection,
            &operation,
            &[vocabulary_id],
            crate::speech_tasks::now()?,
        )
        .map_err(|e| e.to_string())?;
        let item = batch
            .items
            .first()
            .ok_or("Single review result unavailable")?;
        if let Some(error) = &item.error {
            return Err(error.clone());
        }
        app.state::<crate::speech_tasks::SpeechTasks>().wake()?;
        crate::speech_tasks::changed(&app);
        lexique_storage::review_cards(&connection, vocabulary_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|error| format!("Review creation task failed: {error}"))?
}

#[tauri::command]
pub async fn review_pause(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
    paused: Option<bool>,
) -> Result<bool, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = open(&path)?;
        if let Some(value) = paused {
            lexique_storage::set_review_paused(&mut connection, vocabulary_id, value)
                .map_err(|error| error.to_string())?;
            crate::speech_tasks::changed(&app);
        }
        lexique_storage::review_paused(&connection, vocabulary_id)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Review pause task failed: {error}"))?
}

#[tauri::command]
pub async fn review_auto_enroll(
    path: State<'_, DatabasePath>,
    enabled: Option<bool>,
) -> Result<bool, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(value) = enabled {
            lexique_storage::set_auto_enroll(&connection, value)
                .map_err(|error| error.to_string())?;
        }
        lexique_storage::auto_enroll(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Review preference task failed: {error}"))?
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateInput {
    operation: String,
    card_id: i64,
    expected_revision: i64,
    rating: lexique_storage::Rating,
    timezone: String,
}

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RateOutcome {
    Committed { result: serde_json::Value },
    Rejected { code: String, detail: String },
}

#[tauri::command]
pub async fn review_progress(
    path: State<'_, DatabasePath>,
    card_id: i64,
) -> Result<lexique_storage::ReviewProgress, String> {
    if card_id <= 0 || card_id > 9_007_199_254_740_991 {
        return Err("Invalid card ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::review_progress(&open(&path)?, card_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Review read task failed: {e}"))?
}

#[tauri::command]
pub async fn review_availability(
    path: State<'_, DatabasePath>,
    card_id: i64,
    timezone: String,
) -> Result<lexique_storage::ReviewAvailability, String> {
    if card_id <= 0 || card_id > 9_007_199_254_740_991 || timezone.len() > 128 {
        return Err("Invalid card ID or timezone".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        lexique_storage::review_availability(
            &open(&path)?,
            card_id,
            i64::try_from(now).map_err(|e| e.to_string())?,
            &timezone,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Review availability task failed: {e}"))?
}

#[derive(serde::Serialize)]
pub struct QueueView {
    queue: lexique_storage::ReviewQueue,
    record: Option<VocabularyRecord>,
    audio_pending: bool,
    audio_error: Option<String>,
    audio_vocabulary_id: Option<i64>,
}

#[tauri::command]
pub async fn review_undo_target(path: State<'_, DatabasePath>) -> Result<Option<String>, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::undo_target(&open(&path)?).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Undo target task failed: {e}"))?
}

#[tauri::command]
pub async fn review_undo(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    operation: String,
    rating_operation: String,
) -> Result<UndoOutcome, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        match lexique_storage::undo_rating(&mut open(&path)?, &operation, &rating_operation) {
            Ok(queue_rebuilt) => {
                crate::speech_tasks::changed(&app);
                Ok(UndoOutcome::Committed { queue_rebuilt })
            }
            Err(
                error @ (lexique_storage::StorageError::Invalid(_)
                | lexique_storage::StorageError::RatingConflict),
            ) => Ok(UndoOutcome::Rejected {
                detail: error.to_string(),
            }),
            Err(error) => Err(error.to_string()),
        }
    })
    .await
    .map_err(|e| format!("Undo task failed: {e}"))?
}

#[derive(serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UndoOutcome {
    Committed { queue_rebuilt: bool },
    Rejected { detail: String },
}

#[tauri::command]
pub async fn review_queue(
    path: State<'_, DatabasePath>,
    timezone: String,
    skipped: Vec<i64>,
) -> Result<QueueView, String> {
    if timezone.is_empty()
        || timezone.len() > 128
        || skipped.len() > 10_000
        || skipped
            .iter()
            .any(|id| *id <= 0 || *id > 9_007_199_254_740_991)
    {
        return Err("Invalid timezone".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let mut connection = open(&path)?;
        let inspector = crate::speech_cache::Inspector::new(&path.0);
        let inventory =
            lexique_storage::review_card_inventory(&connection).map_err(|e| e.to_string())?;
        let skipped: std::collections::HashSet<i64> = skipped.into_iter().collect();
        let mut verified_audio = std::collections::HashSet::new();
        let mut audio_pending = false;
        let mut audio_error = None;
        let mut audio_vocabulary_id = None;
        let mut checked = std::collections::HashMap::new();
        let mut legacy = Vec::new();
        let mut next_audio_due: Option<i64> = None;
        let limits = lexique_storage::review_settings(&connection).map_err(|e| e.to_string())?;
        for card in inventory
            .iter()
            .filter(|card| card.kind == "listening" && !skipped.contains(&card.id))
        {
            let progress = lexique_storage::review_progress(&connection, card.id)
                .map_err(|e| e.to_string())?;
            if let Some(due) = progress.card.due_at.filter(|due| *due > now as i64) {
                next_audio_due = Some(next_audio_due.map_or(due, |previous| previous.min(due)));
                continue;
            }
            if limits.new_limit == 0 && limits.review_limit == 0 {
                continue;
            }
            let found = checked.entry(card.vocabulary_id).or_insert_with(|| {
                inspector
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|inspector| {
                        crate::speech_tasks::inspect_word(
                            &connection,
                            inspector,
                            card.vocabulary_id,
                            now as i64,
                        )
                    })
                    .map(|(audio, hash)| {
                        if let Some(hash) = hash {
                            legacy.push((card.vocabulary_id, hash));
                        }
                        audio.is_some()
                    })
            });
            match found {
                Ok(true) => {
                    verified_audio.insert(card.id);
                }
                Ok(false) => {
                    audio_pending = true;
                    audio_vocabulary_id.get_or_insert(card.vocabulary_id);
                }
                Err(error) => {
                    audio_pending = true;
                    audio_vocabulary_id.get_or_insert(card.vocabulary_id);
                    audio_error = Some(error.clone());
                }
            }
        }
        lexique_storage::register_legacy_speeches(&mut connection, &legacy)
            .map_err(|e| e.to_string())?;
        let mut queue = lexique_storage::review_queue_filtered(
            &mut connection,
            i64::try_from(now).map_err(|e| e.to_string())?,
            &timezone,
            |card| {
                Ok(!skipped.contains(&card.id)
                    && (card.kind != "listening" || verified_audio.contains(&card.id)))
            },
        )
        .map_err(|e| e.to_string())?;
        if let Some(due) = next_audio_due {
            if queue.available_at.is_none_or(|at| due < at) {
                queue.available_at = Some(due);
                if queue.cards.is_empty() && queue.reason.is_none() {
                    queue.reason = Some("due".into());
                }
            }
        }
        let record = queue
            .cards
            .first()
            .map(|card| lexique_storage::get(&connection, card.vocabulary_id))
            .transpose()
            .map_err(|e| e.to_string())?;
        Ok(QueueView {
            queue,
            record,
            audio_pending,
            audio_error,
            audio_vocabulary_id,
        })
    })
    .await
    .map_err(|e| format!("Review queue task failed: {e}"))?
}

#[tauri::command]
pub async fn review_rate(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    request: RateInput,
) -> Result<RateOutcome, String> {
    if request.card_id <= 0
        || request.card_id > 9_007_199_254_740_991
        || request.expected_revision > 9_007_199_254_740_991
    {
        return Err("Invalid card ID or revision".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();
        let now = i64::try_from(now).map_err(|e| e.to_string())?;
        let rating = lexique_storage::RatingRequest {
            operation: request.operation,
            card_id: request.card_id,
            expected_revision: request.expected_revision,
            rating: request.rating,
            now,
            timezone: request.timezone,
        };
        match lexique_storage::rate_card(&mut open(&path)?, &rating) {
            Ok(result) => {
                crate::speech_tasks::changed(&app);
                Ok(RateOutcome::Committed { result })
            }
            Err(error @ lexique_storage::StorageError::RatingConflict) => {
                Ok(RateOutcome::Rejected {
                    code: "conflict".into(),
                    detail: error.to_string(),
                })
            }
            Err(lexique_storage::StorageError::RatingUnavailable(reason)) => {
                Ok(RateOutcome::Rejected {
                    detail: format!("rating unavailable: {reason}"),
                    code: reason,
                })
            }
            Err(lexique_storage::StorageError::Invalid(detail)) => Ok(RateOutcome::Rejected {
                code: "invalid".into(),
                detail,
            }),
            Err(other) => Err(other.to_string()),
        }
    })
    .await
    .map_err(|e| format!("Review rating task failed: {e}"))?
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
        let connection = open(&path)?;
        crate::speech_tasks::cached_word(
            &connection,
            &crate::speech_cache::Inspector::new(&path.0)?,
            vocabulary_id,
            crate::speech_tasks::now()?,
        )
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
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = open(&path)?;
        let inspector = crate::speech_cache::Inspector::new(&path.0)?;
        if !regenerate.unwrap_or(false) {
            if let Some(bytes) = crate::speech_tasks::cached_word(
                &connection,
                &inspector,
                vocabulary_id,
                crate::speech_tasks::now()?,
            )? {
                return Ok(bytes);
            }
        }
        let record = lexique_storage::get(&connection, vocabulary_id).map_err(|e| e.to_string())?;
        let spec = lexique_storage::SpeechSpec::word(
            &record.headword,
            lexique_storage::speech_preferences(&connection)
                .map_err(|e| e.to_string())?
                .voice,
        );
        let key = lexique_storage::request_speech(
            &mut connection,
            &spec,
            Some(vocabulary_id),
            crate::speech_tasks::now()?,
            regenerate.unwrap_or(false),
        )
        .map_err(|e| e.to_string())?;
        drop(connection);
        let tasks = app.state::<crate::speech_tasks::SpeechTasks>();
        tasks.wake()?;
        tasks.wait(key)
    })
    .await
    .map_err(|e| format!("Speech task failed: {e}"))?
}

#[tauri::command]
pub async fn speech_preview(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    text: String,
) -> Result<Vec<u8>, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = open(&path)?;
        let spec = lexique_storage::SpeechSpec::word(
            &text,
            lexique_storage::speech_preferences(&connection)
                .map_err(|e| e.to_string())?
                .voice,
        );
        let key = spec.key().map_err(|e| e.to_string())?;
        let inspector = crate::speech_cache::Inspector::new(&path.0)?;
        if let Some(bytes) = crate::speech_tasks::cached_asset(
            &connection,
            &inspector,
            &key,
            crate::speech_tasks::now()?,
        )? {
            return Ok(bytes);
        }
        lexique_storage::request_speech(
            &mut connection,
            &spec,
            None,
            crate::speech_tasks::now()?,
            false,
        )
        .map_err(|e| e.to_string())?;
        drop(connection);
        let tasks = app.state::<crate::speech_tasks::SpeechTasks>();
        tasks.wake()?;
        tasks.wait(key)
    })
    .await
    .map_err(|e| format!("Speech preview task failed: {e}"))?
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchInput {
    operation: String,
    vocabulary_ids: Vec<i64>,
}

#[tauri::command]
pub async fn review_add_batch(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    request: BatchInput,
) -> Result<lexique_storage::BatchStatus, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let result = lexique_storage::add_review_batch_observed(
            &mut open(&path)?,
            &request.operation,
            &request.vocabulary_ids,
            crate::speech_tasks::now()?,
            || crate::speech_tasks::changed(&app),
        )
        .map_err(|e| e.to_string())?;
        app.state::<crate::speech_tasks::SpeechTasks>().wake()?;
        crate::speech_tasks::changed(&app);
        Ok(result)
    })
    .await
    .map_err(|e| format!("Review batch task failed: {e}"))?
}

#[tauri::command]
pub async fn review_batch_status(
    path: State<'_, DatabasePath>,
    operation: Option<String>,
) -> Result<Option<lexique_storage::BatchStatus>, String> {
    if operation
        .as_ref()
        .is_some_and(|s| s.is_empty() || s.len() > 128)
    {
        return Err("Invalid batch operation".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        match operation {
            Some(operation) => lexique_storage::batch_status(&connection, &operation)
                .map(Some)
                .map_err(|e| e.to_string()),
            None => lexique_storage::latest_batch(&connection).map_err(|e| e.to_string()),
        }
    })
    .await
    .map_err(|e| format!("Review batch status failed: {e}"))?
}

#[tauri::command]
pub async fn review_cancel_batch(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    operation: String,
) -> Result<(), String> {
    if operation.is_empty() || operation.len() > 128 {
        return Err("Invalid batch operation".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        lexique_storage::cancel_batch(&open(&path)?, &operation).map_err(|e| e.to_string())?;
        app.state::<crate::speech_tasks::SpeechTasks>().wake()?;
        crate::speech_tasks::changed(&app);
        Ok(())
    })
    .await
    .map_err(|e| format!("Review cancellation failed: {e}"))?
}

#[tauri::command]
pub async fn speech_retry_missing(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    vocabulary_id: i64,
) -> Result<u32, String> {
    if vocabulary_id <= 0 || vocabulary_id > 9_007_199_254_740_991 {
        return Err("Invalid vocabulary ID".into());
    }
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = open(&path)?;
        let inspector = crate::speech_cache::Inspector::new(&path.0)?;
        let cards =
            lexique_storage::review_cards(&connection, vocabulary_id).map_err(|e| e.to_string())?;
        if !cards.iter().any(|card| card.kind == "listening") {
            return Err("Listening card does not exist".into());
        }
        let cached = crate::speech_tasks::cached_word(
            &connection,
            &inspector,
            vocabulary_id,
            crate::speech_tasks::now()?,
        );
        let replace = match cached {
            Ok(Some(_)) => return Ok(0),
            Ok(None) => false,
            Err(error)
                if error == "Invalid cached speech audio"
                    || error == "Cached speech content checksum mismatch" =>
            {
                true
            }
            Err(error) => return Err(error),
        };
        let record = lexique_storage::get(&connection, vocabulary_id).map_err(|e| e.to_string())?;
        let spec = lexique_storage::SpeechSpec::word(
            &record.headword,
            lexique_storage::speech_preferences(&connection)
                .map_err(|e| e.to_string())?
                .voice,
        );
        lexique_storage::request_speech(
            &mut connection,
            &spec,
            Some(vocabulary_id),
            crate::speech_tasks::now()?,
            replace,
        )
        .map_err(|e| e.to_string())?;
        app.state::<crate::speech_tasks::SpeechTasks>().wake()?;
        crate::speech_tasks::changed(&app);
        Ok(1)
    })
    .await
    .map_err(|e| format!("Missing speech demand failed: {e}"))?
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
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    preferences: Option<lexique_storage::SpeechPreferences>,
) -> Result<lexique_storage::SpeechPreferences, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(preferences) = preferences {
            lexique_storage::set_speech_preferences(&connection, &preferences)
                .map_err(|error| error.to_string())?;
            crate::speech_tasks::changed(&app);
        }
        lexique_storage::speech_preferences(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Speech preference task failed: {error}"))?
}

#[tauri::command]
pub async fn review_settings(
    app: tauri::AppHandle,
    path: State<'_, DatabasePath>,
    settings: Option<lexique_storage::ReviewSettings>,
) -> Result<lexique_storage::ReviewSettings, String> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let connection = open(&path)?;
        if let Some(settings) = settings {
            lexique_storage::set_review_settings(&connection, &settings)
                .map_err(|error| error.to_string())?;
            crate::speech_tasks::changed(&app);
        }
        lexique_storage::review_settings(&connection).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("Review settings task failed: {error}"))?
}

#[tauri::command]
pub fn finish_close(app: tauri::AppHandle) -> Result<(), String> {
    app.state::<crate::speech_tasks::SpeechTasks>().shutdown()?;
    app.get_webview_window("main")
        .ok_or("Main window unavailable")?
        .destroy()
        .map_err(|error| format!("Could not close window: {error}"))
}
