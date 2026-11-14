use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::Read,
};

use lexique_core::scheduling::{learning_day_for, validate_card, validate_learning_day};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};
use unicode_normalization::UnicodeNormalization;

use crate::*;

fn require(condition: bool, message: &str) -> Result<(), StorageError> {
    if condition {
        Ok(())
    } else {
        Err(StorageError::Invalid(format!(
            "Invalid learning snapshot: {message}"
        )))
    }
}

fn integer(value: i64, minimum: i64) -> Result<(), StorageError> {
    require(
        (minimum..=9_007_199_254_740_991).contains(&value),
        "integer is outside its domain or JavaScript safe range",
    )
}

fn timestamp(value: i64) -> Result<(), StorageError> {
    require(
        (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value),
        "timestamp is outside the JavaScript safe range",
    )?;
    learning_day_for(value, "UTC")
        .map(|_| ())
        .map_err(StorageError::Invalid)
}

fn optional_timestamp(value: Option<i64>) -> Result<(), StorageError> {
    value.map(timestamp).transpose().map(|_| ())
}

fn hash(value: &str) -> Result<(), StorageError> {
    require(
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "invalid content identity",
    )
}

fn operation(value: &str) -> Result<(), StorageError> {
    require(
        !value.is_empty() && value.len() <= 128,
        "invalid operation identity",
    )
}

fn audio_operation(value: &str) -> Result<(), StorageError> {
    operation(value)?;
    require(
        !value.chars().any(char::is_control),
        "invalid audio operation identity",
    )
}

fn reference(value: &Reference) -> Result<(), StorageError> {
    require(
        !value.dictionary.trim().is_empty()
            && !value.version.trim().is_empty()
            && !value.lookup_key.trim().is_empty()
            && value.lookup_key.len() <= 512
            && !value.lookup_key.chars().any(char::is_control),
        "invalid frozen reference",
    )?;
    if let Some(id) = &value.dictionary_id {
        require(
            !id.is_empty() && id.len() <= 64,
            "invalid frozen dictionary identity",
        )?;
    }
    hash(&value.sha256)
}

fn known(value: i64, identities: &HashSet<i64>) -> Result<(), StorageError> {
    integer(value, 1)?;
    require(identities.contains(&value), "dangling learning identity")
}

fn payload(
    status: AudioPayloadStatus,
    reason: Option<AudioExclusionReason>,
) -> Result<(), StorageError> {
    require(
        (status == AudioPayloadStatus::Excluded) == reason.is_some(),
        "audio exclusion reason disagrees with payload status",
    )
}

