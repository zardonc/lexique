use rusqlite::{Connection, TransactionBehavior, params};

use crate::*;

/// Installs a validated portable snapshot in an initialized, empty destination database.
pub fn import_learning_snapshot(
    connection: &mut Connection,
    snapshot: &LearningSnapshot,
    captured_at: i64,
) -> Result<(), StorageError> {
    validate_learning_snapshot(snapshot, captured_at)?;
    let schema: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if schema != lexique_core::DATABASE_SCHEMA_VERSION {
        return Err(StorageError::Invalid(
            "Snapshot import requires an initialized destination database".into(),
        ));
    }
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if tx
        .prepare(
            "SELECT 1 FROM vocabulary UNION ALL SELECT 1 FROM save_operations
             UNION ALL SELECT 1 FROM review_cards UNION ALL SELECT 1 FROM review_progress
             UNION ALL SELECT 1 FROM review_ratings UNION ALL SELECT 1 FROM review_queue_state
             UNION ALL SELECT 1 FROM review_queue_entries UNION ALL SELECT 1 FROM review_paused
             UNION ALL SELECT 1 FROM review_batches UNION ALL SELECT 1 FROM review_batch_items
             UNION ALL SELECT 1 FROM speech_assets UNION ALL SELECT 1 FROM speech_owners
             UNION ALL SELECT 1 FROM speech_tasks UNION ALL SELECT 1 FROM speech_versions
             UNION ALL SELECT 1 FROM word_audio_selections UNION ALL SELECT 1 FROM word_audio_imports
             UNION ALL SELECT 1 FROM legacy_speech UNION ALL SELECT 1 FROM word_audio_operations
             UNION ALL SELECT 1 FROM word_audio_publications UNION ALL SELECT 1 FROM word_audio_repairs
             UNION ALL SELECT 1 FROM app_metadata LIMIT 1",
        )?
        .exists([])?
    {
        return Err(StorageError::Invalid(
            "Snapshot import requires an empty destination database".into(),
        ));
    }
    insert_records(&tx, snapshot)?;
    insert_speech_demands(&tx, snapshot)?;
    insert_review(&tx, snapshot)?;
    insert_batches(&tx, snapshot)?;
    insert_preferences(&tx, &snapshot.preferences)?;
    write_dictionary_catalog(&tx, &snapshot.dictionary_catalog)?;
    if tx.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(StorageError::Invalid(
            "Imported snapshot failed its foreign-key check".into(),
        ));
    }
    let integrity: String = tx.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(StorageError::Invalid(format!(
            "Imported snapshot failed its integrity check: {integrity}"
        )));
    }
    for record in &snapshot.records {
        get(&tx, record.id)?;
    }
    tx.commit()?;
    Ok(())
}

fn insert_records(
    connection: &Connection,
    snapshot: &LearningSnapshot,
) -> Result<(), StorageError> {
    let mut statement = connection.prepare(
        "INSERT INTO vocabulary(id,headword,snapshot,notes,example_fr,example_zh,example_source,
         example_sense,more_examples,revision,created_at,updated_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
    )?;
    for record in &snapshot.records {
        statement.execute(params![
            record.id,
            record.headword,
            serde_json::to_string(&serde_json::json!({
                "reference": record.reference, "digest": record.digest
            }))?,
            record.notes,
            record.example_fr,
            record.example_zh,
            record
                .example_source
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
            record.example_sense,
            serde_json::to_string(&record.more_examples)?,
            record.revision,
            record.created_at,
            record.updated_at,
        ])?;
    }
    let mut statement = connection.prepare(
        "INSERT INTO save_operations(operation,request,outcome,record_id) VALUES(?1,?2,?3,?4)",
    )?;
    for saved in &snapshot.save_operations {
        statement.execute(params![
            saved.operation,
            serde_json::to_string(&saved.request)?,
            saved.outcome,
            saved.record_id,
        ])?;
    }
    Ok(())
}

fn insert_preferences(
    connection: &Connection,
    preferences: &SnapshotPreferences,
) -> Result<(), StorageError> {
    connection.execute(
        "UPDATE review_settings SET retention_percent=?1,new_limit=?2,review_limit=?3 WHERE id=1",
        params![
            preferences.review.retention_percent,
            preferences.review.new_limit,
            preferences.review.review_limit,
        ],
    )?;
    connection.execute(
        "INSERT INTO app_metadata(key,value) VALUES('new-cards-ignore-review-limit',?1)",
        [preferences.review.new_cards_ignore_review_limit.to_string()],
    )?;
    set_auto_enroll(connection, preferences.auto_enroll)?;
    set_speech_preferences(connection, &preferences.speech)?;
    set_interface_preferences(connection, &preferences.interface)?;
    connection.execute(
        "INSERT INTO app_metadata(key,value) VALUES('conjugation-preferences',?1)",
        [serde_json::to_string(&serde_json::json!({
            "mode": preferences.conjugation.mode
        }))?],
    )?;
    set_conjugation_block_order(connection, &preferences.conjugation.block_order)?;
    set_sort_preference(connection, &preferences.vocabulary_sort)?;
    Ok(())
}

