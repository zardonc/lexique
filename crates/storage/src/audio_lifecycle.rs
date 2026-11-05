//! One authority for content validity, ownership and conditional physical reclamation.
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;

use crate::{
    AudioVersion, StorageError,
    selected_audio::{VERSION_COLUMNS, validate_word_spec, version_row},
};

pub const AUDIO_TEMP_SECONDS: i64 = 604_800;
pub const AUDIO_PREVIOUS_SECONDS: i64 = 1_209_600;
pub const AUDIO_SOFT_CAPACITY: i64 = 209_715_200;

fn deadline(now: i64, duration: i64) -> Result<i64, StorageError> {
    now.checked_add(duration)
        .filter(|_| now >= 0)
        .ok_or_else(|| StorageError::Invalid("Invalid audio lifecycle clock".into()))
}

pub fn audio_version(
    connection: &Connection,
    locator: &str,
) -> Result<Option<AudioVersion>, StorageError> {
    Ok(connection
        .query_row(
            &format!("SELECT {VERSION_COLUMNS} FROM speech_versions v WHERE locator=?1"),
            [locator],
            version_row,
        )
        .optional()?)
}

/// A source publication has a fresh physical identity, even for identical spec and bytes.
pub fn prepare_source_audio(
    connection: &Connection,
    key: &str,
    hash: &str,
    bytes: i64,
    generated_at: i64,
) -> Result<AudioVersion, StorageError> {
    if !(4..=10_000_000).contains(&bytes) {
        return Err(StorageError::Invalid("Invalid audio content length".into()));
    }
    reserve_source_audio(connection, key, hash, Some(bytes), generated_at, None)
}

pub(crate) fn reserve_source_audio(
    connection: &Connection,
    key: &str,
    hash: &str,
    bytes: Option<i64>,
    generated_at: i64,
    physical: Option<&str>,
) -> Result<AudioVersion, StorageError> {
    let expires = deadline(generated_at, AUDIO_TEMP_SECONDS)?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(StorageError::Invalid("Invalid audio content hash".into()));
    }
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let (context,execution): (String,Option<i64>) = tx.query_row("SELECT a.context,t.execution_generation FROM speech_assets a LEFT JOIN speech_tasks t USING(semantic_key) WHERE semantic_key=?1",[key],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if let Some(existing) = tx.query_row(&format!("SELECT {VERSION_COLUMNS} FROM speech_versions v JOIN speech_assets a ON a.pending_version=v.locator WHERE a.semantic_key=?1 AND v.content_hash=?2 AND v.generated_at=?3 AND v.execution_generation IS ?4 AND v.gc_state='live'"),params![key,hash,generated_at,execution],version_row).optional()? {
        if let Some(bytes)=bytes { tx.execute("UPDATE speech_versions SET byte_length=?1 WHERE locator=?2",params![bytes,existing.locator])?; }
        tx.commit()?;
        return audio_version(connection,&existing.locator)?.ok_or_else(||StorageError::Invalid("Source publication disappeared".into()));
    }
    let locator: String = tx.query_row("SELECT lower(hex(randomblob(32)))", [], |r| r.get(0))?;
    let physical = physical.unwrap_or(&locator);
    tx.execute("INSERT INTO speech_versions(locator,semantic_key,content_hash,byte_length,generated_at,created_at,execution_generation,physical_locator,context,expires_at,last_used) VALUES(?1,?2,?3,?4,?5,?5,?6,?7,?8,?9,?5)",params![locator,key,hash,bytes,generated_at,execution,physical,context,expires])?;
    tx.execute("UPDATE speech_assets SET pending_version=?1,pending_hash=?2,pending_generated_at=?3,pending_execution=?4 WHERE semantic_key=?5",params![locator,hash,generated_at,execution,key])?;
    tx.commit()?;
    audio_version(connection, &locator)?
        .ok_or_else(|| StorageError::Invalid("Source publication disappeared".into()))
}