pub fn validate_learning_snapshot(
    snapshot: &LearningSnapshot,
    captured_at: i64,
) -> Result<(), StorageError> {
    require(
        snapshot.snapshot_version == 1,
        "unsupported snapshot version",
    )?;
    timestamp(captured_at)?;
    validate_dictionary_catalog(&snapshot.dictionary_catalog)?;
    let mut words = HashSet::new();
    let mut headwords = HashSet::new();
    for record in &snapshot.records {
        integer(record.id, 1)?;
        integer(record.revision, 1)?;
        integer(record.created_at, 0)?;
        integer(record.updated_at, 0)?;
        require(
            words.insert(record.id) && headwords.insert(record.headword.nfc().collect::<String>()),
            "duplicate vocabulary identity",
        )?;
        validate(&SaveRequest {
            operation: "snapshot-record".into(),
            headword: record.headword.clone(),
            notes: record.notes.clone(),
            example_fr: record.example_fr.clone(),
            example_zh: record.example_zh.clone(),
            example_source: record.example_source.clone(),
            example_sense: record.example_sense,
            more_examples: record.more_examples.clone(),
            id: Some(record.id),
            revision: Some(record.revision),
        })?;
        reference(&record.reference)?;
        for source in record.example_source.iter().chain(
            record
                .more_examples
                .iter()
                .filter_map(|example| example.source.as_ref()),
        ) {
            reference(source)?;
        }
    }
    let mut saves = HashSet::new();
    for saved in &snapshot.save_operations {
        validate(&saved.request)?;
        known(saved.record_id, &words)?;
        require(
            saved.operation == saved.request.operation && saves.insert(&saved.operation),
            "duplicate or inconsistent save operation",
        )?;
        require(
            matches!(saved.outcome.as_str(), "created" | "updated" | "existing"),
            "invalid saved outcome",
        )?;
        require(
            saved.request.id.is_none_or(|id| id == saved.record_id),
            "save operation points to another record",
        )?;
        if let Some(revision) = saved.request.revision {
            integer(revision, 1)?;
        }
        let record = snapshot
            .records
            .iter()
            .find(|record| record.id == saved.record_id)
            .unwrap();
        require(
            saved.request.headword.nfc().collect::<String>()
                == record.headword.nfc().collect::<String>(),
            "save request headword disagrees with its record",
        )?;
        for source in saved.request.example_source.iter().chain(
            saved
                .request
                .more_examples
                .iter()
                .filter_map(|example| example.source.as_ref()),
        ) {
            reference(source)?;
        }
    }
    let mut cards = HashSet::new();
    let mut kinds = HashSet::new();
    for card in &snapshot.cards {
        integer(card.id, 1)?;
        known(card.vocabulary_id, &words)?;
        require(
            matches!(card.kind.as_str(), "recognition" | "listening")
                && cards.insert(card.id)
                && kinds.insert((card.vocabulary_id, &card.kind)),
            "duplicate or invalid card",
        )?;
    }
    let mut progress = HashSet::new();
    for state in &snapshot.progress {
        known(state.card_id, &cards)?;
        integer(state.revision, 0)?;
        require(progress.insert(state.card_id), "duplicate progress")?;
        validate_card(&state.card).map_err(StorageError::Invalid)?;
    }
    validate_ratings(snapshot, &cards)?;
    if let Some(context) = &snapshot.queue.context {
        queue_context(&context.learning_day, &context.timezone)?;
    }
    let mut queue_cards = HashSet::new();
    let mut positions = HashSet::new();
    for entry in &snapshot.queue.entries {
        known(entry.card_id, &cards)?;
        integer(entry.position, 0)?;
        require(
            queue_cards.insert(entry.card_id) && positions.insert(entry.position),
            "duplicate queue entry or position",
        )?;
    }
    let mut paused = HashSet::new();
    for id in &snapshot.paused_vocabulary_ids {
        known(*id, &words)?;
        require(paused.insert(id), "duplicate pause")?;
    }
    validate_speech(snapshot, &words, captured_at)?;
    validate_batches(snapshot, &words)?;
    let review = &snapshot.preferences.review;
    require(
        (70..=99).contains(&review.retention_percent)
            && review.new_limit <= 1_000_000
            && review.review_limit <= 1_000_000,
        "invalid review preference",
    )?;
    require(
        (80..=240).contains(&snapshot.preferences.speech.wpm)
            && snapshot.preferences.speech.wpm.is_multiple_of(20),
        "invalid speech preference",
    )?;
    validate_sort(&snapshot.preferences.vocabulary_sort)?;
    Ok(())
}

fn queue_context(day: &str, timezone: &str) -> Result<(), StorageError> {
    validate_learning_day(day).map_err(StorageError::Invalid)?;
    learning_day_for(0, timezone)
        .map(|_| ())
        .map_err(StorageError::Invalid)
}

fn validate_ratings(snapshot: &LearningSnapshot, cards: &HashSet<i64>) -> Result<(), StorageError> {
    let mut operations = HashSet::new();
    let mut undo = HashSet::new();
    for (index, rating) in snapshot.ratings.iter().enumerate() {
        require(
            rating.sequence == index as u64 + 1,
            "rating sequence must preserve contiguous submission order",
        )?;
        operation(&rating.operation)?;
        known(rating.card_id, cards)?;
        integer(rating.request.expected_revision, 0)?;
        require(
            operations.insert(&rating.operation)
                && rating.request.operation == rating.operation
                && rating.request.card_id == rating.card_id
                && rating.request.now == rating.reviewed_at,
            "inconsistent rating identity",
        )?;
        timestamp(rating.reviewed_at)?;
        queue_context(&rating.learning_day, &rating.request.timezone)?;
        validate_card(&rating.before_state).map_err(StorageError::Invalid)?;
        validate_card(&rating.result.card).map_err(StorageError::Invalid)?;
        require(
            matches!(rating.quota.as_str(), "new" | "review"),
            "invalid quota kind",
        )?;
        let result = &rating.result;
        require(
            result.card.last_review_at == Some(rating.reviewed_at)
                && (70..=99).contains(&result.retention_percent)
                && !result.model.version.is_empty()
                && !result.timezone_database.is_empty()
                && !result.model.parameters.is_empty()
                && result
                    .model
                    .parameters
                    .iter()
                    .all(|value| value.is_finite()),
            "invalid recorded scheduling result",
        )?;
        require(
            matches!(
                (result.step_seconds, result.interval_days),
                (Some(1..), None) | (None, Some(1..=36_500))
            ),
            "invalid recorded interval",
        )?;
        if let Some(before) = &rating.queue_before {
            require(
                before.card_id == rating.card_id,
                "rating queue belongs to another card",
            )?;
            if let Some(position) = before.position {
                integer(position, 0)?;
            }
            if let Some(ids) = &before.ordered_cards {
                let mut seen = HashSet::new();
                for id in ids {
                    known(*id, cards)?;
                    require(seen.insert(id), "duplicate prior queue card")?;
                }
            }
            if let Some(day) = &before.learning_day {
                validate_learning_day(day).map_err(StorageError::Invalid)?;
            }
            if let Some(zone) = &before.timezone {
                learning_day_for(0, zone).map_err(StorageError::Invalid)?;
            }
        }
        if let Some(id) = &rating.undo_operation {
            operation(id)?;
            require(undo.insert(id), "duplicate undo identity")?;
        }
        require(
            !rating.undo_queue_rebuilt || rating.undo_operation.is_some(),
            "queue undo flag without an undo",
        )?;
    }
    Ok(())
}

