pub use lexique_core::scheduling::Rating;
use lexique_core::scheduling::{Card, Model, Phase, ScheduleRequest, Scheduled, schedule};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::StorageError;

pub fn auto_enroll(connection: &Connection) -> Result<bool, StorageError> {
    let value: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'review-auto-enroll'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match value.as_deref() {
        None | Some("false") => Ok(false),
        Some("true") => Ok(true),
        _ => Err(StorageError::Invalid(
            "Invalid auto-enrollment preference".into(),
        )),
    }
}

pub fn set_auto_enroll(connection: &Connection, enabled: bool) -> Result<(), StorageError> {
    connection.execute("INSERT INTO app_metadata (key, value) VALUES ('review-auto-enroll', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [enabled.to_string()])?;
    Ok(())
}

pub fn review_paused(connection: &Connection, vocabulary_id: i64) -> Result<bool, StorageError> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM review_paused WHERE vocabulary_id = ?1)",
        [vocabulary_id],
        |row| row.get(0),
    )?)
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewSettings {
    pub retention_percent: u8,
    pub new_limit: u32,
    pub review_limit: u32,
    #[serde(default)]
    pub new_cards_ignore_review_limit: bool,
}

pub fn review_settings(connection: &Connection) -> Result<ReviewSettings, StorageError> {
    let ignore: Option<String> = connection
        .query_row(
            "SELECT value FROM app_metadata WHERE key = 'new-cards-ignore-review-limit'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let new_cards_ignore_review_limit = match ignore.as_deref() {
        None | Some("false") => false,
        Some("true") => true,
        _ => {
            return Err(StorageError::Invalid(
                "Invalid new card limit preference".into(),
            ));
        }
    };
    Ok(connection.query_row(
        "SELECT retention_percent, new_limit, review_limit FROM review_settings WHERE id = 1",
        [],
        |row| {
            Ok(ReviewSettings {
                retention_percent: row.get(0)?,
                new_limit: row.get(1)?,
                review_limit: row.get(2)?,
                new_cards_ignore_review_limit,
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
    let tx = connection.unchecked_transaction()?;
    tx.execute("UPDATE review_settings SET retention_percent = ?1, new_limit = ?2, review_limit = ?3 WHERE id = 1", params![settings.retention_percent, settings.new_limit, settings.review_limit])?;
    tx.execute("INSERT INTO app_metadata (key, value) VALUES ('new-cards-ignore-review-limit', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value", [settings.new_cards_ignore_review_limit.to_string()])?;
    tx.commit()?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum QueueClass {
    New,
    Intraday,
    Interday,
    Review,
}

fn queue_class(card: &Card, day: &str, timezone: &str) -> Result<QueueClass, StorageError> {
    match card.phase {
        Phase::New => Ok(QueueClass::New),
        Phase::Review => Ok(QueueClass::Review),
        _ => {
            let last = card
                .last_review_at
                .ok_or_else(|| StorageError::Invalid("Learning card has no review time".into()))?;
            let last_day = lexique_core::scheduling::learning_day_for(last, timezone)
                .map_err(StorageError::Invalid)?;
            Ok(if last_day == day {
                QueueClass::Intraday
            } else {
                QueueClass::Interday
            })
        }
    }
}

fn daily_usage(connection: &Connection, day: &str, now: i64) -> Result<(u32, u32), StorageError> {
    let mut statement = connection.prepare("SELECT quota, before_state, request FROM review_ratings WHERE undo_operation IS NULL AND learning_day = ?1 AND reviewed_at <= ?2")?;
    let rows = statement.query_map(params![day, now], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let (mut new, mut review) = (0_u32, 0_u32);
    for row in rows {
        let (quota, before, request) = row?;
        if quota == "new" {
            new = new.saturating_add(1);
        } else {
            let card: Card = serde_json::from_str(&before)?;
            let request: RatingRequest = serde_json::from_str(&request)?;
            if matches!(
                queue_class(&card, day, &request.timezone)?,
                QueueClass::Interday | QueueClass::Review
            ) {
                review = review.saturating_add(1);
            }
        }
    }
    Ok((new, review))
}

fn remaining_limits(settings: &ReviewSettings, used_new: u32, used_review: u32) -> (usize, usize) {
    let mut review = settings.review_limit.saturating_sub(used_review);
    if !settings.new_cards_ignore_review_limit {
        review = review.saturating_sub(used_new);
    }
    (
        settings.new_limit.saturating_sub(used_new) as usize,
        review as usize,
    )
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

#[derive(Clone, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueCard {
    pub id: i64,
    pub vocabulary_id: i64,
    pub kind: String,
}

pub struct QueueAudioMetadata {
    pub ready: std::collections::HashSet<i64>,
    pub pending_count: usize,
    pub missing_words: Vec<i64>,
    pub next_due: Option<i64>,
}

pub fn review_learning_day(now: i64, timezone: &str) -> Result<String, StorageError> {
    lexique_core::scheduling::learning_day_for(now, timezone).map_err(StorageError::Invalid)
}

/// Readiness is only a candidate filter. File verification belongs to the platform window.
pub fn queue_audio_metadata(
    connection: &Connection,
    now: i64,
    skipped: &std::collections::HashSet<i64>,
) -> Result<QueueAudioMetadata, StorageError> {
    let mut metadata = QueueAudioMetadata {
        ready: std::collections::HashSet::new(),
        pending_count: 0,
        missing_words: Vec::new(),
        next_due: None,
    };
    let mut query=connection.prepare("SELECT c.id,c.vocabulary_id,json_extract(p.state,'$.due_at'),COALESCE(v.sealed=1 AND v.gc_state='live' AND v.verification IN ('unverified','verified'),0) FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id LEFT JOIN word_audio_selections s USING(vocabulary_id) LEFT JOIN speech_versions v ON v.locator=s.current_version WHERE c.kind='listening' AND NOT EXISTS(SELECT 1 FROM review_paused q WHERE q.vocabulary_id=c.vocabulary_id) ORDER BY c.id")?;
    for row in query.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, Option<i64>>(2)?,
            r.get::<_, bool>(3)?,
        ))
    })? {
        let (card, word, due, ready) = row?;
        if skipped.contains(&card) {
            continue;
        }
        if let Some(due) = due.filter(|due| *due > now) {
            metadata.next_due = Some(metadata.next_due.map_or(due, |old| old.min(due)));
        }
        if ready {
            metadata.ready.insert(word);
        } else if due.is_none_or(|due| due <= now) {
            metadata.pending_count += 1;
            if metadata.missing_words.len() < 100 {
                metadata.missing_words.push(word);
            }
        }
    }
    Ok(metadata)
}

#[derive(Serialize)]
pub struct ReviewQueue {
    pub cards: Vec<QueueCard>,
    pub reason: Option<String>,
    pub available_at: Option<i64>,
}

pub fn review_card_inventory(connection: &Connection) -> Result<Vec<QueueCard>, StorageError> {
    let mut statement =
        connection.prepare("SELECT id, vocabulary_id, kind FROM review_cards WHERE vocabulary_id NOT IN (SELECT vocabulary_id FROM review_paused) ORDER BY id")?;
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
    let mut cache = crate::QueueCache::default();
    for _ in 0..3 {
        match read_queue_from_cache(connection, now, timezone, &mut cache, true, |card, _| {
            available(card)
        }) {
            Err(StorageError::QueueChanged) => continue,
            result => return result,
        }
    }
    Err(StorageError::QueueChanged)
}

pub fn review_queue_cached(
    connection: &mut Connection,
    now: i64,
    timezone: &str,
    skipped: &std::collections::HashSet<i64>,
    cache: &mut crate::QueueCache,
) -> Result<(ReviewQueue, QueueAudioMetadata, crate::QueueCacheMetrics), StorageError> {
    for _ in 0..3 {
        match read_queue_from_cache(connection, now, timezone, cache, true, |card, ready| {
            Ok(!skipped.contains(&card.id) && (card.kind != "listening" || ready))
        }) {
            Err(StorageError::QueueChanged) => continue,
            Ok(queue) => {
                return Ok((
                    queue,
                    cache.audio_metadata(now, skipped),
                    cache.metrics.clone(),
                ));
            }
            Err(error) => return Err(error),
        }
    }
    Err(StorageError::QueueChanged)
}

/// Plans today's actual quota-admitted demand without changing persisted queue positions.
pub fn review_audio_candidates(
    connection: &mut Connection,
    now: i64,
    timezone: &str,
    skipped: &std::collections::HashSet<i64>,
    cache: &mut crate::QueueCache,
) -> Result<ReviewQueue, StorageError> {
    for _ in 0..3 {
        match read_queue_from_cache(connection, now, timezone, cache, false, |card, _| {
            Ok(!skipped.contains(&card.id))
        }) {
            Err(StorageError::QueueChanged) => continue,
            result => return result,
        }
    }
    Err(StorageError::QueueChanged)
}

/// Keeps admission, demand planning and the caller's remaining reads in one snapshot.
/// No filesystem work may run while this transaction is held.
pub struct ReviewQueueSnapshot<'a> {
    pub transaction: rusqlite::Transaction<'a>,
    pub queue: ReviewQueue,
    pub audio: QueueAudioMetadata,
    pub demand_candidates: Vec<QueueCard>,
    pub metrics: crate::QueueCacheMetrics,
}

pub fn review_queue_snapshot<'a>(
    connection: &'a Connection,
    now: i64,
    timezone: &str,
    skipped: &std::collections::HashSet<i64>,
    cache: &mut crate::QueueCache,
) -> Result<ReviewQueueSnapshot<'a>, StorageError> {
    for behavior in [
        TransactionBehavior::Deferred,
        TransactionBehavior::Immediate,
    ] {
        let tx = rusqlite::Transaction::new_unchecked(connection, behavior)?;
        let result = (|| {
            let queue = read_queue_from_cache(&tx, now, timezone, cache, true, |card, ready| {
                Ok(!skipped.contains(&card.id) && (card.kind != "listening" || ready))
            })?;
            let metrics = cache.metrics.clone();
            let audio = cache.audio_metadata(now, skipped);
            let demand_candidates = if audio.pending_count > 0 {
                read_queue_from_cache(&tx, now, timezone, cache, false, |card, _| {
                    Ok(!skipped.contains(&card.id))
                })?
                .cards
            } else {
                Vec::new()
            };
            Ok((queue, audio, demand_candidates, metrics))
        })();
        match result {
            Ok((queue, audio, demand_candidates, metrics)) => {
                return Ok(ReviewQueueSnapshot {
                    transaction: tx,
                    queue,
                    audio,
                    demand_candidates,
                    metrics,
                });
            }
            Err(StorageError::Database(rusqlite::Error::SqliteFailure(error, _)))
                if matches!(behavior, TransactionBehavior::Deferred)
                    && error.code == rusqlite::ErrorCode::DatabaseBusy =>
            {
                // A concurrent commit prevented the position write upgrade. Reserve
                // the writer before rereading rather than chasing changing revisions.
                cache.invalidate();
            }
            Err(error) => {
                cache.invalidate();
                return Err(error);
            }
        }
    }
    unreachable!("the immediate attempt returns its result")
}

