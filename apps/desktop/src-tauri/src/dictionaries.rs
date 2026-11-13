use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use lexique_dictionary::{DictionaryError, Lookup, LookupWarning, Source, mdx};
use lexique_storage::{DictionaryCatalog, DictionaryCatalogEntry, custom_dictionary_number};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

use crate::vocabulary::{DatabasePath, open};

const KEY: &str = lexique_storage::DICTIONARY_CATALOG_KEY;
const FILES_KEY: &str = "dictionary-files";
const MAX_DICTIONARIES: usize = 17;
const BUNDLED_MDX_ID: &str = "builtin-fra-eng";

#[derive(Deserialize)]
struct BundledSource {
    source: Source,
    sha256: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DictionaryConfig {
    id: String,
    title: String,
    content_hash: Option<String>,
    availability: &'static str,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ManagedFile {
    id: String,
    file_id: u32,
    content_hash: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeviceFiles {
    files: Vec<ManagedFile>,
    bindings: BTreeMap<String, u32>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelinkResult {
    Linked {
        catalog: Vec<DictionaryConfig>,
    },
    ConfirmationRequired {
        dictionary_id: String,
        expected_hash: Option<String>,
        content_hash: String,
    },
}

fn failure(code: &'static str, message: impl std::fmt::Display) -> DictionaryError {
    DictionaryError {
        code,
        message: message.to_string(),
    }
}

fn valid_id(id: &str) -> bool {
    matches!(id, "builtin" | BUNDLED_MDX_ID) || custom_dictionary_number(id).is_some()
}

fn read(connection: &Connection) -> Result<DictionaryCatalog, DictionaryError> {
    lexique_storage::read_dictionary_catalog(connection).map_err(|error| {
        failure(
            if matches!(error, lexique_storage::StorageError::Database(_)) {
                "query_failed"
            } else {
                "corrupt"
            },
            format!("Could not read dictionary catalog: {error}"),
        )
    })
}

fn write(connection: &Connection, catalog: &DictionaryCatalog) -> Result<(), DictionaryError> {
    lexique_storage::write_dictionary_catalog(connection, catalog).map_err(|error| {
        failure(
            "query_failed",
            format!("Could not save dictionary catalog: {error}"),
        )
    })
}

fn read_files(connection: &Connection) -> Result<DeviceFiles, DictionaryError> {
    let json: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [FILES_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| failure("query_failed", error))?;
    let files: DeviceFiles = match json {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| failure("corrupt", format!("Invalid dictionary files: {error}")))?,
        None => DeviceFiles::default(),
    };
    if files.files.iter().enumerate().any(|(index, file)| {
        custom_dictionary_number(&file.id).is_none()
            || file.file_id == 0
            || files.files[..index]
                .iter()
                .any(|other| other.file_id == file.file_id)
            || file.content_hash.as_ref().is_some_and(|hash| {
                hash.len() != 64
                    || !hash
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            })
    }) || files.bindings.iter().any(|(id, file_id)| {
        custom_dictionary_number(id).is_none()
            || !files.files.iter().any(|file| file.file_id == *file_id)
    }) {
        return Err(failure("corrupt", "Invalid dictionary file associations"));
    }
    Ok(files)
}

fn write_files(connection: &Connection, files: &DeviceFiles) -> Result<(), DictionaryError> {
    let json = serde_json::to_string(files).map_err(|error| failure("query_failed", error))?;
    connection.execute("INSERT INTO app_metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [FILES_KEY, &json])
        .map_err(|error| failure("query_failed", format!("Could not save dictionary files: {error}")))?;
    Ok(())
}

fn legacy_catalog(connection: &Connection) -> Result<bool, DictionaryError> {
    let json: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| failure("query_failed", error))?;
    let value: Option<serde_json::Value> = json
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|error| failure("corrupt", error))?;
    Ok(value.is_some_and(|value| value.get("dictionaries").is_some()))
}

fn ensure_device_catalog(path: &DatabasePath) -> Result<(), DictionaryError> {
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    if !legacy_catalog(&connection)? {
        return Ok(());
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    if !legacy_catalog(&transaction)? {
        return Ok(());
    }
    let mut catalog = read(&transaction)?;
    let mut files = read_files(&transaction)?;
    for entry in &mut catalog.entries {
        let Some(file_id) = custom_dictionary_number(&entry.id) else {
            continue;
        };
        let hash = mdx::Dictionary::open(&custom_path(path, &entry.id)?, &entry.id, &entry.title)
            .ok()
            .map(|dictionary| dictionary.content_hash().to_owned());
        entry.content_hash = hash.clone();
        if !files.files.iter().any(|file| file.file_id == file_id) {
            files.files.push(ManagedFile {
                id: entry.id.clone(),
                file_id,
                content_hash: hash.clone(),
            });
        }
        if hash.is_some() {
            files.bindings.insert(entry.id.clone(), file_id);
        }
    }
    write_files(&transaction, &files)?;
    write(&transaction, &catalog)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))
}

