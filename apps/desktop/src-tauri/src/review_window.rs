//! Two persistent file workers and a bounded cache of verified, immutable audio bytes.
use crate::{
    speech_cache, speech_tasks,
    vocabulary::{DatabasePath, open},
};
use lexique_storage::{self as storage, AudioVersion, QueueCard, VocabularyRecord};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Condvar, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const METADATA_LIMIT: usize = 50;
const WINDOW_LIMIT: usize = 4;
const PROBE_LIMIT: usize = 8;
const BYTE_LIMIT: usize = 33_554_432;
const WAIT: Duration = Duration::from_millis(75);
const TRANSIENT_RETRY: Duration = Duration::from_secs(1);

#[derive(Clone, Debug)]
pub struct Failure {
    pub message: String,
    pub repairable: bool,
    pub missing: bool,
}
type AudioResult = Result<Arc<Vec<u8>>, Failure>;
struct Entry {
    result: Option<AudioResult>,
    used: u64,
    retry_at: Option<Instant>,
    inflight: bool,
}
#[derive(Default)]
struct Cache {
    entries: HashMap<String, Entry>,
    bytes: usize,
    clock: u64,
    inflight: usize,
}
struct Shared {
    cache: Mutex<Cache>,
    ready: Condvar,
}
enum Job {
    Read {
        version: AudioVersion,
        notify_failure: bool,
    },
    Stop,
}

pub struct WindowValidator {
    sender: mpsc::SyncSender<Job>,
    shared: Arc<Shared>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    metadata: Mutex<storage::QueueCache>,
}

fn probe(database: &std::path::Path, version: &AudioVersion) -> AudioResult {
    let result = speech_cache::Inspector::new(database)
        .and_then(|inspector| speech_tasks::read_selected(&inspector, version));
    match result {
        Ok(Some(bytes)) => Ok(Arc::new(bytes)),
        Ok(None) => Err(Failure {
            message: "Selected audio file is missing".into(),
            repairable: true,
            missing: true,
        }),
        Err(message) => {
            let repairable = message == "Selected audio integrity check failed"
                || message == "Invalid cached speech audio";
            Err(Failure {
                message,
                repairable,
                missing: false,
            })
        }
    }
}

impl WindowValidator {
    pub fn start(app: tauri::AppHandle, database: PathBuf) -> Arc<Self> {
        let (sender, receiver) = mpsc::sync_channel(PROBE_LIMIT);
        let receiver = Arc::new(Mutex::new(receiver));
        let shared = Arc::new(Shared {
            cache: Mutex::new(Cache::default()),
            ready: Condvar::new(),
        });
        let mut threads = Vec::new();
        for _ in 0..2 {
            let receiver = receiver.clone();
            let shared = shared.clone();
            let database = database.clone();
            let app = app.clone();
            threads.push(thread::spawn(move || {
                loop {
                    let job = receiver.lock().expect("audio job receiver").recv();
                    let Ok(Job::Read {
                        version,
                        notify_failure,
                    }) = job
                    else {
                        break;
                    };
                    let result = probe(&database, &version);
                    let transient = matches!(&result, Err(failure) if !failure.repairable);
                    // A failed retry must not trigger an endless completion/refresh loop.
                    let notify = notify_failure || !transient;
                    let mut cache = shared.cache.lock().expect("audio window cache");
                    cache.inflight -= 1;
                    let bytes = result.as_ref().map_or(0, |bytes| bytes.len());
                    while cache.bytes + bytes > BYTE_LIMIT || cache.entries.len() > 100 {
                        let oldest = cache
                            .entries
                            .iter()
                            .filter(|(key, entry)| {
                                key.as_str() != version.locator && !entry.inflight
                            })
                            .min_by_key(|(_, entry)| entry.used)
                            .map(|(key, _)| key.clone());
                        let Some(oldest) = oldest else {
                            break;
                        };
                        if let Some(entry) = cache.entries.remove(&oldest) {
                            cache.bytes -= entry
                                .result
                                .and_then(Result::ok)
                                .map_or(0, |bytes| bytes.len());
                        }
                    }
                    cache.bytes += bytes;
                    if let Some(entry) = cache.entries.get_mut(&version.locator) {
                        entry.result = Some(result);
                        entry.retry_at = transient.then(|| Instant::now() + TRANSIENT_RETRY);
                        entry.inflight = false;
                    }
                    shared.ready.notify_all();
                    drop(cache);
                    if notify {
                        speech_tasks::changed(&app);
                    }
                }
            }));
        }
        Arc::new(Self {
            sender,
            shared,
            threads: Mutex::new(threads),
            metadata: Mutex::new(storage::QueueCache::default()),
        })
    }

