use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::{GrammarCardRequest, GrammarRecord, GrammarSaveRequest, StorageError};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GrammarImportContent {
    pub exchange_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub question: Option<String>,
    pub answer: Option<String>,
}

impl GrammarImportContent {
    pub fn pair(&self) -> Result<Option<(&str, &str)>, StorageError> {
        let question = self.question.as_deref().unwrap_or_default();
        let answer = self.answer.as_deref().unwrap_or_default();
        match (question.trim().is_empty(), answer.trim().is_empty()) {
            (true, true) => Ok(None),
            (false, false) => Ok(Some((question, answer))),
            _ => Err(StorageError::Invalid(
                "Question and answer must be supplied together".into(),
            )),
        }
    }

    pub fn validate(&self) -> Result<(), StorageError> {
        if let Some(identity) = self.exchange_id.as_deref().filter(|id| !id.is_empty()) {
            crate::grammar::validate_exchange_id(identity)?;
        }
        crate::grammar::validate_grammar_request(&GrammarSaveRequest {
            operation: "import-validation".into(),
            id: None,
            revision: None,
            title: self.title.clone(),
            body: self.body.clone().unwrap_or_default(),
            category_ids: vec![],
        })?;
        if let Some((question, answer)) = self.pair()? {
            crate::grammar::validate_grammar_card_request(&GrammarCardRequest {
                operation: "import-validation".into(),
                id: 1,
                revision: 1,
                question: question.into(),
                answer: answer.into(),
            })?;
        }
        Ok(())
    }

    pub fn unchanged(&self, existing: &GrammarRecord) -> Result<bool, StorageError> {
        Ok(self.title.trim() == existing.title
            && self.body.as_ref().is_none_or(|body| *body == existing.body)
            && self.pair()?.is_none_or(|(question, answer)| {
                existing
                    .card
                    .as_ref()
                    .is_some_and(|card| card.question == question && card.answer == answer)
            }))
    }
}

