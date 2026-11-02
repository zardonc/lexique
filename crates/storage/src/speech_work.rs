use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::{SpeechVoice, StorageError};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeechSpec {
    pub text: String,
    pub context: String,
    pub language: String,
    pub engine: String,
    pub model: String,
    pub preprocessing: String,
    pub voice: String,
    pub speed: u8,
    pub steps: u8,
    pub encoding: String,
    pub bitrate: u32,
}

impl SpeechSpec {
    pub fn word(text: &str, voice: SpeechVoice) -> Self {
        Self {
            text: text.nfc().collect(),
            context: "word".into(),
            language: "fr".into(),
            engine: "sherpa-onnx/2.0.1+ort/1.28.2".into(),
            model: concat!(
                "supertonic-3/int8/",
                "c3eb91414d5ff8a7a239b7fe9e34e7e2bf8a8140d8375ffb14718b1c639325db/",
                "c7befd5ea8c3119769e8a6c1486c4edc6a3bc8365c67621c881bbb774b9902ff/",
                "20cd86fa5c6effedfda0e7cffe5b0569ca401c440a0c3a1d72bf39286c0db3fd/",
                "e923d60f53f95eb1ce235f1dc33ec56d9c057823c96fa6f8acf98f32b0da6152/",
                "42078d3aef1cd43ab43021f3c54f47d2d75ceb4e75f627f118890128b06a0d09/",
                "8402ca48e5189a8950138580b0fff64db6f072f24ac07cd54ba8b2fbb9883b30/",
                "67d5209b0ee8ce6c74105ffbe12fe6a7628aea3b4ba2fcb308a4a67938a93ce8"
            )
            .into(),
            preprocessing: "nfc/y-i-grec/v1".into(),
            voice: match voice {
                SpeechVoice::Female => "female",
                SpeechVoice::Male => "male",
            }
            .into(),
            speed: 1,
            steps: 8,
            encoding: "ogg-opus/opus-1.6.1+libopusenc-0.3".into(),
            bitrate: 48000,
        }
    }
    pub fn key(&self) -> Result<String, StorageError> {
        if self.text.trim().is_empty()
            || self.text.chars().count() > 200
            || self.text.chars().any(char::is_control)
        {
            return Err(StorageError::Invalid("Invalid synthesis text".into()));
        }
        Ok(content_hash(&serde_json::to_vec(self)?))
    }
}

pub fn content_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn register_speech(connection: &Connection, spec: &SpeechSpec) -> Result<String, StorageError> {
    let key = spec.key()?;
    let serialized = serde_json::to_string(spec)?;
    if !matches!(spec.context.as_str(), "word" | "sentence") {
        return Err(StorageError::Invalid("Invalid speech context".into()));
    }
    connection.execute("INSERT INTO speech_assets(semantic_key,spec,context) VALUES(?1,?2,?3) ON CONFLICT DO NOTHING",params![key,serialized,spec.context])?;
    let saved: String = connection.query_row(
        "SELECT spec FROM speech_assets WHERE semantic_key=?1",
        [&key],
        |r| r.get(0),
    )?;
    if saved != serialized {
        return Err(StorageError::Invalid(
            "Speech semantic identity conflict".into(),
        ));
    }
    Ok(key)
}

pub fn attach_word_audio(connection: &Connection, id: i64) -> Result<String, StorageError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| StorageError::Invalid(e.to_string()))?
        .as_secs() as i64;
    attach_word_audio_at(connection, id, now)
}

pub fn attach_word_audio_at(
    connection: &Connection,
    id: i64,
    now: i64,
) -> Result<String, StorageError> {
    attach_word_voice(
        connection,
        id,
        crate::speech_preferences(connection)?.voice,
        now,
    )
}

fn attach_word_voice(
    connection: &Connection,
    id: i64,
    voice: SpeechVoice,
    now: i64,
) -> Result<String, StorageError> {
    let record = crate::get(connection, id)?;
    let spec = SpeechSpec::word(&record.headword, voice);
    let key = register_speech(connection, &spec)?;
    connection.execute(
        "INSERT OR IGNORE INTO speech_owners VALUES(?1,?2)",
        params![key, id],
    )?;
    protect_word_audio(connection, &key, now)?;
    Ok(key)
}

