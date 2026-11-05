//! Immutable audio instances and conditional word selection. Files stay in the platform layer.
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::StorageError;

#[derive(Clone, Debug, Serialize)]
pub struct AudioVersion {
    pub locator: String,
    pub semantic_key: Option<String>,
    pub content_hash: String,
    pub byte_length: i64,
    pub generated_at: Option<i64>,
    pub vocabulary_id: i64,
    pub expected_revision: i64,
    pub generation: i64,
    pub execution_generation: Option<i64>,
    pub physical_locator: String,
    pub verification: String,
    pub import_locator: Option<String>,
}

pub(crate) const VERSION_COLUMNS: &str = "v.locator,v.semantic_key,v.content_hash,COALESCE(v.byte_length,0),v.generated_at,COALESCE(v.vocabulary_id,0),v.expected_revision,v.generation,v.execution_generation,v.physical_locator,v.verification,v.import_locator";

pub(crate) fn version_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AudioVersion> {
    Ok(AudioVersion {
        locator: row.get(0)?,
        semantic_key: row.get(1)?,
        content_hash: row.get(2)?,
        byte_length: row.get(3)?,
        generated_at: row.get(4)?,
        vocabulary_id: row.get(5)?,
        expected_revision: row.get(6)?,
        generation: row.get(7)?,
        execution_generation: row.get(8)?,
        physical_locator: row.get(9)?,
        verification: row.get(10)?,
        import_locator: row.get(11)?,
    })
}

pub fn selected_audio(
    connection: &Connection,
    id: i64,
) -> Result<Option<AudioVersion>, StorageError> {
    Ok(connection.query_row("SELECT v.locator,v.semantic_key,v.content_hash,v.byte_length,v.generated_at,s.vocabulary_id,s.revision,s.current_generation,v.execution_generation,v.physical_locator,v.verification,v.import_locator FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id=?1 AND v.sealed=1 AND v.gc_state='live'", [id], version_row).optional()?)
}