pub fn find_grammar_exchange(
    connection: &Connection,
    identity: &str,
) -> Result<Option<GrammarRecord>, StorageError> {
    let id = connection
        .query_row(
            "SELECT record_id FROM grammar_exchange_identities WHERE exchange_id = ?1",
            [identity],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    id.map(|id| crate::get_grammar(connection, id)).transpose()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarImportResult {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub records: Vec<GrammarRecord>,
    /// Local record identities whose newly created cards can explicitly start review.
    pub new_card_ids: Vec<i64>,
}

/// Compatibility entry for the original single-note create/skip contract.
pub fn save_grammar_import(
    connection: &mut Connection,
    operation: &str,
    content: &GrammarImportContent,
    existing: Option<&GrammarRecord>,
    now: i64,
) -> Result<GrammarImportResult, StorageError> {
    if let Some(record) = existing
        && !content.unchanged(record)?
    {
        return Err(StorageError::Invalid(
            "Existing grammar note differs; use selected batch update".into(),
        ));
    }
    save_grammar_import_batch(
        connection,
        operation,
        &[GrammarImportItem {
            source_index: 0,
            content: content.clone(),
            existing: existing.cloned(),
        }],
        0,
        now,
    )
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarImportItem {
    /// Distinguishes independent source groups even when their parsed text is equal.
    pub source_index: usize,
    pub content: GrammarImportContent,
    pub existing: Option<GrammarRecord>,
}

#[derive(Deserialize, Serialize)]
struct BatchReceipt {
    request: String,
    result: GrammarImportResult,
}

/// Recheck the selected preview and save every content/card/pause change together.
pub fn save_grammar_import_batch(
    connection: &mut Connection,
    operation: &str,
    items: &[GrammarImportItem],
    skipped: usize,
    now: i64,
) -> Result<GrammarImportResult, StorageError> {
    crate::grammar::validate_operation(operation)?;
    if !(0..=9_007_199_254_740_991).contains(&now) {
        return Err(StorageError::Invalid("Invalid note timestamp".into()));
    }
    let mut identities = std::collections::HashSet::new();
    for item in items {
        item.content.validate()?;
        if let Some(identity) = item
            .content
            .exchange_id
            .as_deref()
            .filter(|id| !id.is_empty())
            && !identities.insert(identity)
        {
            return Err(StorageError::Invalid(
                "Duplicate selected grammar identity".into(),
            ));
        }
    }
    let request = serde_json::to_string(&(items, skipped))?;
    let key = format!("grammar-import-batch-{operation}");
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(receipt) = tx
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [&key],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        let receipt: BatchReceipt = serde_json::from_str(&receipt)?;
        if receipt.request != request {
            return Err(StorageError::Invalid(
                "Import identifier reused with different selection or content".into(),
            ));
        }
        return Ok(receipt.result);
    }
    let mut result = GrammarImportResult {
        created: 0,
        updated: 0,
        skipped,
        records: vec![],
        new_card_ids: vec![],
    };
    for item in items {
        let identity = item
            .content
            .exchange_id
            .as_deref()
            .filter(|id| !id.is_empty());
        let current = identity
            .map(|id| find_grammar_exchange(&tx, id))
            .transpose()?
            .flatten();
        if current.as_ref().map(|record| (record.id, record.revision))
            != item
                .existing
                .as_ref()
                .map(|record| (record.id, record.revision))
        {
            return Err(StorageError::Invalid(
                "Record changed after preview; preview again".into(),
            ));
        }
        let id = if let Some(current) = current {
            if item.content.unchanged(&current)? {
                result.skipped += 1;
                continue;
            }
            tx.execute("UPDATE vocabulary SET headword = ?1, notes = ?2, revision = revision + 1, updated_at = MAX(updated_at + 1, ?3) WHERE id = ?4", params![item.content.title.trim(), item.content.body.as_deref().unwrap_or(&current.body), now, current.id])?;
            result.updated += 1;
            current.id
        } else {
            tx.execute("INSERT INTO vocabulary(headword, notes, snapshot, record_kind, created_at, updated_at) VALUES (?1, ?2, NULL, 'grammar', ?3, ?3)", params![item.content.title.trim(), item.content.body.as_deref().unwrap_or_default(), now])?;
            let id = tx.last_insert_rowid();
            if let Some(identity) = identity {
                tx.execute(
                    "UPDATE grammar_exchange_identities SET exchange_id = ?1 WHERE record_id = ?2",
                    params![identity, id],
                )?;
            }
            result.created += 1;
            id
        };
        if let Some((question, answer)) = item.content.pair()? {
            let existing_card = crate::get_grammar(&tx, id)?.card;
            let card_id = if let Some(card) = &existing_card {
                card.id
            } else {
                tx.execute(
                    "INSERT INTO review_cards(vocabulary_id, kind) VALUES (?1, 'grammar')",
                    [id],
                )?;
                tx.last_insert_rowid()
            };
            tx.execute("INSERT INTO grammar_cards(card_id, question, answer) VALUES (?1, ?2, ?3) ON CONFLICT(card_id) DO UPDATE SET question = excluded.question, answer = excluded.answer", params![card_id, question, answer])?;
            if existing_card.is_none() {
                tx.execute(
                    "INSERT INTO review_paused(vocabulary_id) VALUES (?1) ON CONFLICT DO NOTHING",
                    [id],
                )?;
                result.new_card_ids.push(id);
            }
        }
        result.records.push(crate::get_grammar(&tx, id)?);
    }
    let receipt = BatchReceipt {
        request,
        result: result.clone(),
    };
    tx.execute(
        "INSERT INTO app_metadata(key, value) VALUES (?1, ?2)",
        params![key, serde_json::to_string(&receipt)?],
    )?;
    tx.commit()?;
    Ok(result)
}
