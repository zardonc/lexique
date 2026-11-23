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
            crate::grammar::validate_operation(identity)?;
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
    pub skipped: usize,
    pub records: Vec<GrammarRecord>,
}

#[derive(Deserialize, Serialize)]
struct Receipt {
    request: String,
    created: usize,
    skipped: usize,
    ids: Vec<i64>,
}

/// A single canonical note, its identity and any paused card share one commit.
pub fn save_grammar_import(
    connection: &mut Connection,
    operation: &str,
    content: &GrammarImportContent,
    existing: Option<&GrammarRecord>,
    now: i64,
) -> Result<GrammarImportResult, StorageError> {
    crate::grammar::validate_operation(operation)?;
    content.validate()?;
    if !(0..=9_007_199_254_740_991).contains(&now) {
        return Err(StorageError::Invalid("Invalid note timestamp".into()));
    }
    let request = serde_json::to_string(&(content, existing))?;
    let key = format!("grammar-import-{operation}");
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let receipt = tx
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [&key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    if let Some(receipt) = receipt {
        let receipt: Receipt = serde_json::from_str(&receipt)?;
        if receipt.request != request {
            return Err(StorageError::Invalid(
                "Import identifier reused with different content".into(),
            ));
        }
        return Ok(GrammarImportResult {
            created: receipt.created,
            skipped: receipt.skipped,
            records: receipt
                .ids
                .into_iter()
                .map(|id| crate::get_grammar(&tx, id))
                .collect::<Result<_, _>>()?,
        });
    }
    let identity = content.exchange_id.as_deref().filter(|id| !id.is_empty());
    let current = identity
        .map(|id| find_grammar_exchange(&tx, id))
        .transpose()?
        .flatten();
    if current.as_ref().map(|record| (record.id, record.revision))
        != existing.map(|record| (record.id, record.revision))
    {
        return Err(StorageError::Invalid(
            "Record changed after preview; preview again".into(),
        ));
    }
    let mut result = GrammarImportResult {
        created: 0,
        skipped: 0,
        records: vec![],
    };
    if let Some(current) = current {
        if !content.unchanged(&current)? {
            return Err(StorageError::Invalid(
                "Existing grammar note differs; updates require the batch import slice".into(),
            ));
        }
        result.skipped = 1;
    } else {
        tx.execute("INSERT INTO vocabulary(headword, notes, snapshot, record_kind, created_at, updated_at) VALUES (?1, ?2, NULL, 'grammar', ?3, ?3)", params![content.title.trim(), content.body.as_deref().unwrap_or_default(), now])?;
        let id = tx.last_insert_rowid();
        if let Some(identity) = identity {
            tx.execute(
                "UPDATE grammar_exchange_identities SET exchange_id = ?1 WHERE record_id = ?2",
                params![identity, id],
            )?;
        }
        if let Some((question, answer)) = content.pair()? {
            tx.execute(
                "INSERT INTO review_cards(vocabulary_id, kind) VALUES (?1, 'grammar')",
                [id],
            )?;
            let card_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO grammar_cards(card_id, question, answer) VALUES (?1, ?2, ?3)",
                params![card_id, question, answer],
            )?;
            tx.execute("INSERT INTO review_paused(vocabulary_id) VALUES (?1)", [id])?;
        }
        result.created = 1;
        result.records.push(crate::get_grammar(&tx, id)?);
    }
    let receipt = Receipt {
        request,
        created: result.created,
        skipped: result.skipped,
        ids: result.records.iter().map(|record| record.id).collect(),
    };
    tx.execute(
        "INSERT INTO app_metadata(key, value) VALUES (?1, ?2)",
        params![key, serde_json::to_string(&receipt)?],
    )?;
    tx.commit()?;
    Ok(result)
}
