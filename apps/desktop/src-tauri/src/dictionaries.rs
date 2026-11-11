use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use lexique_dictionary::{DictionaryError, Lookup, LookupWarning, Source, mdx};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use crate::vocabulary::{DatabasePath, open};

const KEY: &str = "dictionary-settings";
const MAX_DICTIONARIES: usize = 17;
const BUNDLED_MDX_ID: &str = "builtin-fra-eng";

#[derive(Deserialize)]
struct BundledSource {
    source: Source,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DictionaryConfig {
    id: String,
    title: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    next_id: u32,
    dictionaries: Vec<DictionaryConfig>,
}

fn failure(code: &'static str, message: impl std::fmt::Display) -> DictionaryError {
    DictionaryError {
        code,
        message: message.to_string(),
    }
}

fn valid_id(id: &str) -> bool {
    matches!(id, "builtin" | BUNDLED_MDX_ID)
        || id
            .strip_prefix("mdx-")
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|value| value > 0 && id == format!("mdx-{value}"))
}

fn read(connection: &Connection) -> Result<Settings, DictionaryError> {
    let json: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| failure("query_failed", error))?;
    let mut settings: Settings = match json {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| failure("corrupt", format!("Invalid dictionary settings: {error}")))?,
        None => Settings {
            next_id: 1,
            dictionaries: vec![DictionaryConfig {
                id: "builtin".into(),
                title: "FreeDict fra-zho".into(),
            }],
        },
    };
    if !settings
        .dictionaries
        .iter()
        .any(|item| item.id == BUNDLED_MDX_ID)
    {
        settings.dictionaries.push(DictionaryConfig {
            id: BUNDLED_MDX_ID.into(),
            title: "FreeDict fra-eng 0.4.1".into(),
        });
    }
    let items = &settings.dictionaries;
    if items.is_empty()
        || items.len() > MAX_DICTIONARIES
        || settings.next_id == 0
        || items.iter().filter(|item| item.id == "builtin").count() != 1
        || items.iter().enumerate().any(|(index, item)| {
            !valid_id(&item.id)
                || item.title.trim().is_empty()
                || items[..index].iter().any(|other| other.id == item.id)
                || item
                    .id
                    .strip_prefix("mdx-")
                    .is_some_and(|id| id.parse::<u32>().unwrap_or(u32::MAX) >= settings.next_id)
        })
    {
        return Err(failure("corrupt", "Invalid dictionary settings"));
    }
    Ok(settings)
}

