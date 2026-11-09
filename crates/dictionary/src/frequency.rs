//! Fixed subtitle lemma/POS statistics, independent of learning records.

use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::{DictionaryError, Source, failure, snapshot};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Missing,
    Invalid,
    Conflict,
    MissingPos,
    Ambiguous,
    NotFound,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub part_of_speech: Option<String>,
    pub per_million: Option<f64>,
    pub reason: Option<Reason>,
}

#[derive(Debug, Serialize)]
pub struct Frequency {
    pub key: String,
    pub lemma: Option<String>,
    pub per_million: Option<f64>,
    pub reason: Option<Reason>,
    pub parts: Vec<Part>,
    pub candidates: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Lookup {
    pub source: Source,
    pub metric: &'static str,
    pub entries: Vec<Frequency>,
}

pub fn lookup(
    path: &Path,
    keys: &[String],
    expected_sha256: &str,
) -> Result<Lookup, DictionaryError> {
    if keys.is_empty()
        || keys.len() > 100
        || keys.iter().any(|key| {
            key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control)
        })
    {
        return Err(failure(
            "invalid_query",
            "Invalid frequency batch",
            "provide 1–100 queries of 1–512 UTF-8 bytes without control characters",
        ));
    }
    let connection = snapshot::open(path, expected_sha256, 16_000_000)?;
    for (key, expected) in [
        ("format_version", "1"),
        ("version", "3.83"),
        ("metric", "freqlemfilms2"),
    ] {
        let value: String = connection
            .query_row("SELECT value FROM metadata WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .map_err(|error| failure("query_failed", "Could not read frequency metadata", error))?;
        if value != expected {
            return Err(failure("corrupt", "Unsupported frequency metadata", key));
        }
    }
    Ok(Lookup {
        source: serde_json::from_str(include_str!(
            "../../../resources/lexique-3.83/provenance.json"
        ))
        .map_err(|error| failure("corrupt", "Invalid frequency attribution", error))?,
        metric: "freqlemfilms2",
        entries: keys
            .iter()
            .map(|key| read(&connection, key))
            .collect::<Result<_, _>>()?,
    })
}

fn candidates(
    connection: &Connection,
    sql: &str,
    key: &str,
) -> Result<Vec<String>, DictionaryError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| failure("query_failed", "Could not prepare frequency query", error))?;
    statement
        .query_map([key], |row| row.get(0))
        .map_err(|error| failure("query_failed", "Frequency query failed", error))?
        .collect::<Result<_, _>>()
        .map_err(|error| failure("query_failed", "Could not read frequency candidates", error))
}

fn read(connection: &Connection, original: &str) -> Result<Frequency, DictionaryError> {
    let key: String = original.trim().nfc().flat_map(char::to_lowercase).collect();
    let mut matches = candidates(
        connection,
        "SELECT lemma FROM lemmas WHERE search_key = ?1 ORDER BY lemma",
        &key,
    )?;
    if matches.is_empty() {
        matches = candidates(
            connection,
            "SELECT lemma FROM forms WHERE key = ?1 ORDER BY lemma",
            &key,
        )?;
    }
    let mut entry = Frequency {
        key: original.to_owned(),
        lemma: None,
        per_million: None,
        reason: Some(if matches.is_empty() {
            Reason::NotFound
        } else {
            Reason::Ambiguous
        }),
        parts: Vec::new(),
        candidates: matches,
    };
    if entry.candidates.len() == 1 {
        let lemma = &entry.candidates[0];
        let (value, reason, parts): (Option<f64>, Option<String>, String) = connection
            .query_row(
                "SELECT per_million, reason, parts FROM lemmas WHERE lemma = ?1",
                [lemma],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|error| failure("query_failed", "Could not read lemma frequency", error))?;
        entry.lemma = Some(lemma.clone());
        entry.per_million = value;
        entry.reason = reason
            .map(|reason| serde_json::from_value(serde_json::Value::String(reason)))
            .transpose()
            .map_err(|error| failure("corrupt", "Invalid frequency reason", error))?;
        entry.parts = serde_json::from_str(&parts)
            .map_err(|error| failure("corrupt", "Invalid frequency parts", error))?;
    }
    Ok(entry)
}