fn validate_batches(snapshot: &LearningSnapshot, words: &HashSet<i64>) -> Result<(), StorageError> {
    let sources: HashMap<_, _> = snapshot
        .speech_sources
        .iter()
        .map(|source| (&source.semantic_key, source))
        .collect();
    let mut operations = HashSet::new();
    for (index, batch) in snapshot.batches.iter().enumerate() {
        require(
            batch.sequence == index as u64 + 1
                && !batch.operation.is_empty()
                && batch.operation.len() <= 128
                && operations.insert(&batch.operation),
            "invalid batch submission sequence or identity",
        )?;
        timestamp(batch.created_at)?;
        require(
            !batch.requested_vocabulary_ids.is_empty()
                && batch.requested_vocabulary_ids.len() <= 10_000,
            "invalid batch request size",
        )?;
        let mut seen = HashSet::new();
        let mut requested = Vec::new();
        for id in &batch.requested_vocabulary_ids {
            integer(*id, 1)?;
            if seen.insert(*id) {
                requested.push(*id);
            }
        }
        let mut previous = None;
        for item in &batch.items {
            integer(item.ordinal, 0)?;
            require(
                previous.is_none_or(|ordinal| item.ordinal > ordinal)
                    && requested.get(item.ordinal as usize) == Some(&item.vocabulary_id),
                "batch item disagrees with deduplicated request order",
            )?;
            previous = Some(item.ordinal);
            require(item.created_cards <= 2, "invalid batch card count")?;
            if item.error.is_none() {
                known(item.vocabulary_id, words)?;
                let source = item
                    .semantic_key
                    .as_ref()
                    .and_then(|key| sources.get(key))
                    .ok_or_else(|| {
                        StorageError::Invalid("Successful batch item has no speech source".into())
                    })?;
                let record = snapshot
                    .records
                    .iter()
                    .find(|record| record.id == item.vocabulary_id)
                    .unwrap();
                let voice = match batch.voice {
                    SpeechVoice::Female => "female",
                    SpeechVoice::Male => "male",
                };
                // Successful history retains its original source even after current ownership is detached.
                require(
                    source.context == "word"
                        && source.spec.text == record.headword
                        && source.spec.voice == voice,
                    "batch speech relationship disagrees with captured voice or word",
                )?;
            } else {
                require(
                    item.semantic_key.is_none() && item.created_cards == 0,
                    "failed batch item claims successful work",
                )?;
            }
        }
    }
    Ok(())
}