/// The caller verified the exact bytes and hash; metadata-only legacy registration stays unverified.
pub fn verify_audio_content(
    connection: &Connection,
    locator: &str,
    bytes: i64,
) -> Result<(), StorageError> {
    if !(4..=10_000_000).contains(&bytes) {
        return Err(StorageError::Invalid(
            "Invalid verified audio length".into(),
        ));
    }
    if connection.execute("UPDATE speech_versions SET byte_length=?1,verification='verified',sealed=1 WHERE locator=?2 AND gc_state='live'",params![bytes,locator])?!=1 {
        return Err(StorageError::Invalid("Audio version was retired during verification".into()));
    }
    Ok(())
}

pub fn pending_source_audio(
    connection: &Connection,
    after: &str,
) -> Result<Vec<AudioVersion>, StorageError> {
    let mut query=connection.prepare(&format!("SELECT {VERSION_COLUMNS} FROM speech_versions v JOIN speech_assets a ON a.pending_version=v.locator WHERE v.locator>?1 ORDER BY v.locator LIMIT 100"))?;
    Ok(query
        .query_map([after], version_row)?
        .collect::<Result<_, _>>()?)
}

pub fn complete_source_audio(connection: &Connection, locator: &str) -> Result<bool, StorageError> {
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let row: Option<(String,String,i64,Option<i64>)>=tx.query_row("SELECT a.semantic_key,v.content_hash,v.generated_at,v.execution_generation FROM speech_assets a JOIN speech_versions v ON v.locator=a.pending_version WHERE v.locator=?1 AND v.gc_state='live'",[locator],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let Some((key, hash, generated, execution)) = row else {
        tx.commit()?;
        return Ok(false);
    };
    let valid: bool=tx.query_row("SELECT ?2 IS NULL OR EXISTS(SELECT 1 FROM speech_tasks WHERE semantic_key=?1 AND execution_generation=?2 AND state IN ('running','ready'))",params![key,execution],|r|r.get(0))?;
    if valid {
        tx.execute("UPDATE speech_assets SET current_version=?1,content_hash=?2,generated_at=?3,expires_at=?4,gc_pending=0,retain=CASE WHEN context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=speech_assets.semantic_key) THEN 1 ELSE 0 END WHERE semantic_key=?5",params![locator,hash,generated,deadline(generated,AUDIO_TEMP_SECONDS)?,key])?;
        crate::speech_work::finish_speech_execution(&tx, &key)?;
    }
    tx.execute("UPDATE speech_assets SET pending_version=NULL,pending_hash=NULL,pending_generated_at=NULL,pending_execution=NULL WHERE pending_version=?1",[locator])?;
    tx.commit()?;
    Ok(valid)
}

pub fn abandon_audio_publication(
    connection: &Connection,
    locator: &str,
    error: Option<&str>,
) -> Result<(), StorageError> {
    let tx = connection.unchecked_transaction()?;
    tx.execute("UPDATE speech_assets SET pending_version=NULL,pending_hash=NULL,pending_generated_at=NULL,pending_execution=NULL WHERE pending_version=?1",[locator])?;
    tx.execute(
        "DELETE FROM word_audio_publications WHERE locator=?1",
        [locator],
    )?;
    tx.execute("UPDATE speech_versions SET verification='missing',gc_error=?2 WHERE locator=?1 AND verification<>'verified'",params![locator,error.map(|e|e.chars().take(2048).collect::<String>())])?;
    tx.commit()?;
    Ok(())
}

pub(crate) fn protect_last_detached(
    connection: &Connection,
    locator: Option<&str>,
    now: i64,
) -> Result<(), StorageError> {
    if let Some(locator) = locator {
        connection.execute("UPDATE speech_versions SET orphan_until=MAX(COALESCE(orphan_until,0),?2) WHERE locator=?1 AND NOT EXISTS(SELECT 1 FROM word_audio_selections WHERE current_version=?1)",params![locator,deadline(now,AUDIO_TEMP_SECONDS)?])?;
    }
    Ok(())
}

/// Reuses an already verified instance in the same business transaction as acquiring ownership.
pub fn reuse_audio_version(
    connection: &Connection,
    id: i64,
    locator: &str,
    generation: i64,
    now: i64,
) -> Result<bool, StorageError> {
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let changed = reuse_audio_version_in(&tx, id, locator, generation, now)?;
    tx.commit()?;
    Ok(changed)
}

pub(crate) fn reuse_audio_version_in(
    connection: &Connection,
    id: i64,
    locator: &str,
    generation: i64,
    now: i64,
) -> Result<bool, StorageError> {
    let until = deadline(now, AUDIO_PREVIOUS_SECONDS)?;
    let version = audio_version(connection, locator)?
        .ok_or_else(|| StorageError::Invalid("Audio content version is unknown".into()))?;
    validate_word_spec(connection, id, version.semantic_key.as_deref())?;
    let valid:bool=connection.query_row("SELECT gc_state='live' AND verification='verified' AND context='word' AND (owned OR migrating OR expires_at>?2 OR protected_until>?2) FROM audio_version_protection WHERE locator=?1",params![locator,now],|r|r.get(0))?;
    if !valid {
        return Ok(false);
    }
    if generation > 0
        && let Some(execution) = version.execution_generation
    {
        let valid:bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM speech_tasks t JOIN speech_assets a USING(semantic_key) WHERE t.semantic_key=?1 AND t.execution_generation=?2 AND a.current_version=?3 AND t.state='ready' AND t.cancel_requested=0 AND t.refresh_requested=0 AND t.successor_requested=0)",params![version.semantic_key,execution,locator],|r|r.get(0))?;
        if !valid {
            return Ok(false);
        }
    }
    connection.execute(
        "INSERT OR IGNORE INTO word_audio_selections(vocabulary_id) VALUES(?1)",
        [id],
    )?;
    let (current,desired,key):(Option<String>,i64,Option<String>)=connection.query_row("SELECT current_version,desired_generation,desired_key FROM word_audio_selections WHERE vocabulary_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if desired != generation
        || (generation > 0 && key != version.semantic_key)
        || (generation == 0 && current.is_some())
    {
        return Ok(current.as_deref() == Some(locator) && desired == generation);
    }
    if generation == 0 && !crate::pending_word_audio_versions(connection, id)?.is_empty() {
        return Ok(false);
    }
    let changed=connection.execute("UPDATE word_audio_selections SET previous_version=CASE WHEN current_version IS ?2 THEN previous_version ELSE current_version END,previous_until=CASE WHEN current_version IS ?2 THEN previous_until WHEN current_version IS NULL THEN NULL ELSE ?3 END,current_version=?2,current_generation=?4,revision=revision+1 WHERE vocabulary_id=?1 AND revision<9223372036854775807",params![id,locator,until,generation])?==1;
    if changed && current.as_deref() != Some(locator) {
        protect_last_detached(connection, current.as_deref(), now)?;
    }
    Ok(changed)
}

/// Future word deletion must call this inside its own business transaction before removing the word.
pub fn detach_word_audio(
    connection: &mut Connection,
    id: i64,
    now: i64,
) -> Result<(), StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    detach_word_audio_in(&tx, id, now)?;
    tx.commit()?;
    Ok(())
}

