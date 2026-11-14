use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior};
use serde::de::DeserializeOwned;

use crate::*;

fn rows<T>(
    connection: &Connection,
    sql: &str,
    map: impl FnMut(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, StorageError> {
    Ok(connection
        .prepare(sql)?
        .query_map([], map)?
        .collect::<Result<_, _>>()?)
}

fn json<T: DeserializeOwned>(row: &Row<'_>, index: usize) -> rusqlite::Result<T> {
    serde_json::from_str(&row.get::<_, String>(index)?).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn saved_request(row: &Row<'_>) -> rusqlite::Result<SaveRequest> {
    let text: String = row.get(1)?;
    let mut value = crate::snapshot_validation::unique_json(text.as_bytes()).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error))
    })?;
    crate::vocabulary::normalize_save_request_fields(&mut value);
    serde_json::from_value(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error))
    })
}

/// Captures committed domain state without rebuilding the queue or touching audio files.
pub fn export_learning_snapshot(
    connection: &mut Connection,
    captured_at: i64,
) -> Result<LearningSnapshot, StorageError> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Deferred)?;
    let ids = rows(&tx, "SELECT id FROM vocabulary ORDER BY id", |r| {
        r.get::<_, i64>(0)
    })?;
    let mut records = get_batch(&tx, &ids)?;
    let mut snapshot = LearningSnapshot {
        snapshot_version: 1,
        records: ids
            .into_iter()
            .map(|id| records.remove(&id).unwrap())
            .collect(),
        save_operations: rows(
            &tx,
            "SELECT operation,request,outcome,record_id FROM save_operations ORDER BY rowid",
            |r| {
                Ok(SnapshotSaveOperation {
                    operation: r.get(0)?,
                    request: saved_request(r)?,
                    outcome: r.get(2)?,
                    record_id: r.get(3)?,
                })
            },
        )?,
        cards: rows(
            &tx,
            "SELECT id,vocabulary_id,kind FROM review_cards ORDER BY id",
            |r| {
                Ok(QueueCard {
                    id: r.get(0)?,
                    vocabulary_id: r.get(1)?,
                    kind: r.get(2)?,
                })
            },
        )?,
        progress: rows(
            &tx,
            "SELECT card_id,revision,state FROM review_progress ORDER BY card_id",
            |r| {
                Ok(SnapshotProgress {
                    card_id: r.get(0)?,
                    revision: r.get(1)?,
                    card: json(r, 2)?,
                })
            },
        )?,
        ratings: rows(
            &tx,
            "SELECT operation,card_id,request,before_state,result,reviewed_at,learning_day,quota,queue_before,undo_operation,undo_queue_rebuilt FROM review_ratings ORDER BY rowid",
            |r| {
                Ok(SnapshotRating {
                    sequence: 0,
                    operation: r.get(0)?,
                    card_id: r.get(1)?,
                    request: json(r, 2)?,
                    before_state: json(r, 3)?,
                    result: json(r, 4)?,
                    reviewed_at: r.get(5)?,
                    learning_day: r.get(6)?,
                    quota: r.get(7)?,
                    queue_before: r
                        .get::<_, Option<String>>(8)?
                        .map(|text| serde_json::from_str(&text))
                        .transpose()
                        .map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                8,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                    undo_operation: r.get(9)?,
                    undo_queue_rebuilt: r.get(10)?,
                })
            },
        )?,
        queue: SnapshotQueue {
            context: tx
                .query_row(
                    "SELECT learning_day,timezone FROM review_queue_state WHERE id=1",
                    [],
                    |r| {
                        Ok(SnapshotQueueContext {
                            learning_day: r.get(0)?,
                            timezone: r.get(1)?,
                        })
                    },
                )
                .optional()?,
            entries: rows(
                &tx,
                "SELECT card_id,position FROM review_queue_entries ORDER BY position",
                |r| {
                    Ok(SnapshotQueueEntry {
                        card_id: r.get(0)?,
                        position: r.get(1)?,
                    })
                },
            )?,
        },
        paused_vocabulary_ids: rows(
            &tx,
            "SELECT vocabulary_id FROM review_paused ORDER BY vocabulary_id",
            |r| r.get(0),
        )?,
        batches: batches(&tx)?,
        speech_sources: rows(
            &tx,
            "SELECT semantic_key,spec,context,current_version,pending_version,retain FROM speech_assets ORDER BY semantic_key",
            |r| {
                Ok(SnapshotSpeechSource {
                    semantic_key: r.get(0)?,
                    spec: json(r, 1)?,
                    context: r.get(2)?,
                    current_version: r.get(3)?,
                    pending_version: r.get(4)?,
                    retain: r.get(5)?,
                })
            },
        )?,
        speech_owners: rows(
            &tx,
            "SELECT semantic_key,vocabulary_id FROM speech_owners ORDER BY semantic_key,vocabulary_id",
            |r| {
                Ok(SnapshotSpeechOwner {
                    semantic_key: r.get(0)?,
                    vocabulary_id: r.get(1)?,
                })
            },
        )?,
        speech_tasks: rows(
            &tx,
            "SELECT id,semantic_key,priority,state,attempts,wake_at,deadline_at,cancel_requested,refresh_requested,successor_requested,interactive_requested,execution_generation,demand_class,requested_at,error FROM speech_tasks ORDER BY id",
            |r| {
                Ok(SnapshotSpeechTask {
                    id: r.get(0)?,
                    semantic_key: r.get(1)?,
                    priority: r.get(2)?,
                    state: r.get(3)?,
                    attempts: r.get(4)?,
                    wake_at: r.get(5)?,
                    deadline_at: r.get(6)?,
                    cancel_requested: r.get(7)?,
                    refresh_requested: r.get(8)?,
                    successor_requested: r.get(9)?,
                    interactive_requested: r.get(10)?,
                    execution_generation: r.get(11)?,
                    demand_class: r.get(12)?,
                    requested_at: r.get(13)?,
                    error: r.get(14)?,
                })
            },
        )?,
        audio_versions: rows(
            &tx,
            "SELECT locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,execution_generation,context,verification,gc_state,expires_at,orphan_until FROM speech_versions ORDER BY locator",
            |r| {
                Ok(SnapshotAudioVersion {
                    locator: r.get(0)?,
                    semantic_key: r.get(1)?,
                    content_hash: r.get(2)?,
                    byte_length: r.get(3)?,
                    generated_at: r.get(4)?,
                    created_at: r.get(5)?,
                    origin_vocabulary_id: r.get(6)?,
                    expected_revision: r.get(7)?,
                    generation: r.get(8)?,
                    sealed: r.get(9)?,
                    execution_generation: r.get(10)?,
                    context: r.get(11)?,
                    verification: r.get(12)?,
                    gc_state: r.get(13)?,
                    expires_at: r.get(14)?,
                    orphan_until: r.get(15)?,
                    payload_status: AudioPayloadStatus::Excluded,
                    exclusion_reason: Some(AudioExclusionReason::NotSelected),
                })
            },
        )?,
        audio_selections: rows(
            &tx,
            "SELECT vocabulary_id,current_version,previous_version,previous_until,revision,desired_generation,desired_key,current_generation FROM word_audio_selections ORDER BY vocabulary_id",
            |r| {
                Ok(SnapshotAudioSelection {
                    vocabulary_id: r.get(0)?,
                    current_version: r.get(1)?,
                    previous_version: r.get(2)?,
                    previous_until: r.get(3)?,
                    revision: r.get(4)?,
                    desired_generation: r.get(5)?,
                    desired_key: r.get(6)?,
                    current_generation: r.get(7)?,
                })
            },
        )?,
        audio_imports: rows(
            &tx,
            "SELECT vocabulary_id,state,error FROM word_audio_imports ORDER BY vocabulary_id",
            |r| {
                Ok(SnapshotAudioImport {
                    vocabulary_id: r.get(0)?,
                    state: r.get(1)?,
                    error: r.get(2)?,
                })
            },
        )?,
        legacy_audio: rows(
            &tx,
            "SELECT vocabulary_id,content_hash FROM legacy_speech ORDER BY vocabulary_id",
            |r| {
                Ok(SnapshotLegacyAudio {
                    vocabulary_id: r.get(0)?,
                    content_hash: r.get(1)?,
                    payload_status: AudioPayloadStatus::Excluded,
                    exclusion_reason: Some(AudioExclusionReason::NotSelected),
                })
            },
        )?,
        audio_operations: rows(
            &tx,
            "SELECT operation,vocabulary_id,semantic_key,generation,refresh FROM word_audio_operations ORDER BY rowid",
            |r| {
                Ok(SnapshotAudioOperation {
                    operation: r.get(0)?,
                    vocabulary_id: r.get(1)?,
                    semantic_key: r.get(2)?,
                    generation: r.get(3)?,
                    refresh: r.get(4)?,
                })
            },
        )?,
        audio_publications: rows(
            &tx,
            "SELECT locator,vocabulary_id,expected_revision,generation,execution_generation,error FROM word_audio_publications ORDER BY locator",
            |r| {
                Ok(SnapshotAudioPublication {
                    locator: r.get(0)?,
                    vocabulary_id: r.get(1)?,
                    expected_revision: r.get(2)?,
                    generation: r.get(3)?,
                    execution_generation: r.get(4)?,
                    error: r.get(5)?,
                })
            },
        )?,
        audio_repairs: rows(
            &tx,
            "SELECT vocabulary_id,failure_identity,semantic_key,generation,error FROM word_audio_repairs ORDER BY vocabulary_id,failure_identity",
            |r| {
                Ok(SnapshotAudioRepair {
                    vocabulary_id: r.get(0)?,
                    failure_identity: r.get(1)?,
                    semantic_key: r.get(2)?,
                    generation: r.get(3)?,
                    error: r.get(4)?,
                })
            },
        )?,
        preferences: SnapshotPreferences {
            review: review_settings(&tx)?,
            auto_enroll: auto_enroll(&tx)?,
            speech: speech_preferences(&tx)?,
            interface: interface_preferences(&tx)?,
            conjugation: SnapshotConjugationPreferences {
                mode: conjugation_preferences(&tx)?.mode,
                block_order: conjugation_block_order(&tx)?,
            },
            vocabulary_sort: sort_preference(&tx)?,
        },
        dictionary_catalog: read_dictionary_catalog(&tx)?,
    };
    for (index, rating) in snapshot.ratings.iter_mut().enumerate() {
        rating.sequence = index as u64 + 1;
    }
    retain_confirmed_speech(&mut snapshot);
    classify_snapshot_audio(&mut snapshot, captured_at);
    tx.commit()?;
    Ok(snapshot)
}

