//! Exact whole-expression lookup from shared simple-form models.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::Read,
    path::Path,
};

use flate2::read::ZlibDecoder;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{DictionaryError, failure, snapshot};

mod generation;

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
    let connection = snapshot::open(path, expected_sha256, 16 * 1024 * 1024)?;
    let application: u32 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|error| failure("corrupt", "Could not read conjugation format", error))?;
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| failure("corrupt", "Could not read conjugation version", error))?;
    if application != 0x4c58434a || version != 6 {
        return Err(failure(
            "corrupt",
            "Unsupported conjugation resource format",
            format!("application {application:#x}, version {version}"),
        ));
    }
    let models = read_models(&connection)?;
    let construction: String = connection
        .query_row("SELECT construction FROM metadata", [], |row| row.get(0))
        .map_err(|error| failure("corrupt", "Could not read construction forms", error))?;
    let construction: generation::Construction = serde_json::from_str(&construction)
        .map_err(|error| failure("corrupt", "Invalid construction forms", error))?;
    let mut tables = Vec::new();
    for base in candidates(&connection, key, &models)? {
        let mut identities = read_identities(&connection, &base)?;
        let position = identities
            .iter()
            .position(|record| record.lemma == base)
            .ok_or_else(|| failure("corrupt", "Missing ordinary conjugation identity", &base))?;
        let ordinary = identities.remove(position);
        let model = ordinary
            .model
            .and_then(|model| models.get(&model))
            .ok_or_else(|| failure("corrupt", "Missing conjugation model", &base))?;
        let cells = generation::ordinary(
            &base,
            model,
            &ordinary.source_lines,
            &ordinary.info,
            ordinary.h_aspire,
            &construction,
        )?;
        for identity in identities {
            if identity.model.is_some() || !identity.source_lines.is_empty() {
                return Err(failure(
                    "corrupt",
                    "Invalid pronominal conjugation identity",
                    &identity.lemma,
                ));
            }
            let expanded = generation::pronominal(
                &identity.lemma,
                &cells,
                &identity.info,
                ordinary.h_aspire,
                &construction,
            )?;
            append_table(&mut tables, identity, expanded, key)?;
        }
        append_table(&mut tables, ordinary, cells, key)?;
    }
    tables.sort_by(|a, b| a.lemma.cmp(&b.lemma));
    Ok(Lookup {
        query: key.to_owned(),
        tables,
    })
}

fn read_models(
    connection: &Connection,
) -> Result<BTreeMap<i64, Vec<generation::Pattern>>, DictionaryError> {
    let mut statement = connection
        .prepare("SELECT id, cells FROM models")
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not prepare conjugation models",
                error,
            )
        })?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| failure("query_failed", "Could not query conjugation models", error))?;
    let mut models = BTreeMap::new();
    for row in rows {
        let (model, cells) =
            row.map_err(|error| failure("corrupt", "Could not read conjugation model", error))?;
        models.insert(
            model,
            serde_json::from_str(&cells)
                .map_err(|error| failure("corrupt", "Invalid conjugation model", error))?,
        );
    }
    Ok(models)
}