fn write(connection: &Connection, settings: &Settings) -> Result<(), DictionaryError> {
    let json = serde_json::to_string(settings).map_err(|error| failure("query_failed", error))?;
    connection.execute("INSERT INTO app_metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [KEY, &json])
        .map_err(|error| failure("query_failed", format!("Could not save dictionary settings: {error}")))?;
    Ok(())
}

fn settings(
    path: &DatabasePath,
    order: Option<Vec<String>>,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let Some(order) = order else {
        return Ok(read(&connection)?.dictionaries);
    };
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut settings = read(&transaction)?;
    if order.len() != settings.dictionaries.len()
        || order.iter().enumerate().any(|(index, id)| {
            order[..index].contains(id) || !settings.dictionaries.iter().any(|item| &item.id == id)
        })
    {
        return Err(failure(
            "invalid_query",
            "Order must include each registered dictionary exactly once",
        ));
    }
    settings
        .dictionaries
        .sort_by_key(|item| order.iter().position(|id| id == &item.id).unwrap());
    write(&transaction, &settings)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    Ok(settings.dictionaries)
}

fn custom_path(path: &DatabasePath, id: &str) -> Result<PathBuf, DictionaryError> {
    if !valid_id(id) || !id.starts_with("mdx-") {
        return Err(failure("invalid_query", "Invalid custom dictionary ID"));
    }
    Ok(path
        .0
        .parent()
        .ok_or_else(|| failure("query_failed", "Learning directory is unavailable"))?
        .join("dictionaries")
        .join(format!("{id}.mdx")))
}

fn import(
    path: &DatabasePath,
    title: &str,
    bytes: &[u8],
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    if !title.to_lowercase().ends_with(".mdx")
        || title.chars().count() > 200
        || title.contains(['/', '\\'])
        || title.chars().any(char::is_control)
    {
        return Err(failure(
            "invalid_query",
            "Select an MDX file with a plain filename",
        ));
    }
    if bytes.is_empty() || bytes.len() > mdx::MAX_FILE_BYTES {
        return Err(failure(
            "limit",
            "MDX must be non-empty and at most 128 MiB",
        ));
    }
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut preferences = read(&transaction)?;
    if preferences.dictionaries.len() >= MAX_DICTIONARIES {
        return Err(failure(
            "limit",
            "Maximum 17 dictionaries, including two bundled dictionaries",
        ));
    }
    let id = format!("mdx-{}", preferences.next_id);
    preferences.next_id = preferences
        .next_id
        .checked_add(1)
        .ok_or_else(|| failure("limit", "Dictionary ID limit reached"))?;
    // Reserve before writing so an interrupted import cannot overwrite its retained file.
    write(&transaction, &preferences)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    let file_path = custom_path(path, &id)?;
    fs::create_dir_all(file_path.parent().unwrap())
        .map_err(|error| failure("query_failed", error))?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file_path)
        .map_err(|error| failure("query_failed", error))?;
    let result = (|| {
        let mut file = file;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| failure("query_failed", error))?;
        drop(file);
        mdx::Dictionary::open(&file_path, &id, title)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| failure("query_failed", error))?;
        let mut preferences = read(&transaction)?;
        if preferences.dictionaries.len() >= MAX_DICTIONARIES {
            return Err(failure(
                "limit",
                "Maximum 17 dictionaries, including two bundled dictionaries",
            ));
        }
        preferences.dictionaries.push(DictionaryConfig {
            id,
            title: title.into(),
        });
        write(&transaction, &preferences)?;
        transaction
            .commit()
            .map_err(|error| failure("query_failed", error))?;
        Ok(preferences.dictionaries)
    })();
    if let Err(cause) = &result {
        fs::remove_file(&file_path).map_err(|error| {
            failure(
                "query_failed",
                format!("{cause}; could not remove the failed import's new copy: {error}"),
            )
        })?;
    }
    result
}

fn remove(path: &DatabasePath, id: &str) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    custom_path(path, id)?;
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut settings = read(&transaction)?;
    if !settings.dictionaries.iter().any(|item| item.id == id) {
        return Err(failure("missing", "Dictionary is not registered"));
    }
    settings.dictionaries.retain(|item| item.id != id);
    write(&transaction, &settings)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    Ok(settings.dictionaries)
}

#[derive(PartialEq)]
struct Stamp {
    length: u64,
    modified: SystemTime,
}
struct Cached {
    id: String,
    stamp: Stamp,
    dictionary: mdx::Dictionary,
}
#[derive(Default)]
pub struct Registry(Mutex<Vec<Cached>>);

