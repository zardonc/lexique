use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::StorageError;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub dictionary: String,
    pub lookup_key: String,
    pub version: String,
    pub attribution: String,
    pub license: String,
    pub source_url: String,
    pub license_url: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DigestEntry {
    pub pronunciations: Vec<String>,
    pub grammar: Vec<String>,
    pub translation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub reference: Reference,
    pub digest: Vec<DigestEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SaveRequest {
    pub operation: String,
    pub headword: String,
    pub notes: String,
    pub id: Option<i64>,
    pub revision: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VocabularyRecord {
    pub id: i64,
    pub headword: String,
    pub reference: Reference,
    pub digest: Vec<DigestEntry>,
    pub notes: String,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct SaveResult {
    pub outcome: &'static str,
    pub record: VocabularyRecord,
}

pub fn validate(request: &SaveRequest) -> Result<(), StorageError> {
    if request.operation.is_empty()
        || request.operation.len() > 128
        || !request
            .operation
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(StorageError::Invalid(
            "Invalid save operation identifier".into(),
        ));
    }
    if request.headword.trim().is_empty()
        || request.headword.chars().count() > 200
        || request.headword.chars().any(char::is_control)
    {
        return Err(StorageError::Invalid(
            "Headword must be 1–200 characters on one line".into(),
        ));
    }
    if request.notes.chars().count() > 20_000 {
        return Err(StorageError::Invalid(
            "Notes exceed 20,000 Unicode characters".into(),
        ));
    }
    if request.id.is_some() != request.revision.is_some()
        || request.id.is_some_and(|id| id <= 0)
        || request.revision.is_some_and(|revision| revision <= 0)
    {
        return Err(StorageError::Invalid(
            "Invalid record identity or revision".into(),
        ));
    }
    Ok(())
}

fn row_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyRecord> {
    let json: String = row.get(2)?;
    let snapshot: Snapshot = serde_json::from_str(&json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(VocabularyRecord {
        id: row.get(0)?,
        headword: row.get(1)?,
        reference: snapshot.reference,
        digest: snapshot.digest,
        notes: row.get(3)?,
        revision: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

const COLUMNS: &str = "id, headword, snapshot, notes, revision, created_at, updated_at";

pub fn get(connection: &Connection, id: i64) -> Result<VocabularyRecord, StorageError> {
    Ok(connection.query_row(
        &format!("SELECT {COLUMNS} FROM vocabulary WHERE id = ?1"),
        [id],
        row_record,
    )?)
}

pub fn find(
    connection: &Connection,
    headword: &str,
) -> Result<Option<VocabularyRecord>, StorageError> {
    Ok(connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM vocabulary WHERE headword = ?1"),
            [headword.nfc().collect::<String>()],
            row_record,
        )
        .optional()?)
}

pub fn replay(
    connection: &Connection,
    request: &SaveRequest,
) -> Result<Option<SaveResult>, StorageError> {
    validate(request)?;
    let previous = connection
        .query_row(
            "SELECT request, record_id, outcome FROM save_operations WHERE operation = ?1",
            [&request.operation],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;
    match previous {
        None => Ok(None),
        Some((payload, id, outcome)) => {
            if payload != serde_json::to_string(request)? {
                return Err(StorageError::Invalid(
                    "Save identifier reused with different content".into(),
                ));
            }
            Ok(Some(SaveResult {
                outcome: if outcome == "existing" {
                    "existing"
                } else {
                    "replayed"
                },
                record: get(connection, id)?,
            }))
        }
    }
}

pub fn save(
    connection: &mut Connection,
    request: &SaveRequest,
    snapshot: Option<Snapshot>,
) -> Result<SaveResult, StorageError> {
    validate(request)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) = replay(&transaction, request)? {
        return Ok(result);
    }
    let headword = request.headword.nfc().collect::<String>();
    let (outcome, record) = if let Some(id) = request.id {
        let current = get(&transaction, id)?;
        if current.headword != headword {
            return Err(StorageError::Invalid("Headword is read-only".into()));
        }
        if Some(current.revision) != request.revision {
            return Ok(SaveResult {
                outcome: "conflict",
                record: current,
            });
        }
        transaction.execute(
            "UPDATE vocabulary SET notes = ?1, revision = revision + 1,
             updated_at = MAX(updated_at + 1, CAST(unixepoch('subsec') * 1000 AS INTEGER)) WHERE id = ?2",
            params![request.notes, id],
        )?;
        ("updated", get(&transaction, id)?)
    } else if let Some(existing) = find(&transaction, &headword)? {
        ("existing", existing)
    } else {
        let snapshot = snapshot
            .ok_or_else(|| StorageError::Invalid("Verified dictionary content required".into()))?;
        if snapshot.digest.is_empty() {
            return Err(StorageError::Invalid("Dictionary digest is empty".into()));
        }
        transaction.execute(
            "INSERT INTO vocabulary (headword, snapshot, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, CAST(unixepoch('subsec') * 1000 AS INTEGER), CAST(unixepoch('subsec') * 1000 AS INTEGER))",
            params![headword, serde_json::to_string(&snapshot)?, request.notes],
        )?;
        let id = transaction.last_insert_rowid();
        if crate::auto_enroll(&transaction)? {
            for kind in ["recognition", "listening"] {
                transaction.execute(
                    "INSERT INTO review_cards (vocabulary_id, kind) VALUES (?1, ?2)",
                    params![id, kind],
                )?;
            }
        }
        ("created", get(&transaction, id)?)
    };
    transaction.execute(
        "INSERT INTO save_operations (operation, request, record_id, outcome) VALUES (?1, ?2, ?3, ?4)",
        params![
            request.operation,
            serde_json::to_string(request)?,
            record.id, outcome
        ],
    )?;
    transaction.commit()?;
    Ok(SaveResult { outcome, record })
}

fn search_key(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect()
}

pub const DEFAULT_SORT: &str = "updated-desc";
pub fn validate_sort(sort: &str) -> Result<(), StorageError> {
    if [
        "headword-asc",
        "headword-desc",
        "created-asc",
        "created-desc",
        "updated-asc",
        "updated-desc",
    ]
    .contains(&sort)
    {
        Ok(())
    } else {
        Err(StorageError::Invalid("Invalid vocabulary sort".into()))
    }
}

pub fn sort_preference(connection: &Connection) -> Result<String, StorageError> {
    let sort = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'vocabulary-sort'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| DEFAULT_SORT.into());
    validate_sort(&sort)?;
    Ok(sort)
}

pub fn set_sort_preference(connection: &Connection, sort: &str) -> Result<(), StorageError> {
    validate_sort(sort)?;
    connection.execute(
        "INSERT INTO app_metadata (key, value) VALUES ('vocabulary-sort', ?1)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [sort],
    )?;
    Ok(())
}

pub fn list(
    connection: &Connection,
    query: &str,
    sort: &str,
) -> Result<Vec<VocabularyRecord>, StorageError> {
    validate_sort(sort)?;
    if query.chars().count() > 20_000 {
        return Err(StorageError::Invalid("Search is too long".into()));
    }
    let query = search_key(query);
    let mut statement = connection.prepare(&format!("SELECT {COLUMNS} FROM vocabulary"))?;
    let mut records = statement
        .query_map([], row_record)?
        .collect::<Result<Vec<_>, _>>()?;
    records.retain(|record| {
        let digest = record
            .digest
            .iter()
            .flat_map(|entry| {
                entry
                    .pronunciations
                    .iter()
                    .chain(&entry.grammar)
                    .chain(entry.translation.iter())
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        [&record.headword, &digest, &record.notes]
            .iter()
            .any(|value| search_key(value).contains(&query))
    });
    records.sort_by_cached_key(|record| match sort.split('-').next().unwrap_or_default() {
        "headword" => (0, search_key(&record.headword), record.id),
        "created" => (record.created_at, String::new(), record.id),
        _ => (record.updated_at, String::new(), record.id),
    });
    if sort.ends_with("-desc") {
        records.reverse();
    }
    Ok(records)
}
