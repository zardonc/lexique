//! A revisioned metadata snapshot. Normal progress/audio deltas read only affected rows.
use crate::{QueueCard, ReviewProgress, StorageError};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn queue_revision(connection: &Connection) -> Result<i64, StorageError> {
    Ok(connection.query_row(
        "SELECT COALESCE(MAX(seq),0) FROM sqlite_sequence WHERE name='review_queue_changes'",
        [],
        |r| r.get(0),
    )?)
}

#[derive(Debug, Serialize)]
pub struct QueueChange {
    pub revision: i64,
    pub kind: String,
    pub card_id: Option<i64>,
    pub vocabulary_id: Option<i64>,
    pub locator: Option<String>,
}

pub fn queue_changes_since(
    connection: &Connection,
    revision: i64,
) -> Result<Vec<QueueChange>, StorageError> {
    if revision < 0 {
        return Err(StorageError::Invalid("Invalid queue revision".into()));
    }
    let mut query=connection.prepare("SELECT revision,kind,card_id,vocabulary_id,locator FROM review_queue_changes WHERE revision>?1 ORDER BY revision LIMIT 1024")?;
    Ok(query
        .query_map([revision], |r| {
            Ok(QueueChange {
                revision: r.get(0)?,
                kind: r.get(1)?,
                card_id: r.get(2)?,
                vocabulary_id: r.get(3)?,
                locator: r.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?)
}

#[derive(Clone, Default, Serialize)]
pub struct QueueCacheMetrics {
    pub loaded_rows: usize,
    pub parsed_progress: usize,
    pub rebuilt: bool,
    pub reason: Option<String>,
}

#[derive(Default)]
pub struct QueueCache {
    pub(crate) cards: BTreeMap<i64, (QueueCard, ReviewProgress)>,
    pub(crate) positions: HashMap<i64, i64>,
    pub(crate) ready: HashSet<i64>,
    pub(crate) revision: i64,
    initialized: bool,
    day: String,
    timezone: String,
    pub(crate) metrics: QueueCacheMetrics,
}

const CARD_COLUMNS: &str = "c.id,c.vocabulary_id,c.kind,p.revision,p.state";
fn card_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(QueueCard, Option<i64>, Option<String>)> {
    Ok((
        QueueCard {
            id: row.get(0)?,
            vocabulary_id: row.get(1)?,
            kind: row.get(2)?,
        },
        row.get(3)?,
        row.get(4)?,
    ))
}

impl QueueCache {
    pub fn revision(&self) -> i64 {
        self.revision
    }

    pub fn invalidate(&mut self) {
        self.initialized = false;
    }

    pub fn progress(&self, card_id: i64) -> Option<&ReviewProgress> {
        self.cards.get(&card_id).map(|(_, progress)| progress)
    }

    pub(crate) fn refresh(
        &mut self,
        connection: &Connection,
        day: &str,
        timezone: &str,
    ) -> Result<QueueCacheMetrics, StorageError> {
        let revision = queue_revision(connection)?;
        let changes = if self.initialized {
            queue_changes_since(connection, self.revision)?
        } else {
            Vec::new()
        };
        let gap = self.initialized
            && (revision < self.revision
                || (revision > self.revision
                    && changes
                        .first()
                        .is_none_or(|c| c.revision != self.revision + 1)));
        let collection = changes.iter().any(|change| change.kind == "collection");
        let context = self.day != day
            || self.timezone != timezone
            || changes.iter().any(|c| c.kind == "context");
        let mut metrics = QueueCacheMetrics::default();
        if !self.initialized || gap || collection || context {
            metrics.rebuilt = true;
            metrics.reason = Some(
                if !self.initialized {
                    "initial"
                } else if gap {
                    "revision_gap"
                } else if collection {
                    "collection"
                } else {
                    "context"
                }
                .into(),
            );
            self.cards.clear();
            let mut query=connection.prepare(&format!("SELECT {CARD_COLUMNS} FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE NOT EXISTS(SELECT 1 FROM review_paused q WHERE q.vocabulary_id=c.vocabulary_id) ORDER BY c.id"))?;
            for row in query.query_map([], card_row)? {
                let (card, revision, state) = row?;
                metrics.loaded_rows += 1;
                metrics.parsed_progress += usize::from(state.is_some());
                let progress = crate::review::decode_review_progress(revision, state)?;
                self.cards.insert(card.id, (card, progress));
            }
            let mut query =
                connection.prepare("SELECT card_id,position FROM review_queue_entries")?;
            self.positions = query
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<Result<_, _>>()?;
            self.ready.clear();
            let mut query=connection.prepare("SELECT s.vocabulary_id FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE v.sealed=1 AND v.gc_state='live' AND v.verification IN ('verified','unverified')")?;
            self.ready = query
                .query_map([], |r| r.get(0))?
                .collect::<Result<_, _>>()?;
        } else {
            let mut cards = HashSet::new();
            let mut positions = HashSet::new();
            let mut words = HashSet::new();
            let mut locators = HashSet::new();
            for change in changes {
                match change.kind.as_str() {
                    "progress" => {
                        if let Some(card) = change.card_id {
                            cards.insert(card);
                        }
                    }
                    "position" => {
                        if let Some(card) = change.card_id {
                            positions.insert(card);
                        }
                    }
                    "audio" => {
                        if let Some(word) = change.vocabulary_id {
                            words.insert(word);
                        }
                        if let Some(locator) = change.locator {
                            locators.insert(locator);
                        }
                    }
                    "settings" | "quota" | "record" => {}
                    _ => return Err(StorageError::Invalid("Unexpected queue delta".into())),
                }
            }
            for chunk in cards.into_iter().collect::<Vec<_>>().chunks(100) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let mut query=connection.prepare(&format!("SELECT {CARD_COLUMNS} FROM review_cards c LEFT JOIN review_progress p ON p.card_id=c.id WHERE c.id IN ({placeholders})"))?;
                for row in query.query_map(rusqlite::params_from_iter(chunk), card_row)? {
                    let (card, revision, state) = row?;
                    metrics.loaded_rows += 1;
                    metrics.parsed_progress += usize::from(state.is_some());
                    let progress = crate::review::decode_review_progress(revision, state)?;
                    self.cards.insert(card.id, (card, progress));
                }
            }
            for chunk in positions.into_iter().collect::<Vec<_>>().chunks(100) {
                for card in chunk {
                    self.positions.remove(card);
                }
                let placeholders = vec!["?"; chunk.len()].join(",");
                let mut query=connection.prepare(&format!("SELECT card_id,position FROM review_queue_entries WHERE card_id IN ({placeholders})"))?;
                for row in query.query_map(rusqlite::params_from_iter(chunk), |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
                })? {
                    let (card, position) = row?;
                    self.positions.insert(card, position);
                }
            }
            for chunk in locators.into_iter().collect::<Vec<_>>().chunks(100) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let mut query=connection.prepare(&format!("SELECT vocabulary_id FROM word_audio_selections WHERE current_version IN ({placeholders})"))?;
                for word in query.query_map(rusqlite::params_from_iter(chunk), |r| r.get(0))? {
                    words.insert(word?);
                }
            }
            for chunk in words.into_iter().collect::<Vec<_>>().chunks(100) {
                for word in chunk {
                    self.ready.remove(word);
                }
                let placeholders = vec!["?"; chunk.len()].join(",");
                let mut query=connection.prepare(&format!("SELECT s.vocabulary_id FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id IN ({placeholders}) AND v.sealed=1 AND v.gc_state='live' AND v.verification IN ('verified','unverified')"))?;
                for word in query.query_map(rusqlite::params_from_iter(chunk), |r| r.get(0))? {
                    self.ready.insert(word?);
                }
            }
        }
        self.initialized = true;
        self.day = day.into();
        self.timezone = timezone.into();
        self.revision = revision;
        Ok(metrics)
    }

    pub(crate) fn audio_metadata(
        &self,
        now: i64,
        skipped: &HashSet<i64>,
    ) -> crate::QueueAudioMetadata {
        let mut metadata = crate::QueueAudioMetadata {
            ready: self.ready.clone(),
            pending_count: 0,
            missing_words: Vec::new(),
            next_due: None,
        };
        for (card, progress) in self
            .cards
            .values()
            .filter(|(card, _)| card.kind == "listening" && !skipped.contains(&card.id))
        {
            if let Some(due) = progress.card.due_at.filter(|due| *due > now) {
                metadata.next_due = Some(metadata.next_due.map_or(due, |old| old.min(due)));
            }
            if progress.card.due_at.is_none_or(|due| due <= now)
                && !self.ready.contains(&card.vocabulary_id)
            {
                metadata.pending_count += 1;
                if metadata.missing_words.len() < 100 {
                    metadata.missing_words.push(card.vocabulary_id);
                }
            }
        }
        metadata
    }
}