    pub fn stop(&self) {
        let mut threads = self.threads.lock().expect("audio window workers");
        if threads.is_empty() {
            return;
        }
        for _ in 0..threads.len() {
            let _ = self.sender.send(Job::Stop);
        }
        for worker in threads.drain(..) {
            let _ = worker.join();
        }
    }

    fn inspect(&self, cards: &[QueueCard], versions: &HashMap<i64, AudioVersion>) -> Inspection {
        let deadline = Instant::now() + WAIT;
        let mut cache = self.shared.cache.lock().expect("audio window cache");
        let mut probes = 0;
        for card in cards
            .iter()
            .take(METADATA_LIMIT)
            .filter(|card| card.kind == "listening")
        {
            let Some(version) = versions.get(&card.vocabulary_id) else {
                continue;
            };
            cache.clock += 1;
            let clock = cache.clock;
            let notify_failure = if let Some(entry) = cache.entries.get_mut(&version.locator) {
                entry.used = clock;
                if entry.retry_at.is_none_or(|retry| Instant::now() < retry) {
                    continue;
                }
                false
            } else {
                true
            };
            if probes == PROBE_LIMIT || cache.inflight == PROBE_LIMIT {
                break;
            }
            if let Some(entry) = cache.entries.get_mut(&version.locator) {
                // Keep the visible failure while retrying; a slow repeated failure
                // emits no event, so it must not leave the UI showing pending forever.
                entry.retry_at = None;
                entry.inflight = true;
            } else {
                cache.entries.insert(
                    version.locator.clone(),
                    Entry {
                        result: None,
                        used: clock,
                        retry_at: None,
                        inflight: true,
                    },
                );
            }
            match self.sender.try_send(Job::Read {
                version: version.clone(),
                notify_failure,
            }) {
                Ok(()) => {
                    probes += 1;
                    cache.inflight += 1;
                }
                Err(_) => {
                    cache.entries.remove(&version.locator);
                    break;
                }
            }
        }
        loop {
            let mut ready = 0;
            let mut pending = false;
            for card in cards.iter().take(METADATA_LIMIT) {
                if card.kind != "listening" {
                    ready += 1;
                } else if let Some(version) = versions.get(&card.vocabulary_id) {
                    match cache
                        .entries
                        .get(&version.locator)
                        .and_then(|entry| entry.result.as_ref())
                    {
                        Some(Ok(_)) => ready += 1,
                        Some(Err(_)) => {}
                        None => pending = true,
                    }
                }
                if ready >= WINDOW_LIMIT {
                    break;
                }
            }
            if ready >= WINDOW_LIMIT || !pending || Instant::now() >= deadline {
                break;
            }
            let wait = deadline.saturating_duration_since(Instant::now());
            cache = self
                .shared
                .ready
                .wait_timeout(cache, wait)
                .expect("audio validation wait")
                .0;
        }
        let mut results = HashMap::new();
        for version in versions.values() {
            if let Some(result) = cache
                .entries
                .get(&version.locator)
                .and_then(|entry| entry.result.clone())
            {
                results.insert(version.locator.clone(), result);
            }
        }
        Inspection { results, probes }
    }
}

