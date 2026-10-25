//! Read-only access to admitted dictionary resources, separate from learning data.

use std::{fs::File, io::Read, path::Path};

use rusqlite::{Connection, MAIN_DB};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod entry;
pub use entry::{Entry, Sense};

const PROVENANCE: &str =
    include_str!("../../../resources/freedict-fra-zho-2025.11.23/provenance.json");

#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub title: String,
    pub version: String,
    pub attribution: String,
    pub license: String,
    pub source_url: String,
    pub license_url: String,
}

#[derive(Debug, Serialize)]
pub struct Lookup {
    pub source: Source,
    pub sha256: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
pub struct DictionaryError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for DictionaryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DictionaryError {}

fn failure(code: &'static str, context: &str, cause: impl std::fmt::Display) -> DictionaryError {
    DictionaryError {
        code,
        message: format!("{context}: {cause}"),
    }
}

pub fn lookup(path: &Path, key: &str, expected_sha256: &str) -> Result<Lookup, DictionaryError> {
    if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
        return Err(failure(
            "invalid_query",
            "Invalid query",
            "enter 1–512 UTF-8 bytes without control characters",
        ));
    }
    let mut data = Vec::new();
    File::open(path)
        .and_then(|file| file.take(16_000_001).read_to_end(&mut data))
        .map_err(|error| {
            let code = if error.kind() == std::io::ErrorKind::NotFound {
                "missing"
            } else {
                "query_failed"
            };
            failure(
                code,
                &format!("Could not read dictionary {}", path.display()),
                error,
            )
        })?;
    let hash = format!("{:x}", Sha256::digest(&data));
    if data.len() > 16_000_000 || !data.starts_with(b"SQLite format 3\0") || hash != expected_sha256
    {
        return Err(failure("corrupt", "Dictionary checksum mismatch", &hash));
    }
    let mut connection = Connection::open_in_memory().map_err(|error| {
        failure(
            "query_failed",
            "Could not open dictionary connection",
            error,
        )
    })?;
    // Query exactly the verified bytes, even if the resource is replaced concurrently.
    connection
        .deserialize_read_exact(MAIN_DB, data.as_slice(), data.len(), true)
        .map_err(|error| failure("corrupt", "Could not load read-only dictionary", error))?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not restrict dictionary schema",
                error,
            )
        })?;
    let entries = read_entries(&connection, key)?;
    Ok(Lookup {
        source: serde_json::from_str(PROVENANCE)
            .map_err(|error| failure("corrupt", "Invalid source attribution", error))?,
        sha256: hash,
        entries,
    })
}

fn read_entries(connection: &Connection, key: &str) -> Result<Vec<Entry>, DictionaryError> {
    let mut statement = connection.prepare(
        "SELECT ordinal, key, entry_xml FROM entries WHERE key = ?1 COLLATE BINARY ORDER BY ordinal",
    ).map_err(|error| failure("query_failed", "Could not prepare dictionary query", error))?;
    let rows = statement
        .query_map([key], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| failure("query_failed", "Dictionary query failed", error))?;
    rows.map(|row| {
        let (ordinal, key, xml) =
            row.map_err(|error| failure("query_failed", "Could not read dictionary entry", error))?;
        entry::parse(ordinal, key, &xml)
            .map_err(|error| failure("corrupt", "Invalid dictionary entry", error))
    })
    .collect()
}
