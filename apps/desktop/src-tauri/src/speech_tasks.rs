//! Application-owned, bounded CPU work. SQLite owns demand, attempts and cancellation.
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use lexique_storage as storage;
use rusqlite::Connection;
use serde::Deserialize;
use tauri::Emitter;
#[cfg(windows)]
use tauri::Manager;

use crate::{speech_cache, vocabulary::DatabasePath};

pub fn now() -> Result<i64, String> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
    )
    .map_err(|e| e.to_string())
}

pub fn changed(app: &tauri::AppHandle) {
    if let Err(error) = app.emit("lexique-review-changed", ()) {
        eprintln!("Could not notify review change: {error}");
    }
}

pub fn cached_asset(
    connection: &Connection,
    inspector: &speech_cache::Inspector,
    key: &str,
    time: i64,
) -> Result<Option<Vec<u8>>, String> {
    let Some(asset) = storage::speech_asset(connection, key, time).map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };
    let bytes = inspector.read_asset(&asset.physical_locator, &asset.content_hash)?;
    match bytes {
        Some(bytes) if storage::content_hash(&bytes) == asset.content_hash => {
            storage::verify_audio_content(connection, &asset.locator, bytes.len() as i64)
                .map_err(|e| e.to_string())?;
            storage::audio_last_used(connection, &asset.locator, time)
                .map_err(|e| e.to_string())?;
            Ok(Some(bytes))
        }
        Some(_) => Err("Cached speech content checksum mismatch".into()),
        None => Ok(None),
    }
}

pub fn cached_word(
    connection: &Connection,
    inspector: &speech_cache::Inspector,
    id: i64,
    time: i64,
) -> Result<Option<Vec<u8>>, String> {
    if let Some(version) = storage::selected_audio(connection, id).map_err(|e| e.to_string())? {
        return read_selected(inspector, &version);
    }
    let pending =
        storage::pending_word_audio_versions(connection, id).map_err(|e| e.to_string())?;
    for version in &pending {
        if read_selected(inspector, version)?.is_some() {
            storage::complete_audio_version(connection, &version.locator, time)
                .map_err(|e| e.to_string())?;
            if let Some(selected) =
                storage::selected_audio(connection, id).map_err(|e| e.to_string())?
            {
                return read_selected(inspector, &selected);
            }
        }
    }
    if !pending.is_empty() {
        // An in-progress publisher may not have written its file yet. Preserve its
        // intent rather than choosing another source or abandoning live work.
        return Ok(None);
    }
    for key in storage::word_audio_keys(connection, id).map_err(|e| e.to_string())? {
        let Some(asset) =
            storage::speech_asset(connection, &key, time).map_err(|e| e.to_string())?
        else {
            continue;
        };
        if let Some(bytes) = inspector.read_asset(&asset.physical_locator, &asset.content_hash)? {
            if storage::content_hash(&bytes) != asset.content_hash {
                return Err("Cached speech content checksum mismatch".into());
            }
            storage::verify_audio_content(connection, &asset.locator, bytes.len() as i64)
                .map_err(|e| e.to_string())?;
            storage::reuse_audio_version(connection, id, &asset.locator, 0, time)
                .map_err(|e| e.to_string())?;
            if let Some(version) =
                storage::selected_audio(connection, id).map_err(|e| e.to_string())?
            {
                return read_selected(inspector, &version);
            }
            return Err("Initial audio selection was superseded".into());
        }
    }
    if let Some(bytes) = inspector.legacy(id)? {
        return select_initial(connection, inspector, id, None, None, &bytes, time);
    }
    Ok(None)
}

fn select_initial(
    connection: &Connection,
    inspector: &speech_cache::Inspector,
    id: i64,
    key: Option<&str>,
    generated_at: Option<i64>,
    bytes: &[u8],
    time: i64,
) -> Result<Option<Vec<u8>>, String> {
    let publication =
        publish_selection(connection, inspector, id, key, generated_at, bytes, 0, time);
    // Another reader may have committed first. Always return the winning selection's bytes.
    if let Some(version) = storage::selected_audio(connection, id).map_err(|e| e.to_string())? {
        return read_selected(inspector, &version);
    }
    publication?;
    Err("Initial audio selection disappeared".into())
}

pub fn read_selected(
    inspector: &speech_cache::Inspector,
    version: &storage::AudioVersion,
) -> Result<Option<Vec<u8>>, String> {
    let bytes = inspector.read_asset(&version.physical_locator, &version.content_hash)?;
    if let Some(bytes) = &bytes {
        if bytes.len() as i64 != version.byte_length
            || storage::content_hash(bytes) != version.content_hash
        {
            return Err("Selected audio integrity check failed".into());
        }
    }
    Ok(bytes)
}

pub fn publish_selection(
    connection: &Connection,
    inspector: &speech_cache::Inspector,
    id: i64,
    key: Option<&str>,
    generated_at: Option<i64>,
    bytes: &[u8],
    generation: i64,
    time: i64,
) -> Result<(), String> {
    let hash = storage::content_hash(bytes);
    let version = storage::prepare_audio_version(
        connection,
        id,
        key,
        &hash,
        bytes.len() as i64,
        generated_at,
        generation,
        time,
    )
    .map_err(|e| e.to_string())?;
    inspector.publish_version(&version.locator, &hash, bytes)?;
    read_selected(inspector, &version)?.ok_or("Published audio version disappeared")?;
    if !storage::complete_audio_version(connection, &version.locator, time)
        .map_err(|e| e.to_string())?
    {
        return Err("Audio selection was superseded; retry".into());
    }
    storage::finish_audio_import(connection, id, None).map_err(|e| e.to_string())?;
    Ok(())
}

