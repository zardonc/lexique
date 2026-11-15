use lexique_core::scheduling::{Card, Scheduled};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

use crate::{
    ConjugationBlockOrder, ConjugationMode, DictionaryCatalog, InterfacePreferences, QueueCard,
    RatingRequest, ReviewSettings, SaveRequest, SpeechPreferences, SpeechSpec, SpeechVoice,
    VocabularyRecord,
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LearningSnapshot {
    pub snapshot_version: u32,
    #[serde(serialize_with = "serialize_records")]
    pub records: Vec<VocabularyRecord>,
    pub save_operations: Vec<SnapshotSaveOperation>,
    pub cards: Vec<QueueCard>,
    pub progress: Vec<SnapshotProgress>,
    pub ratings: Vec<SnapshotRating>,
    pub queue: SnapshotQueue,
    pub paused_vocabulary_ids: Vec<i64>,
    pub batches: Vec<SnapshotBatch>,
    pub speech_sources: Vec<SnapshotSpeechSource>,
    pub speech_owners: Vec<SnapshotSpeechOwner>,
    pub speech_tasks: Vec<SnapshotSpeechTask>,
    pub audio_versions: Vec<SnapshotAudioVersion>,
    pub audio_selections: Vec<SnapshotAudioSelection>,
    pub audio_imports: Vec<SnapshotAudioImport>,
    pub legacy_audio: Vec<SnapshotLegacyAudio>,
    pub audio_operations: Vec<SnapshotAudioOperation>,
    pub audio_publications: Vec<SnapshotAudioPublication>,
    pub audio_repairs: Vec<SnapshotAudioRepair>,
    pub preferences: SnapshotPreferences,
    pub dictionary_catalog: DictionaryCatalog,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSaveOperation {
    pub operation: String,
    #[serde(serialize_with = "serialize_save_request")]
    pub request: SaveRequest,
    pub outcome: String,
    pub record_id: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotProgress {
    pub card_id: i64,
    pub revision: i64,
    pub card: Card,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRating {
    pub sequence: u64,
    pub operation: String,
    pub card_id: i64,
    pub request: RatingRequest,
    pub before_state: Card,
    pub result: Scheduled,
    pub reviewed_at: i64,
    pub learning_day: String,
    pub quota: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub queue_before: Option<SnapshotQueueBefore>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub undo_operation: Option<String>,
    pub undo_queue_rebuilt: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotQueue {
    #[serde(deserialize_with = "Option::deserialize")]
    pub context: Option<SnapshotQueueContext>,
    pub entries: Vec<SnapshotQueueEntry>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotQueueContext {
    pub learning_day: String,
    pub timezone: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotQueueEntry {
    pub card_id: i64,
    pub position: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotQueueBefore {
    pub card_id: i64,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub ordered_cards: Option<Vec<i64>>,
    #[serde(default)]
    pub learning_day: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotBatch {
    pub sequence: u64,
    pub operation: String,
    pub requested_vocabulary_ids: Vec<i64>,
    pub voice: SpeechVoice,
    pub created_at: i64,
    pub cancelled: bool,
    pub items: Vec<SnapshotBatchItem>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotBatchItem {
    pub ordinal: i64,
    pub vocabulary_id: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub semantic_key: Option<String>,
    pub created_cards: u8,
    #[serde(deserialize_with = "Option::deserialize")]
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSpeechSource {
    pub semantic_key: String,
    pub spec: SpeechSpec,
    pub context: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub current_version: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub pending_version: Option<String>,
    pub retain: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSpeechOwner {
    pub semantic_key: String,
    pub vocabulary_id: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSpeechTask {
    pub id: i64,
    pub semantic_key: String,
    pub priority: u8,
    pub state: String,
    pub attempts: u8,
    pub wake_at: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub deadline_at: Option<i64>,
    pub cancel_requested: bool,
    pub refresh_requested: bool,
    pub successor_requested: bool,
    pub interactive_requested: bool,
    pub execution_generation: i64,
    pub demand_class: u8,
    pub requested_at: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AudioPayloadStatus {
    Included,
    SourceMissing,
    SourceInvalid,
    Excluded,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AudioExclusionReason {
    Expired,
    NotSelected,
    Unpublished,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioVersion {
    pub locator: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub semantic_key: Option<String>,
    pub content_hash: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub byte_length: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub generated_at: Option<i64>,
    pub created_at: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub origin_vocabulary_id: Option<i64>,
    pub expected_revision: i64,
    pub generation: i64,
    pub sealed: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    pub execution_generation: Option<i64>,
    pub context: String,
    pub verification: String,
    pub gc_state: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub expires_at: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub orphan_until: Option<i64>,
    pub payload_status: AudioPayloadStatus,
    #[serde(deserialize_with = "Option::deserialize")]
    pub exclusion_reason: Option<AudioExclusionReason>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioSelection {
    pub vocabulary_id: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub current_version: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub previous_version: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub previous_until: Option<i64>,
    pub revision: i64,
    pub desired_generation: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub desired_key: Option<String>,
    pub current_generation: i64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioImport {
    pub vocabulary_id: i64,
    pub state: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotLegacyAudio {
    pub vocabulary_id: i64,
    pub content_hash: String,
    pub payload_status: AudioPayloadStatus,
    #[serde(deserialize_with = "Option::deserialize")]
    pub exclusion_reason: Option<AudioExclusionReason>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioOperation {
    pub operation: String,
    pub vocabulary_id: i64,
    pub semantic_key: String,
    pub generation: i64,
    pub refresh: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioPublication {
    pub locator: String,
    pub vocabulary_id: i64,
    pub expected_revision: i64,
    pub generation: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub execution_generation: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAudioRepair {
    pub vocabulary_id: i64,
    pub failure_identity: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub semantic_key: Option<String>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub generation: Option<i64>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotPreferences {
    pub review: ReviewSettings,
    pub auto_enroll: bool,
    pub speech: SpeechPreferences,
    pub interface: InterfacePreferences,
    pub conjugation: SnapshotConjugationPreferences,
    pub vocabulary_sort: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotConjugationPreferences {
    pub mode: ConjugationMode,
    pub block_order: ConjugationBlockOrder,
}

struct PortableReference<'a>(&'a crate::Reference);

impl Serialize for PortableReference<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let reference = self.0;
        let mut map = serializer.serialize_map(Some(9))?;
        map.serialize_entry("dictionary_id", &reference.dictionary_id)?;
        map.serialize_entry("dictionary", &reference.dictionary)?;
        map.serialize_entry("lookup_key", &reference.lookup_key)?;
        map.serialize_entry("version", &reference.version)?;
        map.serialize_entry("attribution", &reference.attribution)?;
        map.serialize_entry("license", &reference.license)?;
        map.serialize_entry("source_url", &reference.source_url)?;
        map.serialize_entry("license_url", &reference.license_url)?;
        map.serialize_entry("sha256", &reference.sha256)?;
        map.end()
    }
}

struct PortableExample<'a>(&'a crate::Example);

impl Serialize for PortableExample<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("fr", &self.0.fr)?;
        map.serialize_entry("zh", &self.0.zh)?;
        map.serialize_entry("source", &self.0.source.as_ref().map(PortableReference))?;
        map.end()
    }
}

struct PortableRecord<'a>(&'a VocabularyRecord);

impl Serialize for PortableRecord<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let record = self.0;
        let mut map = serializer.serialize_map(Some(13))?;
        map.serialize_entry("id", &record.id)?;
        map.serialize_entry("headword", &record.headword)?;
        map.serialize_entry("reference", &PortableReference(&record.reference))?;
        map.serialize_entry("digest", &record.digest)?;
        map.serialize_entry("notes", &record.notes)?;
        map.serialize_entry("example_fr", &record.example_fr)?;
        map.serialize_entry("example_zh", &record.example_zh)?;
        map.serialize_entry(
            "example_source",
            &record.example_source.as_ref().map(PortableReference),
        )?;
        map.serialize_entry("example_sense", &record.example_sense)?;
        map.serialize_entry(
            "more_examples",
            &record
                .more_examples
                .iter()
                .map(PortableExample)
                .collect::<Vec<_>>(),
        )?;
        map.serialize_entry("revision", &record.revision)?;
        map.serialize_entry("created_at", &record.created_at)?;
        map.serialize_entry("updated_at", &record.updated_at)?;
        map.end()
    }
}

fn serialize_records<S: Serializer>(
    records: &[VocabularyRecord],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    records
        .iter()
        .map(PortableRecord)
        .collect::<Vec<_>>()
        .serialize(serializer)
}

fn serialize_save_request<S: Serializer>(
    request: &SaveRequest,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(10))?;
    map.serialize_entry("operation", &request.operation)?;
    map.serialize_entry("headword", &request.headword)?;
    map.serialize_entry("notes", &request.notes)?;
    map.serialize_entry("example_fr", &request.example_fr)?;
    map.serialize_entry("example_zh", &request.example_zh)?;
    map.serialize_entry(
        "example_source",
        &request.example_source.as_ref().map(PortableReference),
    )?;
    map.serialize_entry("example_sense", &request.example_sense)?;
    map.serialize_entry(
        "more_examples",
        &request
            .more_examples
            .iter()
            .map(PortableExample)
            .collect::<Vec<_>>(),
    )?;
    map.serialize_entry("id", &request.id)?;
    map.serialize_entry("revision", &request.revision)?;
    map.end()
}
