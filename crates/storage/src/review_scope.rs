use crate::{StorageError, categories::validate_category_ids};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReviewScope {
    All {},
    Vocabulary {
        category_ids: Vec<i64>,
        include_uncategorized: bool,
    },
    NeedsSelection {},
}

impl Default for ReviewScope {
    fn default() -> Self {
        Self::All {}
    }
}

impl ReviewScope {
    pub fn validate(&self) -> Result<(), StorageError> {
        if let Self::Vocabulary {
            category_ids,
            include_uncategorized,
        } = self
        {
            validate_category_ids(category_ids)?;
            if category_ids.is_empty() && !include_uncategorized {
                return Err(StorageError::Invalid(
                    "Review scope has no selected condition".into(),
                ));
            }
        }
        Ok(())
    }
}

pub fn review_scope(connection: &Connection) -> Result<ReviewScope, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'review-scope'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let scope: ReviewScope = value
        .map(|value| serde_json::from_str(&value))
        .transpose()?
        .unwrap_or_default();
    scope.validate()?;
    Ok(scope)
}

pub fn set_review_scope(connection: &Connection, scope: &ReviewScope) -> Result<(), StorageError> {
    scope.validate()?;
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    if let ReviewScope::Vocabulary { category_ids, .. } = scope {
        for id in category_ids {
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE id = ?1)",
                [id],
                |row| row.get::<_, bool>(0),
            )? {
                return Err(StorageError::Invalid("Category no longer exists".into()));
            }
        }
    }
    tx.execute("INSERT INTO app_metadata(key,value) VALUES('review-scope',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(scope)?])?;
    tx.execute(
        "INSERT INTO review_queue_changes(kind) VALUES('context')",
        [],
    )?;
    tx.commit()?;
    Ok(())
}