type Reply = mpsc::SyncSender<Result<Vec<u8>, String>>;
enum Message {
    Wake,
    Prioritize,
    Wait {
        key: String,
        reply: Reply,
    },
    Frame {
        epoch: u64,
        header: Header,
        audio: Vec<u8>,
    },
    Broken {
        epoch: u64,
        error: String,
    },
    Stop,
}

pub struct SpeechTasks {
    sender: mpsc::SyncSender<Message>,
    thread: Mutex<Option<JoinHandle<()>>>,
    planned: Arc<Mutex<Vec<storage::WordDemand>>>,
    #[cfg(windows)]
    maintenance: crate::audio_maintenance::AudioMaintenance,
}

impl SpeechTasks {
    pub fn start(app: tauri::AppHandle, database: PathBuf, resources: PathBuf) -> Self {
        #[cfg(windows)]
        let maintenance =
            crate::audio_maintenance::AudioMaintenance::start(app.clone(), database.clone());
        let (sender, receiver) = mpsc::sync_channel(64);
        let actor_sender = sender.clone();
        let planned = Arc::new(Mutex::new(Vec::new()));
        let actor_planned = planned.clone();
        let thread = thread::spawn(move || {
            let diagnostic_path = database.clone();
            match Actor::new(
                app,
                database,
                resources,
                actor_sender,
                receiver,
                actor_planned,
            )
            .and_then(|mut actor| actor.run())
            {
                Ok(()) => (),
                Err(error) => speech_cache::report_error(
                    &diagnostic_path,
                    &format!("Speech supervisor stopped: {error}"),
                ),
            }
        });
        Self {
            sender,
            thread: Mutex::new(Some(thread)),
            planned,
            #[cfg(windows)]
            maintenance,
        }
    }
    pub fn wake(&self) -> Result<(), String> {
        match self.sender.try_send(Message::Wake) {
            Ok(()) | Err(mpsc::TrySendError::Full(_)) => Ok(()),
            Err(mpsc::TrySendError::Disconnected(_)) => Err("Speech supervisor unavailable".into()),
        }
    }
    pub fn prioritize(&self, demands: Vec<storage::WordDemand>) -> Result<(), String> {
        if demands.is_empty() {
            return Ok(());
        }
        *self
            .planned
            .lock()
            .map_err(|_| "Speech demand planner unavailable")? = demands;
        match self.sender.try_send(Message::Prioritize) {
            Ok(()) | Err(mpsc::TrySendError::Full(_)) => Ok(()),
            Err(mpsc::TrySendError::Disconnected(_)) => Err("Speech supervisor unavailable".into()),
        }
    }
    pub fn wait(&self, key: String) -> Result<Vec<u8>, String> {
        let (reply, receiver) = mpsc::sync_channel(1);
        self.sender
            .try_send(Message::Wait { key, reply })
            .map_err(|_| "Speech subscriber capacity unavailable")?;
        receiver
            .recv_timeout(Duration::from_secs(130))
            .map_err(|_| "Speech result unavailable within task budget")?
    }
    pub fn shutdown(&self) -> Result<(), String> {
        let stopped = self
            .sender
            .send(Message::Stop)
            .map_err(|_| "Speech supervisor unavailable");
        let joined = if let Some(thread) = self.thread.lock().map_err(|e| e.to_string())?.take() {
            thread.join().map_err(|_| "Speech supervisor panicked")
        } else {
            Ok(())
        };
        #[cfg(windows)]
        let maintenance = self.maintenance.shutdown();
        stopped?;
        joined?;
        #[cfg(windows)]
        maintenance?;
        Ok(())
    }
    #[cfg(windows)]
    pub fn cache_status(&self) -> Result<speech_cache::CacheReport, String> {
        self.maintenance.snapshot()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    id: u64,
    audio_bytes: usize,
    ready: Option<bool>,
    voice: Option<String>,
    language: Option<String>,
    error: Option<String>,
    retryable: Option<bool>,
    generated_at: Option<i64>,
}

fn read_frame(reader: &mut impl Read) -> Result<(Header, Vec<u8>), String> {
    let mut prefix = [0; 4];
    reader
        .read_exact(&mut prefix)
        .map_err(|e| format!("Speech frame prefix: {e}"))?;
    let length = u32::from_le_bytes(prefix) as usize;
    if !(1..=8192).contains(&length) {
        return Err("Invalid speech header length".into());
    }
    let mut bytes = vec![0; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|e| format!("Speech frame header: {e}"))?;
    let header: Header =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid speech header: {e}"))?;
    if header.id > 9_007_199_254_740_991 || header.audio_bytes > 10_000_000 {
        return Err("Invalid speech response bounds".into());
    }
    let valid = if header.id == 0 {
        header.ready == Some(true)
            && header.audio_bytes == 0
            && header.voice.is_none()
            && header.language.is_none()
            && header.error.is_none()
            && header.retryable.is_none()
            && header.generated_at.is_none()
    } else {
        header.ready.is_none()
            && matches!(header.voice.as_deref(), Some("female" | "male"))
            && match &header.error {
                Some(error) => {
                    !error.is_empty()
                        && error.len() <= 8192
                        && header.audio_bytes == 0
                        && header.language.is_none()
                        && header.retryable.is_some()
                        && header.generated_at.is_none()
                }
                None => {
                    header.language.as_deref() == Some("fr")
                        && header.audio_bytes > 0
                        && header.retryable.is_none()
                        && header.generated_at.is_some_and(|time| time >= 0)
                }
            }
    };
    if !valid {
        return Err("Invalid speech response fields".into());
    }
    let mut audio = vec![0; header.audio_bytes];
    reader
        .read_exact(&mut audio)
        .map_err(|e| format!("Speech frame audio: {e}"))?;
    if header.audio_bytes > 0 && !audio.starts_with(b"OggS") {
        return Err("Invalid generated speech audio".into());
    }
    Ok((header, audio))
}

struct Worker {
    child: Child,
    input: ChildStdin,
    reader: Option<JoinHandle<()>>,
    stderr: Option<JoinHandle<()>>,
    diagnostics: Arc<Mutex<Vec<u8>>>,
    #[cfg(windows)]
    _configuration: speech_cache::WorkerConfiguration,
    #[cfg(windows)]
    _job: job::Job,
}

fn validate_worker_spec(spec: &storage::SpeechSpec) -> Result<(), String> {
    let unsupported = "Saved audio specification is unsupported; an explicit upgrade is required";
    let voice = match spec.voice.as_str() {
        "female" => storage::SpeechVoice::Female,
        "male" => storage::SpeechVoice::Male,
        _ => return Err(unsupported.into()),
    };
    if spec != &storage::SpeechSpec::word(&spec.text, voice) {
        return Err(unsupported.into());
    }
    Ok(())
}

impl Worker {
    fn start(
        database: &Path,
        resources: &Path,
        epoch: u64,
        sender: mpsc::SyncSender<Message>,
    ) -> Result<Self, String> {
        #[cfg(not(windows))]
        {
            let _ = (database, resources, epoch, sender);
            Err("Offline speech is not admitted on this platform".into())
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let configuration = speech_cache::worker_configuration(database)?;
            let bootstrap = format!(
                "import types,sys\nfor name,source in {}:\n m=types.ModuleType(name);sys.modules[name]=m;exec(source,m.__dict__)\nexec({},{{'__name__':'__main__'}})",
                serde_json::to_string(&[
                    ("speech_worker", include_str!("../speech_worker.py")),
                    ("speech_protocol", include_str!("../speech_protocol.py"))
                ])
                .map_err(|e| e.to_string())?,
                serde_json::to_string(include_str!("../speech_resident.py"))
                    .map_err(|e| e.to_string())?
            );
            let mut child = Command::new(resources.join("runtime/python.exe"))
                .args(["-I", "-X", "utf8", "-c", &bootstrap])
                .arg(resources.join("assets"))
                .arg(resources.join("opusenc.dll"))
                .arg(&configuration.path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(0x0800_0000)
                .spawn()
                .map_err(|e| format!("Could not start offline speech: {e}"))?;
            let job = match job::Job::attach(&child) {
                Ok(job) => job,
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
            };
            let input = child.stdin.take().ok_or("Speech input unavailable")?;
            let mut output = child.stdout.take().ok_or("Speech output unavailable")?;
            let mut errors = child
                .stderr
                .take()
                .ok_or("Speech diagnostics unavailable")?;
            let reader = thread::spawn(move || {
                loop {
                    match read_frame(&mut output) {
                        Ok((header, audio)) => {
                            if sender
                                .try_send(Message::Frame {
                                    epoch,
                                    header,
                                    audio,
                                })
                                .is_err()
                            {
                                let _ = sender.try_send(Message::Broken {
                                    epoch,
                                    error: "Speech response channel capacity exceeded".into(),
                                });
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.try_send(Message::Broken { epoch, error });
                            break;
                        }
                    }
                }
            });
            let diagnostics = Arc::new(Mutex::new(Vec::new()));
            let error_buffer = diagnostics.clone();
            let stderr = thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match errors.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(size) => {
                            if let Ok(mut tail) = error_buffer.lock() {
                                tail.extend_from_slice(&buffer[..size]);
                                let excess = tail.len().saturating_sub(2048);
                                tail.drain(..excess);
                            }
                        }
                        Err(error) => {
                            eprintln!("Could not drain speech diagnostics: {error}");
                            break;
                        }
                    }
                }
            });
            Ok(Self {
                child,
                input,
                reader: Some(reader),
                stderr: Some(stderr),
                diagnostics,
                _configuration: configuration,
                _job: job,
            })
        }
    }
    fn send(&mut self, id: u64, task: &storage::SpeechTask) -> Result<(), String> {
        let mut frame = serde_json::to_vec(
            &serde_json::json!({"id":id,"text":task.spec.text,"voice":task.spec.voice}),
        )
        .map_err(|e| e.to_string())?;
        frame.push(b'\n');
        if frame.len() > 8192 {
            return Err("Speech request frame too large".into());
        }
        self.input
            .write_all(&frame)
            .and_then(|_| self.input.flush())
            .map_err(|e| format!("Could not send speech request: {e}"))
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Err(error) = self.child.kill() {
            eprintln!("Could not terminate speech worker: {error}");
        }
        if let Err(error) = self.child.wait() {
            eprintln!("Could not reap speech worker: {error}");
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        if let Some(stderr) = self.stderr.take() {
            let _ = stderr.join();
        }
    }
}

