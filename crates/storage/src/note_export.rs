use rusqlite::{Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};

use crate::{StorageError, get, list};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum NoteExportCategory {
    Named(NoteExportCategoryName),
    Id(i64),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteExportCategoryName {
    All,
    Uncategorized,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NoteExportScope {
    All {
        sort: String,
    },
    Search {
        query: String,
        sort: String,
        category: NoteExportCategory,
    },
    Single {
        id: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteExportExample {
    pub fr: String,
    pub zh: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteExportRecord {
    pub id: i64,
    pub headword: String,
    pub notes: String,
    pub examples: Vec<NoteExportExample>,
}

#[derive(Clone, Debug)]
pub struct NoteExportSnapshot {
    pub records: Vec<NoteExportRecord>,
}

impl NoteExportSnapshot {
    pub fn row_count(&self) -> usize {
        self.records
            .iter()
            .map(|record| record.examples.len().max(1))
            .sum()
    }
}

/// Scope, ordering and text share one committed read view, including due times and categories.
pub fn capture_note_export(
    connection: &mut Connection,
    scope: &NoteExportScope,
) -> Result<NoteExportSnapshot, StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
    let mut records = match scope {
        NoteExportScope::All { sort } => list(&tx, "", sort)?,
        NoteExportScope::Search { query, sort, .. } => list(&tx, query, sort)?,
        NoteExportScope::Single { id } => {
            if *id <= 0 || *id > 9_007_199_254_740_991 {
                return Err(StorageError::Invalid("Invalid vocabulary ID".into()));
            }
            vec![get(&tx, *id)?]
        }
    };
    if let NoteExportScope::Search { category, .. } = scope {
        match category {
            NoteExportCategory::Named(NoteExportCategoryName::All) => {}
            NoteExportCategory::Named(NoteExportCategoryName::Uncategorized) => {
                records.retain(|record| record.category_ids.is_empty());
            }
            NoteExportCategory::Id(id) => {
                if *id <= 0 || *id > 9_007_199_254_740_991 {
                    return Err(StorageError::Invalid("Invalid category ID".into()));
                }
                records.retain(|record| record.category_ids.contains(id));
            }
        }
    }
    let records = records
        .into_iter()
        .map(|record| {
            let mut examples = Vec::new();
            if !record.example_fr.is_empty() || !record.example_zh.is_empty() {
                examples.push(NoteExportExample {
                    fr: record.example_fr,
                    zh: record.example_zh,
                });
            }
            examples.extend(
                record
                    .more_examples
                    .into_iter()
                    .filter(|example| !example.fr.is_empty() || !example.zh.is_empty())
                    .map(|example| NoteExportExample {
                        fr: example.fr,
                        zh: example.zh,
                    }),
            );
            NoteExportRecord {
                id: record.id,
                headword: record.headword,
                notes: record.notes,
                examples,
            }
        })
        .collect();
    tx.commit()?;
    Ok(NoteExportSnapshot { records })
}
