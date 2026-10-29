use rusqlite::{Connection, TransactionBehavior, params};
use serde::Serialize;

use crate::StorageError;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ReviewCard {
    pub id: i64,
    pub kind: String,
}

pub fn review_cards(
    connection: &Connection,
    vocabulary_id: i64,
) -> Result<Vec<ReviewCard>, StorageError> {
    let mut statement = connection
        .prepare("SELECT id, kind FROM review_cards WHERE vocabulary_id = ?1 ORDER BY id")?;
    let rows = statement.query_map([vocabulary_id], |row| {
        Ok(ReviewCard {
            id: row.get(0)?,
            kind: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn add_review(
    connection: &mut Connection,
    vocabulary_id: i64,
) -> Result<Vec<ReviewCard>, StorageError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for kind in ["recognition", "listening"] {
        transaction.execute(
            "INSERT INTO review_cards (vocabulary_id, kind) VALUES (?1, ?2) ON CONFLICT (vocabulary_id, kind) DO NOTHING",
            params![vocabulary_id, kind],
        )?;
    }
    let cards = review_cards(&transaction, vocabulary_id)?;
    transaction.commit()?;
    Ok(cards)
}
