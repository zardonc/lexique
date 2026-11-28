use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use lexique_dictionary::{
    DictionaryError,
    cefr::{Dataset, Entry, MAX_BYTES, Query, validate_queries},
};
use serde::Serialize;
use tauri::State;

use crate::vocabulary::DatabasePath;

#[derive(Default)]
pub struct CefrResource(Mutex<Option<Dataset>>);

#[derive(Serialize)]
pub struct Settings {
    row_count: Option<usize>,
}

#[derive(Serialize)]
pub struct Lookup {
    imported: bool,
    entries: Vec<Entry>,
}

fn failure(message: impl std::fmt::Display) -> DictionaryError {
    DictionaryError {
        code: "query_failed",
        message: message.to_string(),
    }
}

fn resource_path(database: &Path) -> Result<PathBuf, DictionaryError> {
    Ok(database
        .parent()
        .ok_or_else(|| failure("CEFR data directory is unavailable"))?
        .join("cefr/FleLex_TT_Beacco.tsv"))
}

fn read(path: &Path) -> Result<Option<Dataset>, DictionaryError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(failure(format!(
                "Could not read imported CEFR file: {error}"
            )));
        }
    };
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(failure)?;
    Dataset::parse(&bytes).map(Some)
}

fn replace(path: &Path, bytes: &[u8]) -> Result<Dataset, DictionaryError> {
    let data = Dataset::parse(bytes)?;
    fs::create_dir_all(path.parent().unwrap()).map_err(failure)?;
    let staged = path.with_extension("tsv.tmp");
    let mut file = fs::OpenOptions::new()
        .write(true)
        // Imports share the resource lock; overwrite staging left by an interrupted import.
        .create(true)
        .truncate(true)
        .open(&staged)
        .map_err(|error| failure(format!("Could not stage CEFR import: {error}")))?;
    let result = file.write_all(bytes).and_then(|_| file.sync_all());
    drop(file);
    let result = result.and_then(|_| fs::rename(&staged, path));
    if let Err(error) = result {
        let cleanup = fs::remove_file(&staged);
        return Err(failure(format!(
            "Could not install CEFR file: {error}; staging cleanup: {cleanup:?}"
        )));
    }
    Ok(data)
}

#[tauri::command]
pub async fn cefr_settings(
    path: State<'_, DatabasePath>,
    resource: State<'_, CefrResource>,
) -> Result<Settings, DictionaryError> {
    let path = resource_path(&path.0)?;
    let mut data = resource.0.lock().map_err(failure)?;
    if data.is_none() {
        *data = read(&path)?;
    }
    Ok(Settings {
        row_count: data.as_ref().map(Dataset::row_count),
    })
}

#[tauri::command]
pub async fn cefr_import(
    request: tauri::ipc::Request<'_>,
    path: State<'_, DatabasePath>,
    resource: State<'_, CefrResource>,
) -> Result<Settings, DictionaryError> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(failure("CEFR import requires a raw TSV payload"));
    };
    let path = resource_path(&path.0)?;
    let mut data = resource.0.lock().map_err(failure)?;
    *data = Some(replace(&path, bytes)?);
    Ok(Settings {
        row_count: data.as_ref().map(Dataset::row_count),
    })
}

#[tauri::command]
pub async fn cefr_remove(
    path: State<'_, DatabasePath>,
    resource: State<'_, CefrResource>,
) -> Result<Settings, DictionaryError> {
    let path = resource_path(&path.0)?;
    let mut data = resource.0.lock().map_err(failure)?;
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(failure(format!("Could not remove CEFR file: {error}"))),
    }
    *data = None;
    Ok(Settings { row_count: None })
}

#[tauri::command]
pub async fn cefr_lookup(
    queries: Vec<Query>,
    path: State<'_, DatabasePath>,
    resource: State<'_, CefrResource>,
) -> Result<Lookup, DictionaryError> {
    validate_queries(&queries)?;
    let path = resource_path(&path.0)?;
    let mut data = resource.0.lock().map_err(failure)?;
    if data.is_none() {
        *data = read(&path)?;
    }
    Ok(Lookup {
        imported: data.is_some(),
        entries: queries
            .into_iter()
            .map(|query| Entry {
                level: data.as_ref().and_then(|data| data.lookup(&query)),
                key: query.key,
                sense_id: query.sense.map(|sense| sense.sense_id),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_replace_atomically_and_read_after_restart() {
        let directory = std::env::temp_dir().join(format!("lexique-cefr-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("FleLex_TT_Beacco.tsv");
        let bytes = b"word\ttag\tfreq_A1\tfreq_A2\tfreq_B1\tfreq_B2\tfreq_C1\tfreq_C2\tfreq_total\tlevel\nexemple\tNOM\t0\t0\t0\t0\t0\t0\t0\tB1\n";
        assert!(read(&path).unwrap().is_none());
        replace(&path, bytes).unwrap();
        assert_eq!(read(&path).unwrap().unwrap().row_count(), 1);
        assert!(replace(&path, b"wrong TSV").is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        let replacement = String::from_utf8(bytes.to_vec())
            .unwrap()
            .replace("\tB1\n", "\tC2\n");
        fs::write(path.with_extension("tsv.tmp"), b"interrupted import").unwrap();
        replace(&path, replacement.as_bytes()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), replacement.as_bytes());
        fs::remove_file(&path).unwrap();
        assert!(read(&path).unwrap().is_none());
        fs::remove_dir(&directory).unwrap();
    }
}