fn insert_review(connection: &Connection, snapshot: &LearningSnapshot) -> Result<(), StorageError> {
    let mut statement =
        connection.prepare("INSERT INTO review_cards(id,vocabulary_id,kind) VALUES(?1,?2,?3)")?;
    for card in &snapshot.cards {
        statement.execute(params![card.id, card.vocabulary_id, card.kind])?;
    }
    let mut statement = connection
        .prepare("INSERT INTO review_progress(card_id,revision,state) VALUES(?1,?2,?3)")?;
    for progress in &snapshot.progress {
        statement.execute(params![
            progress.card_id,
            progress.revision,
            serde_json::to_string(&progress.card)?,
        ])?;
    }
    let mut statement = connection.prepare(
        "INSERT INTO review_ratings(operation,card_id,request,before_state,result,reviewed_at,
         learning_day,quota,queue_before,undo_operation,undo_queue_rebuilt)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
    )?;
    // The validated sequence is submission order, including ratings with equal timestamps.
    for rating in &snapshot.ratings {
        statement.execute(params![
            rating.operation,
            rating.card_id,
            serde_json::to_string(&rating.request)?,
            serde_json::to_string(&rating.before_state)?,
            serde_json::to_string(&serde_json::to_value(&rating.result)?)?,
            rating.reviewed_at,
            rating.learning_day,
            rating.quota,
            rating
                .queue_before
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
            rating.undo_operation,
            rating.undo_queue_rebuilt,
        ])?;
    }
    if let Some(context) = &snapshot.queue.context {
        connection.execute(
            "INSERT INTO review_queue_state(id,learning_day,timezone) VALUES(1,?1,?2)",
            params![context.learning_day, context.timezone],
        )?;
    }
    let mut statement =
        connection.prepare("INSERT INTO review_queue_entries(card_id,position) VALUES(?1,?2)")?;
    for entry in &snapshot.queue.entries {
        statement.execute(params![entry.card_id, entry.position])?;
    }
    let mut statement =
        connection.prepare("INSERT INTO review_paused(vocabulary_id) VALUES(?1)")?;
    for id in &snapshot.paused_vocabulary_ids {
        statement.execute([id])?;
    }
    Ok(())
}

fn insert_speech_demands(
    connection: &Connection,
    snapshot: &LearningSnapshot,
) -> Result<(), StorageError> {
    let mut statement = connection.prepare(
        "INSERT INTO speech_assets(semantic_key,spec,context,retain) VALUES(?1,?2,?3,?4)",
    )?;
    for source in &snapshot.speech_sources {
        statement.execute(params![
            source.semantic_key,
            serde_json::to_string(&source.spec)?,
            source.context,
            source.retain,
        ])?;
    }
    let mut statement = connection
        .prepare("INSERT INTO speech_owners(semantic_key,vocabulary_id) VALUES(?1,?2)")?;
    for owner in &snapshot.speech_owners {
        statement.execute(params![owner.semantic_key, owner.vocabulary_id])?;
    }
    let mut statement = connection.prepare(
        "INSERT INTO speech_tasks(id,semantic_key,priority,state,attempts,wake_at,deadline_at,
         cancel_requested,refresh_requested,successor_requested,interactive_requested,
         execution_generation,demand_class,requested_at,error)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
    )?;
    for task in &snapshot.speech_tasks {
        statement.execute(params![
            task.id,
            task.semantic_key,
            task.priority,
            task.state,
            task.attempts,
            task.wake_at,
            task.deadline_at,
            task.cancel_requested,
            task.refresh_requested,
            task.successor_requested,
            task.interactive_requested,
            task.execution_generation,
            task.demand_class,
            task.requested_at,
            task.error,
        ])?;
    }
    Ok(())
}

fn insert_batches(
    connection: &Connection,
    snapshot: &LearningSnapshot,
) -> Result<(), StorageError> {
    let mut batch_statement = connection.prepare(
        "INSERT INTO review_batches(operation,request,voice,created_at,cancelled) VALUES(?1,?2,?3,?4,?5)",
    )?;
    let mut item_statement = connection.prepare(
        "INSERT INTO review_batch_items(operation,ordinal,vocabulary_id,semantic_key,created_cards,error)
         VALUES(?1,?2,?3,?4,?5,?6)",
    )?;
    for batch in &snapshot.batches {
        batch_statement.execute(params![
            batch.operation,
            serde_json::to_string(&batch.requested_vocabulary_ids)?,
            serde_json::to_string(&batch.voice)?,
            batch.created_at,
            batch.cancelled,
        ])?;
        for item in &batch.items {
            item_statement.execute(params![
                batch.operation,
                item.ordinal,
                item.vocabulary_id,
                item.semantic_key,
                item.created_cards,
                item.error,
            ])?;
        }
    }
    Ok(())
}
