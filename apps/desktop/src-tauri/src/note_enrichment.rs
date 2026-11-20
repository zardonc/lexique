use std::path::Path;

use lexique_dictionary::{DictionaryError, LookupWarning};
use lexique_storage::Snapshot;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

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
    let decomposed = key.nfd().collect::<String>();
    let queries = if decomposed == key {
        vec![key.as_str()]
    } else {
        vec![key.as_str(), decomposed.as_str()]
    };
    let mut warnings = vec![];
    for config in catalog.entries {
        for query in &queries {
            match registry.lookup_source(path, builtin, &config, query) {
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
                Err(error) => {
                    warnings.push(LookupWarning {
                        dictionary_id: config.id.clone(),
                        code: error.code,
                        message: error.message,
                    });
                    break;
                }
            }
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