pub(crate) fn portable_catalog(path: &DatabasePath) -> Result<DictionaryCatalog, DictionaryError> {
    ensure_device_catalog(path)?;
    read(&open(path).map_err(|error| failure("query_failed", error))?)
}

pub(crate) fn restore_catalog(
    connection: &mut Connection,
    retained_path: &DatabasePath,
    catalog: &DictionaryCatalog,
) -> Result<(), DictionaryError> {
    lexique_storage::validate_dictionary_catalog(catalog)
        .map_err(|error| failure("corrupt", error))?;
    ensure_device_catalog(retained_path)?;
    let mut files =
        read_files(&open(retained_path).map_err(|error| failure("query_failed", error))?)?;
    files.bindings.clear();
    for entry in &catalog.entries {
        let Some(hash) = &entry.content_hash else {
            continue;
        };
        for file in &files.files {
            if file.content_hash.as_ref() != Some(hash) {
                continue;
            }
            let physical = custom_path(retained_path, &format!("mdx-{}", file.file_id))?;
            if mdx::Dictionary::open(&physical, &entry.id, &entry.title)
                .is_ok_and(|dictionary| dictionary.content_hash() == hash)
            {
                files.bindings.insert(entry.id.clone(), file.file_id);
                break;
            }
        }
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    write_files(&transaction, &files)?;
    write(&transaction, catalog)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))
}

fn bound_file<'a>(
    files: &'a DeviceFiles,
    entry: &DictionaryCatalogEntry,
) -> Option<&'a ManagedFile> {
    let hash = entry.content_hash.as_ref()?;
    let file_id = files.bindings.get(&entry.id)?;
    files
        .files
        .iter()
        .find(|file| file.file_id == *file_id && file.content_hash.as_ref() == Some(hash))
}

fn catalog_status(
    path: &DatabasePath,
    catalog: &DictionaryCatalog,
    files: &DeviceFiles,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    catalog
        .entries
        .iter()
        .map(|entry| {
            let available = if entry.id.starts_with("builtin") {
                true
            } else if let Some(file) = bound_file(files, entry) {
                match fs::metadata(custom_path(path, &format!("mdx-{}", file.file_id))?) {
                    Ok(_) => true,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                    Err(error) => return Err(failure("query_failed", error)),
                }
            } else {
                false
            };
            Ok(DictionaryConfig {
                id: entry.id.clone(),
                title: entry.title.clone(),
                content_hash: entry.content_hash.clone(),
                availability: if available { "available" } else { "missing" },
            })
        })
        .collect()
}