fn batches(connection: &Connection) -> Result<Vec<SnapshotBatch>, StorageError> {
    let mut result = rows(
        connection,
        "SELECT operation,request,voice,created_at,cancelled FROM review_batches ORDER BY rowid",
        |r| {
            Ok(SnapshotBatch {
                sequence: 0,
                operation: r.get(0)?,
                requested_vocabulary_ids: json(r, 1)?,
                voice: json(r, 2)?,
                created_at: r.get(3)?,
                cancelled: r.get(4)?,
                items: Vec::new(),
            })
        },
    )?;
    for (index, batch) in result.iter_mut().enumerate() {
        batch.sequence = index as u64 + 1;
        batch.items = connection.prepare("SELECT ordinal,vocabulary_id,semantic_key,created_cards,error FROM review_batch_items WHERE operation=?1 ORDER BY ordinal")?
            .query_map([&batch.operation], |r| Ok(SnapshotBatchItem { ordinal: r.get(0)?, vocabulary_id: r.get(1)?, semantic_key: r.get(2)?, created_cards: r.get(3)?, error: r.get(4)? }))?
            .collect::<Result<_, _>>()?;
    }
    Ok(result)
}

fn retain_confirmed_speech(snapshot: &mut LearningSnapshot) {
    let mut keys: HashSet<String> = snapshot
        .speech_owners
        .iter()
        .map(|owner| owner.semantic_key.clone())
        .collect();
    keys.extend(
        snapshot
            .batches
            .iter()
            .flat_map(|batch| batch.items.iter())
            .filter_map(|item| item.semantic_key.clone()),
    );
    keys.extend(
        snapshot
            .audio_operations
            .iter()
            .map(|operation| operation.semantic_key.clone()),
    );
    keys.extend(
        snapshot
            .audio_repairs
            .iter()
            .filter_map(|repair| repair.semantic_key.clone()),
    );
    keys.extend(
        snapshot
            .audio_selections
            .iter()
            .filter_map(|selection| selection.desired_key.clone()),
    );
    let mut locators: HashSet<String> = snapshot
        .audio_selections
        .iter()
        .flat_map(|selection| [&selection.current_version, &selection.previous_version])
        .filter_map(|locator| locator.clone())
        .collect();
    locators.extend(
        snapshot
            .audio_publications
            .iter()
            .map(|publication| publication.locator.clone()),
    );
    for version in &snapshot.audio_versions {
        if locators.contains(&version.locator) {
            keys.extend(version.semantic_key.clone());
        }
    }
    snapshot
        .speech_sources
        .retain(|source| keys.contains(&source.semantic_key));
    for source in &snapshot.speech_sources {
        locators.extend(source.current_version.clone());
        locators.extend(source.pending_version.clone());
    }
    snapshot
        .speech_tasks
        .retain(|task| keys.contains(&task.semantic_key));
    snapshot.audio_versions.retain(|version| {
        locators.contains(&version.locator)
            || version
                .semantic_key
                .as_ref()
                .is_some_and(|key| keys.contains(key))
    });
}