/// Capture ready execution and selection generations in one SQLite snapshot.
pub fn ready_audio_selections(
    connection: &Connection,
    key: &str,
) -> Result<Vec<(i64, i64)>, StorageError> {
    let mut query = connection.prepare("SELECT o.vocabulary_id,COALESCE(s.desired_generation,0) FROM speech_owners o JOIN speech_tasks t USING(semantic_key) LEFT JOIN word_audio_selections s USING(vocabulary_id) WHERE o.semantic_key=?1 AND t.state='ready' AND t.cancel_requested=0 AND t.refresh_requested=0 AND t.successor_requested=0 AND ((s.current_version IS NULL AND COALESCE(s.desired_generation,0)=0) OR (s.desired_generation>0 AND s.desired_key=?1 AND s.current_generation<>s.desired_generation))")?;
    Ok(query
        .query_map([key], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<_, _>>()?)
}

/// A genuine new user request supersedes older completions, independently of the spec key.
pub fn begin_audio_change(
    connection: &Connection,
    id: i64,
    key: &str,
) -> Result<i64, StorageError> {
    let tx = connection.unchecked_transaction()?;
    let generation = begin_audio_change_in(&tx, id, key)?;
    tx.commit()?;
    Ok(generation)
}

pub(crate) fn begin_audio_change_in(
    connection: &Connection,
    id: i64,
    key: &str,
) -> Result<i64, StorageError> {
    validate_word_spec(connection, id, Some(key))?;
    connection.execute(
        "INSERT OR IGNORE INTO word_audio_selections(vocabulary_id) VALUES(?1)",
        [id],
    )?;
    if connection.execute("UPDATE word_audio_selections SET desired_generation=desired_generation+1,desired_key=?1 WHERE vocabulary_id=?2 AND desired_generation<9223372036854775807", params![key,id])? != 1 {
        return Err(StorageError::Invalid("Audio selection generation exhausted".into()));
    }
    let generation = connection.query_row(
        "SELECT desired_generation FROM word_audio_selections WHERE vocabulary_id=?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(generation)
}

/// Reserve an instance before file publication. A crash leaves a recoverable intent.
pub fn prepare_audio_version(
    connection: &Connection,
    id: i64,
    key: Option<&str>,
    hash: &str,
    bytes: i64,
    generated_at: Option<i64>,
    generation: i64,
    now: i64,
) -> Result<AudioVersion, StorageError> {
    if now < 0
        || now.checked_add(1_209_600).is_none()
        || generated_at.is_some_and(|at| at < 0 || at > now)
        || hash.len() != 64
        || !hash
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        || !(4..=10_000_000).contains(&bytes)
        || generation < 0
    {
        return Err(StorageError::Invalid(
            "Invalid audio version metadata".into(),
        ));
    }
    let tx = connection.unchecked_transaction()?;
    validate_word_spec(&tx, id, key)?;
    let execution_generation = tx
        .query_row(
            "SELECT execution_generation FROM speech_tasks WHERE semantic_key=?1",
            [key],
            |r| r.get::<_, i64>(0),
        )
        .optional()?;
    tx.execute(
        "INSERT OR IGNORE INTO word_audio_selections(vocabulary_id) VALUES(?1)",
        [id],
    )?;
    let (revision, current, desired): (i64, Option<String>, i64) = tx.query_row(
        "SELECT revision,current_version,desired_generation FROM word_audio_selections WHERE vocabulary_id=?1",
        [id],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
    )?;
    if generation == 0 && (current.is_some() || desired != 0) {
        return Err(StorageError::Invalid(
            "Initial audio selection was superseded".into(),
        ));
    }
    if let Some(existing) = tx.query_row(&format!("SELECT {VERSION_COLUMNS} FROM speech_versions v JOIN word_audio_publications p USING(locator) WHERE p.vocabulary_id=?1 AND p.expected_revision=?2 AND p.generation=?3 AND v.semantic_key IS ?4 AND v.content_hash=?5 AND v.byte_length=?6 AND v.generated_at IS ?7 AND p.execution_generation IS ?8 AND v.sealed=0 AND v.gc_state='live' LIMIT 1"), params![id,revision,generation,key,hash,bytes,generated_at,execution_generation], version_row).optional()? {
        tx.commit()?;
        return Ok(existing);
    }
    if generation == 0 && !pending_word_audio_versions(&tx, id)?.is_empty() {
        return Err(StorageError::Invalid(
            "Initial audio selection has a pending publication".into(),
        ));
    }
    let locator: String =
        tx.query_row("SELECT lower(hex(randomblob(32)))", [], |row| row.get(0))?;
    tx.execute("INSERT INTO speech_versions(locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,execution_generation,physical_locator,context,expires_at,orphan_until,last_used) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?1,'word',?11,?12,?6)", params![locator,key,hash,bytes,generated_at,now,id,revision,generation,execution_generation,generated_at.map(|at|at+604800),generated_at.is_none().then_some(now+604800)])?;
    tx.execute("INSERT INTO word_audio_publications(locator,vocabulary_id,expected_revision,generation,execution_generation) VALUES(?1,?2,?3,?4,?5)", params![locator,id,revision,generation,execution_generation])?;
    tx.commit()?;
    Ok(AudioVersion {
        locator: locator.clone(),
        semantic_key: key.map(str::to_owned),
        content_hash: hash.into(),
        byte_length: bytes,
        generated_at,
        vocabulary_id: id,
        expected_revision: revision,
        generation,
        execution_generation,
        physical_locator: locator.clone(),
        verification: "unverified".into(),
        import_locator: None,
    })
}

/// Called only after platform safe-read/hash verification, never merely after rename.
pub fn complete_audio_version(
    connection: &Connection,
    locator: &str,
    now: i64,
) -> Result<bool, StorageError> {
    let until = now
        .checked_add(1_209_600)
        .filter(|_| now >= 0)
        .ok_or_else(|| StorageError::Invalid("Invalid audio selection clock".into()))?;
    let tx =
        rusqlite::Transaction::new_unchecked(connection, rusqlite::TransactionBehavior::Immediate)?;
    let version = tx.query_row(
        &format!("SELECT {VERSION_COLUMNS} FROM speech_versions v WHERE locator=?1"),
        [locator],
        version_row,
    )?;
    let already: bool = tx
        .query_row(
            "SELECT current_version=?1 FROM word_audio_selections WHERE vocabulary_id=?2",
            params![locator, version.vocabulary_id],
            |row| Ok(row.get::<_, Option<bool>>(0)?.unwrap_or(false)),
        )
        .optional()?
        .unwrap_or(false);
    if already {
        tx.commit()?;
        return Ok(true);
    }
    let intent: Option<(i64,i64,i64,Option<i64>)> = tx.query_row("SELECT vocabulary_id,expected_revision,generation,execution_generation FROM word_audio_publications WHERE locator=?1", [locator], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let Some((id, revision, generation, execution)) = intent else {
        tx.commit()?;
        return Ok(false);
    };
    tx.execute("UPDATE speech_versions SET sealed=1,verification='verified' WHERE locator=?1 AND gc_state='live'", [locator])?;
    let execution_valid = match execution {
        Some(execution) => tx.query_row("SELECT EXISTS(SELECT 1 FROM speech_tasks t JOIN speech_assets a USING(semantic_key) WHERE t.semantic_key=?1 AND t.execution_generation=?2 AND t.state='ready' AND t.cancel_requested=0 AND t.refresh_requested=0 AND t.successor_requested=0 AND a.content_hash=?3 AND a.generated_at IS ?4)", params![version.semantic_key,execution,version.content_hash,version.generated_at], |r| r.get::<_, bool>(0))?,
        None => true,
    };
    let changed = if execution_valid {
        let previous: Option<String> = tx
            .query_row(
                "SELECT current_version FROM word_audio_selections WHERE vocabulary_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let changed = tx.execute("UPDATE word_audio_selections SET previous_version=current_version,previous_until=CASE WHEN current_version IS NULL THEN NULL ELSE ?1 END,current_version=?2,current_generation=?5,revision=revision+1 WHERE vocabulary_id=?3 AND revision=?4 AND desired_generation=?5 AND (desired_generation=0 OR desired_key=?6) AND revision<9223372036854775807 AND EXISTS(SELECT 1 FROM speech_versions WHERE locator=?2 AND gc_state='live')", params![until,locator,id,revision,generation,version.semantic_key])?;
        if changed == 1 {
            crate::audio_lifecycle::protect_last_detached(&tx, previous.as_deref(), now)?;
        }
        changed
    } else {
        0
    };
    tx.execute(
        "DELETE FROM word_audio_publications WHERE locator=?1",
        [locator],
    )?;
    tx.commit()?;
    Ok(changed == 1)
}

/// Keyset pagination lets recovery reach valid imports after missing or damaged ones.
pub fn pending_audio_versions(
    connection: &Connection,
    after: &str,
) -> Result<Vec<AudioVersion>, StorageError> {
    let mut query = connection.prepare(&format!("SELECT {VERSION_COLUMNS} FROM speech_versions v JOIN word_audio_publications p USING(locator) WHERE locator>?1 ORDER BY locator LIMIT 100"))?;
    Ok(query
        .query_map([after], version_row)?
        .collect::<Result<_, _>>()?)
}

/// A cache read may finish the exact durable intent before startup recovery reaches it.
pub fn pending_word_audio_versions(
    connection: &Connection,
    id: i64,
) -> Result<Vec<AudioVersion>, StorageError> {
    let mut query = connection.prepare(&format!("SELECT {VERSION_COLUMNS} FROM speech_versions v JOIN word_audio_publications p USING(locator) JOIN word_audio_selections s ON s.vocabulary_id=p.vocabulary_id WHERE p.vocabulary_id=?1 AND p.expected_revision=s.revision AND p.generation=s.desired_generation AND (p.generation=0 OR s.desired_key IS v.semantic_key) ORDER BY v.locator LIMIT 100"))?;
    Ok(query
        .query_map([id], version_row)?
        .collect::<Result<_, _>>()?)
}

pub(crate) fn validate_word_spec(
    connection: &Connection,
    id: i64,
    key: Option<&str>,
) -> Result<(), StorageError> {
    let headword: String =
        connection.query_row("SELECT headword FROM vocabulary WHERE id=?1", [id], |row| {
            row.get(0)
        })?;
    if let Some(key) = key {
        let saved: String = connection.query_row(
            "SELECT spec FROM speech_assets WHERE semantic_key=?1",
            [key],
            |row| row.get(0),
        )?;
        let spec: crate::SpeechSpec = serde_json::from_str(&saved)?;
        if spec.context != "word" || spec.text != headword || spec.key()? != key {
            return Err(StorageError::Invalid(
                "Audio specification does not match the word".into(),
            ));
        }
    }
    Ok(())
}

pub fn selected_audio_batch(
    connection: &Connection,
    ids: &[i64],
) -> Result<std::collections::HashMap<i64, AudioVersion>, StorageError> {
    let mut result = std::collections::HashMap::new();
    for chunk in ids.chunks(100) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let mut query = connection.prepare(&format!("SELECT v.locator,v.semantic_key,v.content_hash,v.byte_length,v.generated_at,s.vocabulary_id,s.revision,s.current_generation,v.execution_generation,v.physical_locator,v.verification,v.import_locator FROM word_audio_selections s JOIN speech_versions v ON v.locator=s.current_version WHERE s.vocabulary_id IN ({placeholders}) AND v.sealed=1 AND v.gc_state='live'"))?;
        for row in query.query_map(rusqlite::params_from_iter(chunk), version_row)? {
            let version = row?;
            result.insert(version.vocabulary_id, version);
        }
    }
    Ok(result)
}

#[derive(Clone, Default, Serialize)]
pub struct AudioMigrationStatus {
    pub pending: u32,
    pub failed: u32,
}

pub fn audio_migration_status(
    connection: &Connection,
) -> Result<AudioMigrationStatus, StorageError> {
    let mut status = AudioMigrationStatus::default();
    let mut query = connection.prepare(
        "SELECT state,COUNT(*) FROM word_audio_imports WHERE state<>'finished' GROUP BY state",
    )?;
    for row in query.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))? {
        match row? {
            (state, count) if state == "pending" => status.pending = count,
            (_, count) => status.failed = count,
        }
    }
    Ok(status)
}

pub fn pending_audio_imports(connection: &Connection) -> Result<Vec<i64>, StorageError> {
    let mut query = connection.prepare("SELECT i.vocabulary_id FROM word_audio_imports i JOIN vocabulary v ON v.id=i.vocabulary_id WHERE i.state='pending' ORDER BY v.updated_at DESC,v.id DESC LIMIT 16")?;
    Ok(query
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?)
}

pub fn finish_audio_import(
    connection: &Connection,
    id: i64,
    error: Option<&str>,
) -> Result<(), StorageError> {
    connection.execute("UPDATE word_audio_imports SET state=?1,error=?2 WHERE vocabulary_id=?3 AND (state<>?1 OR error IS NOT ?2)", params![if error.is_some() {"failed"}else{"finished"},error.map(|e|e.chars().take(2048).collect::<String>()),id])?;
    Ok(())
}
