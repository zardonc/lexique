//! Verified read-only access to the separately admitted four-field core.

use std::{collections::HashSet, path::Path};

use lexique_core::dictionary::CoreSense;
use rusqlite::{Connection, OptionalExtension};
use unicode_normalization::UnicodeNormalization;

use crate::{CORE_DICTIONARY_ID, DictionaryError, Entry, Lookup, Source, failure, mdx, snapshot};

pub const MAX_RESOURCE_BYTES: u64 = 20_000_000;
pub const MAX_MATCHING_SENSES: usize = 200;

fn metadata(connection: &Connection, key: &str) -> Result<String, DictionaryError> {
    connection
        .query_row("SELECT value FROM metadata WHERE key=?1", [key], |row| {
            row.get(0)
        })
        .map_err(|error| failure("corrupt", &format!("Invalid core metadata {key}"), error))
}

fn source(connection: &Connection) -> Result<Source, DictionaryError> {
    if metadata(connection, "format_version")? != "1" {
        return Err(failure(
            "unsupported",
            "Unsupported core resource format",
            "expected format version 1",
        ));
    }
    let version = metadata(connection, "content_version")?;
    let value: serde_json::Value = serde_json::from_str(&metadata(connection, "source")?)
        .map_err(|error| failure("corrupt", "Invalid core source metadata", error))?;
    let names = [
        "title",
        "version",
        "attribution",
        "license",
        "source_url",
        "license_url",
    ];
    if !value.as_object().is_some_and(|fields| {
        fields.len() == names.len() && names.iter().all(|name| fields.contains_key(*name))
    }) {
        return Err(failure(
            "corrupt",
            "Invalid core source metadata",
            "unknown or missing source fields",
        ));
    }
    let source: Source = serde_json::from_value(value)
        .map_err(|error| failure("corrupt", "Invalid core source attribution", error))?;
    if source.version != version
        || version.trim().is_empty()
        || version.chars().count() > 200
        || [&source.title, &source.version]
            .iter()
            .any(|value| value.trim().is_empty() || value.chars().any(char::is_control))
        || [
            &source.attribution,
            &source.license,
            &source.source_url,
            &source.license_url,
        ]
        .iter()
        .any(|value| {
            value
                .as_ref()
                .is_none_or(|value| value.trim().is_empty() || value.chars().any(char::is_control))
        })
        || [&source.source_url, &source.license_url]
            .iter()
            .any(|value| {
                value.as_ref().is_none_or(|value| {
                    !(value.starts_with("https://") || value.starts_with("http://"))
                        || value.chars().any(char::is_whitespace)
                })
            })
    {
        return Err(failure(
            "corrupt",
            "Invalid core version or source attribution",
            "source metadata is incomplete or inconsistent",
        ));
    }
    Ok(source)
}

pub fn lookup(path: &Path, key: &str, expected_sha256: &str) -> Result<Lookup, DictionaryError> {
    crate::validate_query(key)?;
    let connection = snapshot::open(path, expected_sha256, MAX_RESOURCE_BYTES)?;
    let source = source(&connection)?;
    let normalized = key.nfc().collect::<String>();
    let canonical = connection
        .query_row(
            "SELECT headword FROM aliases WHERE query=?1 COLLATE BINARY",
            [&normalized],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| failure("corrupt", "Invalid core query binding", error))?
        .unwrap_or(normalized);
    let mut statement = connection.prepare(
        "SELECT frozen_row,sense_index,headword,payload FROM senses WHERE headword=?1 COLLATE BINARY ORDER BY frozen_row,sense_index LIMIT 201",
    ).map_err(|error| failure("corrupt", "Could not prepare core query", error))?;
    let rows = statement
        .query_map([&canonical], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| failure("query_failed", "Core query failed", error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| failure("corrupt", "Invalid core query record", error))?;
    if rows.len() > MAX_MATCHING_SENSES
        || rows.iter().map(|row| row.3.len()).sum::<usize>() > mdx::MAX_RESPONSE_BYTES
    {
        return Err(failure(
            "limit",
            "Core lookup exceeds the content limit",
            "maximum 200 matching senses and 4 MiB response",
        ));
    }
    let mut entries: Vec<Entry> = Vec::new();
    let mut identities = HashSet::new();
    for (frozen_row, sense_index, headword, payload) in rows {
        let core: CoreSense = serde_json::from_str(&payload)
            .map_err(|error| failure("corrupt", "Invalid four-field core sense", error))?;
        core.validate()
            .map_err(|error| failure("corrupt", "Invalid four-field core sense", error))?;
        if core.frozen_row != frozen_row
            || core.headword != headword
            || core.headword != canonical
            || sense_index == 0
            || !identities.insert(core.sense_id.clone())
        {
            return Err(failure(
                "corrupt",
                "Invalid frozen core identity",
                "sense disagrees with its resource row or query binding",
            ));
        }
        if let Some(entry) = entries
            .last_mut()
            .filter(|entry| entry.ordinal == frozen_row)
        {
            if entry.grammar != [core.pos.clone()] {
                return Err(failure(
                    "corrupt",
                    "Invalid frozen core POS",
                    "one frozen group has conflicting POS classes",
                ));
            }
            if !entry.pronunciations.contains(&core.ipa) {
                entry.pronunciations.push(core.ipa.clone());
            }
            entry.core_senses.push(core);
        } else {
            entries.push(Entry {
                ordinal: frozen_row,
                key: headword,
                pronunciations: vec![core.ipa.clone()],
                grammar: vec![core.pos.clone()],
                senses: Vec::new(),
                core_senses: vec![core],
                html: None,
            });
        }
    }
    let lookup = Lookup {
        dictionary_id: Some(CORE_DICTIONARY_ID.into()),
        source,
        sha256: expected_sha256.into(),
        entries,
        warnings: Vec::new(),
    };
    let bytes = serde_json::to_vec(&lookup)
        .map_err(|error| failure("query_failed", "Could not serialize core response", error))?;
    if bytes.len() > mdx::MAX_RESPONSE_BYTES {
        return Err(failure(
            "limit",
            "Core lookup exceeds the content limit",
            "maximum 4 MiB response",
        ));
    }
    Ok(lookup)
}