fn validate_speech(
    snapshot: &LearningSnapshot,
    words: &HashSet<i64>,
    captured_at: i64,
) -> Result<(), StorageError> {
    let mut sources = HashMap::new();
    for source in &snapshot.speech_sources {
        require(
            sources.insert(&source.semantic_key, source).is_none()
                && source.spec.key()? == source.semantic_key
                && source.context == source.spec.context
                && matches!(source.context.as_str(), "word" | "sentence"),
            "invalid speech source identity",
        )?;
    }
    let mut owners = HashSet::new();
    for owner in &snapshot.speech_owners {
        known(owner.vocabulary_id, words)?;
        require(
            owners.insert((&owner.semantic_key, owner.vocabulary_id))
                && sources
                    .get(&owner.semantic_key)
                    .is_some_and(|source| source.context == "word"),
            "invalid speech owner",
        )?;
    }
    let mut task_ids = HashSet::new();
    let mut task_keys = HashSet::new();
    for task in &snapshot.speech_tasks {
        integer(task.id, 1)?;
        integer(task.execution_generation, 0)?;
        integer(task.requested_at, 0)?;
        timestamp(task.wake_at)?;
        optional_timestamp(task.deadline_at)?;
        require(
            task_ids.insert(task.id)
                && task_keys.insert(&task.semantic_key)
                && sources.contains_key(&task.semantic_key)
                && task.priority <= 1
                && task.attempts <= 4
                && (1..=3).contains(&task.demand_class)
                && matches!(
                    task.state.as_str(),
                    "queued" | "running" | "retry_wait" | "ready" | "failed" | "cancelled"
                ),
            "invalid speech task",
        )?;
    }
    let mut versions = HashMap::new();
    for version in &snapshot.audio_versions {
        hash(&version.locator)?;
        hash(&version.content_hash)?;
        require(
            versions.insert(&version.locator, version).is_none(),
            "duplicate audio version",
        )?;
        if let Some(key) = &version.semantic_key {
            require(
                sources
                    .get(key)
                    .is_some_and(|source| source.context == version.context),
                "audio version has no source with a matching context",
            )?;
        }
        if let Some(id) = version.origin_vocabulary_id {
            integer(id, 1)?;
        }
        for value in [
            version.expected_revision,
            version.generation,
            version.execution_generation.unwrap_or(0),
        ] {
            integer(value, 0)?;
        }
        timestamp(version.created_at)?;
        optional_timestamp(version.generated_at)?;
        optional_timestamp(version.expires_at)?;
        optional_timestamp(version.orphan_until)?;
        require(
            version
                .byte_length
                .is_none_or(|length| (4..=10_000_000).contains(&length))
                && matches!(version.context.as_str(), "word" | "sentence")
                && matches!(
                    version.verification.as_str(),
                    "unverified" | "verified" | "suspect" | "missing" | "invalid"
                )
                && matches!(version.gc_state.as_str(), "live" | "marked" | "deleted"),
            "invalid audio version fields",
        )?;
        payload(version.payload_status, version.exclusion_reason)?;
        require(
            version.payload_status != AudioPayloadStatus::Included
                || (version.sealed
                    && version.context == "word"
                    && version.gc_state == "live"
                    && version.byte_length.is_some()
                    && matches!(version.verification.as_str(), "unverified" | "verified")),
            "included audio is not sealed live word content with a known length and valid verification",
        )?;
        require(
            version.payload_status != AudioPayloadStatus::SourceMissing
                || version.verification == "missing",
            "missing audio disagrees with its verification",
        )?;
        require(
            version.payload_status != AudioPayloadStatus::SourceInvalid
                || matches!(version.verification.as_str(), "invalid" | "suspect"),
            "invalid audio disagrees with its verification",
        )?;
    }
    for source in &snapshot.speech_sources {
        for locator in source
            .current_version
            .iter()
            .chain(source.pending_version.iter())
        {
            require(
                versions.get(locator).is_some_and(|version| {
                    version.semantic_key.as_ref() == Some(&source.semantic_key)
                        && version.context == source.context
                }),
                "speech pointer has no matching version or context",
            )?;
        }
        require(
            source.current_version.is_none() || source.current_version != source.pending_version,
            "speech current and pending pointers are identical",
        )?;
        if let Some(locator) = &source.current_version {
            require(
                versions[locator].sealed,
                "current speech source is not sealed",
            )?;
        }
        // Verification seals in a separate transaction before publication completes. A sealed
        // pending instance is a valid persisted interruption; its bytes remain unpublished.
        if let Some(locator) = &source.pending_version {
            require(
                versions[locator].payload_status == AudioPayloadStatus::Excluded,
                "pending speech content claims published bytes",
            )?;
        }
    }
    let mut selected_words = HashSet::new();
    let mut protected = HashSet::new();
    for selection in &snapshot.audio_selections {
        known(selection.vocabulary_id, words)?;
        for value in [
            selection.revision,
            selection.desired_generation,
            selection.current_generation,
        ] {
            integer(value, 0)?;
        }
        require(
            selected_words.insert(selection.vocabulary_id)
                && selection.current_generation <= selection.desired_generation
                && selection.previous_version.is_some() == selection.previous_until.is_some()
                && (selection.current_version.is_none()
                    || selection.current_version != selection.previous_version),
            "invalid selected audio relationship",
        )?;
        optional_timestamp(selection.previous_until)?;
        if let Some(key) = &selection.desired_key {
            let record = snapshot
                .records
                .iter()
                .find(|record| record.id == selection.vocabulary_id)
                .unwrap();
            require(
                sources.get(key).is_some_and(|source| {
                    source.context == "word" && source.spec.text == record.headword
                }),
                "desired audio has no matching word source",
            )?;
        }
        if let Some(locator) = &selection.current_version {
            let version = versions
                .get(locator)
                .ok_or_else(|| StorageError::Invalid("Selected audio version is missing".into()))?;
            require(
                version.sealed
                    && version.context == "word"
                    && version.gc_state == "live"
                    && version.payload_status != AudioPayloadStatus::Excluded,
                "selected audio is retired or excluded",
            )?;
            if let Some(key) = &version.semantic_key {
                let record = snapshot
                    .records
                    .iter()
                    .find(|record| record.id == selection.vocabulary_id)
                    .unwrap();
                require(
                    sources[key].spec.text == record.headword,
                    "selected audio specification belongs to another word",
                )?;
            }
            protected.insert(locator);
        }
        if let Some(locator) = &selection.previous_version {
            let version = versions
                .get(locator)
                .ok_or_else(|| StorageError::Invalid("Previous audio version is missing".into()))?;
            if selection.previous_until.unwrap() > captured_at {
                require(
                    version.sealed
                        && version.context == "word"
                        && version.gc_state == "live"
                        && version.payload_status != AudioPayloadStatus::Excluded,
                    "protected previous audio is retired or excluded",
                )?;
                protected.insert(locator);
            }
        }
    }
    for owner in &snapshot.speech_owners {
        if initial_snapshot_audio_allowed(snapshot, owner.vocabulary_id)
            && let Some(locator) = &sources[&owner.semantic_key].current_version
        {
            protected.insert(locator);
        }
    }
    for version in &snapshot.audio_versions {
        require(
            version.payload_status == AudioPayloadStatus::Excluded
                || protected.contains(&version.locator),
            "audio bytes are outside selected or initial owned content",
        )?;
    }
    let mut imports = HashSet::new();
    for import in &snapshot.audio_imports {
        known(import.vocabulary_id, words)?;
        require(
            imports.insert(import.vocabulary_id)
                && matches!(import.state.as_str(), "pending" | "finished" | "failed"),
            "invalid legacy import",
        )?;
    }
    let mut legacy_words = HashSet::new();
    for legacy in &snapshot.legacy_audio {
        known(legacy.vocabulary_id, words)?;
        hash(&legacy.content_hash)?;
        payload(legacy.payload_status, legacy.exclusion_reason)?;
        require(
            legacy_words.insert(legacy.vocabulary_id)
                && (legacy.payload_status == AudioPayloadStatus::Excluded
                    || initial_snapshot_audio_allowed(snapshot, legacy.vocabulary_id)),
            "invalid legacy current relationship",
        )?;
    }
    for record in &snapshot.records {
        if !initial_snapshot_audio_allowed(snapshot, record.id) {
            continue;
        }
        let candidates: HashSet<_> = snapshot
            .speech_owners
            .iter()
            .filter(|owner| owner.vocabulary_id == record.id)
            .filter_map(|owner| sources[&owner.semantic_key].current_version.as_ref())
            .filter(|locator| versions[*locator].payload_status != AudioPayloadStatus::Excluded)
            .collect();
        let legacy = usize::from(snapshot.legacy_audio.iter().any(|audio| {
            audio.vocabulary_id == record.id && audio.payload_status != AudioPayloadStatus::Excluded
        }));
        require(
            candidates.len() + legacy <= 1,
            "initial audio has more than one playback winner",
        )?;
    }
    let mut operations = HashSet::new();
    for change in &snapshot.audio_operations {
        audio_operation(&change.operation)?;
        known(change.vocabulary_id, words)?;
        integer(change.generation, 1)?;
        require(
            operations.insert(&change.operation) && sources.contains_key(&change.semantic_key),
            "invalid audio operation",
        )?;
    }
    let mut publications = HashSet::new();
    for publication in &snapshot.audio_publications {
        known(publication.vocabulary_id, words)?;
        for value in [
            publication.expected_revision,
            publication.generation,
            publication.execution_generation.unwrap_or(0),
        ] {
            integer(value, 0)?;
        }
        let version = versions
            .get(&publication.locator)
            .ok_or_else(|| StorageError::Invalid("Publication version is missing".into()))?;
        require(
            publications.insert(&publication.locator)
                && version.origin_vocabulary_id == Some(publication.vocabulary_id)
                && version.context == "word"
                && version.payload_status == AudioPayloadStatus::Excluded
                && version.exclusion_reason == Some(AudioExclusionReason::Unpublished)
                && version.expected_revision == publication.expected_revision
                && version.generation == publication.generation
                && version.execution_generation == publication.execution_generation,
            "invalid unfinished audio publication",
        )?;
        if let Some(key) = &version.semantic_key {
            let record = snapshot
                .records
                .iter()
                .find(|record| record.id == publication.vocabulary_id)
                .unwrap();
            require(
                sources[key].spec.text == record.headword,
                "publication has no matching word source",
            )?;
        }
    }
    let mut repairs = HashSet::new();
    for repair in &snapshot.audio_repairs {
        known(repair.vocabulary_id, words)?;
        require(
            !repair.failure_identity.is_empty()
                && repairs.insert((repair.vocabulary_id, &repair.failure_identity))
                && repair
                    .semantic_key
                    .as_ref()
                    .is_none_or(|key| sources.contains_key(key)),
            "invalid repair identity",
        )?;
        if let Some(generation) = repair.generation {
            integer(generation, 0)?;
        }
    }
    Ok(())
}

