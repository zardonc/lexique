use crate::{NoteExportCategory, NoteExportCategoryName, NoteExportScope, StorageError};
use rusqlite::{Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GrammarExchangeIdentity {
    pub record_id: i64,
    pub exchange_id: String,
}

pub(crate) fn grammar_exchange_identities(
    connection: &Connection,
) -> Result<Vec<GrammarExchangeIdentity>, StorageError> {
    Ok(connection
        .prepare(
            "SELECT record_id, exchange_id FROM grammar_exchange_identities ORDER BY record_id",
        )?
        .query_map([], |row| {
            Ok(GrammarExchangeIdentity {
                record_id: row.get(0)?,
                exchange_id: row.get(1)?,
            })
        })?
        .collect::<Result<_, _>>()?)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarExportRecord {
    pub id: i64,
    pub exchange_id: String,
    pub title: String,
    pub body: String,
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarExportSnapshot {
    pub records: Vec<GrammarExportRecord>,
}

/// Scope, count, ordering, saved text and exchange identities share one read view.
pub fn capture_grammar_export(
    connection: &mut Connection,
    scope: &NoteExportScope,
) -> Result<GrammarExportSnapshot, StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
    let mut records = match scope {
        NoteExportScope::All { sort } => crate::list_grammar(&tx, "", sort)?,
        NoteExportScope::Search { query, sort, .. } => crate::list_grammar(&tx, query, sort)?,
        NoteExportScope::Single { id } => vec![crate::get_grammar(&tx, *id)?],
    };
    if let NoteExportScope::Search { category, .. } = scope {
        match category {
            NoteExportCategory::Named(NoteExportCategoryName::All) => {}
            NoteExportCategory::Named(NoteExportCategoryName::Uncategorized) => {
                records.retain(|record| record.category_ids.is_empty())
            }
            NoteExportCategory::Id(id) => {
                if !(1..=9_007_199_254_740_991).contains(id) {
                    return Err(StorageError::Invalid("Invalid category ID".into()));
                }
                records.retain(|record| record.category_ids.contains(id));
            }
        }
    }
    let records = records
        .into_iter()
        .map(|record| {
            let exchange_id = tx.query_row(
                "SELECT exchange_id FROM grammar_exchange_identities WHERE record_id = ?1",
                [record.id],
                |row| row.get(0),
            )?;
            let (question, answer) = record
                .card
                .map(|card| (card.question, card.answer))
                .unwrap_or_default();
            Ok(GrammarExportRecord {
                id: record.id,
                exchange_id,
                title: record.title,
                body: record.body,
                question,
                answer,
            })
        })
        .collect::<Result<_, StorageError>>()?;
    tx.commit()?;
    Ok(GrammarExportSnapshot { records })
}
