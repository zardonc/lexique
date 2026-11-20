use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::{Example, ImportSource, SaveRequest, Snapshot, StorageError, VocabularyRecord};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoteExample {
    pub fr: String,
    pub zh: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteImportItem {
    pub content: NoteImportContent,
    pub existing: Option<VocabularyRecord>,
    pub source: ImportSource,
    pub snapshot: Option<Snapshot>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NoteImportResult {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub records: Vec<VocabularyRecord>,
}

#[derive(Deserialize, Serialize)]
struct ImportReceipt {
    request: String,
    created: usize,
    updated: usize,
    skipped: usize,
    ids: Vec<i64>,
}

pub fn save_note_import_batch(
    connection: &mut Connection,
    operation: &str,
    items: &[NoteImportItem],
    skipped: usize,
) -> Result<NoteImportResult, StorageError> {
    crate::grammar::validate_operation(operation)?;
    let request = serde_json::to_string(&(items, skipped))?;
    let key = format!("note-import-{operation}");
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let receipt: Option<String> = transaction
        .query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            [&key],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(receipt) = receipt {
        let receipt: ImportReceipt = serde_json::from_str(&receipt)?;
        if receipt.request != request {
            return Err(StorageError::Invalid(
                "Import identifier reused with different selection or content".into(),
            ));
        }
        return Ok(NoteImportResult {
            created: receipt.created,
            updated: receipt.updated,
            skipped: receipt.skipped,
            records: receipt
                .ids
                .into_iter()
                .map(|id| crate::get(&transaction, id))
                .collect::<Result<_, _>>()?,
        });
    }
    let mut requests = Vec::new();
    let mut identities = std::collections::HashSet::new();
    for (index, item) in items.iter().enumerate() {
        item.source.validate()?;
        if !identities.insert(crate::normalize_headword(&item.content.headword)) {
            return Err(StorageError::Invalid("Duplicate import identity".into()));
        }
        let current = crate::find(&transaction, &item.content.headword)?;
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
        let save = item
            .content
            .save_request(&format!("{operation}-{index}"), item.existing.as_ref())?;
        requests.push(save);
    }
    let mut result = NoteImportResult {
        created: 0,
        updated: 0,
        skipped,
        records: vec![],
    };
    for (item, save) in items.iter().zip(&requests) {
        let saved = crate::vocabulary::save_in_transaction(
            &transaction,
            save,
            item.snapshot.clone(),
            Some(item.source.clone()),
        )?;
        match saved.outcome {
            "created" => result.created += 1,
            "updated" => result.updated += 1,
            _ => {
                return Err(StorageError::Invalid(
                    "Record changed after preview; preview again".into(),
                ));
            }
        }
        result.records.push(saved.record);
    }
    let receipt = ImportReceipt {
        request,
        created: result.created,
        updated: result.updated,
        skipped,
        ids: result.records.iter().map(|record| record.id).collect(),
    };
    transaction.execute(
        "INSERT INTO app_metadata (key, value) VALUES (?1, ?2)",
        params![key, serde_json::to_string(&receipt)?],
    )?;
    transaction.commit()?;
    Ok(result)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoteImportContent {
    pub headword: String,
    pub notes: Option<String>,
    pub examples: Option<Vec<NoteExample>>,
}

impl NoteImportContent {
    pub fn save_request(
        &self,
        operation: &str,
        existing: Option<&VocabularyRecord>,
    ) -> Result<SaveRequest, StorageError> {
        let mut request = self.create_request(operation)?;
        let Some(record) = existing else {
            return Ok(request);
        };
        request.id = Some(record.id);
        request.revision = Some(record.revision);
        request.category_ids = record.category_ids.clone();
        if self.notes.is_none() {
            request.notes = record.notes.clone();
        }
        if let Some(examples) = &self.examples {
            let mut old = std::iter::once(Example {
                fr: record.example_fr.clone(),
                zh: record.example_zh.clone(),
                source: record.example_source.clone(),
                sense: record.example_sense,
            })
            .chain(record.more_examples.clone())
            .map(Some)
            .collect::<Vec<_>>();
            for (index, incoming) in examples.iter().enumerate() {
                let found = old.iter().position(|item| {
                    item.as_ref()
                        .is_some_and(|item| item.fr == incoming.fr && item.zh == incoming.zh)
                });
                let matched = found.and_then(|found| old[found].take());
                if index == 0 {
                    request.example_sense = matched.as_ref().and_then(|item| item.sense);
                    request.example_source = matched.and_then(|item| item.source);
                } else if let Some(matched) = matched {
                    request.more_examples[index - 1] = matched;
                }
            }
        } else {
            request.example_fr = record.example_fr.clone();
            request.example_zh = record.example_zh.clone();
            request.example_source = record.example_source.clone();
            request.example_sense = record.example_sense;
            request.more_examples = record.more_examples.clone();
        }
        crate::validate(&request)?;
        Ok(request)
    }

    pub fn create_request(&self, operation: &str) -> Result<SaveRequest, StorageError> {
        let examples = self.examples.as_deref().unwrap_or_default();
        if examples.len() > 101 {
            return Err(StorageError::Invalid("Examples exceed 101 pairs".into()));
        }
        let request = SaveRequest {
            operation: operation.into(),
            headword: self.headword.clone(),
            notes: self.notes.clone().unwrap_or_default(),
            example_fr: examples.first().map(|e| e.fr.clone()).unwrap_or_default(),
            example_zh: examples.first().map(|e| e.zh.clone()).unwrap_or_default(),
            example_source: None,
            example_sense: None,
            more_examples: examples
                .iter()
                .skip(1)
                .map(|e| Example {
                    sense: None,
                    fr: e.fr.clone(),
                    zh: e.zh.clone(),
                    source: None,
                })
                .collect(),
            id: None,
            revision: None,
            category_ids: vec![],
        };
        crate::validate(&request)?;
        Ok(request)
    }

    pub fn matches(&self, record: &VocabularyRecord) -> bool {
        let examples = std::iter::once(NoteExample {
            fr: record.example_fr.clone(),
            zh: record.example_zh.clone(),
        })
        .chain(record.more_examples.iter().map(|e| NoteExample {
            fr: e.fr.clone(),
            zh: e.zh.clone(),
        }))
        .filter(|e| !e.fr.is_empty() || !e.zh.is_empty())
        .collect::<Vec<_>>();
        self.notes
            .as_ref()
            .is_none_or(|notes| *notes == record.notes)
            && self
                .examples
                .as_ref()
                .is_none_or(|provided| *provided == examples)
    }
}