struct UniqueJson(Value);

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = UniqueJson;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with unique object fields")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Number::from_f64(value)
                    .map(|number| UniqueJson(number.into()))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(UniqueJson(value.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element::<UniqueJson>()? {
                    values.push(value.0);
                }
                Ok(UniqueJson(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut fields: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some((key, value)) = fields.next_entry::<String, UniqueJson>()? {
                    if values.insert(key, value.0).is_some() {
                        return Err(de::Error::custom("duplicate JSON field"));
                    }
                }
                Ok(UniqueJson(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

pub(crate) fn unique_json(reader: impl Read) -> Result<Value, StorageError> {
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let value = UniqueJson::deserialize(&mut deserializer)?.0;
    deserializer.end()?;
    Ok(value)
}

fn required(value: &Value, fields: &[&str]) -> Result<(), StorageError> {
    require(
        value
            .as_object()
            .is_some_and(|object| fields.iter().all(|field| object.contains_key(*field))),
        "required nullable field is absent",
    )
}

pub fn read_learning_snapshot(
    reader: impl Read,
    captured_at: i64,
) -> Result<LearningSnapshot, StorageError> {
    let mut value = unique_json(reader)?;
    if let Some(operations) = value["save_operations"].as_array_mut() {
        for operation in operations {
            required(&operation["request"], &["id", "revision"])?;
            crate::vocabulary::normalize_save_request_fields(&mut operation["request"]);
        }
    }
    if let Some(records) = value["records"].as_array() {
        for record in records {
            required(record, &["example_sense"])?;
            if let Some(digest) = record["digest"].as_array() {
                for entry in digest {
                    required(entry, &["translation"])?;
                }
            }
        }
    }
    if let Some(progress) = value["progress"].as_array() {
        for state in progress {
            required(&state["card"], &["memory", "last_review_at", "due_at"])?;
        }
    }
    if let Some(ratings) = value["ratings"].as_array() {
        for rating in ratings {
            if !rating["queue_before"].is_null() {
                required(
                    &rating["queue_before"],
                    &["position", "ordered_cards", "learning_day", "timezone"],
                )?;
            }
            required(
                &rating["before_state"],
                &["memory", "last_review_at", "due_at"],
            )?;
            required(
                &rating["result"]["card"],
                &["memory", "last_review_at", "due_at"],
            )?;
            required(&rating["result"], &["step_seconds", "interval_days"])?;
        }
    }
    if let Some(batches) = value["batches"].as_array() {
        for batch in batches {
            require(
                matches!(batch["voice"].as_str(), Some("female" | "male")),
                "batch voice must use its portable spelling",
            )?;
        }
    }
    required(
        &value["preferences"]["review"],
        &["new_cards_ignore_review_limit"],
    )?;
    required(&value["preferences"]["speech"], &["voice", "wpm"])?;
    require(
        matches!(
            value["preferences"]["speech"]["voice"].as_str(),
            Some("female" | "male")
        ),
        "snapshot speech voice must use its portable spelling",
    )?;
    let snapshot: LearningSnapshot = serde_json::from_value(value)?;
    validate_learning_snapshot(&snapshot, captured_at)?;
    Ok(snapshot)
}
