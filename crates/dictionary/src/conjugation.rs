//! Exact whole-expression lookup in the admitted ordinary active resource.
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
    pub meanings: serde_json::Value,
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
    pub expressions: Vec<String>,
}

pub fn lookup(path: &Path, key: &str, expected_sha256: &str) -> Result<Lookup, DictionaryError> {
    if key.trim().is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
        return Err(failure(
            "invalid_query",
            "Invalid conjugation query",
            "enter 1–512 UTF-8 bytes without control characters",
        ));
    }
    let connection = snapshot::open(path, expected_sha256, 128 * 1024 * 1024)?;
    let base = query_base(key);
    let mut statement = connection
        .prepare(
            "SELECT DISTINCT s.lemma, v.information, v.meanings FROM slots s JOIN verbs v ON v.lemma = s.lemma
         WHERE s.form IN (?1, ?2) COLLATE BINARY ORDER BY s.lemma COLLATE BINARY",
        )
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not prepare conjugation lookup",
                error,
            )
        })?;
    let rows = statement
        .query_map([key, base], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
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
        let (lemma, metadata, meanings) =
            row.map_err(|error| failure("corrupt", "Could not read conjugation metadata", error))?;
        let cells = read_cells(&connection, &lemma)?;
        if !cells.iter().any(|cell| {
            cell.form == key || cell.expressions.iter().any(|expression| expression == key)
        }) {
            continue;
        }
        tables.push(Table {
            lemma,
            cells,
            verb: serde_json::from_str(&metadata)
                .map_err(|error| failure("corrupt", "Invalid conjugation metadata", error))?,
            meanings: serde_json::from_str(&meanings)
                .map_err(|error| failure("corrupt", "Invalid English meanings", error))?,
        });
    }
    Ok(Lookup {
        query: key.to_owned(),
        tables,
    })
}

fn read_cells(connection: &Connection, lemma: &str) -> Result<Vec<Cell>, DictionaryError> {
    let mut statement = connection.prepare(
        "SELECT source_line, form, lemma, raw_tag, tense, person, number, gender, expressions
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
                expressions: serde_json::from_str(&row.get::<_, String>(8)?).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?,
            })
        })
        .map_err(|error| failure("query_failed", "Could not query conjugation table", error))?;
    rows.map(|row| {
        row.map_err(|error| failure("corrupt", "Could not read conjugation cell", error))
    })
    .collect()
}

fn query_base(key: &str) -> &str {
    let verbal = key
        .strip_prefix("que ")
        .or_else(|| key.strip_prefix("qu’"))
        .unwrap_or(key);
    for subject in [
        "je ", "j’", "tu ", "il ", "elle ", "on ", "nous ", "vous ", "ils ", "elles ",
    ] {
        if let Some(base) = verbal.strip_prefix(subject) {
            return base;
        }
    }
    key
}
