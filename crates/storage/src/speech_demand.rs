//! Bounded review demand and idempotent local repair. Sentence synthesis still needs M3 admission.
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashSet;

use crate::{SpeechSpec, StorageError};

pub struct AudioCandidate {
    pub card_id: i64,
    pub word: SpeechSpec,
    pub sentence: Option<SpeechSpec>,
    pub due_at: Option<i64>,
    pub paused: bool,
    pub skipped: bool,
    pub quota_available: bool,
    pub deferred: bool,
}

pub struct PlannedAudio<'a> {
    pub semantic_key: String,
    pub spec: &'a SpeechSpec,
    pub priority: u8,
}

/// Preserve supplied review order, select current plus at most three candidates,
/// and deduplicate shared word audio. No schedule, quota or played state is written.
pub fn plan_speech_window(
    candidates: &[AudioCandidate],
    current_id: Option<i64>,
    now: i64,
) -> Result<Vec<PlannedAudio<'_>>, StorageError> {
    let horizon = now
        .checked_add(604800)
        .ok_or_else(|| StorageError::Invalid("Invalid prefetch clock".into()))?;
    let mut selected: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            !candidate.paused
                && !candidate.skipped
                && candidate.quota_available
                && !candidate.deferred
                && candidate.due_at.is_none_or(|due| due <= horizon)
        })
        .collect();
    if let Some(index) = selected
        .iter()
        .position(|candidate| Some(candidate.card_id) == current_id)
    {
        selected.drain(..index);
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for candidate in selected.into_iter().take(4) {
        for spec in std::iter::once(&candidate.word).chain(candidate.sentence.iter()) {
            let semantic_key = spec.key()?;
            if seen.insert(semantic_key.clone()) {
                result.push(PlannedAudio {
                    semantic_key,
                    spec,
                    priority: if Some(candidate.card_id) == current_id {
                        0
                    } else {
                        1
                    },
                });
            }
        }
    }
    Ok(result)
}

