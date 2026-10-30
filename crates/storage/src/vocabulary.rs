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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub fr: String,
    pub zh: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SaveRequest {
    pub operation: String,
    pub headword: String,
    pub notes: String,
    pub example_fr: String,
    pub example_zh: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub example_sense: Option<u32>,
    pub more_examples: Vec<Example>,
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
    pub example_fr: String,
    pub example_zh: String,
    pub example_sense: Option<u32>,
    pub more_examples: Vec<Example>,
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
    if request.example_fr.chars().count() > 20_000 || request.example_zh.chars().count() > 20_000 {
        return Err(StorageError::Invalid(
            "Example exceeds 20,000 Unicode characters".into(),
        ));
    }
    if request.more_examples.len() > 100
        || request
            .more_examples
            .iter()
            .any(|e| e.fr.chars().count() > 20_000 || e.zh.chars().count() > 20_000)
    {
        return Err(StorageError::Invalid(
            "Too many or oversized examples".into(),
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
        example_fr: row.get(7)?,
        example_zh: row.get(8)?,
        example_sense: row.get(10)?,
        more_examples: serde_json::from_str(&row.get::<_, String>(9)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
    })
}

const COLUMNS: &str = "id, headword, snapshot, notes, revision, created_at, updated_at, example_fr, example_zh, more_examples, example_sense";

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
            let mut stored: serde_json::Value = serde_json::from_str(&payload)?;
            if let Some(fields) = stored.as_object_mut() {
                fields.entry("example_fr").or_insert_with(|| "".into());
                fields.entry("example_zh").or_insert_with(|| "".into());
                fields
                    .entry("example_sense")
                    .or_insert(serde_json::Value::Null);
                fields
                    .entry("more_examples")
                    .or_insert_with(|| serde_json::json!([]));
            }
            if serde_json::from_value::<SaveRequest>(stored)? != *request {
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
    let has_examples = !request.example_fr.trim().is_empty()
        || !request.example_zh.trim().is_empty()
        || request
            .more_examples
            .iter()
            .any(|example| !example.fr.trim().is_empty() || !example.zh.trim().is_empty());
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
        let unchanged_legacy = current.example_sense.is_none()
            && current.example_fr == request.example_fr
            && current.example_zh == request.example_zh
            && current.more_examples == request.more_examples;
        if has_examples && request.example_sense.is_none() && !unchanged_legacy {
            return Err(StorageError::Invalid(
                "Select a meaning for new or edited examples".into(),
            ));
        }
        if request
            .example_sense
            .is_some_and(|index| index as usize >= current.digest.len())
        {
            return Err(StorageError::Invalid(
                "Example meaning is not in the saved summary".into(),
            ));
        }
        transaction.execute(
            "UPDATE vocabulary SET notes = ?1, example_fr = ?2, example_zh = ?3, more_examples = ?4, example_sense = ?5, revision = revision + 1,
             updated_at = MAX(updated_at + 1, CAST(unixepoch('subsec') * 1000 AS INTEGER)) WHERE id = ?6",
            params![request.notes, request.example_fr, request.example_zh, serde_json::to_string(&request.more_examples)?, request.example_sense, id],
        )?;
        ("updated", get(&transaction, id)?)
    } else if let Some(existing) = find(&transaction, &headword)? {
        ("existing", existing)
    } else {
        if has_examples && request.example_sense.is_none() {
            return Err(StorageError::Invalid(
                "Select a meaning for new or edited examples".into(),
            ));
        }
        let snapshot = snapshot
            .ok_or_else(|| StorageError::Invalid("Verified dictionary content required".into()))?;
        if snapshot.digest.is_empty() {
            return Err(StorageError::Invalid("Dictionary digest is empty".into()));
        }
        if request
            .example_sense
            .is_some_and(|index| index as usize >= snapshot.digest.len())
        {
            return Err(StorageError::Invalid(
                "Example meaning is not in the saved summary".into(),
            ));
        }
        transaction.execute(
            "INSERT INTO vocabulary (headword, snapshot, notes, example_fr, example_zh, more_examples, example_sense, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CAST(unixepoch('subsec') * 1000 AS INTEGER), CAST(unixepoch('subsec') * 1000 AS INTEGER))",
            params![headword, serde_json::to_string(&snapshot)?, request.notes, request.example_fr, request.example_zh, serde_json::to_string(&request.more_examples)?, request.example_sense],
        )?;
        let id = transaction.last_insert_rowid();
        if crate::auto_enroll(&transaction)? {
            crate::review::ensure_review_cards(&transaction, id)?;
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
