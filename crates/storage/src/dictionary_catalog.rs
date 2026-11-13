use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::StorageError;

pub const DICTIONARY_CATALOG_KEY: &str = "dictionary-settings";
pub const MAX_DICTIONARIES: usize = 17;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DictionaryCatalogEntry {
    pub id: String,
    pub title: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub content_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DictionaryCatalog {
    pub next_id: u32,
    pub entries: Vec<DictionaryCatalogEntry>,
}

impl Default for DictionaryCatalog {
    fn default() -> Self {
        Self {
            next_id: 1,
            entries: vec![
                DictionaryCatalogEntry {
                    id: "builtin".into(),
                    title: "FreeDict fra-zho".into(),
                    content_hash: None,
                },
                DictionaryCatalogEntry {
                    id: "builtin-fra-eng".into(),
                    title: "FreeDict fra-eng 0.4.1".into(),
                    content_hash: None,
                },
            ],
        }
    }
}

pub fn custom_dictionary_number(id: &str) -> Option<u32> {
    id.strip_prefix("mdx-")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0 && id == format!("mdx-{value}"))
}

pub fn validate_dictionary_catalog(catalog: &DictionaryCatalog) -> Result<(), StorageError> {
    let entries = &catalog.entries;
    if entries.len() > MAX_DICTIONARIES
        || catalog.next_id == 0
        || ["builtin", "builtin-fra-eng"]
            .iter()
            .any(|id| entries.iter().filter(|entry| entry.id == *id).count() != 1)
        || entries.iter().enumerate().any(|(index, entry)| {
            let custom = custom_dictionary_number(&entry.id);
            let builtin = matches!(entry.id.as_str(), "builtin" | "builtin-fra-eng");
            (!builtin && custom.is_none())
                || custom.is_some_and(|number| number >= catalog.next_id)
                || entry.title.trim().is_empty()
                || entry.title.chars().count() > 200
                || entry.title.chars().any(char::is_control)
                || entries[..index].iter().any(|other| other.id == entry.id)
                || entry.content_hash.as_ref().is_some_and(|hash| {
                    builtin
                        || hash.len() != 64
                        || !hash
                            .bytes()
                            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                })
        })
    {
        return Err(StorageError::Invalid("Invalid dictionary catalog".into()));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEntry {
    id: String,
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCatalog {
    next_id: u32,
    dictionaries: Vec<LegacyEntry>,
}

pub fn read_dictionary_catalog(connection: &Connection) -> Result<DictionaryCatalog, StorageError> {
    let json: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [DICTIONARY_CATALOG_KEY],
            |row| row.get(0),
        )
        .optional()?;
    let catalog = match json {
        None => DictionaryCatalog::default(),
        Some(json) => match serde_json::from_str::<DictionaryCatalog>(&json) {
            Ok(catalog) => catalog,
            Err(_) => {
                let legacy: LegacyCatalog = serde_json::from_str(&json)?;
                let mut entries: Vec<_> = legacy
                    .dictionaries
                    .into_iter()
                    .map(|entry| DictionaryCatalogEntry {
                        id: entry.id,
                        title: entry.title,
                        content_hash: None,
                    })
                    .collect();
                if !entries.iter().any(|entry| entry.id == "builtin-fra-eng") {
                    entries.push(DictionaryCatalog::default().entries.remove(1));
                }
                DictionaryCatalog {
                    next_id: legacy.next_id,
                    entries,
                }
            }
        },
    };
    validate_dictionary_catalog(&catalog)?;
    Ok(catalog)
}

pub fn write_dictionary_catalog(
    connection: &Connection,
    catalog: &DictionaryCatalog,
) -> Result<(), StorageError> {
    validate_dictionary_catalog(catalog)?;
    connection.execute(
        "INSERT INTO app_metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [DICTIONARY_CATALOG_KEY, &serde_json::to_string(catalog)?],
    )?;
    Ok(())
}

pub fn referenced_custom_dictionary_ids(
    connection: &Connection,
) -> Result<BTreeSet<u32>, StorageError> {
    fn collect(value: &serde_json::Value, ids: &mut BTreeSet<u32>) {
        match value {
            serde_json::Value::Object(object) => {
                if let Some(id) = object
                    .get("dictionary_id")
                    .and_then(serde_json::Value::as_str)
                    .and_then(custom_dictionary_number)
                {
                    ids.insert(id);
                }
                for value in object.values() {
                    collect(value, ids);
                }
            }
            serde_json::Value::Array(array) => {
                for value in array {
                    collect(value, ids);
                }
            }
            _ => {}
        }
    }
    let mut ids = BTreeSet::new();
    let mut statement = connection.prepare(
        "SELECT snapshot FROM vocabulary UNION ALL SELECT example_source FROM vocabulary WHERE example_source IS NOT NULL UNION ALL SELECT more_examples FROM vocabulary UNION ALL SELECT request FROM save_operations",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    for row in rows {
        collect(&serde_json::from_str(&row?)?, &mut ids);
    }
    Ok(ids)
}