struct Running {
    task: storage::SpeechTask,
    started: Instant,
    budget: Duration,
}
struct Actor {
    app: tauri::AppHandle,
    database: PathBuf,
    resources: PathBuf,
    connection: Connection,
    sender: mpsc::SyncSender<Message>,
    receiver: mpsc::Receiver<Message>,
    worker: Option<Worker>,
    epoch: u64,
    ready: bool,
    starting: Option<Instant>,
    request: u64,
    running: HashMap<u64, Running>,
    waiters: HashMap<String, Vec<Reply>>,
    migration_pending: bool,
    planned: Arc<Mutex<Vec<storage::WordDemand>>>,
}

impl Actor {
    fn new(
        app: tauri::AppHandle,
        database: PathBuf,
        resources: PathBuf,
        sender: mpsc::SyncSender<Message>,
        receiver: mpsc::Receiver<Message>,
        planned: Arc<Mutex<Vec<storage::WordDemand>>>,
    ) -> Result<Self, String> {
        let connection = crate::vocabulary::open(&DatabasePath(database.clone()))?;
        Ok(Self {
            app,
            database,
            resources,
            connection,
            sender,
            receiver,
            worker: None,
            epoch: 0,
            ready: false,
            starting: None,
            request: 0,
            running: HashMap::new(),
            waiters: HashMap::new(),
            migration_pending: true,
            planned,
        })
    }
    fn recover(&mut self) -> Result<(), String> {
        storage::restore_owned_speech_gc(&self.connection).map_err(|e| e.to_string())?;
        self.connection
            .execute(
                "UPDATE word_audio_imports SET state='pending',error=NULL WHERE state='failed'",
                [],
            )
            .map_err(|e| e.to_string())?;
        let inspector = speech_cache::Inspector::new(&self.database);
        let mut cursor = String::new();
        loop {
            let pending = storage::pending_source_audio(&self.connection, &cursor)
                .map_err(|e| e.to_string())?;
            if pending.is_empty() {
                break;
            }
            for version in pending {
                cursor.clone_from(&version.locator);
                let result = (|| {
                    let inspector = inspector.as_ref().map_err(Clone::clone)?;
                    let mut bytes =
                        inspector.read_asset(&version.physical_locator, &version.content_hash)?;
                    if bytes.is_none() {
                        if let Some(import) = &version.import_locator {
                            if let Some(imported) =
                                inspector.read_asset(import, &version.content_hash)?
                            {
                                if storage::content_hash(&imported) != version.content_hash {
                                    return Err("Interrupted source checksum mismatch".into());
                                }
                                inspector.publish_version(
                                    &version.physical_locator,
                                    &version.content_hash,
                                    &imported,
                                )?;
                                bytes = inspector
                                    .read_asset(&version.physical_locator, &version.content_hash)?;
                            }
                        }
                    }
                    Ok::<_, String>(bytes)
                })();
                match result {
                    Ok(Some(bytes)) if storage::content_hash(&bytes) == version.content_hash => {
                        storage::verify_audio_content(
                            &self.connection,
                            &version.locator,
                            bytes.len() as i64,
                        )
                        .map_err(|e| e.to_string())?;
                        storage::complete_source_audio(&self.connection, &version.locator)
                            .map_err(|e| e.to_string())?;
                    }
                    Ok(_) => {
                        storage::abandon_audio_publication(
                            &self.connection,
                            &version.locator,
                            None,
                        )
                        .map_err(|e| e.to_string())?;
                        speech_cache::report_error(
                            &self.database,
                            "Interrupted speech publication has no verified file; intent released",
                        );
                    }
                    Err(error) => {
                        self.connection.execute("UPDATE speech_tasks SET state='failed',error=?2 WHERE semantic_key=?1 AND state='running'",rusqlite::params![version.semantic_key,error.chars().take(2048).collect::<String>()]).map_err(|e|e.to_string())?;
                        speech_cache::report_error(
                            &self.database,
                            &format!("Could not recover speech publication: {error}"),
                        );
                    }
                }
            }
        }
        cursor.clear();
        loop {
            let pending = storage::pending_audio_versions(&self.connection, &cursor)
                .map_err(|e| e.to_string())?;
            if pending.is_empty() {
                break;
            }
            for version in pending {
                cursor.clone_from(&version.locator);
                let result = (|| {
                    if inspector
                        .as_ref()
                        .map_err(Clone::clone)
                        .and_then(|inspector| read_selected(inspector, &version))?
                        .is_some()
                    {
                        storage::complete_audio_version(&self.connection, &version.locator, now()?)
                            .map_err(|e| e.to_string())?;
                    } else {
                        storage::abandon_audio_publication(
                            &self.connection,
                            &version.locator,
                            None,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    Ok::<_, String>(())
                })();
                if let Err(error) = result {
                    self.connection
                        .execute(
                            "UPDATE word_audio_publications SET error=?2 WHERE locator=?1",
                            rusqlite::params![
                                version.locator,
                                error.chars().take(2048).collect::<String>()
                            ],
                        )
                        .map_err(|e| e.to_string())?;
                    speech_cache::report_error(
                        &self.database,
                        &format!("Could not recover audio selection: {error}"),
                    );
                }
            }
        }
        for (id, key) in storage::interrupted_speech(&self.connection).map_err(|e| e.to_string())? {
            if storage::pending_publication(&self.connection, &key)
                .map_err(|e| e.to_string())?
                .is_some()
            {
                continue;
            }
            storage::resume_speech(&self.connection, id, now()?).map_err(|e| e.to_string())?;
        }
        let mut query = self.connection.prepare("SELECT DISTINCT s.desired_key FROM word_audio_selections s JOIN speech_tasks t ON t.semantic_key=s.desired_key WHERE s.desired_generation>0 AND t.state='ready' AND t.refresh_requested=0").map_err(|e| e.to_string())?;
        let ready = query
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        drop(query);
        for key in ready {
            self.adopt_published(&key);
        }
        changed(&self.app);
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        self.recover()?;
        loop {
            self.maintenance()?;
            self.dispatch()?;
            match self.receiver.recv_timeout(self.timeout()?) {
                Ok(Message::Stop) => break,
                Ok(message) => self.handle(message)?,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        // Stop Python before detaching running work. Attempts/deadlines remain persisted.
        self.worker.take();
        for (_, running) in self.running.drain() {
            storage::resume_speech(&self.connection, running.task.id, now()?)
                .map_err(|e| e.to_string())?;
        }
        for (_, replies) in self.waiters.drain() {
            for reply in replies {
                let _ = reply.try_send(Err("Application is closing".into()));
            }
        }
        Ok(())
    }
    fn handle(&mut self, message: Message) -> Result<(), String> {
        match message {
            Message::Wake => self.migration_pending = true,
            Message::Prioritize => (),
            Message::Wait { key, reply } => {
                let inspector = match speech_cache::Inspector::new(&self.database) {
                    Ok(inspector) => inspector,
                    Err(error) => {
                        let _ = reply.try_send(Err(error));
                        return Ok(());
                    }
                };
                let refresh: bool = self
                    .connection
                    .query_row(
                        "SELECT refresh_requested FROM speech_tasks WHERE semantic_key=?1",
                        [&key],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                match if refresh {
                    Ok(None)
                } else {
                    cached_asset(&self.connection, &inspector, &key, now()?)
                } {
                    Ok(Some(bytes)) => {
                        self.adopt_published(&key);
                        let _ = reply.try_send(Ok(bytes));
                    }
                    Err(error) => {
                        let _ = reply.try_send(Err(error));
                    }
                    Ok(None) => {
                        let count: usize = self.waiters.values().map(Vec::len).sum();
                        let replies = self.waiters.entry(key).or_default();
                        if count >= 32 || replies.len() >= 8 {
                            let _ = reply
                                .try_send(Err("Speech subscriber capacity unavailable".into()));
                        } else {
                            replies.push(reply);
                        }
                    }
                }
            }
            Message::Frame {
                epoch,
                header,
                audio,
            } if epoch == self.epoch => {
                if header.id == 0 {
                    if self.ready {
                        self.break_worker("Duplicate speech startup response", true)?;
                    } else {
                        self.ready = true;
                        self.starting = None;
                    }
                } else if let Some(running) = self.running.remove(&header.id) {
                    if header.voice.as_deref() != Some(&running.task.spec.voice) {
                        self.running.insert(header.id, running);
                        self.break_worker("Speech response identity mismatch", true)?;
                    } else if let Some(error) = header.error {
                        storage::fail_speech(
                            &self.connection,
                            running.task.id,
                            now()?,
                            &error,
                            !header.retryable.unwrap_or(false),
                        )
                        .map_err(|e| e.to_string())?;
                        changed(&self.app);
                    } else {
                        let hash = storage::content_hash(&audio);
                        let result = (|| {
                            let version = storage::prepare_source_audio(
                                &self.connection,
                                &running.task.semantic_key,
                                &hash,
                                audio.len() as i64,
                                header
                                    .generated_at
                                    .ok_or("Speech generation timestamp unavailable")?,
                            )
                            .map_err(|e| e.to_string())?;
                            let inspector = speech_cache::Inspector::new(&self.database)?;
                            inspector.publish_version(&version.physical_locator, &hash, &audio)?;
                            let bytes = inspector
                                .read_asset(&version.physical_locator, &hash)?
                                .ok_or("Published audio disappeared")?;
                            if storage::content_hash(&bytes) != hash {
                                return Err("Published speech checksum mismatch".into());
                            }
                            storage::verify_audio_content(
                                &self.connection,
                                &version.locator,
                                bytes.len() as i64,
                            )
                            .map_err(|e| e.to_string())?;
                            storage::complete_source_audio(&self.connection, &version.locator)
                                .map_err(|e| e.to_string())?;
                            self.adopt_published(&running.task.semantic_key);
                            #[cfg(windows)]
                            self.app.state::<SpeechTasks>().maintenance.wake();
                            Ok::<_, String>(())
                        })();
                        if let Err(error) = result {
                            storage::fail_speech(
                                &self.connection,
                                running.task.id,
                                now()?,
                                &error,
                                error != "Speech cache is busy",
                            )
                            .map_err(|e| e.to_string())?;
                        }
                        changed(&self.app);
                    }
                } else {
                    self.break_worker("Unknown or repeated speech response identity", true)?;
                }
            }
            Message::Broken { epoch, error } if epoch == self.epoch => {
                self.break_worker(&error, false)?;
            }
            Message::Frame { .. } | Message::Broken { .. } | Message::Stop => (),
        }
        self.finish_waiters()
    }
    fn dispatch(&mut self) -> Result<(), String> {
        if self.worker.is_none() && !self.settle_cached()? {
            return self.finish_waiters();
        }
        if self.worker.is_none()
            && storage::next_speech_wake(&self.connection)
                .map_err(|e| e.to_string())?
                .is_some_and(|wake| wake <= now().unwrap_or(0))
        {
            self.epoch += 1;
            match Worker::start(
                &self.database,
                &self.resources,
                self.epoch,
                self.sender.clone(),
            ) {
                Ok(worker) => {
                    self.worker = Some(worker);
                    self.starting = Some(Instant::now());
                }
                Err(error) => {
                    self.fail_backlog(&error)?;
                    return self.finish_waiters();
                }
            }
        }
        if !self.ready {
            return Ok(());
        }
        while let Some(task) =
            storage::claim_speech(&mut self.connection, now()?, 2).map_err(|e| e.to_string())?
        {
            let inspector = match speech_cache::Inspector::new(&self.database) {
                Ok(inspector) => inspector,
                Err(error) => {
                    storage::fail_speech(&self.connection, task.id, now()?, &error, true)
                        .map_err(|e| e.to_string())?;
                    changed(&self.app);
                    continue;
                }
            };
            match if task.refresh {
                Ok(None)
            } else {
                cached_asset(&self.connection, &inspector, &task.semantic_key, now()?)
            } {
                Ok(Some(_)) => {
                    storage::ready_speech_task(&self.connection, &task.semantic_key)
                        .map_err(|e| e.to_string())?;
                    changed(&self.app);
                    continue;
                }
                Err(error) => {
                    storage::fail_speech(&self.connection, task.id, now()?, &error, true)
                        .map_err(|e| e.to_string())?;
                    changed(&self.app);
                    continue;
                }
                Ok(None) => (),
            }
            if let Err(error) = validate_worker_spec(&task.spec) {
                storage::fail_speech(&self.connection, task.id, now()?, &error, true)
                    .map_err(|e| e.to_string())?;
                changed(&self.app);
                continue;
            }
            self.request += 1;
            let result = self
                .worker
                .as_mut()
                .ok_or("Speech worker unavailable")?
                .send(self.request, &task);
            self.running.insert(
                self.request,
                Running {
                    budget: Duration::from_secs(
                        task.deadline_at.saturating_sub(now()?).max(0) as u64
                    )
                    .min(Duration::from_secs(60)),
                    task,
                    started: Instant::now(),
                },
            );
            changed(&self.app);
            if let Err(error) = result {
                self.break_worker(&error, false)?;
                break;
            }
        }
        self.finish_waiters()
    }
    fn finish_waiters(&mut self) -> Result<(), String> {
        let keys: Vec<_> = self.waiters.keys().cloned().collect();
        for key in keys {
            let state =
                storage::speech_task_state(&self.connection, &key).map_err(|e| e.to_string())?;
            if let Some((state, error)) = state
                .filter(|(state, _)| matches!(state.as_str(), "ready" | "failed" | "cancelled"))
            {
                let result = if state == "ready" {
                    self.adopt_published(&key);
                    speech_cache::Inspector::new(&self.database)
                        .and_then(|inspector| {
                            cached_asset(&self.connection, &inspector, &key, now()?)
                        })
                        .and_then(|audio| {
                            audio.ok_or_else(|| "Completed speech cache unavailable".to_string())
                        })
                } else {
                    Err(error.unwrap_or_else(|| format!("Speech task {state}")))
                };
                for reply in self.waiters.remove(&key).unwrap_or_default() {
                    let _ = reply.try_send(result.clone());
                }
            }
        }
        Ok(())
    }

    fn settle_cached(&self) -> Result<bool, String> {
        let pending =
            storage::pending_speech_keys(&self.connection, now()?).map_err(|e| e.to_string())?;
        if pending.is_empty() {
            return Ok(false);
        }
        let inspector = match speech_cache::Inspector::new(&self.database) {
            Ok(inspector) => inspector,
            Err(error) => {
                self.fail_backlog(&error)?;
                return Ok(false);
            }
        };
        let mut missing = false;
        for (key, refresh) in pending {
            if refresh {
                missing = true;
                continue;
            }
            let hit = match cached_asset(&self.connection, &inspector, &key, now()?) {
                Ok(Some(_)) => true,
                Ok(None) => {
                    let legacy = storage::speech_owner(&self.connection, &key)
                        .map_err(|e| e.to_string())?
                        .map(|id| inspector.legacy(id))
                        .transpose();
                    match legacy {
                        Ok(audio) => audio.flatten().is_some(),
                        Err(error) => {
                            speech_cache::report_error(&self.database, &error);
                            false
                        }
                    }
                }
                Err(error) => {
                    speech_cache::report_error(&self.database, &error);
                    false
                }
            };
            if hit {
                storage::cached_speech_task(&self.connection, &key).map_err(|e| e.to_string())?;
                self.adopt_published(&key);
                changed(&self.app);
            } else {
                missing = true;
            }
        }
        Ok(missing)
    }
    fn break_worker(&mut self, error: &str, permanent: bool) -> Result<(), String> {
        let startup_failure = !self.ready;
        let diagnostics = self
            .worker
            .as_ref()
            .map(|worker| worker.diagnostics.clone());
        self.worker.take();
        let detail = diagnostics
            .and_then(|buffer| {
                buffer
                    .lock()
                    .ok()
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            })
            .unwrap_or_default();
        let error = if detail.trim().is_empty() {
            error.to_string()
        } else {
            format!("{error}: {}", detail.trim())
        };
        self.ready = false;
        self.starting = None;
        self.epoch += 1;
        for (_, running) in self.running.drain() {
            storage::fail_speech(&self.connection, running.task.id, now()?, &error, permanent)
                .map_err(|e| e.to_string())?;
        }
        if permanent || startup_failure {
            self.fail_backlog(&error)?;
        }
        changed(&self.app);
        self.finish_waiters()
    }
    fn fail_backlog(&self, error: &str) -> Result<(), String> {
        self.connection.execute("UPDATE speech_tasks SET state='failed',error=?1 WHERE state IN ('queued','retry_wait')",[error.chars().take(2048).collect::<String>()]).map_err(|e|e.to_string())?;
        changed(&self.app);
        Ok(())
    }
    fn adopt_published(&self, key: &str) {
        let result = (|| {
            let inspector = speech_cache::Inspector::new(&self.database)?;
            let ids = storage::ready_audio_selections(&self.connection, key)
                .map_err(|e| e.to_string())?;
            for (id, generation) in ids {
                let result = (|| {
                    let time = now()?;
                    if generation == 0 {
                        cached_word(&self.connection, &inspector, id, time)?;
                    } else {
                        let asset = storage::speech_asset(&self.connection, key, time)
                            .map_err(|e| e.to_string())?
                            .ok_or("Published audio disappeared")?;
                        cached_asset(&self.connection, &inspector, key, time)?
                            .ok_or("Published audio file disappeared")?;
                        if !storage::reuse_audio_version(
                            &self.connection,
                            id,
                            &asset.locator,
                            generation,
                            time,
                        )
                        .map_err(|e| e.to_string())?
                        {
                            return Err("Audio selection was superseded; retry".into());
                        }
                        storage::finish_audio_import(&self.connection, id, None)
                            .map_err(|e| e.to_string())?;
                    }
                    Ok::<_, String>(())
                })();
                if let Err(error) = result {
                    speech_cache::report_error(
                        &self.database,
                        &format!("Could not select published word audio: {error}"),
                    );
                }
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            speech_cache::report_error(
                &self.database,
                &format!("Could not select published audio: {error}"),
            );
        }
    }
    fn migrate_audio(&mut self) -> Result<(), String> {
        let ids = storage::pending_audio_imports(&self.connection).map_err(|e| e.to_string())?;
        self.migration_pending = ids.len() == 16;
        let inspector = match speech_cache::Inspector::new(&self.database) {
            Ok(inspector) => inspector,
            Err(error) => {
                self.migration_pending = false;
                for id in ids {
                    storage::finish_audio_import(&self.connection, id, Some(&error))
                        .map_err(|e| e.to_string())?;
                }
                speech_cache::report_error(
                    &self.database,
                    &format!("Could not open audio migration cache: {error}"),
                );
                changed(&self.app);
                return Ok(());
            }
        };
        for id in ids {
            let result = cached_word(&self.connection, &inspector, id, now()?);
            if let Err(error) = &result {
                speech_cache::report_error(
                    &self.database,
                    &format!("Could not migrate word audio: {error}"),
                );
            }
            storage::finish_audio_import(&self.connection, id, result.err().as_deref())
                .map_err(|e| e.to_string())?;
        }
        changed(&self.app);
        Ok(())
    }
    fn maintenance(&mut self) -> Result<(), String> {
        if self.migration_pending {
            self.migrate_audio()?;
        }
        let planned: Vec<_> = {
            let mut planned = self
                .planned
                .lock()
                .map_err(|_| "Speech demand planner unavailable")?;
            let count = planned.len().min(16);
            planned.drain(..count).collect()
        };
        let mut updated = false;
        for demand in planned {
            updated |= storage::demand_word_repair(&mut self.connection, &demand, now()?)
                .map_err(|e| e.to_string())?;
        }
        if updated {
            changed(&self.app);
        }
        if self
            .starting
            .is_some_and(|start| start.elapsed() >= Duration::from_secs(30))
        {
            self.break_worker("Offline model startup timed out", true)?;
        }
        if self
            .running
            .values()
            .any(|run| run.started.elapsed() >= run.budget)
        {
            self.break_worker("Offline synthesis execution timed out", false)?;
        }
        Ok(())
    }
    fn timeout(&self) -> Result<Duration, String> {
        let mut delay = Duration::from_secs(6 * 60 * 60);
        if self.migration_pending {
            delay = delay.min(Duration::from_millis(50));
        }
        if !self
            .planned
            .lock()
            .map_err(|_| "Speech demand planner unavailable")?
            .is_empty()
        {
            delay = delay.min(Duration::from_millis(50));
        }
        if let Some(start) = self.starting {
            delay = delay.min(Duration::from_secs(30).saturating_sub(start.elapsed()));
        }
        for running in self.running.values() {
            delay = delay.min(running.budget.saturating_sub(running.started.elapsed()));
        }
        if self.starting.is_none() {
            if let Some(wake) =
                storage::next_speech_wake(&self.connection).map_err(|e| e.to_string())?
            {
                delay = delay.min(Duration::from_secs(
                    wake.saturating_sub(now()?).max(0) as u64
                ));
            }
        }
        Ok(delay.max(Duration::from_millis(1)))
    }
}

#[cfg(windows)]
mod job {
    use std::{
        ffi::c_void,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::Child,
    };
    // Windows JOBOBJECT_EXTENDED_LIMIT_INFORMATION; pointers/size_t use native width.
    #[repr(C)]
    #[derive(Default)]
    struct Basic {
        process_time: i64,
        job_time: i64,
        flags: u32,
        min: usize,
        max: usize,
        active: u32,
        affinity: usize,
        priority: u32,
        scheduling: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Extended {
        basic: Basic,
        io: [u64; 6],
        process_memory: usize,
        job_memory: usize,
        peak_process: usize,
        peak_job: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> *mut c_void;
        fn SetInformationJobObject(
            job: *mut c_void,
            class: u32,
            info: *const c_void,
            length: u32,
        ) -> i32;
        fn AssignProcessToJobObject(job: *mut c_void, process: *mut c_void) -> i32;
    }
    pub struct Job(OwnedHandle);
    impl Job {
        pub fn attach(child: &Child) -> Result<Self, String> {
            // SAFETY: unnamed job, no inherited handles; owned handle closes on all paths.
            let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if raw.is_null() {
                return Err(format!(
                    "Could not create speech process job: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let job = Self(unsafe { OwnedHandle::from_raw_handle(raw) });
            let mut limits = Extended::default();
            limits.basic.flags = 0x2000; // KILL_ON_JOB_CLOSE
            let success = unsafe {
                SetInformationJobObject(
                    job.0.as_raw_handle(),
                    9,
                    (&limits as *const Extended).cast(),
                    std::mem::size_of::<Extended>() as u32,
                )
            };
            if success == 0
                || unsafe { AssignProcessToJobObject(job.0.as_raw_handle(), child.as_raw_handle()) }
                    == 0
            {
                return Err(format!(
                    "Could not contain speech process: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(job)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    fn cached_word_finishes_pending_initial_intent_before_adopting_a_source() {
        let root = std::env::temp_dir().join(format!(
            "lexique-pending-initial-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let data = root.join("Lexique-data/data");
        std::fs::create_dir_all(&data).unwrap();
        let database = data.join("lexique.sqlite3");
        let connection = Connection::open(&database).unwrap();
        storage::initialize(&connection).unwrap();
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(1,'école','{}','keep',1,1)", []).unwrap();
        let key = storage::register_speech(
            &connection,
            &storage::SpeechSpec::word("école", storage::SpeechVoice::Female),
        )
        .unwrap();
        connection
            .execute("INSERT INTO speech_owners VALUES(?1,1)", [&key])
            .unwrap();
        let source_bytes = b"OggSsource";
        let source = storage::prepare_source_audio(
            &connection,
            &key,
            &storage::content_hash(source_bytes),
            source_bytes.len() as i64,
            1000,
        )
        .unwrap();
        let inspector = speech_cache::Inspector::new(&database).unwrap();
        inspector
            .publish_version(&source.physical_locator, &source.content_hash, source_bytes)
            .unwrap();
        storage::verify_audio_content(&connection, &source.locator, source_bytes.len() as i64)
            .unwrap();
        storage::complete_source_audio(&connection, &source.locator).unwrap();
        let intended = b"OggSintended";
        let pending = storage::prepare_audio_version(
            &connection,
            1,
            Some(&key),
            &storage::content_hash(intended),
            intended.len() as i64,
            Some(1000),
            0,
            1001,
        )
        .unwrap();
        assert!(
            cached_word(&connection, &inspector, 1, 1002)
                .unwrap()
                .is_none(),
            "an unfinished live publication must retain its intent"
        );
        assert!(storage::selected_audio(&connection, 1).unwrap().is_none());
        inspector
            .publish_version(&pending.physical_locator, &pending.content_hash, intended)
            .unwrap();
        assert_eq!(
            cached_word(&connection, &inspector, 1, 1002)
                .unwrap()
                .unwrap(),
            intended
        );
        assert_eq!(
            storage::selected_audio(&connection, 1)
                .unwrap()
                .unwrap()
                .locator,
            pending.locator
        );
        assert!(
            storage::pending_word_audio_versions(&connection, 1)
                .unwrap()
                .is_empty()
        );
        drop(inspector);
        drop(connection);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn frame(header: serde_json::Value, audio: &[u8]) -> Vec<u8> {
        let header = serde_json::to_vec(&header).unwrap();
        [&(header.len() as u32).to_le_bytes()[..], &header, audio].concat()
    }
    #[test]
    fn private_frames_reject_unbounded_unknown_and_mismatched_payloads() {
        assert!(
            read_frame(
                &mut &frame(
                    serde_json::json!({"id":1,"audio_bytes":4,"voice":"female","language":"fr","generated_at":1000}),
                    b"OggS"
                )[..]
            )
            .is_ok()
        );
        for header in [
            serde_json::json!({"id":1,"audio_bytes":10000001}),
            serde_json::json!({"id":0,"audio_bytes":0,"ready":true,"text":"secret"}),
            serde_json::json!({"id":1,"audio_bytes":4,"voice":"male","language":"en"}),
        ] {
            assert!(read_frame(&mut &frame(header, b"OggS")[..]).is_err());
        }
        assert!(
            read_frame(
                &mut &frame(
                    serde_json::json!({"id":1,"audio_bytes":4,"voice":"female","language":"fr"}),
                    b"Ogg"
                )[..]
            )
            .is_err()
        );
    }
}