pub fn usable_through(generated_at: i64, required_until: i64) -> bool {
    generated_at
        .checked_add(604800)
        .is_some_and(|expiry| required_until < expiry)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordDemand {
    pub vocabulary_id: i64,
    pub class: u8,
}

pub fn plan_review_words(
    recent: &[crate::QueueCard],
    today: &[crate::QueueCard],
    ready: &HashSet<i64>,
) -> Vec<WordDemand> {
    let mut seen = HashSet::new();
    recent
        .iter()
        .take(4)
        .map(|card| (card, 1))
        .chain(
            today
                .iter()
                .filter(|card| card.kind == "listening")
                .map(|card| (card, 2)),
        )
        .filter(|(card, _)| !ready.contains(&card.vocabulary_id) && seen.insert(card.vocabulary_id))
        .take(100)
        .map(|(card, class)| WordDemand {
            vocabulary_id: card.vocabulary_id,
            class,
        })
        .collect()
}

/// A known specification is repaired exactly; only unknown legacy provenance adopts preferences.
pub fn word_repair_spec(connection: &Connection, id: i64) -> Result<SpeechSpec, StorageError> {
    let record = crate::get(connection, id)?;
    if let Some(version) = crate::selected_audio(connection, id)?
        && let Some(key) = version.semantic_key
    {
        let saved: String = connection.query_row(
            "SELECT spec FROM speech_assets WHERE semantic_key=?1",
            [key],
            |row| row.get(0),
        )?;
        let previous: SpeechSpec = serde_json::from_str(&saved)?;
        let voice =
            match previous.voice.as_str() {
                "female" => crate::SpeechVoice::Female,
                "male" => crate::SpeechVoice::Male,
                _ => return Err(StorageError::Invalid(
                    "Selected audio specification is unsupported; an explicit upgrade is required"
                        .into(),
                )),
            };
        if previous.key()? != SpeechSpec::word(&record.headword, voice).key()? {
            return Err(StorageError::Invalid(
                "Selected audio specification is unsupported; an explicit upgrade is required"
                    .into(),
            ));
        }
        return Ok(previous);
    }
    Ok(SpeechSpec::word(
        &record.headword,
        crate::speech_preferences(connection)?.voice,
    ))
}

/// One record per failed version/reason. Repeated refresh never resets an execution budget.
pub fn demand_word_repair(
    connection: &mut Connection,
    demand: &WordDemand,
    now: i64,
) -> Result<bool, StorageError> {
    if !(1..=2).contains(&demand.class) || demand.vocabulary_id <= 0 || now < 0 {
        return Err(StorageError::Invalid(
            "Invalid automatic audio demand".into(),
        ));
    }
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let row = tx.query_row(
        "SELECT COALESCE(s.current_version,'absent')||':'||COALESCE(v.verification,'missing'),COALESCE(v.verification IN ('verified','unverified') AND v.gc_state='live',0),s.desired_key,COALESCE(s.desired_generation,0),COALESCE(s.current_generation,0),i.error FROM vocabulary w LEFT JOIN word_audio_selections s ON s.vocabulary_id=w.id LEFT JOIN speech_versions v ON v.locator=s.current_version LEFT JOIN word_audio_imports i ON i.vocabulary_id=w.id WHERE w.id=?1 AND NOT EXISTS(SELECT 1 FROM review_paused WHERE vocabulary_id=w.id) AND COALESCE(i.state,'ready')<>'pending'",
        [demand.vocabulary_id], |r| Ok((r.get::<_, String>(0)?,r.get::<_, bool>(1)?,r.get::<_, Option<String>>(2)?,r.get::<_, i64>(3)?,r.get::<_, i64>(4)?,r.get::<_, Option<String>>(5)?))).optional()?;
    let Some((identity, ready, desired_key, desired, current, migration_error)) = row else {
        return Ok(false);
    };
    if ready {
        return Ok(false);
    }
    let previous: Option<Option<String>> = tx.query_row("SELECT semantic_key FROM word_audio_repairs WHERE vocabulary_id=?1 AND failure_identity=?2", params![demand.vocabulary_id, identity], |r|r.get(0)).optional()?;
    if let Some(key) = previous {
        if let Some(key) = key {
            tx.execute("UPDATE speech_tasks SET demand_class=MIN(demand_class,?2) WHERE semantic_key=?1 AND priority=1 AND state IN ('queued','retry_wait')",params![key,demand.class])?;
        }
        tx.commit()?;
        return Ok(false);
    }
    if desired > current && desired_key.is_some() {
        tx.execute("UPDATE speech_tasks SET demand_class=MIN(demand_class,?2) WHERE semantic_key=?1 AND priority=1 AND state IN ('queued','retry_wait')",params![desired_key,demand.class])?;
        tx.execute("INSERT INTO word_audio_repairs(vocabulary_id,failure_identity,semantic_key,generation) VALUES(?1,?2,?3,?4)", params![demand.vocabulary_id,identity,desired_key,desired])?;
        tx.commit()?;
        return Ok(true);
    }
    let spec = if let Some(error) = migration_error.filter(|error| {
        !matches!(
            error.as_str(),
            "Invalid cached speech audio"
                | "Cached speech content checksum mismatch"
                | "Selected audio integrity check failed"
        )
    }) {
        Err(StorageError::Invalid(error))
    } else {
        word_repair_spec(&tx, demand.vocabulary_id)
    };
    let spec = match spec {
        Ok(spec) => spec,
        Err(StorageError::Invalid(error)) => {
            tx.execute("INSERT INTO word_audio_repairs(vocabulary_id,failure_identity,error) VALUES(?1,?2,?3)", params![demand.vocabulary_id,identity,error])?;
            tx.commit()?;
            return Ok(true);
        }
        Err(error) => return Err(error),
    };
    let key = crate::register_speech(&tx, &spec)?;
    tx.execute(
        "INSERT OR IGNORE INTO speech_owners VALUES(?1,?2)",
        params![key, demand.vocabulary_id],
    )?;
    crate::demand_speech(&tx, &spec, 1, now, false)?;
    let generation = crate::selected_audio::begin_audio_change_in(&tx, demand.vocabulary_id, &key)?;
    tx.execute("UPDATE speech_tasks SET demand_class=MIN(demand_class,?2),refresh_requested=CASE WHEN state IN ('queued','retry_wait') AND ?3 NOT LIKE 'absent:%' THEN 1 ELSE refresh_requested END WHERE semantic_key=?1",params![key,demand.class,identity])?;
    tx.execute("INSERT INTO word_audio_repairs(vocabulary_id,failure_identity,semantic_key,generation) VALUES(?1,?2,?3,?4)",params![demand.vocabulary_id,identity,key,generation])?;
    tx.commit()?;
    Ok(true)
}

pub struct AudioDemandDiagnostics {
    pub error: Option<String>,
    pub unknown_legacy: bool,
}
pub fn audio_demand_diagnostics(
    connection: &Connection,
    ids: &[i64],
) -> Result<AudioDemandDiagnostics, StorageError> {
    let mut result = AudioDemandDiagnostics {
        error: None,
        unknown_legacy: false,
    };
    for chunk in ids.chunks(100) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let mut query = connection.prepare(&format!("SELECT COALESCE(r.error,t.error,i.error),(v.locator IS NOT NULL AND v.semantic_key IS NULL) OR (s.current_version IS NULL AND EXISTS(SELECT 1 FROM legacy_speech l WHERE l.vocabulary_id=w.id)) FROM vocabulary w LEFT JOIN word_audio_selections s ON s.vocabulary_id=w.id LEFT JOIN speech_versions v ON v.locator=s.current_version LEFT JOIN word_audio_repairs r ON r.vocabulary_id=w.id AND r.failure_identity=COALESCE(s.current_version,'absent')||':'||COALESCE(v.verification,'missing') LEFT JOIN speech_tasks t ON t.semantic_key=COALESCE(s.desired_key,r.semantic_key) LEFT JOIN word_audio_imports i ON i.vocabulary_id=w.id WHERE w.id IN ({placeholders})"))?;
        for row in query.query_map(rusqlite::params_from_iter(chunk), |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, bool>(1)?))
        })? {
            let (error, unknown) = row?;
            if result.error.is_none() {
                result.error = error;
            }
            result.unknown_legacy |= unknown;
        }
    }
    Ok(result)
}