fn read_queue_from_cache(
    connection: &Connection,
    now: i64,
    timezone: &str,
    cache: &mut crate::QueueCache,
    persist: bool,
    mut available: impl FnMut(&QueueCard, bool) -> Result<bool, StorageError>,
) -> Result<ReviewQueue, StorageError> {
    let day =
        lexique_core::scheduling::learning_day_for(now, timezone).map_err(StorageError::Invalid)?;
    let owned = connection.is_autocommit();
    let tx = if owned {
        Some(rusqlite::Transaction::new_unchecked(
            connection,
            TransactionBehavior::Deferred,
        )?)
    } else {
        None
    };
    let snapshot = tx.as_deref().unwrap_or(connection);
    let previous: Option<(String, String)> = snapshot
        .query_row(
            "SELECT learning_day, timezone FROM review_queue_state WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let reset = previous
        .as_ref()
        .is_none_or(|(previous_day, _)| previous_day != &day);
    cache.metrics = cache.refresh(snapshot, &day, timezone)?;
    let source_revision = cache.revision;
    let all: Vec<_> = cache.cards.values().cloned().collect();
    let settings = review_settings(snapshot)?;
    let (used_new, used_review) = daily_usage(snapshot, &day, now)?;
    let latest: Option<i64> = snapshot.query_row(
        "SELECT MAX(reviewed_at) FROM review_ratings WHERE undo_operation IS NULL",
        [],
        |row| row.get(0),
    )?;
    let (remaining_new, remaining_review) = remaining_limits(&settings, used_new, used_review);
    let existing: Vec<_> = cache
        .positions
        .iter()
        .map(|(card, position)| (*card, *position))
        .collect();
    let version: i64 = snapshot.pragma_query_value(None, "data_version", |row| row.get(0))?;
    if let Some(tx) = tx {
        tx.commit()?;
    }
    // The callback can perform slow filesystem checks. Never retain a database
    // snapshot (or a day-reset write reservation) across those checks.
    let mut pending = Vec::new();
    let mut reason = None;
    let mut available_at: Option<i64> = None;
    for (card, progress) in all {
        if !available(&card, cache.ready.contains(&card.vocabulary_id))? {
            continue;
        }
        let state = &progress.card;
        let (blocked, at) = if latest.is_some_and(|time| now < time) {
            (Some("clock".to_owned()), latest)
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
    let mut interday = Vec::new();
    let mut intraday = Vec::new();
    for (card, state) in &pending {
        match queue_class(state, &day, timezone)? {
            QueueClass::New => new.push(card.id),
            QueueClass::Intraday => intraday.push(card.id),
            QueueClass::Interday => interday.push(card.id),
            QueueClass::Review => {
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
    }
    old.sort_unstable();
    let total_candidates = new.len() + old.len() + interday.len() + intraday.len();
    interday.truncate(remaining_review);
    old.truncate(remaining_review.saturating_sub(interday.len()));
    let new_limit = if settings.new_cards_ignore_review_limit {
        remaining_new
    } else {
        remaining_new.min(remaining_review.saturating_sub(interday.len() + old.len()))
    };
    new.truncate(new_limit);
    if new.len() + old.len() + interday.len() + intraday.len() < total_candidates {
        let next_day = lexique_core::scheduling::next_learning_day_at(now, timezone)
            .map_err(StorageError::Invalid)?;
        if reason.is_none() || (reason.as_deref() != Some("clock") && Some(next_day) < available_at)
        {
            reason = Some("quota".into());
            available_at = Some(next_day);
        }
    }
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
    order.extend(interday.iter().copied());
    order.extend(intraday.iter().copied());
    let admitted: std::collections::HashSet<i64> = order.iter().copied().collect();
    let existing = if reset { Vec::new() } else { existing };
    let mut positions: std::collections::HashMap<i64, i64> = existing.iter().copied().collect();
    let mut occupied: std::collections::BTreeMap<i64, i64> = existing
        .into_iter()
        .map(|(id, position)| (position, id))
        .collect();
    let needs_write = reset || order.iter().any(|id| !positions.contains_key(id));
    let connection: &Connection = connection;
    let tx = if needs_write && persist && owned {
        Some(rusqlite::Transaction::new_unchecked(
            connection,
            TransactionBehavior::Immediate,
        )?)
    } else {
        None
    };
    let current_version: i64 = connection_version(tx.as_deref().unwrap_or(connection))?;
    if (owned && current_version != version)
        || crate::queue_revision(tx.as_deref().unwrap_or(connection))? != source_revision
    {
        return Err(StorageError::QueueChanged);
    }
    let writer = persist.then(|| tx.as_deref().unwrap_or(connection));
    if reset && persist {
        let tx = writer.expect("day reset requires a writer");
        tx.execute("DELETE FROM review_queue_entries", [])?;
        tx.execute("INSERT INTO review_queue_state VALUES (1, ?1, ?2) ON CONFLICT(id) DO UPDATE SET learning_day = excluded.learning_day, timezone = excluded.timezone", params![day, timezone])?;
    }
    if needs_write
        && occupied
            .last_key_value()
            .is_some_and(|(&position, _)| position.checked_add(1_000_000).is_none())
    {
        renumber_positions(writer, &mut positions, &mut occupied)?;
    }
    if positions.is_empty() {
        for (index, &card_id) in order.iter().enumerate() {
            let position = i64::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(1_000_000))
                .ok_or_else(|| StorageError::Invalid("Review queue position overflow".into()))?;
            if let Some(tx) = writer {
                tx.execute(
                    "INSERT INTO review_queue_entries(card_id, position) VALUES (?1, ?2)",
                    params![card_id, position],
                )?;
            }
            positions.insert(card_id, position);
            occupied.insert(position, card_id);
        }
    }
    let mut next: Option<i64> = None;
    for card_id in order.into_iter().rev() {
        if positions.contains_key(&card_id) {
            next = Some(card_id);
            continue;
        }
        let tx = writer;
        if let Some(next_id) = next {
            let right = positions[&next_id];
            let left = occupied
                .range(..right)
                .next_back()
                .map(|(&value, _)| value)
                .unwrap_or(0);
            if right - left <= 1 {
                renumber_positions(tx, &mut positions, &mut occupied)?;
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
                .map_or(Some(1_000_000), |(&position, _)| {
                    position.checked_add(1_000_000)
                })
                .ok_or_else(|| StorageError::Invalid("Review queue position overflow".into()))?
        };
        if let Some(tx) = tx {
            tx.execute(
                "INSERT INTO review_queue_entries(card_id, position) VALUES (?1, ?2)",
                params![card_id, position],
            )?;
        }
        occupied.insert(position, card_id);
        positions.insert(card_id, position);
        next = Some(card_id);
    }
    let mut ordered = Vec::new();
    for (card, state) in pending {
        if !admitted.contains(&card.id) {
            continue;
        }
        let Some(&position) = positions.get(&card.id) else {
            continue;
        };
        let minute = matches!(
            state.phase,
            Phase::LearningFirst | Phase::LearningLast | Phase::Relearning
        );
        ordered.push((!minute, position, card));
    }
    ordered.sort_by_key(|(not_minute, position, _)| (*not_minute, *position));
    let cards = ordered.into_iter().map(|(_, _, card)| card).collect();
    let committed_revision = crate::queue_revision(tx.as_deref().unwrap_or(connection))?;
    if let Some(tx) = tx {
        tx.commit()?;
    }
    if persist {
        cache.positions = positions;
        cache.revision = committed_revision;
    }
    Ok(ReviewQueue {
        cards,
        reason,
        available_at,
    })
}

fn connection_version(connection: &Connection) -> Result<i64, StorageError> {
    Ok(connection.pragma_query_value(None, "data_version", |row| row.get(0))?)
}

fn renumber_positions(
    tx: Option<&Connection>,
    positions: &mut std::collections::HashMap<i64, i64>,
    occupied: &mut std::collections::BTreeMap<i64, i64>,
) -> Result<(), StorageError> {
    let ids: Vec<i64> = occupied.values().copied().collect();
    // Move every old positive position out of the destination range first;
    // SQLite enforces the UNIQUE constraint after each individual update.
    if let Some(tx) = tx {
        tx.execute("UPDATE review_queue_entries SET position = -position", [])?;
    }
    occupied.clear();
    for (index, id) in ids.into_iter().enumerate() {
        let position = i64::try_from(index)
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(1_000_000))
            .ok_or_else(|| StorageError::Invalid("Review queue position overflow".into()))?;
        if let Some(tx) = tx {
            tx.execute(
                "UPDATE review_queue_entries SET position = ?1 WHERE card_id = ?2",
                params![position, id],
            )?;
        }
        positions.insert(id, position);
        occupied.insert(position, id);
    }
    Ok(())
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
    let paused: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM review_cards c JOIN review_paused p ON p.vocabulary_id = c.vocabulary_id WHERE c.id = ?1)",
        [card_id], |row| row.get(0),
    )?;
    if paused {
        return Ok((Some("paused".into()), None));
    }
    let day =
        lexique_core::scheduling::learning_day_for(now, timezone).map_err(StorageError::Invalid)?;
    let latest: Option<i64> = connection.query_row(
        "SELECT MAX(reviewed_at) FROM review_ratings WHERE undo_operation IS NULL",
        [],
        |row| row.get(0),
    )?;
    if latest.is_some_and(|time| now < time) {
        return Ok((Some("clock".into()), latest));
    }
    if card.due_at.is_some_and(|due| now < due) {
        return Ok((Some("due".into()), card.due_at));
    }
    let settings = review_settings(connection)?;
    let (used_new, used_review) = daily_usage(connection, &day, now)?;
    let (new, review) = remaining_limits(&settings, used_new, used_review);
    let blocked = match queue_class(card, &day, timezone)? {
        QueueClass::New => new == 0 || (!settings.new_cards_ignore_review_limit && review == 0),
        QueueClass::Interday | QueueClass::Review => review == 0,
        QueueClass::Intraday => false,
    };
    let reason = if blocked { Some("quota") } else { None };
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
    let row: Option<(i64, Option<i64>, Option<String>)> = connection.prepare_cached(
        "SELECT c.id, p.revision, p.state FROM review_cards c LEFT JOIN review_progress p ON p.card_id = c.id WHERE c.id = ?1")?.query_row(
        [card_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).optional()?;
    match row {
        Some((_, revision, state)) => decode_review_progress(revision, state),
        None => Err(StorageError::Invalid("Review card not found".into())),
    }
}

pub fn review_progress_batch(
    connection: &Connection,
    card_ids: &[i64],
) -> Result<Vec<ReviewProgress>, StorageError> {
    let mut progress = Vec::with_capacity(card_ids.len());
    for chunk in card_ids.chunks(100) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let mut query = connection.prepare(&format!(
            "SELECT c.id,p.revision,p.state FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE c.id IN ({placeholders})"
        ))?;
        let rows = query
            .query_map(rusqlite::params_from_iter(chunk.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    (
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ),
                ))
            })?
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
        for id in chunk {
            let (revision, state) = rows
                .get(id)
                .ok_or_else(|| StorageError::Invalid("Review card not found".into()))?;
            progress.push(decode_review_progress(*revision, state.clone())?);
        }
    }
    Ok(progress)
}

