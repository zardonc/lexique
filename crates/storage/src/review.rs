pub use lexique_core::scheduling::Rating;
use lexique_core::scheduling::{Card, Model, Phase, ScheduleRequest, Scheduled, schedule};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::StorageError;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewSettings {
    pub retention_percent: u8,
    pub new_limit: u32,
    pub review_limit: u32,
}

pub fn review_settings(connection: &Connection) -> Result<ReviewSettings, StorageError> {
    Ok(connection.query_row(
        "SELECT retention_percent, new_limit, review_limit FROM review_settings WHERE id = 1",
        [],
        |row| {
            Ok(ReviewSettings {
                retention_percent: row.get(0)?,
                new_limit: row.get(1)?,
                review_limit: row.get(2)?,
            })
        },
    )?)
}

pub fn set_review_settings(
    connection: &Connection,
    settings: &ReviewSettings,
) -> Result<(), StorageError> {
    if !(70..=99).contains(&settings.retention_percent)
        || settings.new_limit > 1_000_000
        || settings.review_limit > 1_000_000
    {
        return Err(StorageError::Invalid("Invalid review settings".into()));
    }
    connection.execute("UPDATE review_settings SET retention_percent = ?1, new_limit = ?2, review_limit = ?3 WHERE id = 1", params![settings.retention_percent, settings.new_limit, settings.review_limit])?;
    Ok(())
}

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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct QueueCard {
    pub id: i64,
    pub vocabulary_id: i64,
    pub kind: String,
}

#[derive(Serialize)]
pub struct ReviewQueue {
    pub cards: Vec<QueueCard>,
    pub reason: Option<String>,
    pub available_at: Option<i64>,
}

