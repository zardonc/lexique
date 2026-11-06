//! Exact source-form lookup. Missing forms are never generated or declared defective.
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

use crate::{DictionaryError, failure, snapshot};

#[derive(Debug, Serialize)]
pub struct Lookup {
    pub query: String,
    pub tables: Vec<Table>,
}

#[derive(Debug, Serialize)]
pub struct Table {
    pub lemma: String,
    pub verb: serde_json::Value,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Serialize)]
pub struct Cell {
    pub source_line: u32,
    pub form: String,
    pub lemma: String,
    pub raw_tag: String,
    pub tense: Option<String>,
    pub person: Option<String>,
    pub number: Option<String>,
    pub gender: Option<String>,
}

pub fn lookup(path: &Path, key: &str, expected_sha256: &str) -> Result<Lookup, DictionaryError> {
    if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
        return Err(failure(
            "invalid_query",
            "Invalid conjugation query",
            "enter 1–512 UTF-8 bytes without control characters",
        ));
    }
    let connection = snapshot::open(path, expected_sha256, 40 * 1024 * 1024)?;
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT s.lemma, v.information FROM slots s JOIN verbs v ON v.lemma = s.lemma
         WHERE s.form = ?1 COLLATE BINARY ORDER BY s.lemma COLLATE BINARY",
        )
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not prepare conjugation lookup",
                error,
            )
        })?;
    let rows = statement
        .query_map([key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not query conjugation resource",
                error,
            )
        })?;
    let mut tables = Vec::new();
    for row in rows {
        let (lemma, metadata) =
            row.map_err(|error| failure("corrupt", "Could not read conjugation metadata", error))?;
        let cells = read_cells(&connection, &lemma)?;
        tables.push(Table {
            lemma,
            cells,
            verb: serde_json::from_str(&metadata)
                .map_err(|error| failure("corrupt", "Invalid conjugation metadata", error))?,
        });
    }
    Ok(Lookup {
        query: key.to_owned(),
        tables,
    })
}

fn read_cells(connection: &Connection, lemma: &str) -> Result<Vec<Cell>, DictionaryError> {
    let mut statement = connection.prepare(
        "SELECT source_line, form, lemma, raw_tag, tense, person, number, gender
         FROM slots WHERE lemma = ?1 COLLATE BINARY ORDER BY tense, person, number, gender, source_line",
    ).map_err(|error| failure("query_failed", "Could not prepare conjugation table", error))?;
    let rows = statement
        .query_map([lemma], |row| {
            Ok(Cell {
                source_line: row.get(0)?,
                form: row.get(1)?,
                lemma: row.get(2)?,
                raw_tag: row.get(3)?,
                tense: row.get(4)?,
                person: row.get(5)?,
                number: row.get(6)?,
                gender: row.get(7)?,
            })
        })
        .map_err(|error| failure("query_failed", "Could not query conjugation table", error))?;
    rows.map(|row| {
        row.map_err(|error| failure("corrupt", "Could not read conjugation cell", error))
    })
    .collect()
}