pub fn initial_snapshot_audio_allowed(snapshot: &LearningSnapshot, vocabulary_id: i64) -> bool {
    snapshot
        .audio_selections
        .iter()
        .find(|selection| selection.vocabulary_id == vocabulary_id)
        .is_none_or(|selection| {
            selection.current_version.is_none() && selection.desired_generation == 0
        })
        && !snapshot
            .audio_publications
            .iter()
            .any(|publication| publication.vocabulary_id == vocabulary_id)
}

fn classify_snapshot_audio(snapshot: &mut LearningSnapshot, captured_at: i64) {
    let pending: HashSet<_> = snapshot
        .speech_sources
        .iter()
        .filter_map(|source| source.pending_version.clone())
        .collect();
    let mut included: HashSet<String> = snapshot
        .audio_selections
        .iter()
        .filter_map(|selection| selection.current_version.clone())
        .collect();
    included.extend(
        snapshot
            .audio_selections
            .iter()
            .filter(|selection| {
                selection
                    .previous_until
                    .is_some_and(|until| until > captured_at)
            })
            .filter_map(|selection| selection.previous_version.clone()),
    );
    let expired: HashSet<_> = snapshot
        .audio_selections
        .iter()
        .filter(|selection| {
            selection
                .previous_until
                .is_some_and(|until| until <= captured_at)
        })
        .filter_map(|selection| selection.previous_version.clone())
        .collect();
    for version in &mut snapshot.audio_versions {
        if included.contains(&version.locator)
            && version.context == "word"
            && version.gc_state == "live"
        {
            version.payload_status = match version.verification.as_str() {
                "missing" => AudioPayloadStatus::SourceMissing,
                "invalid" | "suspect" => AudioPayloadStatus::SourceInvalid,
                _ => AudioPayloadStatus::Included,
            };
            version.exclusion_reason = None;
        } else {
            version.exclusion_reason = Some(
                if expired.contains(&version.locator) || version.gc_state != "live" {
                    AudioExclusionReason::Expired
                } else if !version.sealed
                    || pending.contains(&version.locator)
                    || snapshot
                        .audio_publications
                        .iter()
                        .any(|publication| publication.locator == version.locator)
                {
                    AudioExclusionReason::Unpublished
                } else {
                    AudioExclusionReason::NotSelected
                },
            );
        }
    }
}