fn protect_word_audio(connection: &Connection, key: &str, now: i64) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_assets SET retain=1 WHERE semantic_key=?1 AND context='word' AND gc_pending=0 AND expires_at>?2",params![key,now])?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct SpeechAsset {
    pub semantic_key: String,
    pub content_hash: String,
    pub generated_at: i64,
}

pub fn speech_asset(
    connection: &Connection,
    key: &str,
    now: i64,
) -> Result<Option<SpeechAsset>, StorageError> {
    Ok(connection.query_row("SELECT semantic_key,content_hash,generated_at FROM speech_assets a WHERE semantic_key=?1 AND content_hash IS NOT NULL AND gc_pending=0 AND (expires_at>?2 OR (context='word' AND retain=1 AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=a.semantic_key)))",params![key,now],|r|Ok(SpeechAsset{semantic_key:r.get(0)?,content_hash:r.get(1)?,generated_at:r.get(2)?})).optional()?)
}

pub fn word_audio_keys(connection: &Connection, id: i64) -> Result<Vec<String>, StorageError> {
    let mut query=connection.prepare("SELECT a.semantic_key FROM speech_assets a JOIN speech_owners o USING(semantic_key) WHERE o.vocabulary_id=?1 AND a.content_hash IS NOT NULL ORDER BY a.generated_at DESC,a.semantic_key LIMIT 20")?;
    Ok(query
        .query_map([id], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}

pub fn publish_speech(
    connection: &Connection,
    key: &str,
    hash: &str,
    now: i64,
) -> Result<(), StorageError> {
    valid_hash(hash)?;
    let tx = connection.unchecked_transaction()?;
    tx.execute("UPDATE speech_assets SET content_hash=?1,generated_at=?2,expires_at=?3,pending_hash=NULL,pending_generated_at=NULL,gc_pending=0,retain=CASE WHEN context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=speech_assets.semantic_key) THEN 1 ELSE retain END WHERE semantic_key=?4",params![hash,now,now.checked_add(604800).ok_or_else(||StorageError::Invalid("Invalid audio clock".into()))?,key])?;
    tx.execute("UPDATE speech_tasks SET state='ready',error=NULL,refresh_requested=0 WHERE semantic_key=?1 AND state='running'",[key])?;
    tx.commit()?;
    Ok(())
}

pub fn expired_speech(
    connection: &Connection,
    now: i64,
    limit: u32,
) -> Result<Vec<String>, StorageError> {
    let mut query=connection.prepare("SELECT semantic_key FROM speech_assets a WHERE expires_at<=?1 AND retain=0 AND content_hash IS NOT NULL AND NOT (context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=a.semantic_key)) AND NOT EXISTS(SELECT 1 FROM speech_tasks t WHERE t.semantic_key=a.semantic_key AND t.state='running') ORDER BY expires_at LIMIT ?2")?;
    Ok(query
        .query_map(params![now, limit.min(100)], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}

pub fn demand_speech(
    connection: &Connection,
    spec: &SpeechSpec,
    priority: u8,
    now: i64,
    retry: bool,
) -> Result<i64, StorageError> {
    if priority > 1 {
        return Err(StorageError::Invalid("Invalid speech priority".into()));
    }
    let key = register_speech(connection, spec)?;
    let pending: i64 = connection.query_row(
        "SELECT COUNT(*) FROM speech_tasks WHERE priority=1 AND state IN ('queued','running','retry_wait')",
        [],
        |r| r.get(0),
    )?;
    let existing: Option<(u8, String)> = connection
        .query_row(
            "SELECT priority,state FROM speech_tasks WHERE semantic_key=?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let interactive: i64 = if priority == 0 {
        connection.query_row("SELECT COUNT(*) FROM speech_tasks WHERE priority=0 AND state IN ('queued','running','retry_wait')",[],|r|r.get(0))?
    } else {
        0
    };
    let additional = match &existing {
        None => true,
        Some((old_priority, state)) if matches!(state.as_str(), "queued" | "retry_wait") => {
            priority < *old_priority
        }
        Some((_, state)) if matches!(state.as_str(), "ready" | "failed" | "cancelled") => retry,
        _ => false,
    };
    if additional && ((priority == 1 && pending >= 10000) || (priority == 0 && interactive >= 32)) {
        return Err(StorageError::Invalid("Speech backlog is full".into()));
    }
    registered_demand(connection, &key, priority, now, retry)
}

fn registered_demand(
    connection: &Connection,
    key: &str,
    priority: u8,
    now: i64,
    retry: bool,
) -> Result<i64, StorageError> {
    connection.execute("INSERT INTO speech_tasks(semantic_key,priority,state,wake_at) VALUES(?1,?2,'queued',?3) ON CONFLICT(semantic_key) DO UPDATE SET priority=CASE WHEN state='running' THEN priority ELSE MIN(priority,excluded.priority) END",params![key,priority,now])?;
    if retry {
        connection.execute("UPDATE speech_tasks SET state='queued',attempts=0,deadline_at=NULL,wake_at=?1,cancel_requested=0,error=NULL WHERE semantic_key=?2 AND state IN ('failed','cancelled','ready')",params![now,key])?;
    }
    Ok(connection.query_row(
        "SELECT id FROM speech_tasks WHERE semantic_key=?1",
        [key],
        |r| r.get(0),
    )?)
}

#[derive(Serialize)]
pub struct BatchItem {
    pub vocabulary_id: i64,
    pub created_cards: u8,
    pub error: Option<String>,
    pub audio_state: Option<String>,
    pub audio_error: Option<String>,
}
#[derive(Serialize)]
pub struct BatchStatus {
    pub operation: String,
    pub cancelled: bool,
    pub items: Vec<BatchItem>,
}

pub fn batch_status(connection: &Connection, operation: &str) -> Result<BatchStatus, StorageError> {
    let cancelled = connection.query_row(
        "SELECT cancelled FROM review_batches WHERE operation=?1",
        [operation],
        |r| r.get(0),
    )?;
    let mut query=connection.prepare("SELECT i.vocabulary_id,i.created_cards,i.error,t.state,t.error FROM review_batch_items i LEFT JOIN speech_tasks t USING(semantic_key) WHERE i.operation=?1 ORDER BY i.ordinal LIMIT 10000")?;
    let items = query
        .query_map([operation], |r| {
            Ok(BatchItem {
                vocabulary_id: r.get(0)?,
                created_cards: r.get(1)?,
                error: r.get(2)?,
                audio_state: r.get(3)?,
                audio_error: r.get(4)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(BatchStatus {
        operation: operation.into(),
        cancelled,
        items,
    })
}

pub fn latest_batch(connection: &Connection) -> Result<Option<BatchStatus>, StorageError> {
    let operation: Option<String> = connection
        .query_row(
            "SELECT operation FROM review_batches ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    operation
        .map(|id| batch_status(connection, &id))
        .transpose()
}

pub fn add_review_batch(
    connection: &mut Connection,
    operation: &str,
    ids: &[i64],
    now: i64,
) -> Result<BatchStatus, StorageError> {
    add_review_batch_observed(connection, operation, ids, now, || {})
}

pub fn add_review_batch_observed(
    connection: &mut Connection,
    operation: &str,
    ids: &[i64],
    now: i64,
    mut committed: impl FnMut(),
) -> Result<BatchStatus, StorageError> {
    if operation.is_empty() || operation.len() > 128 || ids.is_empty() || ids.len() > 10000 {
        return Err(StorageError::Invalid("Invalid batch request".into()));
    }
    let request = serde_json::to_string(ids)?;
    let original: Option<(String, String)> = connection
        .query_row(
            "SELECT request,voice FROM review_batches WHERE operation=?1",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if original.as_ref().is_some_and(|(r, _)| r != &request) {
        return Err(StorageError::Invalid(
            "Batch operation identity conflict".into(),
        ));
    }
    let voice = match original {
        Some((_, voice)) => voice,
        None => serde_json::to_string(&crate::speech_preferences(connection)?.voice)?,
    };
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<_> = ids.iter().copied().filter(|id| seen.insert(*id)).collect();
    for (index, chunk) in ids.chunks(100).enumerate() {
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT OR IGNORE INTO review_batches(operation,request,voice,created_at) VALUES(?1,?2,?3,?4)",params![operation,request,voice,now])?;
        let (saved_request, saved_voice, cancelled): (String, String, bool) = tx.query_row(
            "SELECT request,voice,cancelled FROM review_batches WHERE operation=?1",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        if saved_request != request {
            return Err(StorageError::Invalid(
                "Batch operation identity conflict".into(),
            ));
        }
        let pending:i64=tx.query_row("SELECT COUNT(*) FROM speech_tasks WHERE priority=1 AND state IN ('queued','running','retry_wait')",[],|r|r.get(0))?;
        let mut remaining = 10000_i64.saturating_sub(pending);
        for (offset, &id) in chunk.iter().enumerate() {
            let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM review_batch_items WHERE operation=?1 AND vocabulary_id=?2)",params![operation,id],|r|r.get(0))?;
            if exists {
                continue;
            }
            tx.execute_batch("SAVEPOINT word")?;
            let result = build_word(
                &tx,
                id,
                now,
                serde_json::from_str(&saved_voice)?,
                &mut remaining,
            );
            let (key, created, error) = match result {
                Ok((key, created)) => (Some(key), created, None),
                Err(error) => {
                    tx.execute_batch("ROLLBACK TO word")?;
                    (None, 0, Some(error.to_string()))
                }
            };
            tx.execute_batch("RELEASE word")?;
            tx.execute(
                "INSERT INTO review_batch_items VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    operation,
                    (index * 100 + offset) as i64,
                    id,
                    key,
                    created,
                    error
                ],
            )?;
        }
        if cancelled {
            cancel_batch_tasks(&tx, operation)?;
        }
        tx.commit()?;
        committed();
    }
    batch_status(connection, operation)
}

fn build_word(
    tx: &rusqlite::Transaction<'_>,
    id: i64,
    now: i64,
    voice: SpeechVoice,
    remaining: &mut i64,
) -> Result<(String, u8), StorageError> {
    if id <= 0 || id > 9_007_199_254_740_991 {
        return Err(StorageError::Invalid("Invalid vocabulary ID".into()));
    }
    let record = crate::get(tx, id)?;
    let before = crate::review_cards(tx, id)?.len();
    crate::review::ensure_review_cards(tx, id)?;
    let spec = SpeechSpec::word(&record.headword, voice);
    let key = register_speech(tx, &spec)?;
    tx.execute(
        "INSERT OR IGNORE INTO speech_owners VALUES(?1,?2)",
        params![key, id],
    )?;
    protect_word_audio(tx, &key, now)?;
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM speech_tasks WHERE semantic_key=?1)",
        [&key],
        |r| r.get(0),
    )?;
    if !exists && *remaining <= 0 {
        return Err(StorageError::Invalid("Speech backlog is full".into()));
    }
    registered_demand(tx, &key, 1, now, false)?;
    if !exists {
        *remaining -= 1;
    }
    Ok((key, (crate::review_cards(tx, id)?.len() - before) as u8))
}

#[derive(Debug)]
pub struct SpeechTask {
    pub id: i64,
    pub semantic_key: String,
    pub spec: SpeechSpec,
    pub priority: u8,
    pub attempts: u8,
    pub refresh: bool,
    pub deadline_at: i64,
}

pub fn claim_speech(
    connection: &mut Connection,
    now: i64,
    background_limit: u8,
) -> Result<Option<SpeechTask>, StorageError> {
    if background_limit > 2 || now < 0 || now.checked_add(120).is_none() {
        return Err(StorageError::Invalid(
            "Invalid speech execution limits".into(),
        ));
    }
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute("UPDATE speech_tasks SET state='failed',error='Speech task budget exhausted' WHERE state IN ('queued','retry_wait') AND deadline_at<=?1",[now])?;
    let row=tx.query_row("SELECT t.id,t.semantic_key,a.spec,t.priority,t.attempts FROM speech_tasks t JOIN speech_assets a USING(semantic_key) WHERE t.state IN ('queued','retry_wait') AND t.wake_at<=?1 AND t.attempts<4 AND ((t.priority=0 AND NOT EXISTS(SELECT 1 FROM speech_tasks WHERE state='running' AND priority=0)) OR (t.priority=1 AND (SELECT COUNT(*) FROM speech_tasks WHERE state='running' AND priority=1)<?2)) ORDER BY t.priority,t.id LIMIT 1",params![now,background_limit],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,u8>(3)?,r.get::<_,u8>(4)?))).optional()?;
    let task = row
        .map(|(id, semantic_key, spec, priority, attempts)| {
            let refresh = tx.query_row(
                "SELECT refresh_requested FROM speech_tasks WHERE id=?1",
                [id],
                |r| r.get(0),
            )?;
            Ok::<_, StorageError>(SpeechTask {
                id,
                semantic_key,
                spec: serde_json::from_str(&spec)?,
                priority,
                attempts: attempts + 1,
                refresh,
                deadline_at: tx.query_row(
                    "SELECT COALESCE(deadline_at,?1) FROM speech_tasks WHERE id=?2",
                    params![now + 120, id],
                    |r| r.get(0),
                )?,
            })
        })
        .transpose()?;
    if let Some(task) = &task {
        tx.execute("UPDATE speech_tasks SET state='running',attempts=attempts+1,deadline_at=COALESCE(deadline_at,?1) WHERE id=?2",params![now+120,task.id])?;
    }
    tx.commit()?;
    Ok(task)
}

pub fn fail_speech(
    connection: &Connection,
    id: i64,
    now: i64,
    error: &str,
    permanent: bool,
) -> Result<(), StorageError> {
    let (attempts,deadline,cancel):(u8,i64,bool)=connection.query_row("SELECT attempts,deadline_at,cancel_requested FROM speech_tasks WHERE id=?1 AND state='running'",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    let delay = [2, 8, 20][usize::from(attempts.saturating_sub(1)).min(2)];
    let state = if cancel {
        "cancelled"
    } else if permanent || attempts >= 4 || now + delay >= deadline {
        "failed"
    } else {
        "retry_wait"
    };
    connection.execute(
        "UPDATE speech_tasks SET state=?1,wake_at=?2,error=?3 WHERE id=?4",
        params![
            state,
            now + delay,
            error.chars().take(2048).collect::<String>(),
            id
        ],
    )?;
    Ok(())
}

pub fn next_speech_wake(connection: &Connection) -> Result<Option<i64>, StorageError> {
    Ok(connection.query_row("SELECT MIN(wake_at) FROM speech_tasks t WHERE state IN ('queued','retry_wait') AND ((priority=0 AND NOT EXISTS(SELECT 1 FROM speech_tasks WHERE state='running' AND priority=0)) OR (priority=1 AND (SELECT COUNT(*) FROM speech_tasks WHERE state='running' AND priority=1)<2))",[],|r|r.get(0))?)
}

pub fn interrupted_speech(connection: &Connection) -> Result<Vec<(i64, String)>, StorageError> {
    let mut query = connection
        .prepare("SELECT id,semantic_key FROM speech_tasks WHERE state='running' LIMIT 8")?;
    Ok(query
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?)
}

pub fn resume_speech(connection: &Connection, id: i64, now: i64) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_tasks SET state=CASE WHEN cancel_requested=1 THEN 'cancelled' WHEN attempts>=4 OR deadline_at<=?1 THEN 'failed' ELSE 'queued' END,wake_at=?1,error='Speech worker interrupted' WHERE id=?2 AND state='running'",params![now,id])?;
    Ok(())
}

pub fn cancel_batch(connection: &Connection, operation: &str) -> Result<(), StorageError> {
    let tx = connection.unchecked_transaction()?;
    if tx.execute(
        "UPDATE review_batches SET cancelled=1 WHERE operation=?1",
        [operation],
    )? != 1
    {
        return Err(StorageError::Invalid("Batch not found".into()));
    }
    cancel_batch_tasks(&tx, operation)?;
    tx.commit()?;
    Ok(())
}

fn cancel_batch_tasks(connection: &Connection, operation: &str) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_tasks SET cancel_requested=1,state=CASE WHEN state IN ('queued','retry_wait') THEN 'cancelled' ELSE state END WHERE priority=1 AND semantic_key IN (SELECT semantic_key FROM review_batch_items WHERE operation=?1) AND NOT EXISTS(SELECT 1 FROM review_batch_items i JOIN review_batches b USING(operation) WHERE i.semantic_key=speech_tasks.semantic_key AND b.cancelled=0)",[operation])?;
    Ok(())
}

pub fn request_speech(
    connection: &mut Connection,
    spec: &SpeechSpec,
    owner: Option<i64>,
    now: i64,
    refresh: bool,
) -> Result<String, StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let key = register_speech(&tx, spec)?;
    if let Some(id) = owner {
        let record = crate::get(&tx, id)?;
        if spec.context != "word" || record.headword.nfc().collect::<String>() != spec.text {
            return Err(StorageError::Invalid(
                "Word audio ownership mismatch".into(),
            ));
        }
        tx.execute(
            "INSERT OR IGNORE INTO speech_owners VALUES(?1,?2)",
            params![key, id],
        )?;
        protect_word_audio(&tx, &key, now)?;
    }
    demand_speech(&tx, spec, 0, now, true)?;
    if refresh {
        force_speech_refresh(&tx, &key)?;
    }
    tx.commit()?;
    Ok(key)
}

fn valid_hash(hash: &str) -> Result<(), StorageError> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(StorageError::Invalid("Invalid audio content hash".into()));
    }
    Ok(())
}

pub fn prepare_speech_publication(
    connection: &Connection,
    key: &str,
    hash: &str,
    generated_at: i64,
) -> Result<(), StorageError> {
    valid_hash(hash)?;
    if generated_at < 0 || generated_at.checked_add(604800).is_none() {
        return Err(StorageError::Invalid("Invalid audio clock".into()));
    }
    if connection.execute("UPDATE speech_assets SET pending_hash=?1,pending_generated_at=?2 WHERE semantic_key=?3 AND gc_pending=0",params![hash,generated_at,key])?!=1 {
        return Err(StorageError::Invalid("Speech asset unavailable for publication".into()));
    }
    Ok(())
}

pub fn pending_publication(
    connection: &Connection,
    key: &str,
) -> Result<Option<(String, i64)>, StorageError> {
    Ok(connection.query_row("SELECT pending_hash,pending_generated_at FROM speech_assets WHERE semantic_key=?1 AND pending_hash IS NOT NULL",[key],|r|Ok((r.get(0)?,r.get(1)?))).optional()?)
}

pub fn complete_speech_publication(
    connection: &Connection,
    key: &str,
    hash: &str,
) -> Result<(), StorageError> {
    let (_, generated_at) = pending_publication(connection, key)?
        .filter(|(pending, _)| pending == hash)
        .ok_or_else(|| StorageError::Invalid("Speech publication identity conflict".into()))?;
    publish_speech(connection, key, hash, generated_at)
}

pub fn mark_speech_gc(
    connection: &Connection,
    key: &str,
    now: i64,
) -> Result<Option<String>, StorageError> {
    let tx = connection.unchecked_transaction()?;
    let changed=tx.execute("UPDATE speech_assets SET gc_pending=1 WHERE semantic_key=?1 AND content_hash IS NOT NULL AND expires_at<=?2 AND retain=0 AND NOT(context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=speech_assets.semantic_key)) AND NOT EXISTS(SELECT 1 FROM speech_tasks t WHERE t.semantic_key=speech_assets.semantic_key AND state='running')",params![key,now])?;
    let hash = if changed == 1 {
        Some(tx.query_row(
            "SELECT content_hash FROM speech_assets WHERE semantic_key=?1",
            [key],
            |r| r.get(0),
        )?)
    } else {
        None
    };
    tx.commit()?;
    Ok(hash)
}

pub fn finish_speech_gc(
    connection: &Connection,
    key: &str,
    hash: &str,
) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_assets SET content_hash=NULL,generated_at=NULL,expires_at=NULL,gc_pending=0 WHERE semantic_key=?1 AND content_hash=?2 AND gc_pending=1",params![key,hash])?;
    Ok(())
}

