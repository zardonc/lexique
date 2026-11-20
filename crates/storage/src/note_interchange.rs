use serde::{Deserialize, Serialize};

use crate::{Example, SaveRequest, StorageError, VocabularyRecord};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoteExample {
    pub fr: String,
    pub zh: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NoteImportContent {
    pub headword: String,
    pub notes: Option<String>,
    pub examples: Option<Vec<NoteExample>>,
}

impl NoteImportContent {
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
