use std::path::Path;

use lexique_dictionary::{DictionaryError, LookupWarning};
use lexique_storage::Snapshot;
use serde::Serialize;

use crate::{dictionaries::Registry, vocabulary::DatabasePath};

#[derive(Clone, Debug, Serialize)]
pub struct NoteEnrichment {
    pub status: &'static str,
    pub snapshot: Option<Snapshot>,
    pub warnings: Vec<LookupWarning>,
}

impl NoteEnrichment {
    pub fn retained() -> Self {
        Self {
            status: "retained",
            snapshot: None,
            warnings: vec![],
        }
    }
    pub fn failed(error: DictionaryError) -> Self {
        Self {
            status: "error",
            snapshot: None,
            warnings: vec![LookupWarning {
                dictionary_id: "catalog".into(),
                code: error.code,
                message: error.message,
            }],
        }
    }
}

pub(crate) fn enrich_new_note(
    registry: &Registry,
    path: &DatabasePath,
    builtin: &Path,
    headword: &str,
) -> NoteEnrichment {
    let catalog = match crate::dictionaries::portable_catalog(path) {
        Ok(catalog) => catalog,
        Err(error) => return NoteEnrichment::failed(error),
    };
    let key = lexique_storage::normalize_headword(headword);
    let mut warnings = vec![];
    for config in catalog.entries {
        match registry.lookup_source(path, builtin, &config, &key) {
            Ok(mut lookup) => {
                lookup
                    .entries
                    .retain(|entry| lexique_storage::normalize_headword(&entry.key) == key);
                if !lookup.entries.is_empty() {
                    return NoteEnrichment {
                        status: "linked",
                        snapshot: Some(crate::vocabulary::snapshot(lookup)),
                        warnings,
                    };
                }
            }
            Err(error) => warnings.push(LookupWarning {
                dictionary_id: config.id,
                code: error.code,
                message: error.message,
            }),
        }
    }
    NoteEnrichment {
        status: if warnings.is_empty() {
            "not_found"
        } else {
            "error"
        },
        snapshot: None,
        warnings,
    }
}