fn settings(
    path: &DatabasePath,
    order: Option<Vec<String>>,
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    ensure_device_catalog(path)?;
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let Some(order) = order else {
        return catalog_status(path, &read(&connection)?, &read_files(&connection)?);
    };
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut settings = read(&transaction)?;
    if order.len() != settings.entries.len()
        || order.iter().enumerate().any(|(index, id)| {
            order[..index].contains(id) || !settings.entries.iter().any(|item| &item.id == id)
        })
    {
        return Err(failure(
            "invalid_query",
            "Order must include each registered dictionary exactly once",
        ));
    }
    settings
        .entries
        .sort_by_key(|item| order.iter().position(|id| id == &item.id).unwrap());
    write(&transaction, &settings)?;
    let catalog = catalog_status(path, &settings, &read_files(&transaction)?)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    Ok(catalog)
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

fn reserve_id(
    path: &DatabasePath,
    connection: &Connection,
    catalog: &mut DictionaryCatalog,
) -> Result<u32, DictionaryError> {
    let files = read_files(connection)?;
    let mut highest = catalog.next_id - 1;
    for number in catalog
        .entries
        .iter()
        .filter_map(|entry| custom_dictionary_number(&entry.id))
        .chain(
            files
                .files
                .iter()
                .flat_map(|file| [custom_dictionary_number(&file.id).unwrap(), file.file_id]),
        )
        .chain(
            files
                .bindings
                .keys()
                .filter_map(|id| custom_dictionary_number(id)),
        )
        .chain(
            lexique_storage::referenced_custom_dictionary_ids(connection)
                .map_err(|error| failure("query_failed", error))?,
        )
    {
        highest = highest.max(number);
    }
    let directory = custom_path(path, "mdx-1")?.parent().unwrap().to_owned();
    match fs::read_dir(directory) {
        Ok(entries) => {
            for entry in entries {
                let filename = entry
                    .map_err(|error| failure("query_failed", error))?
                    .file_name()
                    .to_string_lossy()
                    .to_lowercase();
                if let Some(number) = filename
                    .strip_suffix(".mdx")
                    .and_then(custom_dictionary_number)
                {
                    highest = highest.max(number);
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(failure("query_failed", error)),
    }
    let number = highest
        .checked_add(1)
        .ok_or_else(|| failure("limit", "Dictionary ID limit reached"))?;
    catalog.next_id = number
        .checked_add(1)
        .ok_or_else(|| failure("limit", "Dictionary ID limit reached"))?;
    Ok(number)
}

fn import(
    path: &DatabasePath,
    title: &str,
    bytes: &[u8],
) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    match import_file(path, title, bytes, None, None)? {
        RelinkResult::Linked { catalog } => Ok(catalog),
        RelinkResult::ConfirmationRequired { .. } => {
            unreachable!("New imports do not replace a registered dictionary")
        }
    }
}

fn relink(
    path: &DatabasePath,
    id: &str,
    title: &str,
    bytes: &[u8],
    confirmed_hash: Option<&str>,
) -> Result<RelinkResult, DictionaryError> {
    if custom_dictionary_number(id).is_none() {
        return Err(failure(
            "invalid_query",
            "Select a registered custom dictionary",
        ));
    }
    if confirmed_hash.is_some_and(|hash| {
        hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return Err(failure(
            "invalid_query",
            "Invalid confirmed dictionary content identity",
        ));
    }
    import_file(path, title, bytes, Some(id), confirmed_hash)
}

fn import_file(
    path: &DatabasePath,
    title: &str,
    bytes: &[u8],
    relink_id: Option<&str>,
    confirmed_hash: Option<&str>,
) -> Result<RelinkResult, DictionaryError> {
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
    ensure_device_catalog(path)?;
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut preferences = read(&transaction)?;
    let original = relink_id
        .map(|id| {
            preferences
                .entries
                .iter()
                .find(|entry| entry.id == id)
                .cloned()
                .ok_or_else(|| failure("missing", "Dictionary is not registered"))
        })
        .transpose()?;
    if original.is_none() && preferences.entries.len() >= MAX_DICTIONARIES {
        return Err(failure(
            "limit",
            "Maximum 17 dictionaries, including two bundled dictionaries",
        ));
    }
    let file_id = reserve_id(path, &transaction, &mut preferences)?;
    let id = original
        .as_ref()
        .map(|entry| entry.id.clone())
        .unwrap_or_else(|| format!("mdx-{file_id}"));
    // Reserve before writing so an interrupted import cannot overwrite its retained file.
    write(&transaction, &preferences)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    let file_path = custom_path(path, &format!("mdx-{file_id}"))?;
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
        let content_hash = mdx::Dictionary::open(&file_path, &id, title)?
            .content_hash()
            .to_owned();
        if confirmed_hash.is_some_and(|hash| hash != content_hash) {
            return Err(failure(
                "invalid_query",
                "Selected MDX differs from the confirmed file; select it again",
            ));
        }
        if let Some(entry) = &original {
            if entry.content_hash.as_ref() != Some(&content_hash) && confirmed_hash.is_none() {
                return Ok(RelinkResult::ConfirmationRequired {
                    dictionary_id: id.clone(),
                    expected_hash: entry.content_hash.clone(),
                    content_hash,
                });
            }
        }
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| failure("query_failed", error))?;
        let mut preferences = read(&transaction)?;
        if original.is_none() && preferences.entries.len() >= MAX_DICTIONARIES {
            return Err(failure(
                "limit",
                "Maximum 17 dictionaries, including two bundled dictionaries",
            ));
        }
        let mut files = read_files(&transaction)?;
        files.files.push(ManagedFile {
            id: id.clone(),
            file_id,
            content_hash: Some(content_hash.clone()),
        });
        files.bindings.insert(id.clone(), file_id);
        if let Some(original) = &original {
            let entry = preferences
                .entries
                .iter_mut()
                .find(|entry| entry.id == id)
                .filter(|entry| {
                    entry.content_hash == original.content_hash && entry.title == original.title
                })
                .ok_or_else(|| {
                    failure(
                        "query_failed",
                        "Dictionary registration changed; select its file again",
                    )
                })?;
            entry.content_hash = Some(content_hash);
        } else {
            preferences.entries.push(DictionaryCatalogEntry {
                id,
                title: title.into(),
                content_hash: Some(content_hash),
            });
        }
        let catalog = catalog_status(path, &preferences, &files)?;
        write_files(&transaction, &files)?;
        write(&transaction, &preferences)?;
        transaction
            .commit()
            .map_err(|error| failure("query_failed", error))?;
        Ok(RelinkResult::Linked { catalog })
    })();
    if !matches!(&result, Ok(RelinkResult::Linked { .. })) {
        let context = result
            .as_ref()
            .err()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Dictionary confirmation is required".into());
        fs::remove_file(&file_path).map_err(|error| {
            failure(
                "query_failed",
                format!("{context}; could not remove the unregistered MDX copy: {error}"),
            )
        })?;
    }
    result
}

fn remove(path: &DatabasePath, id: &str) -> Result<Vec<DictionaryConfig>, DictionaryError> {
    custom_path(path, id)?;
    ensure_device_catalog(path)?;
    let mut connection = open(path).map_err(|error| failure("query_failed", error))?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| failure("query_failed", error))?;
    let mut settings = read(&transaction)?;
    if !settings.entries.iter().any(|item| item.id == id) {
        return Err(failure("missing", "Dictionary is not registered"));
    }
    settings.entries.retain(|item| item.id != id);
    write(&transaction, &settings)?;
    let catalog = catalog_status(path, &settings, &read_files(&transaction)?)?;
    transaction
        .commit()
        .map_err(|error| failure("query_failed", error))?;
    Ok(catalog)
}