impl Registry {
    fn lookup_source(
        &self,
        path: &DatabasePath,
        builtin: &Path,
        config: &DictionaryConfig,
        key: &str,
    ) -> Result<Lookup, DictionaryError> {
        if config.id == "builtin" {
            let mut result =
                lexique_dictionary::lookup(builtin, key, env!("LEXIQUE_DICTIONARY_SHA256"))?;
            result.dictionary_id = Some("builtin".into());
            return Ok(result);
        }
        let file_path = if config.id == BUNDLED_MDX_ID {
            builtin
                .with_file_name("freedict-fra-eng-mdx")
                .join("freedict-fra-eng-0.4.1.mdx")
        } else {
            custom_path(path, &config.id)?
        };
        let mut cache = self
            .0
            .lock()
            .map_err(|error| failure("query_failed", error))?;
        let metadata = fs::metadata(&file_path).map_err(|error| {
            cache.retain(|item| item.id != config.id);
            failure(
                if error.kind() == std::io::ErrorKind::NotFound {
                    "missing"
                } else {
                    "query_failed"
                },
                format!("{}: {error}", config.title),
            )
        })?;
        let stamp = Stamp {
            length: metadata.len(),
            modified: metadata
                .modified()
                .map_err(|error| failure("query_failed", error))?,
        };
        let existing = cache
            .iter()
            .position(|item| item.id == config.id && item.stamp == stamp)
            .map(|index| cache.remove(index));
        cache.retain(|item| item.id != config.id);
        if cache.len() == 2 {
            cache.remove(0);
        }
        let cached = match existing {
            Some(cached) => cached,
            None => Cached {
                id: config.id.clone(),
                stamp,
                dictionary: mdx::Dictionary::open(&file_path, &config.id, &config.title)?,
            },
        };
        let result = cached.dictionary.lookup(key);
        cache.push(cached);
        let mut result = result?;
        if config.id == BUNDLED_MDX_ID {
            let provenance: BundledSource = serde_json::from_str(include_str!(
                "../../../../resources/freedict-fra-eng-mdx-0.4.1/manifest.json"
            ))
            .map_err(|error| failure("corrupt", error))?;
            if result.sha256 != provenance.sha256 {
                return Err(failure(
                    "corrupt",
                    "Bundled FreeDict MDX differs from its admitted source",
                ));
            }
            result.source = provenance.source;
        }
        Ok(result)
    }

    fn lookup(
        &self,
        path: &DatabasePath,
        builtin: &Path,
        key: &str,
        id: Option<&str>,
    ) -> Result<Lookup, DictionaryError> {
        if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
            return Err(failure(
                "invalid_query",
                "Enter 1–512 UTF-8 bytes without control characters",
            ));
        }
        let catalog = settings(path, None)?;
        if let Some(id) = id {
            let config = catalog
                .iter()
                .find(|item| item.id == id)
                .ok_or_else(|| failure("missing", "Dictionary is not registered"))?;
            return self.lookup_source(path, builtin, config, key);
        }
        let mut warnings = Vec::new();
        for config in &catalog {
            match self.lookup_source(path, builtin, config, key) {
                Ok(mut lookup) => {
                    lookup.warnings = warnings;
                    return Ok(lookup);
                }
                Err(error) => warnings.push(LookupWarning {
                    dictionary_id: config.id.clone(),
                    code: error.code,
                    message: error.message,
                }),
            }
        }
        Err(failure(
            warnings[0].code,
            warnings
                .iter()
                .map(|item| format!("{}: {}", item.dictionary_id, item.message))
                .collect::<Vec<_>>()
                .join("; "),
        ))
    }
}

pub(crate) fn lookup_from_app(
    app: &tauri::AppHandle,
    key: &str,
    id: Option<&str>,
) -> Result<Lookup, DictionaryError> {
    let builtin = app
        .path()
        .resource_dir()
        .map_err(|error| failure("query_failed", error))?
        .join("dictionary/dictionary.sqlite3");
    app.state::<Registry>()
        .lookup(&app.state::<DatabasePath>(), &builtin, key, id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn dictionary_lookup(
    app: tauri::AppHandle,
    key: String,
    dictionary_id: Option<String>,
) -> Result<Lookup, DictionaryError> {
    tauri::async_runtime::spawn_blocking(move || {
        lookup_from_app(&app, &key, dictionary_id.as_deref())
    })
    .await
    .map_err(|error| failure("query_failed", format!("Dictionary task failed: {error}")))?
}

#[tauri::command]
pub async fn dictionary_settings(
    path: State<'_, DatabasePath>,
    order: Option<Vec<String>>,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    let path = DatabasePath(path.0.clone());
    tauri::async_runtime::spawn_blocking(move || settings(&path, order))
        .await
        .map_err(|error| failure("query_failed", error))?
}

#[tauri::command]
pub async fn dictionary_import(
    app: tauri::AppHandle,
    request: tauri::ipc::Request<'_>,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    let encoded = request
        .headers()
        .get("x-lexique-dictionary-name")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() <= 2048)
        .ok_or_else(|| failure("invalid_query", "Dictionary filename is missing"))?;
    let title = String::from_utf8(
        STANDARD
            .decode(encoded)
            .map_err(|error| failure("invalid_query", error))?,
    )
    .map_err(|error| failure("invalid_query", error))?;
    let bytes = match request.body() {
        tauri::ipc::InvokeBody::Raw(bytes) if bytes.len() <= mdx::MAX_FILE_BYTES => bytes.clone(),
        _ => {
            return Err(failure(
                "limit",
                "MDX import requires at most 128 MiB of binary data",
            ));
        }
    };
    let path = DatabasePath(app.state::<DatabasePath>().0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let registry = app.state::<Registry>();
        let mut cache = registry
            .0
            .lock()
            .map_err(|error| failure("query_failed", error))?;
        if cache.len() == 2 {
            cache.remove(0);
        }
        import(&path, &title, &bytes)
    })
    .await
    .map_err(|error| failure("query_failed", error))?
}

