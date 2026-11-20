use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::StorageError;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dictionary_id: Option<String>,
    pub dictionary: String,
    pub lookup_key: String,
    pub version: String,
    pub attribution: Option<String>,
    pub license: Option<String>,
    pub source_url: Option<String>,
    pub license_url: Option<String>,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DigestEntry {
    pub pronunciations: Vec<String>,
    pub grammar: Vec<String>,
    pub translation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core: Option<lexique_core::dictionary::CoreSense>,
}

pub(crate) fn validate_core_digest(
    headword: &str,
    reference: &Reference,
    digest: &[DigestEntry],
) -> Result<(), StorageError> {
    let is_core =
        reference.dictionary_id.as_deref() == Some(lexique_core::dictionary::CORE_DICTIONARY_ID);
    if is_core && (digest.is_empty() || digest.len() > 200) {
        return Err(StorageError::Invalid(
            "Core records require 1–200 complete senses".into(),
        ));
    }
    let mut senses = std::collections::HashSet::new();
    for entry in digest {
        if let Some(core) = &entry.core {
            if !is_core
                || core.headword.nfc().collect::<String>() != headword
                || reference.lookup_key.nfc().collect::<String>() != headword
                || entry.translation.is_some()
                || entry.pronunciations != [core.ipa.clone()]
                || entry.grammar != [core.pos.clone()]
                || !senses.insert(&core.sense_id)
            {
                return Err(StorageError::Invalid(
                    "Core sense disagrees with its frozen record, provider or digest".into(),
                ));
            }
            core.validate().map_err(StorageError::Invalid)?;
        } else if is_core {
            return Err(StorageError::Invalid(
                "Core record is missing four-field senses".into(),
            ));
        }
    }
    Ok(())
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Reference>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SaveRequest {
    pub operation: String,
    pub headword: String,
    pub notes: String,
    pub example_fr: String,
    pub example_zh: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_source: Option<Reference>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub example_sense: Option<u32>,
    pub more_examples: Vec<Example>,
    pub id: Option<i64>,
    pub revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_ids: Vec<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VocabularyRecord {
    pub id: i64,
    pub headword: String,
    pub reference: Reference,
    pub digest: Vec<DigestEntry>,
    pub notes: String,
    pub example_fr: String,
    pub example_zh: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example_source: Option<Reference>,
    pub example_sense: Option<u32>,
    pub more_examples: Vec<Example>,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct SaveResult {
    pub outcome: &'static str,
    pub record: VocabularyRecord,
}

pub fn validate(request: &SaveRequest) -> Result<(), StorageError> {
    crate::categories::validate_category_ids(&request.category_ids)?;
    crate::grammar::validate_operation(&request.operation)?;
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
        category_ids: serde_json::from_str(&row.get::<_, String>(12)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
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
        example_source: row
            .get::<_, Option<String>>(11)?
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        more_examples: serde_json::from_str(&row.get::<_, String>(9)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
    })
}

const COLUMNS: &str = "id, headword, snapshot, notes, revision, created_at, updated_at, example_fr, example_zh, more_examples, example_sense, example_source, (SELECT json_group_array(category_id) FROM record_categories WHERE record_id = vocabulary.id)";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ReviewRecord {
    Vocabulary(Box<VocabularyRecord>),
    Grammar(crate::GrammarRecord),
}

impl ReviewRecord {
    pub fn revision(&self) -> i64 {
        match self {
            Self::Vocabulary(record) => record.revision,
            Self::Grammar(record) => record.revision,
        }
    }
}

pub fn get_review_records(
    connection: &Connection,
    ids: &[i64],
) -> Result<std::collections::HashMap<i64, ReviewRecord>, StorageError> {
    let mut records = get_batch(connection, ids)?
        .into_iter()
        .map(|(id, record)| (id, ReviewRecord::Vocabulary(Box::new(record))))
        .collect::<std::collections::HashMap<_, _>>();
    let candidates = ids
        .iter()
        .filter(|id| !records.contains_key(id))
        .copied()
        .collect::<Vec<_>>();
    for chunk in candidates.chunks(100) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let mut query = connection.prepare(&format!(
            "SELECT id FROM vocabulary WHERE record_kind = 'grammar' AND id IN ({placeholders})"
        ))?;
        let grammars = query
            .query_map(rusqlite::params_from_iter(chunk), |row| {
                row.get::<_, i64>(0)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for id in grammars {
            records.insert(
                id,
                ReviewRecord::Grammar(crate::get_grammar(connection, id)?),
            );
        }
    }
    Ok(records)
}

pub(crate) fn review_due_key(
    due: &std::collections::HashMap<i64, Option<i64>>,
    id: i64,
    title: &str,
) -> (u8, i64, String, i64) {
    let (group, time) = match due.get(&id) {
        Some(Some(time)) => (0, *time),
        Some(None) => (1, 0),
        None => (2, 0),
    };
    (group, time, search_key(title), id)
}

pub fn get(connection: &Connection, id: i64) -> Result<VocabularyRecord, StorageError> {
    Ok(connection.query_row(
        &format!("SELECT {COLUMNS} FROM vocabulary WHERE id = ?1 AND record_kind = 'vocabulary'"),
        [id],
        row_record,
    )?)
}

pub fn get_batch(
    connection: &Connection,
    ids: &[i64],
) -> Result<std::collections::HashMap<i64, VocabularyRecord>, StorageError> {
    let mut records = std::collections::HashMap::new();
    for chunk in ids.chunks(100) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let mut query = connection.prepare(&format!(
            "SELECT {COLUMNS} FROM vocabulary WHERE record_kind = 'vocabulary' AND id IN ({placeholders})"
        ))?;
        for record in query.query_map(rusqlite::params_from_iter(chunk), row_record)? {
            let record = record?;
            records.insert(record.id, record);
        }
    }
    Ok(records)
}

pub fn find(
    connection: &Connection,
    headword: &str,
) -> Result<Option<VocabularyRecord>, StorageError> {
    Ok(connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM vocabulary WHERE headword = ?1 AND record_kind = 'vocabulary'"),
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
    crate::grammar::check_save_operation(connection, &request.operation)?;
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
            normalize_save_request_fields(&mut stored);
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

pub(crate) fn normalize_save_request_fields(value: &mut serde_json::Value) {
    if let Some(fields) = value.as_object_mut() {
        fields.entry("example_fr").or_insert_with(|| "".into());
        fields.entry("example_zh").or_insert_with(|| "".into());
        fields
            .entry("example_source")
            .or_insert(serde_json::Value::Null);
        fields
            .entry("example_sense")
            .or_insert(serde_json::Value::Null);
        fields
            .entry("more_examples")
            .or_insert_with(|| serde_json::json!([]));
        fields
            .entry("category_ids")
            .or_insert_with(|| serde_json::json!([]));
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
            "UPDATE vocabulary SET notes = ?1, example_fr = ?2, example_zh = ?3, more_examples = ?4, example_sense = ?5, example_source = ?7, revision = revision + 1,
             updated_at = MAX(updated_at + 1, CAST(unixepoch('subsec') * 1000 AS INTEGER)) WHERE id = ?6",
            params![request.notes, request.example_fr, request.example_zh, serde_json::to_string(&request.more_examples)?, request.example_sense, id, request.example_source.as_ref().map(serde_json::to_string).transpose()?],
        )?;
        crate::categories::save_record_categories(&transaction, id, &request.category_ids)?;
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
        validate_core_digest(&headword, &snapshot.reference, &snapshot.digest)?;
        if request
            .example_sense
            .is_some_and(|index| index as usize >= snapshot.digest.len())
        {
            return Err(StorageError::Invalid(
                "Example meaning is not in the saved summary".into(),
            ));
        }
        transaction.execute(
            "INSERT INTO vocabulary (headword, snapshot, notes, example_fr, example_zh, more_examples, example_sense, example_source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CAST(unixepoch('subsec') * 1000 AS INTEGER), CAST(unixepoch('subsec') * 1000 AS INTEGER))",
            params![headword, serde_json::to_string(&snapshot)?, request.notes, request.example_fr, request.example_zh, serde_json::to_string(&request.more_examples)?, request.example_sense, request.example_source.as_ref().map(serde_json::to_string).transpose()?],
        )?;
        let id = transaction.last_insert_rowid();
        crate::categories::save_record_categories(&transaction, id, &request.category_ids)?;
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
    let audio_now = transaction.query_row("SELECT unixepoch()", [], |row| row.get(0))?;
    crate::attach_word_audio_at(&transaction, record.id, audio_now)?;
    if outcome == "created" && crate::auto_enroll(&transaction)? {
        let spec = crate::SpeechSpec::word(
            &record.headword,
            crate::speech_preferences(&transaction)?.voice,
        );
        crate::demand_speech(&transaction, &spec, 1, audio_now, false)?;
    }
    transaction.commit()?;
    Ok(SaveResult { outcome, record })
}

pub(crate) fn search_key(value: &str) -> String {
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
        "review-due-asc",
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
    let mut statement = connection.prepare(&format!(
        "SELECT {COLUMNS} FROM vocabulary WHERE record_kind = 'vocabulary'"
    ))?;
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
                    .chain(entry.core.iter().flat_map(|core| {
                        [
                            &core.definition_fr,
                            &core.ipa,
                            &core.meaning_en,
                            &core.meaning_zh,
                            &core.usage,
                            &core.source_headword,
                        ]
                    }))
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        [&record.headword, &digest, &record.notes]
            .iter()
            .any(|value| search_key(value).contains(&query))
    });
    if sort == "review-due-asc" {
        let due = crate::review::review_due_times(connection)?;
        records.sort_by_cached_key(|record| review_due_key(&due, record.id, &record.headword));
        return Ok(records);
    }
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
