pub use lexique_core::scheduling::Rating;
use lexique_core::scheduling::{Card, Model, Phase, ScheduleRequest, Scheduled, schedule};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RatingRequest {
    pub operation: String,
    pub card_id: i64,
    pub expected_revision: i64,
    pub rating: Rating,
    pub now: i64,
    pub timezone: String,
}

#[derive(Serialize)]
pub struct ReviewProgress {
    pub revision: i64,
    pub card: Card,
}

#[derive(Serialize)]
pub struct ReviewAvailability {
    pub progress: ReviewProgress,
    pub reason: Option<String>,
    pub available_at: Option<i64>,
}

pub fn review_availability(
    connection: &Connection,
    card_id: i64,
    now: i64,
    timezone: &str,
) -> Result<ReviewAvailability, StorageError> {
    let progress = review_progress(connection, card_id)?;
    let (reason, available_at) = eligibility(connection, card_id, &progress.card, now, timezone)?;
    Ok(ReviewAvailability {
        progress,
        reason,
        available_at,
    })
}

fn eligibility(
    connection: &Connection,
    card_id: i64,
    card: &Card,
    now: i64,
    timezone: &str,
) -> Result<(Option<String>, Option<i64>), StorageError> {
    let day =
        lexique_core::scheduling::learning_day_for(now, timezone).map_err(StorageError::Invalid)?;
    let quota = if card.phase == Phase::New {
        "new"
    } else {
        "review"
    };
    let latest: Option<i64> =
        connection.query_row("SELECT MAX(reviewed_at) FROM review_ratings", [], |row| {
            row.get(0)
        })?;
    if latest.is_some_and(|time| now < time) {
        return Ok((Some("clock".into()), latest));
    }
    if card.due_at.is_some_and(|due| now < due) {
        return Ok((Some("due".into()), card.due_at));
    }
    let mut used = 0;
    let mut deferred = false;
    let mut statement = connection.prepare("SELECT r.reviewed_at, r.quota, r.card_id != ?1 AND other.vocabulary_id = current.vocabulary_id FROM review_ratings r JOIN review_cards other ON other.id = r.card_id JOIN review_cards current ON current.id = ?1 WHERE r.reviewed_at >= ?2 AND r.reviewed_at <= ?3")?;
    let rows = statement.query_map(params![card_id, now.saturating_sub(172_800), now], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, bool>(2)?,
        ))
    })?;
    for row in rows {
        let (time, previous_quota, sibling) = row?;
        if lexique_core::scheduling::learning_day_for(time, timezone)
            .map_err(StorageError::Invalid)?
            == day
        {
            used += i64::from(previous_quota == quota);
            deferred |= sibling;
        }
    }
    let reason = if used >= if quota == "new" { 20 } else { 50 } {
        Some("quota")
    } else if deferred && matches!(card.phase, Phase::New | Phase::Review) {
        Some("deferred")
    } else {
        None
    };
    let available_at = reason
        .map(|_| {
            lexique_core::scheduling::next_learning_day_at(now, timezone)
                .map_err(StorageError::Invalid)
        })
        .transpose()?;
    Ok((reason.map(str::to_owned), available_at))
}

pub fn review_progress(
    connection: &Connection,
    card_id: i64,
) -> Result<ReviewProgress, StorageError> {
    let row: Option<(i64, Option<i64>, Option<String>)> = connection.query_row(
        "SELECT c.id, p.revision, p.state FROM review_cards c LEFT JOIN review_progress p ON p.card_id = c.id WHERE c.id = ?1",
        [card_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).optional()?;
    match row {
        Some((_, Some(revision), Some(state))) => Ok(ReviewProgress {
            revision,
            card: serde_json::from_str(&state)?,
        }),
        Some((_, None, None)) => Ok(ReviewProgress {
            revision: 0,
            card: Card {
                phase: Phase::New,
                memory: None,
                last_review_at: None,
                due_at: None,
            },
        }),
        Some(_) => Err(StorageError::Invalid("Invalid review progress".into())),
        None => Err(StorageError::Invalid("Review card not found".into())),
    }
}

pub fn rate_card(
    connection: &mut Connection,
    request: &RatingRequest,
) -> Result<serde_json::Value, StorageError> {
    if request.operation.is_empty()
        || request.operation.len() > 128
        || request.card_id <= 0
        || request.expected_revision < 0
    {
        return Err(StorageError::Invalid("Invalid rating request".into()));
    }
    let encoded = serde_json::to_string(request)?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let previous: Option<(String, String)> = tx
        .query_row(
            "SELECT request, result FROM review_ratings WHERE operation = ?1",
            [&request.operation],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((original, result)) = previous {
        let original: RatingRequest = serde_json::from_str(&original)?;
        if original.card_id != request.card_id
            || original.expected_revision != request.expected_revision
            || original.rating != request.rating
            || original.timezone != request.timezone
        {
            return Err(StorageError::Invalid(
                "Rating operation already used for another request".into(),
            ));
        }
        return Ok(serde_json::from_str(&result)?);
    }
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM review_cards WHERE id = ?1)",
        [request.card_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(StorageError::Invalid("Review card not found".into()));
    }
    let progress: Option<(i64, String)> = tx
        .query_row(
            "SELECT revision, state FROM review_progress WHERE card_id = ?1",
            [request.card_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let (revision, card) = match progress {
        Some((revision, state)) => (revision, serde_json::from_str::<Card>(&state)?),
        None => (
            0,
            Card {
                phase: Phase::New,
                memory: None,
                last_review_at: None,
                due_at: None,
            },
        ),
    };
    if revision != request.expected_revision {
        return Err(StorageError::RatingConflict);
    }
    let day = lexique_core::scheduling::learning_day_for(request.now, &request.timezone)
        .map_err(StorageError::Invalid)?;
    let quota = if card.phase == Phase::New {
        "new"
    } else {
        "review"
    };
    let latest: Option<i64> =
        tx.query_row("SELECT MAX(reviewed_at) FROM review_ratings", [], |row| {
            row.get(0)
        })?;
    if let (Some(reason), _) =
        eligibility(&tx, request.card_id, &card, request.now, &request.timezone)?
    {
        return Err(StorageError::RatingUnavailable(reason));
    }
    let before = serde_json::to_string(&card)?;
    let scheduled: Scheduled = schedule(ScheduleRequest {
        card,
        rating: request.rating,
        now: request.now,
        timezone: request.timezone.clone(),
        timezone_database: lexique_core::scheduling::timezone_database(),
        latest_review_at: latest,
        retention_percent: 90,
        model: Model::default(),
    })
    .map_err(StorageError::Invalid)?;
    let state = serde_json::to_string(&scheduled.card)?;
    let result = serde_json::to_string(&serde_json::to_value(&scheduled)?)?;
    tx.execute("INSERT INTO review_progress (card_id, revision, state) VALUES (?1, ?2, ?3) ON CONFLICT(card_id) DO UPDATE SET revision = excluded.revision, state = excluded.state", params![request.card_id, revision + 1, state])?;
    tx.execute("INSERT INTO review_ratings (operation, card_id, request, before_state, result, reviewed_at, learning_day, quota) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![request.operation, request.card_id, encoded, before, result, request.now, day, quota])?;
    tx.commit()?;
    Ok(serde_json::from_str(&result)?)
}

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