#[derive(PartialEq)]
struct Stamp {
    path: PathBuf,
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
        config: &DictionaryCatalogEntry,
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
            let connection = open(path).map_err(|error| failure("query_failed", error))?;
            let files = read_files(&connection)?;
            let file = bound_file(&files, config).ok_or_else(|| {
                failure(
                    "missing",
                    format!("{}: select its MDX file in settings", config.title),
                )
            })?;
            custom_path(path, &format!("mdx-{}", file.file_id))?
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
            path: file_path.clone(),
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
        if config
            .content_hash
            .as_ref()
            .is_some_and(|hash| *hash != result.sha256)
        {
            return Err(failure(
                "corrupt",
                "Dictionary file has changed; select and confirm its MDX file in settings",
            ));
        }
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
        let catalog = portable_catalog(path)?.entries;
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
    let (title, bytes) = uploaded_file(&request)?;
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

fn uploaded_file(request: &tauri::ipc::Request<'_>) -> Result<(String, Vec<u8>), DictionaryError> {
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
    Ok((title, bytes))
}

#[tauri::command]
pub async fn dictionary_relink(
    app: tauri::AppHandle,
    request: tauri::ipc::Request<'_>,
) -> Result<RelinkResult, DictionaryError> {
    let id = request
        .headers()
        .get("x-lexique-dictionary-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() <= 64)
        .ok_or_else(|| failure("invalid_query", "Registered dictionary identity is missing"))?
        .to_owned();
    let confirmed_hash = request
        .headers()
        .get("x-lexique-dictionary-confirmed-hash")
        .map(|value| value.to_str().map(str::to_owned))
        .transpose()
        .map_err(|error| failure("invalid_query", error))?;
    let (title, bytes) = uploaded_file(&request)?;
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
        let result = relink(&path, &id, &title, &bytes, confirmed_hash.as_deref())?;
        if matches!(&result, RelinkResult::Linked { .. }) {
            cache.retain(|entry| entry.id != id);
        }
        Ok(result)
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
        assert_eq!(lexique_storage::initialize(&connection).unwrap(), 18);
    }

    #[test]
    fn a_full_legacy_catalog_keeps_fifteen_custom_dictionaries_and_their_order() {
        let path = database();
        let connection = open(&path).unwrap();
        let bytes = fixture("v2-utf8-none.mdx");
        let mut original: Vec<_> = (1..=15)
            .map(|index| DictionaryCatalogEntry {
                id: format!("mdx-{index}"),
                title: format!("{index}.mdx"),
                content_hash: None,
            })
            .collect();
        original.insert(
            7,
            DictionaryCatalogEntry {
                id: "builtin".into(),
                title: "FreeDict fra-zho".into(),
                content_hash: None,
            },
        );
        fs::create_dir_all(custom_path(&path, "mdx-1").unwrap().parent().unwrap()).unwrap();
        for item in original.iter().filter(|item| item.id.starts_with("mdx-")) {
            fs::write(custom_path(&path, &item.id).unwrap(), &bytes).unwrap();
        }
        let legacy: Vec<_> = original
            .iter()
            .map(|entry| serde_json::json!({"id":entry.id,"title":entry.title}))
            .collect();
        connection
            .execute(
                "INSERT INTO app_metadata(key,value) VALUES(?1,?2)",
                [
                    KEY,
                    &serde_json::json!({"next_id":16,"dictionaries":legacy}).to_string(),
                ],
            )
            .unwrap();
        let upgraded = settings(&path, None).unwrap();
        assert_eq!(
            upgraded[..16]
                .iter()
                .map(|entry| (&entry.id, &entry.title))
                .collect::<Vec<_>>(),
            original
                .iter()
                .map(|entry| (&entry.id, &entry.title))
                .collect::<Vec<_>>()
        );
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
        assert_eq!(lexique_storage::initialize(&connection).unwrap(), 18);
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
    fn a_restored_catalog_does_not_guess_a_file_from_its_id_or_title() {
        let path = database();
        let retained = fixture("v1-utf16-none.mdx");
        import(&path, "same-title.mdx", &retained).unwrap();
        let mut catalog = portable_catalog(&path).unwrap();
        catalog.entries.last_mut().unwrap().content_hash =
            Some("a09421f5f4b008eb9ab2ed9e4b988405b28ae0dbaea8232e95a7762e6b9c9aee".into());
        restore_catalog(&mut open(&path).unwrap(), &path, &catalog).unwrap();
        let restored = settings(&path, None).unwrap();
        assert_eq!(restored.last().unwrap().id, "mdx-1");
        assert_eq!(restored.last().unwrap().availability, "missing");
        assert_eq!(
            Registry::default()
                .lookup(
                    &path,
                    &path.0.with_file_name("unused"),
                    "double",
                    Some("mdx-1")
                )
                .unwrap_err()
                .code,
            "missing"
        );
        assert_eq!(
            fs::read(custom_path(&path, "mdx-1").unwrap()).unwrap(),
            retained
        );
    }

    #[test]
    fn a_new_import_reserves_retained_files_and_all_frozen_source_ids() {
        let path = database();
        let bytes = fixture("v2-utf8-none.mdx");
        import(&path, "retained.mdx", &bytes).unwrap();
        fs::write(custom_path(&path, "mdx-2").unwrap(), &bytes).unwrap();
        let source = |id| serde_json::json!({"dictionary_id":format!("mdx-{id}"),"dictionary":"frozen","lookup_key":"école","version":"1","attribution":null,"license":null,"source_url":null,"license_url":null,"sha256":"a".repeat(64)});
        let snapshot: lexique_storage::Snapshot = serde_json::from_value(serde_json::json!({
            "reference": source(3), "digest": [{"pronunciations":[],"grammar":[],"translation":"saved meaning"}]
        })).unwrap();
        let mut request: lexique_storage::SaveRequest = serde_json::from_value(serde_json::json!({
            "operation":"reserved-old-source", "headword":"école", "notes":"keep my note", "example_fr":"école", "example_zh":"学校", "example_source":source(6),
            "example_sense":0,"more_examples":[{"fr":"une école","zh":"一所学校","source":source(5)}],"id":null,"revision":null
        })).unwrap();
        let mut connection = open(&path).unwrap();
        let record = lexique_storage::save(&mut connection, &request, Some(snapshot))
            .unwrap()
            .record;
        request.operation = "reserved-current-source".into();
        request.id = Some(record.id);
        request.revision = Some(record.revision);
        request.example_source = Some(serde_json::from_value(source(4)).unwrap());
        let record = lexique_storage::save(&mut connection, &request, None)
            .unwrap()
            .record;
        let preserved = serde_json::to_value(record).unwrap();
        restore_catalog(&mut connection, &path, &DictionaryCatalog::default()).unwrap();
        let imported = import(&path, "new.mdx", &bytes).unwrap();
        assert_eq!(imported.last().unwrap().id, "mdx-7");
        assert_eq!(
            fs::read(custom_path(&path, "mdx-1").unwrap()).unwrap(),
            bytes
        );
        assert_eq!(
            fs::read(custom_path(&path, "mdx-2").unwrap()).unwrap(),
            bytes
        );
        assert_eq!(
            serde_json::to_value(lexique_storage::get(&connection, request.id.unwrap()).unwrap())
                .unwrap(),
            preserved
        );
    }

    #[test]
    fn matching_mdx_relinks_the_original_identity_without_confirmation() {
        let path = database();
        let bytes = fixture("v2-utf8-none.mdx");
        import(&path, "original.mdx", &bytes).unwrap();
        let order = vec!["mdx-1".into(), "builtin".into(), BUNDLED_MDX_ID.into()];
        settings(&path, Some(order.clone())).unwrap();
        let registry = Registry::default();
        let builtin = path.0.with_file_name("unused");
        registry
            .lookup(&path, &builtin, "double", Some("mdx-1"))
            .unwrap();
        fs::remove_file(custom_path(&path, "mdx-1").unwrap()).unwrap();
        assert_eq!(settings(&path, None).unwrap()[0].availability, "missing");
        let RelinkResult::Linked { catalog } =
            relink(&path, "mdx-1", "other-name.mdx", &bytes, None).unwrap()
        else {
            panic!("Identical content should not need confirmation");
        };
        assert_eq!(
            catalog
                .iter()
                .map(|entry| entry.id.clone())
                .collect::<Vec<_>>(),
            order
        );
        assert_eq!(catalog[0].title, "original.mdx");
        assert_eq!(catalog[0].availability, "available");
        assert!(!custom_path(&path, "mdx-1").unwrap().exists());
        assert_eq!(
            fs::read(custom_path(&path, "mdx-2").unwrap()).unwrap(),
            bytes
        );
        assert_eq!(
            registry
                .lookup(&path, &builtin, "double", Some("mdx-1"))
                .unwrap()
                .entries
                .len(),
            2
        );
        assert_eq!(
            Registry::default()
                .lookup(&path, &builtin, "double", Some("mdx-1"))
                .unwrap()
                .dictionary_id
                .as_deref(),
            Some("mdx-1")
        );
    }

    #[test]
    fn different_and_unknown_content_need_confirmation_of_the_actual_candidate() {
        let current_hash = "a09421f5f4b008eb9ab2ed9e4b988405b28ae0dbaea8232e95a7762e6b9c9aee";
        let candidate_hash = "65ea82c45458d01cf7131a33f6bf162220b97b9dbe7eecf8e1b55243a9b08174";
        for unknown in [false, true] {
            let path = database();
            let current = fixture("v2-utf8-none.mdx");
            let candidate = fixture("v1-utf16-none.mdx");
            import(&path, "original.mdx", &current).unwrap();
            let mut connection = open(&path).unwrap();
            let source = serde_json::json!({"dictionary_id":"mdx-1","dictionary":"original.mdx","lookup_key":"double","version":current_hash,"attribution":null,"license":null,"source_url":null,"license_url":null,"sha256":current_hash});
            let snapshot: lexique_storage::Snapshot = serde_json::from_value(serde_json::json!({"reference":source,"digest":[{"pronunciations":[],"grammar":[],"translation":"original meaning"}]})).unwrap();
            let request: lexique_storage::SaveRequest = serde_json::from_value(serde_json::json!({"operation":"frozen-relink-note","headword":"double","notes":"my note","example_fr":"original example","example_zh":"原例句","example_source":source,"example_sense":0,"more_examples":[{"fr":"another example","zh":"另一例句","source":source}],"id":null,"revision":null})).unwrap();
            let record = lexique_storage::save(&mut connection, &request, Some(snapshot))
                .unwrap()
                .record;
            let preserved = serde_json::to_value(&record).unwrap();
            if unknown {
                let mut catalog = portable_catalog(&path).unwrap();
                catalog.entries.last_mut().unwrap().content_hash = None;
                restore_catalog(&mut connection, &path, &catalog).unwrap();
            }
            let before = portable_catalog(&path).unwrap().entries;
            let RelinkResult::ConfirmationRequired {
                dictionary_id,
                expected_hash,
                content_hash,
            } = relink(&path, "mdx-1", "candidate.mdx", &candidate, None).unwrap()
            else {
                panic!("Changed or unknown identity requires confirmation");
            };
            assert_eq!(dictionary_id, "mdx-1");
            assert_eq!(
                expected_hash.as_deref(),
                if unknown { None } else { Some(current_hash) }
            );
            assert_eq!(content_hash, candidate_hash);
            assert_eq!(portable_catalog(&path).unwrap().entries, before);
            assert!(!custom_path(&path, "mdx-2").unwrap().exists());
            assert_eq!(
                relink(
                    &path,
                    "mdx-1",
                    "changed-candidate.mdx",
                    &current,
                    Some(candidate_hash)
                )
                .unwrap_err()
                .code,
                "invalid_query"
            );
            assert_eq!(portable_catalog(&path).unwrap().entries, before);
            let RelinkResult::Linked { catalog } = relink(
                &path,
                "mdx-1",
                "candidate.mdx",
                &candidate,
                Some(candidate_hash),
            )
            .unwrap() else {
                panic!("The confirmed candidate should be linked");
            };
            assert_eq!(catalog.last().unwrap().id, "mdx-1");
            assert_eq!(catalog.last().unwrap().title, "original.mdx");
            assert_eq!(
                fs::read(custom_path(&path, "mdx-1").unwrap()).unwrap(),
                current
            );
            let lookup = Registry::default()
                .lookup(
                    &path,
                    &path.0.with_file_name("unused"),
                    "double",
                    Some("mdx-1"),
                )
                .unwrap();
            assert_eq!(lookup.sha256, candidate_hash);
            assert_eq!(
                serde_json::to_value(lexique_storage::get(&connection, record.id).unwrap())
                    .unwrap(),
                preserved
            );
        }
    }

    #[test]
    fn failed_settings_validation_keeps_the_original_order_and_registration() {
        for removing in [false, true] {
            let path = database();
            let bytes = fixture("v2-utf8-none.mdx");
            import(&path, "original.mdx", &bytes).unwrap();
            let before = portable_catalog(&path).unwrap().entries;
            let connection = open(&path).unwrap();
            connection.execute_batch("CREATE TRIGGER damage_dictionary_bindings AFTER UPDATE ON app_metadata WHEN NEW.key = 'dictionary-settings' BEGIN UPDATE app_metadata SET value = 'invalid bindings' WHERE key = 'dictionary-files'; END;").unwrap();
            let result = if removing {
                remove(&path, "mdx-1")
            } else {
                settings(
                    &path,
                    Some(vec![
                        "mdx-1".into(),
                        "builtin".into(),
                        BUNDLED_MDX_ID.into(),
                    ]),
                )
            };
            assert!(result.is_err());
            assert_eq!(
                lexique_storage::read_dictionary_catalog(&connection)
                    .unwrap()
                    .entries,
                before
            );
            assert_eq!(
                Registry::default()
                    .lookup(
                        &path,
                        &path.0.with_file_name("unused"),
                        "double",
                        Some("mdx-1")
                    )
                    .unwrap()
                    .sha256,
                "a09421f5f4b008eb9ab2ed9e4b988405b28ae0dbaea8232e95a7762e6b9c9aee"
            );
        }
    }

    #[test]
    fn failed_file_validation_publication_and_registration_keep_the_original_binding() {
        let path = database();
        let bytes = fixture("v2-utf8-none.mdx");
        import(&path, "original.mdx", &bytes).unwrap();
        let before = portable_catalog(&path).unwrap().entries;
        for (title, input, code) in [
            ("../outside.mdx", bytes.as_slice(), "invalid_query"),
            ("empty.mdx", b"".as_slice(), "limit"),
            ("damaged.mdx", b"damaged".as_slice(), "corrupt"),
        ] {
            assert_eq!(
                relink(&path, "mdx-1", title, input, None).unwrap_err().code,
                code
            );
            assert_eq!(portable_catalog(&path).unwrap().entries, before);
        }
        assert_eq!(
            relink(&path, "builtin", "file.mdx", &bytes, None)
                .unwrap_err()
                .code,
            "invalid_query"
        );
        assert_eq!(
            relink(&path, "mdx-99", "file.mdx", &bytes, None)
                .unwrap_err()
                .code,
            "missing"
        );
        let connection = open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_dictionary_binding BEFORE UPDATE ON app_metadata WHEN NEW.key = 'dictionary-files' BEGIN SELECT RAISE(ABORT, 'synthetic binding failure'); END;").unwrap();
        let failed = relink(&path, "mdx-1", "file.mdx", &bytes, None).unwrap_err();
        assert_eq!(failed.code, "query_failed");
        assert!(failed.message.contains("synthetic binding failure"));
        assert_eq!(portable_catalog(&path).unwrap().entries, before);
        assert_eq!(
            fs::read(custom_path(&path, "mdx-1").unwrap()).unwrap(),
            bytes
        );
        assert!(!custom_path(&path, "mdx-3").unwrap().exists());
        connection
            .execute_batch("DROP TRIGGER reject_dictionary_binding;")
            .unwrap();
        let directory = custom_path(&path, "mdx-1")
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let retained = directory.with_file_name("retained-dictionaries");
        fs::rename(&directory, &retained).unwrap();
        fs::write(&directory, b"synthetic unwritable directory").unwrap();
        assert_eq!(
            relink(&path, "mdx-1", "file.mdx", &bytes, None)
                .unwrap_err()
                .code,
            "query_failed"
        );
        assert_eq!(portable_catalog(&path).unwrap().entries, before);
        assert_eq!(fs::read(retained.join("mdx-1.mdx")).unwrap(), bytes);
        fs::remove_file(&directory).unwrap();
        fs::rename(&retained, &directory).unwrap();
        assert_eq!(
            Registry::default()
                .lookup(
                    &path,
                    &path.0.with_file_name("unused"),
                    "double",
                    Some("mdx-1")
                )
                .unwrap()
                .sha256,
            "a09421f5f4b008eb9ab2ed9e4b988405b28ae0dbaea8232e95a7762e6b9c9aee"
        );
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