fn candidates(
    connection: &Connection,
    key: &str,
    models: &BTreeMap<i64, Vec<generation::Pattern>>,
) -> Result<BTreeSet<String>, DictionaryError> {
    let mut suffixes: HashMap<&str, HashSet<(i64, u32)>> = HashMap::new();
    for (&model, cells) in models {
        for cell in cells {
            suffixes
                .entry(&cell.suffix)
                .or_default()
                .insert((model, cell.cut));
        }
    }
    let mut probes = BTreeSet::from([key, query_base(key)]);
    if let Some(last) = key.split_whitespace().last() {
        probes.insert(last);
    }
    for probe in probes.clone() {
        for prefix in ["m’", "t’", "s’"] {
            if let Some(form) = probe.strip_prefix(prefix) {
                probes.insert(form);
            }
        }
        for suffix in ["-toi", "-nous", "-vous", "-t’en", "-nous-en", "-vous-en"] {
            if let Some(form) = probe.strip_suffix(suffix) {
                probes.insert(form);
            }
        }
    }
    let mut statement = connection
        .prepare("SELECT model, cut, lemma FROM stems WHERE stem = ?1 COLLATE BINARY")
        .map_err(|error| failure("query_failed", "Could not prepare conjugation stems", error))?;
    let mut bases = BTreeSet::new();
    for probe in probes {
        for boundary in probe
            .char_indices()
            .map(|(index, _)| index)
            .chain([probe.len()])
        {
            let Some(patterns) = suffixes.get(&probe[boundary..]) else {
                continue;
            };
            let rows = statement
                .query_map([&probe[..boundary]], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, u32>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(|error| {
                    failure("query_failed", "Could not query conjugation stems", error)
                })?;
            for row in rows {
                let (model, cut, lemma) = row.map_err(|error| {
                    failure("corrupt", "Could not read conjugation stem", error)
                })?;
                if patterns.contains(&(model, cut)) {
                    bases.insert(lemma);
                }
            }
        }
    }
    Ok(bases)
}

struct Identity {
    lemma: String,
    model: Option<i64>,
    source_lines: Vec<u8>,
    verb: serde_json::Value,
    info: generation::Verb,
    meanings: serde_json::Value,
    h_aspire: bool,
    slots: u32,
}

fn read_identities(connection: &Connection, base: &str) -> Result<Vec<Identity>, DictionaryError> {
    let mut statement = connection
        .prepare(
            "SELECT lemma, model, source_lines, information, meanings, h_aspire, slots
                  FROM verbs WHERE base = ?1 COLLATE BINARY ORDER BY lemma COLLATE BINARY",
        )
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not prepare conjugation identities",
                error,
            )
        })?;
    let rows = statement
        .query_map([base], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, bool>(5)?,
                row.get::<_, u32>(6)?,
            ))
        })
        .map_err(|error| {
            failure(
                "query_failed",
                "Could not query conjugation identities",
                error,
            )
        })?;
    rows.map(|row| {
        let (lemma, model, source_lines, information, meanings, h_aspire, slots) =
            row.map_err(|error| failure("corrupt", "Could not read conjugation identity", error))?;
        let mut metadata = Vec::new();
        ZlibDecoder::new(information.as_slice())
            .read_to_end(&mut metadata)
            .map_err(|error| {
                failure(
                    "corrupt",
                    "Could not decompress conjugation metadata",
                    error,
                )
            })?;
        let verb = serde_json::from_slice(&metadata)
            .map_err(|error| failure("corrupt", "Invalid conjugation metadata", error))?;
        let info = generation::Verb::deserialize(&verb).map_err(|error| {
            failure(
                "corrupt",
                "Invalid conjugation construction metadata",
                error,
            )
        })?;
        Ok(Identity {
            lemma,
            model,
            source_lines,
            verb,
            info,
            h_aspire,
            slots,
            meanings: serde_json::from_str(&meanings)
                .map_err(|error| failure("corrupt", "Invalid English meanings", error))?,
        })
    })
    .collect()
}

fn append_table(
    tables: &mut Vec<Table>,
    identity: Identity,
    mut cells: Vec<Cell>,
    key: &str,
) -> Result<(), DictionaryError> {
    if cells.len() != identity.slots as usize {
        return Err(failure(
            "corrupt",
            "Conjugation slot count mismatch",
            &identity.lemma,
        ));
    }
    if cells
        .iter()
        .any(|cell| cell.form == key || cell.expressions.iter().any(|expression| expression == key))
    {
        cells.sort_by(|a, b| {
            a.tense
                .cmp(&b.tense)
                .then(a.person.cmp(&b.person))
                .then(a.number.cmp(&b.number))
                .then(a.gender.cmp(&b.gender))
                .then(a.source_line.cmp(&b.source_line))
        });
        tables.push(Table {
            lemma: identity.lemma,
            verb: identity.verb,
            meanings: identity.meanings,
            cells,
        });
    }
    Ok(())
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