pub fn invalidate_speech(connection: &Connection, key: &str) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_assets SET content_hash=NULL,generated_at=NULL,expires_at=NULL WHERE semantic_key=?1",[key])?;
    Ok(())
}

pub fn register_legacy_speech(
    connection: &Connection,
    id: i64,
    hash: &str,
) -> Result<(), StorageError> {
    valid_hash(hash)?;
    let existing: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM legacy_speech WHERE vocabulary_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    if existing.as_deref() == Some(hash) {
        return Ok(());
    }
    connection.execute("INSERT INTO legacy_speech VALUES(?1,?2) ON CONFLICT(vocabulary_id) DO UPDATE SET content_hash=excluded.content_hash WHERE content_hash<>excluded.content_hash",params![id,hash])?;
    Ok(())
}

pub fn register_legacy_speeches(
    connection: &mut Connection,
    assets: &[(i64, String)],
) -> Result<(), StorageError> {
    for chunk in assets.chunks(100) {
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        for (id, hash) in chunk {
            register_legacy_speech(&tx, *id, hash)?;
        }
        tx.commit()?;
    }
    Ok(())
}

pub fn speech_task_state(
    connection: &Connection,
    key: &str,
) -> Result<Option<(String, Option<String>)>, StorageError> {
    Ok(connection
        .query_row(
            "SELECT state,error FROM speech_tasks WHERE semantic_key=?1",
            [key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
}

pub fn ready_speech_task(connection: &Connection, key: &str) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_tasks SET state='ready',error=NULL,attempts=MAX(0,attempts-1) WHERE semantic_key=?1 AND state='running'",[key])?;
    Ok(())
}

pub fn force_speech_refresh(connection: &Connection, key: &str) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_tasks SET refresh_requested=1 WHERE semantic_key=?1 AND state IN ('queued','retry_wait')",[key])?;
    Ok(())
}

pub fn pending_speech_keys(
    connection: &Connection,
    now: i64,
) -> Result<Vec<(String, bool)>, StorageError> {
    let mut query=connection.prepare("SELECT semantic_key,refresh_requested FROM speech_tasks WHERE state IN ('queued','retry_wait') AND wake_at<=?1 ORDER BY priority,id LIMIT 100")?;
    Ok(query
        .query_map([now], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?)
}

pub fn cached_speech_task(connection: &Connection, key: &str) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_tasks SET state='ready',error=NULL WHERE semantic_key=?1 AND state IN ('queued','retry_wait') AND refresh_requested=0",[key])?;
    Ok(())
}

pub fn speech_owner(connection: &Connection, key: &str) -> Result<Option<i64>, StorageError> {
    Ok(connection.query_row("SELECT vocabulary_id FROM speech_owners WHERE semantic_key=?1 ORDER BY vocabulary_id LIMIT 1",[key],|r|r.get(0)).optional()?)
}

pub fn pending_speech_gc(connection: &Connection) -> Result<Vec<(String, String)>, StorageError> {
    let mut query=connection.prepare("SELECT semantic_key,content_hash FROM speech_assets WHERE gc_pending=1 AND content_hash IS NOT NULL LIMIT 100")?;
    Ok(query
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?)
}