#[tauri::command]
pub async fn dictionary_remove(
    app: tauri::AppHandle,
    id: String,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    let path = DatabasePath(app.state::<DatabasePath>().0.clone());
    tauri::async_runtime::spawn_blocking(move || {
        let result = remove(&path, &id)?;
        app.state::<Registry>()
            .0
            .lock()
            .map_err(|error| failure("query_failed", error))?
            .retain(|item| item.id != id);
        Ok(result)
    })
    .await
    .map_err(|error| failure("query_failed", error))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn fixture(name: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../tools/mdict-probe/tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    fn database_root(nonce: u128) -> PathBuf {
        static NEXT_DATABASE_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/issue-9/registry-tests")
            .join(format!("{}-{nonce}-{id}", std::process::id()))
    }

    #[test]
    fn database_roots_stay_unique_when_clock_values_repeat() {
        assert_ne!(database_root(0), database_root(0));
    }

    fn database() -> DatabasePath {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = database_root(nonce);
        fs::create_dir_all(&root).unwrap();
        let path = DatabasePath(root.join("lexique.sqlite3"));
        lexique_storage::initialize(&Connection::open(&path.0).unwrap()).unwrap();
        path
    }

    #[test]
    fn imports_restore_order_isolate_errors_and_preserve_files_and_learning_records() {
        let path = database();
        let mut connection = open(&path).unwrap();
        let snapshot: lexique_storage::Snapshot = serde_json::from_value(serde_json::json!({
            "reference":{"dictionary":"legacy","lookup_key":"école","version":"1","attribution":"original","license":"MIT", "source_url":"https://example.org","license_url":"https://example.org","sha256":"a".repeat(64)},
            "digest":[{"pronunciations":[],"grammar":["n"],"translation":"original school"}]})).unwrap();
        let request: lexique_storage::SaveRequest = serde_json::from_value(serde_json::json!({"operation":"preserved-note", "headword":"école", "notes":"keep my note",
            "example_fr":"", "example_zh":"", "example_sense":null, "more_examples":[], "id":null, "revision":null})).unwrap();
        let record = lexique_storage::save(&mut connection, &request, Some(snapshot))
            .unwrap()
            .record;
        let preserved = serde_json::to_value(&record).unwrap();
        let first = fixture("v2-utf8-none.mdx");
        let second = fixture("v1-utf16-none.mdx");
        import(&path, "first.mdx", &first).unwrap();
        import(&path, "第二本.mdx", &second).unwrap();
        let order = vec![
            "mdx-1".into(),
            "mdx-2".into(),
            "builtin".into(),
            BUNDLED_MDX_ID.into(),
        ];
        let catalog = settings(&path, Some(order.clone())).unwrap();
        assert_eq!(settings(&path, None).unwrap(), catalog);
        assert_eq!(
            settings(&path, Some(vec!["mdx-1".into(); 4]))
                .unwrap_err()
                .code,
            "invalid_query"
        );
        assert_eq!(settings(&path, None).unwrap(), catalog);
        let registry = Registry::default();
        let builtin = path.0.with_file_name("missing-resource.sqlite3");
        let initial = registry.lookup(&path, &builtin, "double", None).unwrap();
        assert_eq!(initial.dictionary_id.as_deref(), Some("mdx-1"));
        assert!(
            registry
                .lookup(&path, &builtin, "not-in-either", None)
                .unwrap()
                .entries
                .is_empty()
        );
        let first_path = custom_path(&path, "mdx-1").unwrap();
        fs::write(&first_path, b"damaged MDX").unwrap();
        assert_eq!(
            registry
                .lookup(&path, &builtin, "double", Some("mdx-1"))
                .unwrap_err()
                .code,
            "corrupt"
        );
        let fallback = registry.lookup(&path, &builtin, "double", None).unwrap();
        assert_eq!(fallback.dictionary_id.as_deref(), Some("mdx-2"));
        assert_eq!(fallback.warnings[0].dictionary_id, "mdx-1");
        fs::write(&first_path, &second).unwrap();
        assert_ne!(
            registry
                .lookup(&path, &builtin, "double", None)
                .unwrap()
                .sha256,
            initial.sha256
        );
        remove(&path, "mdx-1").unwrap();
        assert_eq!(fs::read(&first_path).unwrap(), second);
        assert_eq!(
            Registry::default()
                .lookup(&path, &builtin, "double", Some("mdx-1"))
                .unwrap_err()
                .code,
            "missing"
        );
        assert_eq!(
            import(&path, "bad.mdx", b"bad").unwrap_err().code,
            "corrupt"
        );
        assert!(!custom_path(&path, "mdx-3").unwrap().exists());
        let final_catalog = import(&path, "third.mdx", &first).unwrap();
        assert_eq!(final_catalog.last().unwrap().id, "mdx-4");
        assert_eq!(settings(&path, None).unwrap(), final_catalog);
        assert_eq!(
            serde_json::to_value(lexique_storage::get(&open(&path).unwrap(), record.id).unwrap())
                .unwrap(),
            preserved
        );
        assert_eq!(lexique_storage::initialize(&connection).unwrap(), 17);
    }

    #[test]
    fn a_full_legacy_catalog_keeps_fifteen_custom_dictionaries_and_their_order() {
        let path = database();
        let connection = open(&path).unwrap();
        let bytes = fixture("v2-utf8-none.mdx");
        let mut original: Vec<_> = (1..=15)
            .map(|index| DictionaryConfig {
                id: format!("mdx-{index}"),
                title: format!("{index}.mdx"),
            })
            .collect();
        original.insert(
            7,
            DictionaryConfig {
                id: "builtin".into(),
                title: "FreeDict fra-zho".into(),
            },
        );
        fs::create_dir_all(custom_path(&path, "mdx-1").unwrap().parent().unwrap()).unwrap();
        for item in original.iter().filter(|item| item.id.starts_with("mdx-")) {
            fs::write(custom_path(&path, &item.id).unwrap(), &bytes).unwrap();
        }
        write(
            &connection,
            &Settings {
                next_id: 16,
                dictionaries: original.clone(),
            },
        )
        .unwrap();
        let upgraded = settings(&path, None).unwrap();
        assert_eq!(&upgraded[..16], &original);
        assert_eq!(upgraded[16].id, BUNDLED_MDX_ID);
        assert_eq!(
            import(&path, "over-limit.mdx", &bytes).unwrap_err().code,
            "limit"
        );
        let order = upgraded.iter().map(|item| item.id.clone()).rev().collect();
        let reordered = settings(&path, Some(order)).unwrap();
        assert_eq!(settings(&path, None).unwrap(), reordered);
        for item in original.iter().filter(|item| item.id.starts_with("mdx-")) {
            assert_eq!(
                fs::read(custom_path(&path, &item.id).unwrap()).unwrap(),
                bytes
            );
        }
        assert_eq!(lexique_storage::initialize(&connection).unwrap(), 17);
    }

    #[test]
    fn bundled_lookup_checks_bytes_before_assigning_freedict_provenance() {
        let path = database();
        let builtin = path
            .0
            .parent()
            .unwrap()
            .join("dictionary/dictionary.sqlite3");
        let mdx_path = builtin
            .with_file_name("freedict-fra-eng-mdx")
            .join("freedict-fra-eng-0.4.1.mdx");
        fs::create_dir_all(mdx_path.parent().unwrap()).unwrap();
        let original = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../resources/freedict-fra-eng-mdx-0.4.1/freedict-fra-eng-0.4.1.mdx"),
        )
        .unwrap();
        fs::write(&mdx_path, &original).unwrap();
        let registry = Registry::default();
        let lookup = registry
            .lookup(&path, &builtin, "école", Some(BUNDLED_MDX_ID))
            .unwrap();
        assert_eq!(lookup.dictionary_id.as_deref(), Some(BUNDLED_MDX_ID));
        assert_eq!(lookup.source.version, "0.4.1");
        assert_eq!(lookup.source.license.as_deref(), Some("GPL-2.0-or-later"));
        assert!(lookup.entries.iter().any(|entry| {
            entry
                .html
                .as_deref()
                .is_some_and(|html| html.contains("school"))
        }));
        fs::write(&mdx_path, b"damaged MDX").unwrap();
        assert_eq!(
            registry
                .lookup(&path, &builtin, "école", Some(BUNDLED_MDX_ID))
                .unwrap_err()
                .code,
            "corrupt"
        );
        fs::remove_file(&mdx_path).unwrap();
        assert_eq!(
            registry
                .lookup(&path, &builtin, "école", Some(BUNDLED_MDX_ID))
                .unwrap_err()
                .code,
            "missing"
        );
        fs::write(&mdx_path, fixture("v2-utf8-none.mdx")).unwrap();
        let replaced = registry
            .lookup(&path, &builtin, "double", Some(BUNDLED_MDX_ID))
            .unwrap_err();
        assert_eq!(replaced.code, "corrupt");
        assert!(replaced.message.contains("admitted source"));
        fs::write(&mdx_path, &original).unwrap();
        let restored = Registry::default()
            .lookup(&path, &builtin, "école", Some(BUNDLED_MDX_ID))
            .unwrap();
        assert_eq!(
            serde_json::to_value(restored.source).unwrap(),
            serde_json::to_value(lookup.source).unwrap()
        );
    }

    #[test]
    fn bundled_dictionaries_cannot_be_removed() {
        let path = database();
        let before = settings(&path, None).unwrap();
        for id in ["builtin", BUNDLED_MDX_ID] {
            assert_eq!(remove(&path, id).unwrap_err().code, "invalid_query");
        }
        assert_eq!(settings(&path, None).unwrap(), before);
    }

    #[test]
    fn import_limits_and_invalid_filenames_leave_the_catalog_unchanged() {
        let path = database();
        let bytes = fixture("v2-utf8-none.mdx");
        assert_eq!(
            import(&path, "../outside.mdx", &bytes).unwrap_err().code,
            "invalid_query"
        );
        assert_eq!(import(&path, "empty.mdx", &[]).unwrap_err().code, "limit");
        assert_eq!(remove(&path, "builtin").unwrap_err().code, "invalid_query");
        for index in 0..15 {
            import(&path, &format!("{index}.mdx"), &bytes).unwrap();
        }
        let before = settings(&path, None).unwrap();
        assert_eq!(before.len(), 17);
        assert_eq!(
            import(&path, "too-many.mdx", &bytes).unwrap_err().code,
            "limit"
        );
        assert_eq!(settings(&path, None).unwrap(), before);
    }
}