pub(crate) fn decode_review_progress(
    revision: Option<i64>,
    state: Option<String>,
) -> Result<ReviewProgress, StorageError> {
    match (revision, state) {
        (Some(revision), Some(state)) => Ok(ReviewProgress {
            revision,
            card: serde_json::from_str(&state)?,
        }),
        (None, None) => Ok(ReviewProgress {
            revision: 0,
            card: Card {
                phase: Phase::New,
                memory: None,
                last_review_at: None,
                due_at: None,
            },
        }),
        _ => Err(StorageError::Invalid("Invalid review progress".into())),
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
    let previous: Option<(String, String, Option<String>)> = tx
        .query_row(
            "SELECT request, result, undo_operation FROM review_ratings WHERE operation = ?1",
            [&request.operation],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    if let Some((original, result, undone)) = previous {
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
        if undone.is_some() {
            return Err(StorageError::RatingConflict);
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
    let latest: Option<i64> = tx.query_row(
        "SELECT MAX(reviewed_at) FROM review_ratings WHERE undo_operation IS NULL",
        [],
        |row| row.get(0),
    )?;
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueueSnapshot {
    card_id: i64,
    position: Option<i64>,
    ordered_cards: Option<Vec<i64>>,
    learning_day: String,
    timezone: String,
}

pub fn undo_target(connection: &Connection) -> Result<Option<String>, StorageError> {
    let row: Option<(String, Option<String>)> = connection
        .query_row(
            "SELECT operation, undo_operation FROM review_ratings ORDER BY rowid DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    Ok(row.and_then(|(operation, undone)| undone.is_none().then_some(operation)))
}

pub fn undo_rating(
    connection: &mut Connection,
    operation: &str,
    rating_operation: &str,
) -> Result<bool, StorageError> {
    if operation.is_empty()
        || operation.len() > 128
        || rating_operation.is_empty()
        || rating_operation.len() > 128
    {
        return Err(StorageError::Invalid("Invalid undo request".into()));
    }
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let retry: Option<(String, bool)> = tx
        .query_row(
            "SELECT operation, undo_queue_rebuilt FROM review_ratings WHERE undo_operation = ?1",
            [operation],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((original, rebuilt)) = retry {
        return if original == rating_operation {
            Ok(rebuilt)
        } else {
            Err(StorageError::Invalid("Undo operation already used".into()))
        };
    }
    type UndoRatingRow = (String, i64, String, String, Option<String>, Option<String>);
    let latest: Option<UndoRatingRow> = tx.query_row(
        "SELECT operation, card_id, request, before_state, queue_before, undo_operation FROM review_ratings ORDER BY rowid DESC LIMIT 1", [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
    ).optional()?;
    let (rating, card_id, request, before, queue, undone) =
        latest.ok_or_else(|| StorageError::Invalid("No rating to undo".into()))?;
    if rating != rating_operation || undone.is_some() {
        return Err(StorageError::Invalid(
            "Rating is no longer available for undo".into(),
        ));
    }
    let request: RatingRequest = serde_json::from_str(&request)?;
    let progress = review_progress(&tx, card_id)?;
    if progress.revision != request.expected_revision + 1 {
        return Err(StorageError::RatingConflict);
    }
    let previous: Card = serde_json::from_str(&before)?;
    lexique_core::scheduling::validate_card(&previous).map_err(StorageError::Invalid)?;
    tx.execute(
        "UPDATE review_progress SET state = ?1, revision = revision + 1 WHERE card_id = ?2",
        params![before, card_id],
    )?;
    if let Some(queue) = queue {
        let snapshot: QueueSnapshot = serde_json::from_str(&queue)?;
        if snapshot.card_id != card_id
            || snapshot.timezone.is_empty()
            || snapshot.learning_day.is_empty()
        {
            return Err(StorageError::Invalid("Invalid queue snapshot".into()));
        }
        let state: Option<(String, String)> = tx
            .query_row(
                "SELECT learning_day, timezone FROM review_queue_state WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if snapshot.position.is_some() && snapshot.ordered_cards.is_none() {
            if state
                .as_ref()
                .is_some_and(|(day, _)| snapshot.learning_day == *day)
            {
                tx.execute("DELETE FROM review_queue_entries", [])?;
            }
            tx.execute("UPDATE review_ratings SET undo_operation = ?1, undo_queue_rebuilt = 1 WHERE operation = ?2", params![operation, rating_operation])?;
            tx.commit()?;
            return Ok(true);
        }
        if state
            .as_ref()
            .is_some_and(|(day, _)| snapshot.learning_day == *day)
            && snapshot.position.is_some()
        {
            let Some(ids) = snapshot.ordered_cards else {
                return Err(StorageError::Invalid(
                    "Legacy queue snapshot cannot be restored safely".into(),
                ));
            };
            if ids.iter().any(|id| *id <= 0) {
                return Err(StorageError::Invalid("Invalid queue snapshot".into()));
            }
            let mut current =
                tx.prepare("SELECT card_id FROM review_queue_entries ORDER BY position")?;
            let mut order = current
                .query_map([], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            drop(current);
            order.retain(|id| *id != card_id);
            let index = ids
                .iter()
                .position(|id| *id == card_id)
                .ok_or_else(|| StorageError::Invalid("Card missing from queue snapshot".into()))?;
            let insert = ids[index + 1..]
                .iter()
                .find_map(|id| order.iter().position(|current| current == id))
                .unwrap_or(order.len());
            order.insert(insert, card_id);
            tx.execute("DELETE FROM review_queue_entries", [])?;
            for (index, id) in order.into_iter().enumerate() {
                tx.execute(
                    "INSERT INTO review_queue_entries(card_id, position) VALUES (?1, ?2)",
                    params![id, (index as i64 + 1) * 1_000_000],
                )?;
            }
        }
    }
    tx.execute(
        "UPDATE review_ratings SET undo_operation = ?1 WHERE operation = ?2",
        params![operation, rating_operation],
    )?;
    tx.commit()?;
    Ok(false)
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
    ensure_review_cards(&transaction, vocabulary_id)?;
    let cards = review_cards(&transaction, vocabulary_id)?;
    transaction.commit()?;
    Ok(cards)
}

pub(crate) fn ensure_review_cards(
    transaction: &rusqlite::Transaction<'_>,
    vocabulary_id: i64,
) -> Result<(), StorageError> {
    for kind in ["recognition", "listening"] {
        transaction.execute(
            "INSERT INTO review_cards (vocabulary_id, kind) VALUES (?1, ?2) ON CONFLICT (vocabulary_id, kind) DO NOTHING",
            params![vocabulary_id, kind],
        )?;
    }
    Ok(())
}

pub fn set_review_paused(
    connection: &mut Connection,
    vocabulary_id: i64,
    paused: bool,
) -> Result<(), StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM review_cards WHERE vocabulary_id = ?1)",
        [vocabulary_id],
        |row| row.get(0),
    )?;
    if !exists {
        return Err(StorageError::Invalid("Review cards not found".into()));
    }
    if paused {
        tx.execute(
            "INSERT INTO review_paused (vocabulary_id) VALUES (?1) ON CONFLICT DO NOTHING",
            [vocabulary_id],
        )?;
    } else {
        tx.execute(
            "DELETE FROM review_paused WHERE vocabulary_id = ?1",
            [vocabulary_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}
