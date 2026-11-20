use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::StorageError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LearningCategory {
    pub id: i64,
    pub name: String,
}

pub(crate) fn category_name_key(name: &str) -> String {
    name.trim()
        .nfc()
        .flat_map(char::to_lowercase)
        .nfc()
        .collect()
}

fn validate_name(name: &str) -> Result<(), StorageError> {
    crate::grammar::validate_title(name)
}

pub fn learning_categories(connection: &Connection) -> Result<Vec<LearningCategory>, StorageError> {
    Ok(connection
        .prepare("SELECT id, name FROM learning_categories ORDER BY name_key, id")?
        .query_map([], |row| {
            Ok(LearningCategory {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn create_category(
    connection: &Connection,
    name: &str,
) -> Result<LearningCategory, StorageError> {
    validate_name(name)?;
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let key = category_name_key(name);
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE name_key = ?1)",
        [&key],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(StorageError::Invalid("Category name already exists".into()));
    }
    tx.execute(
        "INSERT INTO learning_categories(name, name_key) VALUES (?1, ?2)",
        params![name.trim(), key],
    )?;
    let category = LearningCategory {
        id: tx.last_insert_rowid(),
        name: name.trim().into(),
    };
    tx.commit()?;
    Ok(category)
}

pub fn rename_category(
    connection: &Connection,
    id: i64,
    name: &str,
) -> Result<LearningCategory, StorageError> {
    validate_category_ids(&[id])?;
    validate_name(name)?;
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let key = category_name_key(name);
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE name_key = ?1 AND id <> ?2)",
        params![key, id],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(StorageError::Invalid("Category name already exists".into()));
    }
    if tx.execute(
        "UPDATE learning_categories SET name = ?1, name_key = ?2 WHERE id = ?3",
        params![name.trim(), key, id],
    )? != 1
    {
        return Err(StorageError::Invalid("Category no longer exists".into()));
    }
    tx.commit()?;
    Ok(LearningCategory {
        id,
        name: name.trim().into(),
    })
}

pub fn delete_category(connection: &Connection, id: i64) -> Result<(), StorageError> {
    validate_category_ids(&[id])?;
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    if !tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE id = ?1)",
        [id],
        |row| row.get::<_, bool>(0),
    )? {
        return Ok(());
    }
    tx.execute("UPDATE vocabulary SET revision = revision + 1, updated_at = MAX(updated_at + 1, CAST(unixepoch('subsec') * 1000 AS INTEGER)) WHERE id IN (SELECT record_id FROM record_categories WHERE category_id = ?1)", [id])?;
    tx.execute("DELETE FROM learning_categories WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

pub(crate) fn validate_category_ids(ids: &[i64]) -> Result<(), StorageError> {
    let mut unique = std::collections::HashSet::new();
    if ids
        .iter()
        .any(|id| *id <= 0 || *id > 9_007_199_254_740_991 || !unique.insert(id))
    {
        return Err(StorageError::Invalid(
            "Invalid or duplicate category identity".into(),
        ));
    }
    Ok(())
}

pub(crate) fn record_category_ids(
    connection: &Connection,
    id: i64,
) -> Result<Vec<i64>, StorageError> {
    Ok(connection
        .prepare(
            "SELECT category_id FROM record_categories WHERE record_id = ?1 ORDER BY category_id",
        )?
        .query_map([id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?)
}

pub(crate) fn save_record_categories(
    connection: &Connection,
    id: i64,
    ids: &[i64],
) -> Result<(), StorageError> {
    validate_category_ids(ids)?;
    for category in ids {
        if !connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE id = ?1)",
            [category],
            |row| row.get::<_, bool>(0),
        )? {
            return Err(StorageError::Invalid("Category no longer exists".into()));
        }
    }
    connection.execute("DELETE FROM record_categories WHERE record_id = ?1", [id])?;
    for category in ids {
        connection.execute(
            "INSERT INTO record_categories(record_id, category_id) VALUES (?1, ?2)",
            params![id, category],
        )?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryBulkRequest {
    pub record_kind: String,
    pub ids: Vec<i64>,
    pub category_id: i64,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryBulkResult {
    pub id: i64,
    pub status: String,
    pub error: Option<String>,
}

pub fn bulk_classify(
    connection: &Connection,
    request: &CategoryBulkRequest,
) -> Result<Vec<CategoryBulkResult>, StorageError> {
    let unique = request.ids.iter().collect::<std::collections::HashSet<_>>();
    if !matches!(request.record_kind.as_str(), "vocabulary" | "grammar")
        || !matches!(request.action.as_str(), "add" | "remove")
        || request.ids.len() > 10_000
        || unique.len() != request.ids.len()
        || request
            .ids
            .iter()
            .any(|id| *id <= 0 || *id > 9_007_199_254_740_991)
    {
        return Err(StorageError::Invalid(
            "Invalid batch classification request".into(),
        ));
    }
    validate_category_ids(&[request.category_id])?;
    Ok(request
        .ids
        .iter()
        .map(|id| {
            let result = classify_record(connection, *id, request);
            CategoryBulkResult {
                id: *id,
                status: if result.is_ok() { "saved" } else { "failed" }.into(),
                error: result.err().map(|error| error.to_string()),
            }
        })
        .collect())
}

fn classify_record(
    connection: &Connection,
    id: i64,
    request: &CategoryBulkRequest,
) -> Result<(), StorageError> {
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    if !tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM vocabulary WHERE id = ?1 AND record_kind = ?2)",
        params![id, request.record_kind],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(StorageError::Invalid(
            "Record no longer exists in this tab".into(),
        ));
    }
    if !tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM learning_categories WHERE id = ?1)",
        [request.category_id],
        |row| row.get::<_, bool>(0),
    )? {
        return Err(StorageError::Invalid("Category no longer exists".into()));
    }
    let changed = if request.action == "add" {
        tx.execute("INSERT INTO record_categories(record_id, category_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING", params![id, request.category_id])?
    } else {
        tx.execute(
            "DELETE FROM record_categories WHERE record_id = ?1 AND category_id = ?2",
            params![id, request.category_id],
        )?
    };
    if changed > 0 {
        tx.execute("UPDATE vocabulary SET revision = revision + 1, updated_at = MAX(updated_at + 1, CAST(unixepoch('subsec') * 1000 AS INTEGER)) WHERE id = ?1", [id])?;
    }
    tx.commit()?;
    Ok(())
}