pub fn detach_word_audio_in(
    connection: &Connection,
    id: i64,
    now: i64,
) -> Result<(), StorageError> {
    deadline(now, AUDIO_TEMP_SECONDS)?;
    let current: Option<String> = connection
        .query_row(
            "SELECT current_version FROM word_audio_selections WHERE vocabulary_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    connection.execute(
        "DELETE FROM word_audio_selections WHERE vocabulary_id=?1",
        [id],
    )?;
    protect_last_detached(connection, current.as_deref(), now)?;
    connection.execute(
        "DELETE FROM word_audio_publications WHERE vocabulary_id=?1",
        [id],
    )?;
    connection.execute("DELETE FROM speech_owners WHERE vocabulary_id=?1", [id])?;
    connection.execute("DELETE FROM legacy_speech WHERE vocabulary_id=?1", [id])?;
    connection.execute(
        "DELETE FROM word_audio_imports WHERE vocabulary_id=?1",
        [id],
    )?;
    Ok(())
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AudioUsage {
    pub logical_bytes: i64,
    pub permanent_bytes: i64,
    pub protected_bytes: i64,
    pub reclaimable_bytes: i64,
    pub publication_bytes: i64,
    pub unowned_bytes: i64,
    pub marked_bytes: i64,
    pub over_capacity: bool,
    pub gc_failures: u32,
}

pub fn audio_usage(connection: &Connection, now: i64) -> Result<AudioUsage, StorageError> {
    deadline(now, 0)?;
    let mut usage=connection.query_row("SELECT COALESCE(SUM(byte_length),0),COALESCE(SUM(CASE WHEN owned THEN byte_length ELSE 0 END),0),COALESCE(SUM(CASE WHEN NOT owned AND (protected_until>?1 OR migrating) THEN byte_length ELSE 0 END),0),COALESCE(SUM(CASE WHEN NOT owned AND protected_until<=?1 AND NOT migrating AND NOT publishing THEN byte_length ELSE 0 END),0),COALESCE(SUM(CASE WHEN publishing THEN byte_length ELSE 0 END),0),COALESCE(SUM(CASE WHEN NOT owned THEN byte_length ELSE 0 END),0),COALESCE(SUM(CASE WHEN NOT owned AND gc_state='marked' THEN byte_length ELSE 0 END),0),COUNT(CASE WHEN gc_error IS NOT NULL THEN 1 END) FROM audio_version_protection WHERE gc_state<>'deleted'",[now],|r|Ok(AudioUsage{logical_bytes:r.get(0)?,permanent_bytes:r.get(1)?,protected_bytes:r.get(2)?,reclaimable_bytes:r.get(3)?,publication_bytes:r.get(4)?,unowned_bytes:r.get(5)?,marked_bytes:r.get(6)?,gc_failures:r.get(7)?,..AudioUsage::default()}))?;
    usage.over_capacity = usage.unowned_bytes > AUDIO_SOFT_CAPACITY;
    Ok(usage)
}

#[derive(Debug)]
pub struct AudioGc {
    pub version: AudioVersion,
    pub generation: i64,
}

pub fn audio_gc_candidates(
    connection: &Connection,
    now: i64,
    limit: u32,
) -> Result<Vec<String>, StorageError> {
    let usage = audio_usage(connection, now)?;
    let capacity = usage.unowned_bytes - usage.marked_bytes > AUDIO_SOFT_CAPACITY;
    let mut query=connection.prepare("SELECT locator FROM audio_version_protection WHERE gc_state='live' AND NOT owned AND NOT publishing AND NOT migrating AND protected_until<=?1 AND (expires_at<=?1 OR expires_at IS NULL OR ?2) ORDER BY CASE WHEN expires_at<=?1 OR expires_at IS NULL THEN 0 ELSE 1 END,last_used,locator LIMIT ?3")?;
    Ok(query
        .query_map(params![now, capacity, limit.min(100)], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}

pub fn mark_audio_gc(
    connection: &Connection,
    locator: &str,
    now: i64,
) -> Result<Option<AudioGc>, StorageError> {
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let usage = audio_usage(&tx, now)?;
    let result = mark_audio_gc_in(
        &tx,
        locator,
        now,
        usage.unowned_bytes - usage.marked_bytes > AUDIO_SOFT_CAPACITY,
    )?;
    tx.commit()?;
    Ok(result)
}

fn mark_audio_gc_in(
    connection: &Connection,
    locator: &str,
    now: i64,
    capacity: bool,
) -> Result<Option<AudioGc>, StorageError> {
    let columns = VERSION_COLUMNS.replace("v.", "");
    Ok(connection.query_row(&format!("UPDATE speech_versions SET gc_state='marked',gc_generation=gc_generation+1,gc_error=NULL WHERE locator=?1 AND gc_generation<9223372036854775807 AND locator IN (SELECT locator FROM audio_version_protection WHERE gc_state='live' AND NOT owned AND NOT publishing AND NOT migrating AND protected_until<=?2 AND (expires_at<=?2 OR expires_at IS NULL OR ?3)) RETURNING {columns},gc_generation"),params![locator,now,capacity],|r| Ok(AudioGc{version:version_row(r)?,generation:r.get(12)?})).optional()?)
}

/// One capacity snapshot and short transaction for a bounded batch; already marked bytes count.
pub fn mark_audio_gc_batch(
    connection: &Connection,
    now: i64,
    limit: u32,
) -> Result<Vec<AudioGc>, StorageError> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let tx = rusqlite::Transaction::new_unchecked(connection, TransactionBehavior::Immediate)?;
    let usage = audio_usage(&tx, now)?;
    let mut remaining_bytes = usage.unowned_bytes - usage.marked_bytes;
    let mut query = tx.prepare("SELECT locator,expires_at<=?1 OR expires_at IS NULL FROM audio_version_protection WHERE gc_state='live' AND gc_generation<9223372036854775807 AND NOT owned AND NOT publishing AND NOT migrating AND protected_until<=?1 AND (expires_at<=?1 OR expires_at IS NULL OR ?2) ORDER BY CASE WHEN expires_at<=?1 OR expires_at IS NULL THEN 0 ELSE 1 END,last_used,locator LIMIT ?3")?;
    let candidates = query
        .query_map(
            params![now, remaining_bytes > AUDIO_SOFT_CAPACITY, limit.min(100)],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?)),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(query);
    let mut work = Vec::with_capacity(candidates.len());
    for (locator, expired) in candidates {
        if !expired && remaining_bytes <= AUDIO_SOFT_CAPACITY {
            break;
        }
        if let Some(gc) =
            mark_audio_gc_in(&tx, &locator, now, remaining_bytes > AUDIO_SOFT_CAPACITY)?
        {
            remaining_bytes = remaining_bytes.saturating_sub(gc.version.byte_length);
            work.push(gc);
        }
    }
    tx.commit()?;
    Ok(work)
}

pub fn pending_audio_gc(connection: &Connection) -> Result<Vec<AudioGc>, StorageError> {
    pending_audio_gc_after(connection, "", 100)
}

/// A maintenance sweep advances past failed deletes before retrying them in the next sweep.
pub fn pending_audio_gc_after(
    connection: &Connection,
    after: &str,
    limit: u32,
) -> Result<Vec<AudioGc>, StorageError> {
    let mut query=connection.prepare(&format!("SELECT {VERSION_COLUMNS},v.gc_generation FROM speech_versions v WHERE gc_state='marked' AND locator>?1 ORDER BY locator LIMIT ?2"))?;
    Ok(query
        .query_map(params![after, limit.min(100)], |r| {
            Ok(AudioGc {
                version: version_row(r)?,
                generation: r.get(12)?,
            })
        })?
        .collect::<Result<_, _>>()?)
}

pub fn finish_audio_gc(
    connection: &Connection,
    locator: &str,
    generation: i64,
) -> Result<bool, StorageError> {
    let tx = connection.unchecked_transaction()?;
    let changed=tx.execute("UPDATE speech_versions SET gc_state='deleted',gc_error=NULL WHERE locator=?1 AND gc_state='marked' AND gc_generation=?2",params![locator,generation])?==1;
    if changed {
        tx.execute("UPDATE speech_assets SET current_version=NULL,content_hash=NULL,generated_at=NULL,expires_at=NULL,gc_pending=0,retain=0 WHERE current_version=?1",[locator])?;
    }
    tx.commit()?;
    Ok(changed)
}

pub fn fail_audio_gc(
    connection: &Connection,
    locator: &str,
    generation: i64,
    error: &str,
) -> Result<(), StorageError> {
    connection.execute("UPDATE speech_versions SET gc_error=?3 WHERE locator=?1 AND gc_state='marked' AND gc_generation=?2",params![locator,generation,error.chars().take(2048).collect::<String>()])?;
    Ok(())
}

pub fn mark_audio_unavailable(
    connection: &Connection,
    locator: &str,
    state: &str,
) -> Result<(), StorageError> {
    if !matches!(state, "suspect" | "missing" | "invalid") {
        return Err(StorageError::Invalid(
            "Invalid audio verification state".into(),
        ));
    }
    connection.execute("UPDATE speech_versions SET verification=?2 WHERE locator=?1 AND gc_state='live' AND verification<>?2",params![locator,state])?;
    Ok(())
}

pub fn audio_last_used(
    connection: &Connection,
    locator: &str,
    now: i64,
) -> Result<(), StorageError> {
    deadline(now, 0)?;
    connection.execute("UPDATE speech_versions SET last_used=?2 WHERE locator=?1 AND gc_state='live' AND last_used<?2-300",params![locator,now])?;
    Ok(())
}