impl Drop for WindowValidator {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Inspection {
    results: HashMap<String, AudioResult>,
    probes: usize,
}

#[derive(Clone, Serialize)]
pub struct WindowEntry {
    pub card: QueueCard,
    pub record: VocabularyRecord,
    pub progress: storage::ReviewProgress,
    pub version: Option<String>,
    pub content_hash: Option<String>,
    pub audio: Option<Vec<u8>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnownEntry {
    pub card: QueueCard,
    pub record_revision: i64,
    pub progress_revision: i64,
    pub version: Option<String>,
    pub content_hash: Option<String>,
}
impl KnownEntry {
    fn matches(&self, entry: &WindowEntry) -> bool {
        self.card == entry.card
            && self.record_revision == entry.record.revision
            && self.progress_revision == entry.progress.revision
            && self.version == entry.version
            && self.content_hash == entry.content_hash
    }
    fn same_audio(&self, entry: &WindowEntry) -> bool {
        self.card == entry.card
            && self.version == entry.version
            && self.content_hash == entry.content_hash
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueCursor {
    pub revision: i64,
    pub window: Vec<KnownEntry>,
}
impl QueueCursor {
    pub fn valid(&self) -> bool {
        fn id(value: i64) -> bool {
            value > 0 && value <= 9_007_199_254_740_991
        }
        fn hash(value: &Option<String>) -> bool {
            value.as_ref().is_none_or(|value| {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
        }
        self.revision >= 0
            && self.revision <= 9_007_199_254_740_991
            && self.window.len() <= WINDOW_LIMIT
            && self.window.iter().all(|entry| {
                id(entry.card.id)
                    && id(entry.card.vocabulary_id)
                    && matches!(entry.card.kind.as_str(), "recognition" | "listening")
                    && entry.record_revision >= 0
                    && entry.record_revision <= 9_007_199_254_740_991
                    && entry.progress_revision >= 0
                    && entry.progress_revision <= 9_007_199_254_740_991
                    && hash(&entry.version)
                    && hash(&entry.content_hash)
            })
            && self
                .window
                .iter()
                .map(|entry| entry.card.id)
                .collect::<std::collections::HashSet<_>>()
                .len()
                == self.window.len()
    }
}
#[derive(Serialize)]
pub struct QueueDelta {
    pub before_revision: Option<i64>,
    pub after_revision: i64,
    pub removed: Vec<i64>,
    pub order: Vec<i64>,
}
#[derive(Serialize)]
pub struct Summary {
    pub admitted_count: usize,
    pub audio_pending_count: usize,
    pub missing_vocabulary_ids: Vec<i64>,
    pub verification_pending: bool,
    pub learning_day: String,
    pub timezone: String,
    pub revision: i64,
    pub unknown_legacy: bool,
}
#[derive(Serialize)]
pub struct Metrics {
    pub metadata_candidates: usize,
    pub strict_probes: usize,
    pub compressed_bytes: usize,
    pub payload_bytes: usize,
    pub validation_wait_ms: f64,
    pub metadata_loaded_rows: usize,
    pub parsed_progress: usize,
    pub metadata_rebuilt: bool,
    pub rebuild_reason: Option<String>,
    pub database_ms: f64,
}
#[derive(Serialize)]
pub struct QueueView {
    pub queue: storage::ReviewQueue,
    pub record: Option<VocabularyRecord>,
    pub audio_pending: bool,
    pub audio_error: Option<String>,
    pub audio_vocabulary_id: Option<i64>,
    pub summary: Summary,
    pub window: Vec<WindowEntry>,
    pub delta: QueueDelta,
    pub metrics: Metrics,
    #[serde(skip)]
    pub demands: Vec<storage::WordDemand>,
}

pub fn read(
    path: &DatabasePath,
    validator: &WindowValidator,
    now: i64,
    timezone: &str,
    skipped: &std::collections::HashSet<i64>,
    cursor: Option<&QueueCursor>,
) -> Result<QueueView, String> {
    let mut cache = validator
        .metadata
        .lock()
        .map_err(|_| "Queue metadata cache unavailable")?;
    let database_started = Instant::now();
    let connection = open(path)?;
    let (initial, tx) = metadata_snapshot(&connection, &mut cache, now, timezone, skipped)?;
    tx.commit().map_err(|e| e.to_string())?;
    let mut metrics = initial.metrics.clone();
    let candidates: Vec<_> = initial
        .queue
        .cards
        .iter()
        .take(METADATA_LIMIT)
        .cloned()
        .collect();
    drop(connection);
    let mut database_ms = database_started.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    let checked = validator.inspect(&candidates, &initial.versions);
    let waited = started.elapsed().as_secs_f64() * 1000.0;
    let database_started = Instant::now();
    let connection = open(path)?;
    let mut audio_error = None;
    let mut audio_error_locator = None;
    let mut audio_vocabulary_id = initial.audio.missing_words.first().copied();
    for (locator, result) in &checked.results {
        if let Err(failure) = result {
            if audio_error.is_none() {
                audio_error = Some(failure.message.clone());
                audio_error_locator = Some(locator.clone());
            }
            if let Some(word) = initial
                .versions
                .iter()
                .find_map(|(word, v)| (&v.locator == locator).then_some(*word))
            {
                audio_vocabulary_id.get_or_insert(word);
            }
            if failure.repairable {
                storage::mark_audio_unavailable(
                    &connection,
                    locator,
                    if failure.missing {
                        "missing"
                    } else {
                        "invalid"
                    },
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    // Reuse unchanged metadata, or re-evaluate it inside the transaction used for
    // records and diagnostics. Background repairs can commit without invalidating
    // this coherent read snapshot or forcing another optimistic revision chase.
    let tx =
        rusqlite::Transaction::new_unchecked(&connection, rusqlite::TransactionBehavior::Deferred)
            .map_err(|e| e.to_string())?;
    let (mut snapshot, tx) = if storage::queue_revision(&tx).map_err(|e| e.to_string())?
        == cache.revision()
    {
        (initial, tx)
    } else {
        drop(tx);
        let (snapshot, tx) = metadata_snapshot(&connection, &mut cache, now, timezone, skipped)?;
        metrics.loaded_rows += snapshot.metrics.loaded_rows;
        metrics.parsed_progress += snapshot.metrics.parsed_progress;
        metrics.rebuilt |= snapshot.metrics.rebuilt;
        if metrics.reason.is_none() {
            metrics.reason = snapshot.metrics.reason.clone();
        }
        (snapshot, tx)
    };
    let (results, compressed_bytes): (HashMap<_, _>, usize) = {
        let bytes = validator
            .shared
            .cache
            .lock()
            .map_err(|_| "Audio window cache unavailable")?;
        let results = snapshot
            .versions
            .values()
            .filter_map(|version| {
                bytes
                    .entries
                    .get(&version.locator)
                    .and_then(|entry| entry.result.clone())
                    .map(|result| (version.locator.clone(), result))
            })
            .collect();
        (results, bytes.bytes)
    };
    let mut selected = Vec::new();
    let mut payload_bytes = 0;
    let mut pending = false;
    for card in snapshot.queue.cards.iter().take(METADATA_LIMIT).cloned() {
        let audio = if card.kind == "listening" {
            let Some(version) = snapshot.versions.get(&card.vocabulary_id) else {
                pending = true;
                continue;
            };
            match results.get(&version.locator) {
                Some(Ok(bytes)) if payload_bytes + bytes.len() <= BYTE_LIMIT => {
                    payload_bytes += bytes.len();
                    Some(bytes.as_ref().clone())
                }
                Some(Err(_)) => {
                    continue;
                }
                _ => {
                    pending = true;
                    continue;
                }
            }
        } else {
            None
        };
        selected.push((card, audio));
        if selected.len() == WINDOW_LIMIT {
            break;
        }
    }
    let words: Vec<_> = selected
        .iter()
        .map(|(card, _)| card.vocabulary_id)
        .collect();
    let records = storage::get_batch(&tx, &words).map_err(|e| e.to_string())?;
    let mut window = Vec::new();
    for (card, audio) in selected {
        let record = records
            .get(&card.vocabulary_id)
            .ok_or("Queue word disappeared during verification")?
            .clone();
        let version = snapshot.versions.get(&card.vocabulary_id);
        let progress = cache
            .progress(card.id)
            .ok_or("Queue progress disappeared")?
            .clone();
        window.push(WindowEntry {
            card,
            record,
            progress,
            version: version.map(|v| v.locator.clone()),
            content_hash: version.map(|v| v.content_hash.clone()),
            audio,
        });
    }
    let diagnostics = storage::audio_demand_diagnostics(&tx, &snapshot.audio.missing_words)
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    let admitted_count = snapshot.queue.cards.len();
    snapshot.queue.cards = window.iter().map(|entry| entry.card.clone()).collect();
    let record = window.first().map(|entry| entry.record.clone());
    if audio_error_locator
        .as_ref()
        .is_some_and(|locator| matches!(results.get(locator), Some(Ok(_))))
    {
        audio_error = None;
    }
    if audio_error.is_none() {
        audio_error = diagnostics.error;
    }
    let summary = Summary {
        admitted_count,
        audio_pending_count: snapshot.audio.pending_count,
        missing_vocabulary_ids: snapshot.audio.missing_words,
        verification_pending: pending,
        learning_day: storage::review_learning_day(now, timezone).map_err(|e| e.to_string())?,
        timezone: timezone.into(),
        revision: cache.revision(),
        unknown_legacy: diagnostics.unknown_legacy,
    };
    let cursor = cursor.filter(|cursor| {
        cursor.revision <= summary.revision && summary.revision - cursor.revision <= 1024
    });
    let order: Vec<_> = window.iter().map(|entry| entry.card.id).collect();
    let removed = cursor.map_or_else(Vec::new, |cursor| {
        cursor
            .window
            .iter()
            .filter(|entry| !order.contains(&entry.card.id))
            .map(|entry| entry.card.id)
            .collect()
    });
    let mut changed = Vec::new();
    payload_bytes = 0;
    for mut entry in window {
        if let Some(known) = cursor.and_then(|cursor| {
            cursor
                .window
                .iter()
                .find(|known| known.card.id == entry.card.id)
        }) {
            if known.matches(&entry) {
                continue;
            }
            if known.same_audio(&entry) {
                entry.audio = None;
            }
        }
        payload_bytes += entry.audio.as_ref().map_or(0, Vec::len);
        changed.push(entry);
    }
    database_ms += database_started.elapsed().as_secs_f64() * 1000.0;
    let delta = QueueDelta {
        before_revision: cursor.map(|cursor| cursor.revision),
        after_revision: summary.revision,
        removed,
        order,
    };
    Ok(QueueView {
        queue: snapshot.queue,
        record,
        audio_pending: summary.audio_pending_count > 0 || pending,
        audio_error,
        audio_vocabulary_id,
        summary,
        window: changed,
        delta,
        metrics: Metrics {
            metadata_candidates: admitted_count.min(METADATA_LIMIT),
            strict_probes: checked.probes,
            compressed_bytes,
            payload_bytes,
            validation_wait_ms: waited,
            metadata_loaded_rows: metrics.loaded_rows,
            parsed_progress: metrics.parsed_progress,
            metadata_rebuilt: metrics.rebuilt,
            rebuild_reason: metrics.reason,
            database_ms,
        },
        demands: snapshot.demands,
    })
}

struct MetadataSnapshot {
    queue: storage::ReviewQueue,
    audio: storage::QueueAudioMetadata,
    versions: HashMap<i64, AudioVersion>,
    metrics: storage::QueueCacheMetrics,
    demands: Vec<storage::WordDemand>,
}
fn metadata_snapshot<'a>(
    connection: &'a rusqlite::Connection,
    cache: &mut storage::QueueCache,
    now: i64,
    timezone: &str,
    skipped: &std::collections::HashSet<i64>,
) -> Result<(MetadataSnapshot, rusqlite::Transaction<'a>), String> {
    let storage::ReviewQueueSnapshot {
        transaction: tx,
        mut queue,
        audio,
        demand_candidates,
        metrics,
    } = storage::review_queue_snapshot(connection, now, timezone, skipped, cache)
        .map_err(|e| e.to_string())?;
    let demands = storage::plan_review_words(&queue.cards, &demand_candidates, &audio.ready);
    let words: Vec<_> = queue
        .cards
        .iter()
        .take(METADATA_LIMIT)
        .map(|card| card.vocabulary_id)
        .collect();
    let versions = storage::selected_audio_batch(&tx, &words).map_err(|e| e.to_string())?;
    if let Some(due) = audio.next_due {
        if queue.available_at.is_none_or(|at| due < at) {
            queue.available_at = Some(due);
        }
    }
    Ok((
        MetadataSnapshot {
            queue,
            audio,
            versions,
            metrics,
            demands,
        },
        tx,
    ))
}
