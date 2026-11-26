use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::{
    StorageError,
    categories::{record_category_ids, save_record_categories, validate_category_ids},
    vocabulary::search_key,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GrammarCard {
    pub id: i64,
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GrammarRecord {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub category_ids: Vec<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub card: Option<GrammarCard>,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GrammarSaveRequest {
    pub operation: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub id: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub revision: Option<i64>,
    pub title: String,
    pub body: String,
    pub category_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct GrammarSaveResult {
    pub outcome: &'static str,
    pub record: GrammarRecord,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarCardRequest {
    pub operation: String,
    pub id: i64,
    pub revision: i64,
    pub question: String,
    pub answer: String,
}

fn is_ascii_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(crate) fn validate_operation(operation: &str) -> Result<(), StorageError> {
    if !is_ascii_identifier(operation) {
        return Err(StorageError::Invalid(
            "Invalid save operation identifier".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_exchange_id(identity: &str) -> Result<(), StorageError> {
    if !is_ascii_identifier(identity) {
        return Err(StorageError::Invalid(
            "Invalid grammar exchange identity".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_title(title: &str) -> Result<(), StorageError> {
    if title.trim().is_empty()
        || title.trim().chars().count() > 200
        || title
            .chars()
            .any(|character| character.is_control() || matches!(character, '\u{2028}' | '\u{2029}'))
    {
        return Err(StorageError::Invalid(
            "Title must be 1–200 characters on one line".into(),
        ));
    }
    Ok(())
}

pub fn get_grammar(connection: &Connection, id: i64) -> Result<GrammarRecord, StorageError> {
    if !(1..=9_007_199_254_740_991).contains(&id) {
        return Err(StorageError::Invalid(
            "Invalid grammar record identity".into(),
        ));
    }
    let mut record = connection.query_row(
        "SELECT id, headword, notes, revision, created_at, updated_at FROM vocabulary WHERE record_kind = 'grammar' AND id = ?1",
        [id], |row| Ok(GrammarRecord { id: row.get(0)?, title: row.get(1)?, body: row.get(2)?,
            revision: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)?,
            category_ids: vec![], card: None }))?;
    record.category_ids = record_category_ids(connection, id)?;
    record.card = connection.query_row(
        "SELECT c.id, g.question, g.answer FROM review_cards c JOIN grammar_cards g ON g.card_id = c.id WHERE c.vocabulary_id = ?1 AND c.kind = 'grammar'",
        [id], |row| Ok(GrammarCard { id: row.get(0)?, question: row.get(1)?, answer: row.get(2)? })).optional()?;
    Ok(record)
}

pub(crate) fn validate_grammar_request(request: &GrammarSaveRequest) -> Result<(), StorageError> {
    validate_operation(&request.operation)?;
    validate_title(&request.title)?;
    validate_category_ids(&request.category_ids)?;
    if request.body.chars().count() > 20_000
        || request.id.is_some() != request.revision.is_some()
        || request
            .id
            .is_some_and(|id| id <= 0 || id > 9_007_199_254_740_991)
        || request
            .revision
            .is_some_and(|revision| revision <= 0 || revision > 9_007_199_254_740_991)
    {
        return Err(StorageError::Invalid(
            "Invalid grammar note content or revision".into(),
        ));
    }
    Ok(())
}

pub fn save_grammar(
    connection: &Connection,
    request: &GrammarSaveRequest,
    now: i64,
) -> Result<GrammarSaveResult, StorageError> {
    validate_grammar_request(request)?;
    if !(0..=9_007_199_254_740_991).contains(&now) {
        return Err(StorageError::Invalid("Invalid note timestamp".into()));
    }
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let payload = serde_json::to_string(request)?;
    if let Some(previous) = replay_grammar(&tx, &request.operation, &payload)? {
        return Ok(previous);
    }
    let (outcome, id) = if let Some(id) = request.id {
        let current = get_grammar(&tx, id)?;
        if Some(current.revision) != request.revision {
            return Ok(GrammarSaveResult {
                outcome: "conflict",
                record: current,
            });
        }
        tx.execute("UPDATE vocabulary SET headword = ?1, notes = ?2, revision = revision + 1, updated_at = MAX(updated_at + 1, ?3) WHERE id = ?4", params![request.title.trim(), request.body, now, id])?;
        ("updated", id)
    } else {
        tx.execute("INSERT INTO vocabulary(headword, notes, snapshot, record_kind, created_at, updated_at) VALUES (?1, ?2, NULL, 'grammar', ?3, ?3)", params![request.title.trim(), request.body, now])?;
        ("created", tx.last_insert_rowid())
    };
    save_record_categories(&tx, id, &request.category_ids)?;
    tx.execute("INSERT INTO save_operations(operation, request, record_id, outcome) VALUES (?1, ?2, ?3, ?4)", params![request.operation, payload, id, outcome])?;
    let record = get_grammar(&tx, id)?;
    tx.commit()?;
    Ok(GrammarSaveResult { outcome, record })
}

pub(crate) fn check_save_operation(
    connection: &Connection,
    operation: &str,
) -> Result<(), StorageError> {
    if connection
        .prepare("SELECT 1 FROM deleted_grammar_operations WHERE operation = ?1")?
        .exists([operation])?
    {
        return Err(StorageError::Invalid(
            "Save operation belongs to a deleted grammar note".into(),
        ));
    }
    Ok(())
}

fn replay_grammar(
    connection: &Connection,
    operation: &str,
    payload: &str,
) -> Result<Option<GrammarSaveResult>, StorageError> {
    check_save_operation(connection, operation)?;
    let previous = connection
        .query_row(
            "SELECT request, record_id FROM save_operations WHERE operation = ?1",
            [operation],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?;
    let Some((saved, id)) = previous else {
        return Ok(None);
    };
    if saved != payload {
        return Err(StorageError::Invalid(
            "Save identifier reused with different content".into(),
        ));
    }
    Ok(Some(GrammarSaveResult {
        outcome: "replayed",
        record: get_grammar(connection, id)?,
    }))
}

pub(crate) fn validate_grammar_card_request(
    request: &GrammarCardRequest,
) -> Result<(), StorageError> {
    validate_operation(&request.operation)?;
    if request.id <= 0
        || request.revision <= 0
        || request.id > 9_007_199_254_740_991
        || request.revision > 9_007_199_254_740_991
        || [&request.question, &request.answer]
            .iter()
            .any(|text| text.trim().is_empty() || text.chars().count() > 20_000)
    {
        return Err(StorageError::Invalid(
            "Question and answer must contain 1–20,000 characters".into(),
        ));
    }
    Ok(())
}

pub fn save_grammar_card(
    connection: &Connection,
    request: &GrammarCardRequest,
    now: i64,
) -> Result<GrammarSaveResult, StorageError> {
    validate_grammar_card_request(request)?;
    if !(0..=9_007_199_254_740_991).contains(&now) {
        return Err(StorageError::Invalid("Invalid note timestamp".into()));
    }
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let payload = serde_json::to_string(request)?;
    if let Some(previous) = replay_grammar(&tx, &request.operation, &payload)? {
        return Ok(previous);
    }
    let current = get_grammar(&tx, request.id)?;
    if current.revision != request.revision {
        return Ok(GrammarSaveResult {
            outcome: "conflict",
            record: current,
        });
    }
    tx.execute("INSERT INTO review_cards(vocabulary_id, kind) VALUES (?1, 'grammar') ON CONFLICT(vocabulary_id, kind) DO NOTHING", [request.id])?;
    let card_id = tx.query_row(
        "SELECT id FROM review_cards WHERE vocabulary_id = ?1 AND kind = 'grammar'",
        [request.id],
        |row| row.get::<_, i64>(0),
    )?;
    tx.execute("INSERT INTO grammar_cards(card_id, question, answer) VALUES (?1, ?2, ?3) ON CONFLICT(card_id) DO UPDATE SET question = excluded.question, answer = excluded.answer", params![card_id, request.question, request.answer])?;
    tx.execute("UPDATE vocabulary SET revision = revision + 1, updated_at = MAX(updated_at + 1, ?1) WHERE id = ?2", params![now, request.id])?;
    tx.execute("INSERT INTO save_operations(operation, request, record_id, outcome) VALUES (?1, ?2, ?3, 'updated')", params![request.operation, payload, request.id])?;
    let record = get_grammar(&tx, request.id)?;
    tx.commit()?;
    Ok(GrammarSaveResult {
        outcome: "updated",
        record,
    })
}

pub fn delete_grammar(connection: &Connection, id: i64, revision: i64) -> Result<(), StorageError> {
    if [id, revision]
        .iter()
        .any(|value| !(1..=9_007_199_254_740_991).contains(value))
    {
        return Err(StorageError::Invalid(
            "Invalid grammar record identity or revision".into(),
        ));
    }
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let current = tx
        .query_row(
            "SELECT record_kind, revision FROM vocabulary WHERE id = ?1",
            [id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?;
    let Some((kind, current_revision)) = current else {
        return Ok(());
    };
    if kind != "grammar" {
        return Err(StorageError::Invalid("Record is not a grammar note".into()));
    }
    if current_revision != revision {
        return Err(StorageError::RatingConflict);
    }
    crate::review::delete_grammar_learning(&tx, id)?;
    tx.execute(
        "INSERT INTO deleted_grammar_operations SELECT operation FROM save_operations WHERE record_id = ?1",
        [id],
    )?;
    tx.execute("DELETE FROM save_operations WHERE record_id = ?1", [id])?;
    tx.execute(
        "DELETE FROM vocabulary WHERE id = ?1 AND record_kind = 'grammar'",
        [id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn grammar_sort_preference(connection: &Connection) -> Result<String, StorageError> {
    let value = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'grammar-sort'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| crate::DEFAULT_SORT.into());
    crate::validate_sort(&value)?;
    Ok(value)
}

pub fn set_grammar_sort_preference(
    connection: &Connection,
    sort: &str,
) -> Result<(), StorageError> {
    crate::validate_sort(sort)?;
    connection.execute("INSERT INTO app_metadata(key, value) VALUES ('grammar-sort', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [sort])?;
    Ok(())
}

pub fn list_grammar(
    connection: &Connection,
    query: &str,
    sort: &str,
) -> Result<Vec<GrammarRecord>, StorageError> {
    crate::validate_sort(sort)?;
    if query.chars().count() > 20_000 {
        return Err(StorageError::Invalid("Search is too long".into()));
    }
    let query = search_key(query);
    let ids = connection
        .prepare("SELECT id FROM vocabulary WHERE record_kind = 'grammar'")?
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut records = ids
        .into_iter()
        .map(|id| get_grammar(connection, id))
        .collect::<Result<Vec<_>, _>>()?;
    records.retain(|record| {
        [&record.title, &record.body]
            .iter()
            .any(|value| search_key(value).contains(&query))
    });
    if sort == "review-due-asc" {
        let due = crate::review::review_due_times(connection)?;
        records.sort_by_cached_key(|record| {
            crate::vocabulary::review_due_key(&due, record.id, &record.title)
        });
        return Ok(records);
    }
    records.sort_by_cached_key(|record| match sort.split('-').next().unwrap_or_default() {
        "headword" => (0, search_key(&record.title), record.id),
        "created" => (record.created_at, String::new(), record.id),
        _ => (record.updated_at, String::new(), record.id),
    });
    if sort.ends_with("-desc") {
        records.reverse();
    }
    Ok(records)
}

pub fn deleted_grammar_operations(connection: &Connection) -> Result<Vec<String>, StorageError> {
    Ok(connection
        .prepare("SELECT operation FROM deleted_grammar_operations ORDER BY operation")?
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?)
}

pub fn preserve_deleted_grammar_operations(
    connection: &Connection,
    operations: &[String],
) -> Result<(), StorageError> {
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let mut statement = tx.prepare(
        "INSERT OR IGNORE INTO deleted_grammar_operations(operation) SELECT ?1 WHERE NOT EXISTS (SELECT 1 FROM save_operations WHERE operation = ?1)",
    )?;
    for operation in operations {
        validate_operation(operation)?;
        statement.execute([operation])?;
    }
    drop(statement);
    tx.commit()?;
    Ok(())
}

pub fn learning_identities(
    connection: &Connection,
) -> Result<crate::SnapshotIdentities, StorageError> {
    let sequence = |name| -> Result<i64, StorageError> {
        Ok(connection.query_row(
            "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name = ?1), 0)",
            [name],
            |row| row.get(0),
        )?)
    };
    Ok(crate::SnapshotIdentities {
        records: sequence("vocabulary")?,
        cards: sequence("review_cards")?,
        categories: sequence("learning_categories")?,
    })
}

pub fn preserve_learning_identities(
    connection: &Connection,
    identities: crate::SnapshotIdentities,
) -> Result<(), StorageError> {
    let current = learning_identities(connection)?;
    let sequences = [
        ("vocabulary", current.records.max(identities.records)),
        ("review_cards", current.cards.max(identities.cards)),
        (
            "learning_categories",
            current.categories.max(identities.categories),
        ),
    ];
    if sequences
        .iter()
        .any(|(_, value)| !(0..=9_007_199_254_740_991).contains(value))
    {
        return Err(StorageError::Invalid(
            "Invalid learning identity watermark".into(),
        ));
    }
    for (name, value) in sequences {
        connection.execute("DELETE FROM sqlite_sequence WHERE name = ?1", [name])?;
        connection.execute(
            "INSERT INTO sqlite_sequence(name, seq) VALUES (?1, ?2)",
            params![name, value],
        )?;
    }
    Ok(())
}