pub fn review_card_inventory(connection: &Connection) -> Result<Vec<QueueCard>, StorageError> {
    let mut statement =
        connection.prepare("SELECT id, vocabulary_id, kind FROM review_cards ORDER BY id")?;
    let rows = statement.query_map([], |row| {
        Ok(QueueCard {
            id: row.get(0)?,
            vocabulary_id: row.get(1)?,
            kind: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn review_queue(
    connection: &mut Connection,
    now: i64,
    timezone: &str,
) -> Result<ReviewQueue, StorageError> {
    review_queue_filtered(connection, now, timezone, |_| Ok(true))
}

pub fn review_queue_filtered(
    connection: &mut Connection,
    now: i64,
    timezone: &str,
    mut available: impl FnMut(&QueueCard) -> Result<bool, StorageError>,
) -> Result<ReviewQueue, StorageError> {
    let day =
        lexique_core::scheduling::learning_day_for(now, timezone).map_err(StorageError::Invalid)?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let previous: Option<(String, String)> = tx
        .query_row(
            "SELECT learning_day, timezone FROM review_queue_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if previous
        .as_ref()
        .is_none_or(|(previous_day, _)| previous_day != &day)
    {
        tx.execute("DELETE FROM review_queue_entries", [])?;
        tx.execute("INSERT INTO review_queue_state VALUES (1, ?1, ?2) ON CONFLICT(id) DO UPDATE SET learning_day = excluded.learning_day, timezone = excluded.timezone", params![day, timezone])?;
    }
    let all = review_card_inventory(&tx)?;
    let settings = review_settings(&tx)?;
    let mut used_new = 0_u32;
    let mut used_review = 0_u32;
    let mut siblings: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    let latest: Option<i64> =
        tx.query_row("SELECT MAX(reviewed_at) FROM review_ratings", [], |row| {
            row.get(0)
        })?;
    let mut statement = tx.prepare("SELECT r.reviewed_at, r.quota, c.vocabulary_id, c.id FROM review_ratings r JOIN review_cards c ON c.id = r.card_id WHERE r.reviewed_at >= ?1 AND r.reviewed_at <= ?2")?;
    let ratings = statement.query_map(params![now.saturating_sub(172_800), now], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;
    for rating in ratings {
        let (time, quota, vocabulary_id, card_id) = rating?;
        if lexique_core::scheduling::learning_day_for(time, timezone)
            .map_err(StorageError::Invalid)?
            == day
        {
            if quota == "new" {
                used_new = used_new.saturating_add(1);
            } else {
                used_review = used_review.saturating_add(1);
            }
            siblings.entry(vocabulary_id).or_default().push(card_id);
        }
    }
    drop(statement);
    let remaining_new = settings.new_limit.saturating_sub(used_new) as usize;
    let remaining_review = settings.review_limit.saturating_sub(used_review) as usize;
    let mut pending = Vec::new();
    let mut reason = None;
    let mut available_at: Option<i64> = None;
    for card in all {
        if !available(&card)? {
            continue;
        }
        let progress = review_progress(&tx, card.id)?;
        let state = &progress.card;
        let quota = if state.phase == Phase::New {
            used_new >= settings.new_limit
        } else {
            used_review >= settings.review_limit
        };
        let deferred = matches!(state.phase, Phase::New | Phase::Review)
            && siblings
                .get(&card.vocabulary_id)
                .is_some_and(|ids| ids.iter().any(|id| *id != card.id));
        let (blocked, at) = if latest.is_some_and(|time| now < time) {
            (Some("clock".to_owned()), latest)
        } else if quota || deferred {
            (
                Some(if quota { "quota" } else { "deferred" }.to_owned()),
                Some(
                    lexique_core::scheduling::next_learning_day_at(now, timezone)
                        .map_err(StorageError::Invalid)?,
                ),
            )
        } else if state.due_at.is_some_and(|due| now < due) {
            (Some("due".to_owned()), state.due_at)
        } else {
            (None, None)
        };
        if let Some(blocked) = blocked {
            if reason.is_none()
                || blocked == "clock"
                || (reason.as_deref() != Some("clock") && at < available_at)
            {
                reason = Some(blocked);
                available_at = at;
            }
        } else {
            pending.push((card, progress.card));
        }
    }
    let mut new = Vec::new();
    let mut old = Vec::new();
    for (card, state) in &pending {
        if state.phase == Phase::New {
            new.push(card.id);
        } else if state.phase == Phase::Review {
            let due = state
                .due_at
                .ok_or_else(|| StorageError::Invalid("Review card has no due time".into()))?;
            let due_day = lexique_core::scheduling::learning_day_for(due, timezone)
                .map_err(StorageError::Invalid)?;
            let mut hash = 14695981039346656037_u64;
            for byte in due_day.bytes().chain(card.id.to_le_bytes()) {
                hash = (hash ^ u64::from(byte)).wrapping_mul(1099511628211);
            }
            old.push((due_day, hash, card.id));
        }
    }
    old.sort_unstable();
    let minute_count = pending
        .iter()
        .filter(|(_, state)| {
            matches!(
                state.phase,
                Phase::LearningFirst | Phase::LearningLast | Phase::Relearning
            )
        })
        .count();
    new.truncate(remaining_new);
    old.truncate(remaining_review.saturating_sub(minute_count));
    let old: Vec<i64> = old.into_iter().map(|(_, _, id)| id).collect();
    let mut order = Vec::new();
    let (mut ni, mut oi) = (0, 0);
    while ni < new.len() || oi < old.len() {
        if ni < new.len()
            && (oi == old.len() || (2 * ni + 1) * old.len() <= (2 * oi + 1) * new.len())
        {
            order.push(new[ni]);
            ni += 1;
        } else {
            order.push(old[oi]);
            oi += 1;
        }
    }
    order.extend(
        pending
            .iter()
            .filter(|(_, state)| {
                matches!(
                    state.phase,
                    Phase::LearningFirst | Phase::LearningLast | Phase::Relearning
                )
            })
            .map(|(card, _)| card.id),
    );
    let mut statement = tx.prepare("SELECT card_id, position FROM review_queue_entries")?;
    let existing = statement
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let mut positions: std::collections::HashMap<i64, i64> = existing.iter().copied().collect();
    let mut occupied: std::collections::BTreeMap<i64, i64> = existing
        .into_iter()
        .map(|(id, position)| (position, id))
        .collect();
    let mut next: Option<i64> = None;
    for card_id in order.into_iter().rev() {
        if positions.contains_key(&card_id) {
            next = Some(card_id);
            continue;
        }
        if let Some(next_id) = next {
            let right = positions[&next_id];
            let left = occupied
                .range(..right)
                .next_back()
                .map(|(&value, _)| value)
                .unwrap_or(0);
            if right - left <= 1 {
                let ids: Vec<i64> = occupied.values().copied().collect();
                tx.execute("UPDATE review_queue_entries SET position = -position", [])?;
                occupied.clear();
                for (index, id) in ids.into_iter().enumerate() {
                    let position = (index as i64 + 1) * 1_000_000;
                    tx.execute(
                        "UPDATE review_queue_entries SET position = ?1 WHERE card_id = ?2",
                        params![position, id],
                    )?;
                    positions.insert(id, position);
                    occupied.insert(position, id);
                }
            }
        }
        let position = if let Some(next_id) = next {
            let right = positions[&next_id];
            let left = occupied
                .range(..right)
                .next_back()
                .map(|(&value, _)| value)
                .unwrap_or(0);
            left + (right - left) / 2
        } else {
            occupied
                .last_key_value()
                .map_or(1_000_000, |(&position, _)| {
                    position.saturating_add(1_000_000)
                })
        };
        tx.execute(
            "INSERT INTO review_queue_entries(card_id, position) VALUES (?1, ?2)",
            params![card_id, position],
        )?;
        occupied.insert(position, card_id);
        positions.insert(card_id, position);
        next = Some(card_id);
    }
    let mut ordered = Vec::new();
    for (card, state) in pending {
        let position: Option<i64> = tx
            .query_row(
                "SELECT position FROM review_queue_entries WHERE card_id = ?1",
                [card.id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(position) = position else {
            continue;
        };
        let minute = matches!(
            state.phase,
            Phase::LearningFirst | Phase::LearningLast | Phase::Relearning
        );
        ordered.push((!minute, position, card));
    }
    ordered.sort_by_key(|(not_minute, position, _)| (*not_minute, *position));
    let mut remaining_new = remaining_new;
    let mut remaining_review = remaining_review;
    let mut cards = Vec::new();
    for (_, _, card) in ordered {
        let phase = review_progress(&tx, card.id)?.card.phase;
        let remaining = if phase == Phase::New {
            &mut remaining_new
        } else {
            &mut remaining_review
        };
        if *remaining > 0 {
            *remaining -= 1;
            cards.push(card);
        } else if reason.is_none() {
            reason = Some("quota".into());
        }
    }
    tx.commit()?;
    Ok(ReviewQueue {
        cards,
        reason,
        available_at,
    })
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
    let settings = review_settings(connection)?;
    let limit = if quota == "new" {
        settings.new_limit
    } else {
        settings.review_limit
    };
    let reason = if used >= i64::from(limit) {
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
        retention_percent: review_settings(&tx)?.retention_percent,
        model: Model::default(),
    })
    .map_err(StorageError::Invalid)?;
    let state = serde_json::to_string(&scheduled.card)?;
    let result = serde_json::to_string(&serde_json::to_value(&scheduled)?)?;
    tx.execute("INSERT INTO review_progress (card_id, revision, state) VALUES (?1, ?2, ?3) ON CONFLICT(card_id) DO UPDATE SET revision = excluded.revision, state = excluded.state", params![request.card_id, revision + 1, state])?;
    let position: Option<i64> = tx
        .query_row(
            "SELECT position FROM review_queue_entries WHERE card_id = ?1",
            [request.card_id],
            |row| row.get(0),
        )
        .optional()?;
    let mut statement = tx.prepare("SELECT card_id FROM review_queue_entries ORDER BY position")?;
    let ordered_cards = statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    let queue_before = serde_json::json!({ "card_id": request.card_id, "position": position, "ordered_cards": ordered_cards, "learning_day": day, "timezone": request.timezone }).to_string();
    tx.execute(
        "DELETE FROM review_queue_entries WHERE card_id = ?1",
        [request.card_id],
    )?;
    tx.execute("INSERT INTO review_ratings (operation, card_id, request, before_state, result, reviewed_at, learning_day, quota, queue_before) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![request.operation, request.card_id, encoded, before, result, request.now, day, quota, queue_before])?;
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
