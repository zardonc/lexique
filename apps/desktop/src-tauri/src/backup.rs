//! Versioned portable files. Native paths and archive bytes never cross the WebView boundary.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use lexique_storage::{self as storage, AudioPayloadStatus, LearningSnapshot};
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, ZipArchive, ZipReadOptions, ZipWriter, write::SimpleFileOptions};

use crate::{
    maintenance::MaintenanceCoordinator, speech_tasks::SpeechTasks, vocabulary::DatabasePath,
};

const SAFE_INTEGER: u64 = 9_007_199_254_740_991;
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupPhase {
    Draining,
    Collecting,
    Packaging,
    Validating,
    Publishing,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BackupCounts {
    pub vocabulary: u64,
    pub cards: u64,
    pub ratings: u64,
    pub batches: u64,
    pub custom_dictionaries: u64,
    pub legacy_audio: u64,
    pub included_audio_versions: u64,
    pub included_legacy_audio: u64,
    pub audio_files: u64,
    pub source_missing_audio: u64,
    pub source_invalid_audio: u64,
}

impl BackupCounts {
    pub fn from_snapshot(snapshot: &LearningSnapshot) -> Self {
        let statuses = snapshot
            .audio_versions
            .iter()
            .map(|audio| audio.payload_status)
            .chain(
                snapshot
                    .legacy_audio
                    .iter()
                    .map(|audio| audio.payload_status),
            )
            .collect::<Vec<_>>();
        Self {
            vocabulary: snapshot.records.len() as u64,
            cards: snapshot.cards.len() as u64,
            ratings: snapshot.ratings.len() as u64,
            batches: snapshot.batches.len() as u64,
            custom_dictionaries: snapshot
                .dictionary_catalog
                .entries
                .iter()
                .filter(|entry| storage::custom_dictionary_number(&entry.id).is_some())
                .count() as u64,
            legacy_audio: snapshot.legacy_audio.len() as u64,
            included_audio_versions: snapshot
                .audio_versions
                .iter()
                .filter(|audio| audio.payload_status == AudioPayloadStatus::Included)
                .count() as u64,
            included_legacy_audio: snapshot
                .legacy_audio
                .iter()
                .filter(|audio| audio.payload_status == AudioPayloadStatus::Included)
                .count() as u64,
            audio_files: included_hashes(snapshot).len() as u64,
            source_missing_audio: statuses
                .iter()
                .filter(|status| **status == AudioPayloadStatus::SourceMissing)
                .count() as u64,
            source_invalid_audio: statuses
                .iter()
                .filter(|status| **status == AudioPayloadStatus::SourceInvalid)
                .count() as u64,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackupManifest {
    pub kind: String,
    pub format_version: u32,
    pub snapshot_version: u32,
    pub created_at: i64,
    pub application_version: String,
    pub counts: BackupCounts,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackupResult {
    pub destination: String,
    pub byte_length: u64,
    pub created_at: i64,
    pub counts: BackupCounts,
}

pub struct ValidatedBackup {
    pub byte_length: u64,
    pub manifest: BackupManifest,
    pub snapshot: LearningSnapshot,
    pub audio_directory: PathBuf,
    pub audio_lengths: BTreeMap<String, u64>,
}

pub struct OwnedDirectory(pub PathBuf);

impl OwnedDirectory {
    pub fn create(parent: &Path) -> Result<Self, String> {
        for _ in 0..10 {
            let name = format!(
                ".lexique-backup-{}-{}-{}",
                std::process::id(),
                crate::speech_tasks::now()?,
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            );
            let path = parent.join(name);
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => {
                    return Err(format!(
                        "Could not create backup staging directory: {error}"
                    ));
                }
            }
        }
        Err("Could not reserve a new backup staging directory".into())
    }

    pub(crate) fn keep(mut self) -> PathBuf {
        let path = std::mem::take(&mut self.0);
        std::mem::forget(self);
        path
    }
}

impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("Could not clean backup staging directory: {error}");
        }
    }
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        Err("backup_cancelled".into())
    } else {
        Ok(())
    }
}

pub(crate) struct CancelledReader<'a, R> {
    pub(crate) reader: R,
    pub(crate) cancelled: &'a AtomicBool,
}
impl<R: Read> Read for CancelledReader<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::other("backup_cancelled"));
        }
        let length = bytes.len().min(64 * 1024);
        self.reader.read(&mut bytes[..length])
    }
}

struct CancelledWriter<'a, W> {
    writer: W,
    cancelled: &'a AtomicBool,
}
impl<W: Write> Write for CancelledWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::other("backup_cancelled"));
        }
        self.writer.write(&bytes[..bytes.len().min(64 * 1024)])
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::other("backup_cancelled"));
        }
        self.writer.flush()
    }
}
impl<W: Seek> Seek for CancelledWriter<'_, W> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(io::Error::other("backup_cancelled"));
        }
        self.writer.seek(position)
    }
}

fn included_hashes(snapshot: &LearningSnapshot) -> BTreeSet<&str> {
    snapshot
        .audio_versions
        .iter()
        .filter(|audio| audio.payload_status == AudioPayloadStatus::Included)
        .map(|audio| audio.content_hash.as_str())
        .chain(
            snapshot
                .legacy_audio
                .iter()
                .filter(|audio| audio.payload_status == AudioPayloadStatus::Included)
                .map(|audio| audio.content_hash.as_str()),
        )
        .collect()
}

fn protected_audio(
    directory: &Path,
    hash: &str,
    bytes: &[u8],
    copied: &mut BTreeSet<String>,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    check_cancelled(cancelled)?;
    if copied.contains(hash) {
        return Ok(());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(format!("{hash}.ogg")))
        .map_err(|e| format!("Could not create protected audio copy: {e}"))?;
    CancelledWriter {
        writer: &mut file,
        cancelled,
    }
    .write_all(bytes)
    .and_then(|()| file.sync_all())
    .map_err(|e| format!("Could not copy and sync selected audio: {e}"))?;
    check_cancelled(cancelled)?;
    copied.insert(hash.into());
    Ok(())
}

fn inspect_version(
    connection: &rusqlite::Connection,
    inspector: &crate::speech_cache::Inspector,
    audio: &storage::SnapshotAudioVersion,
) -> Result<crate::speech_cache::AudioInspection, String> {
    let physical: String = connection
        .query_row(
            "SELECT physical_locator FROM speech_versions WHERE locator=?1",
            [&audio.locator],
            |row| row.get(0),
        )
        .map_err(|e| format!("Could not locate selected audio: {e}"))?;
    inspector
        .collect_version(&physical, &audio.content_hash, audio.byte_length)
        .map_err(|e| format!("Could not inspect selected audio: {e}"))
}

fn collect_version(
    audio: &mut storage::SnapshotAudioVersion,
    inspection: crate::speech_cache::AudioInspection,
    directory: &Path,
    copied: &mut BTreeSet<String>,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    use crate::speech_cache::AudioInspection;
    audio.exclusion_reason = None;
    match inspection {
        AudioInspection::Present(bytes) => {
            protected_audio(directory, &audio.content_hash, &bytes, copied, cancelled)?;
            audio.payload_status = AudioPayloadStatus::Included;
            audio.byte_length = Some(bytes.len() as i64);
            audio.verification = "verified".into();
        }
        AudioInspection::Missing => {
            audio.payload_status = AudioPayloadStatus::SourceMissing;
            audio.verification = "missing".into();
        }
        AudioInspection::Invalid { .. } => {
            audio.payload_status = AudioPayloadStatus::SourceInvalid;
            audio.verification = "invalid".into();
        }
    }
    Ok(())
}

fn collect_audio(
    connection: &rusqlite::Connection,
    database: &DatabasePath,
    snapshot: &mut LearningSnapshot,
    directory: &Path,
    captured_at: i64,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    use crate::speech_cache::{AudioInspection, Inspector};
    let audio_directory = directory.join("audio");
    fs::create_dir(&audio_directory).map_err(|e| format!("Could not stage selected audio: {e}"))?;
    let inspector = Inspector::new(&database.0)?;
    let mut copied = BTreeSet::new();
    for audio in &mut snapshot.audio_versions {
        if audio.payload_status == AudioPayloadStatus::Excluded {
            continue;
        }
        check_cancelled(cancelled)?;
        let inspection = inspect_version(connection, &inspector, audio)?;
        collect_version(audio, inspection, &audio_directory, &mut copied, cancelled)?;
    }

    let initial: Vec<_> = snapshot
        .records
        .iter()
        .filter(|record| storage::initial_snapshot_audio_allowed(snapshot, record.id))
        .map(|record| record.id)
        .collect();
    let assets = storage::word_audio_assets(connection, &initial, captured_at)
        .map_err(|e| format!("Could not resolve initial audio candidates: {e}"))?;
    for id in initial {
        check_cancelled(cancelled)?;
        let mut resolved = false;
        let mut first_missing = None;
        for asset in assets.get(&id).into_iter().flatten() {
            let index = snapshot
                .audio_versions
                .iter()
                .position(|version| version.locator == asset.locator)
                .ok_or("Initial audio version is outside the confirmed snapshot")?;
            let audio = &snapshot.audio_versions[index];
            if !audio.sealed || audio.context != "word" || audio.gc_state != "live" {
                return Err("Initial owned audio is not sealed live word content".into());
            }
            let inspection = inspect_version(connection, &inspector, audio)?;
            if matches!(inspection, AudioInspection::Missing) {
                first_missing.get_or_insert(index);
                continue;
            }
            collect_version(
                &mut snapshot.audio_versions[index],
                inspection,
                &audio_directory,
                &mut copied,
                cancelled,
            )?;
            resolved = true;
            break;
        }
        if resolved {
            continue;
        }

        let registered = snapshot
            .legacy_audio
            .iter()
            .position(|audio| audio.vocabulary_id == id);
        let inspection = inspector
            .collect_legacy(id)
            .map_err(|e| format!("Could not inspect legacy audio: {e}"))?;
        let identity = match &inspection {
            AudioInspection::Present(bytes) => Some(storage::content_hash(bytes)),
            AudioInspection::Invalid { observed_hash, .. } => observed_hash.clone(),
            AudioInspection::Missing => None,
        };
        let index = match registered {
            Some(index) => Some(index),
            None => identity.map(|hash| {
                snapshot.legacy_audio.push(storage::SnapshotLegacyAudio {
                    vocabulary_id: id,
                    content_hash: hash,
                    payload_status: AudioPayloadStatus::Excluded,
                    exclusion_reason: Some(storage::AudioExclusionReason::NotSelected),
                });
                snapshot.legacy_audio.len() - 1
            }),
        };
        if matches!(inspection, AudioInspection::Missing) && first_missing.is_some() {
            collect_version(
                &mut snapshot.audio_versions[first_missing.unwrap()],
                AudioInspection::Missing,
                &audio_directory,
                &mut copied,
                cancelled,
            )?;
            continue;
        }
        if let Some(index) = index {
            let audio = &mut snapshot.legacy_audio[index];
            audio.exclusion_reason = None;
            match inspection {
                AudioInspection::Present(bytes)
                    if storage::content_hash(&bytes) == audio.content_hash =>
                {
                    protected_audio(
                        &audio_directory,
                        &audio.content_hash,
                        &bytes,
                        &mut copied,
                        cancelled,
                    )?;
                    audio.payload_status = AudioPayloadStatus::Included;
                }
                AudioInspection::Missing => {
                    audio.payload_status = AudioPayloadStatus::SourceMissing
                }
                AudioInspection::Present(_) | AudioInspection::Invalid { .. } => {
                    audio.payload_status = AudioPayloadStatus::SourceInvalid
                }
            }
        } else if let Some(index) = first_missing {
            // No content identity is invented for an unregistered non-audio path.
            collect_version(
                &mut snapshot.audio_versions[index],
                AudioInspection::Missing,
                &audio_directory,
                &mut copied,
                cancelled,
            )?;
        }
    }
    snapshot
        .legacy_audio
        .sort_by_key(|audio| audio.vocabulary_id);
    Ok(())
}

/// `staging` must be a newly created, empty directory owned by the caller. No archive path is extracted.
pub fn validate_backup(
    archive_path: &Path,
    staging: &Path,
    cancelled: &AtomicBool,
) -> Result<ValidatedBackup, String> {
    check_cancelled(cancelled)?;
    let metadata =
        fs::symlink_metadata(staging).map_err(|e| format!("Backup staging directory: {e}"))?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || fs::read_dir(staging)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err("Backup validation requires an empty owned staging directory".into());
    }
    let file = File::open(archive_path).map_err(|e| format!("Could not read backup: {e}"))?;
    let byte_length = file
        .metadata()
        .map_err(|e| format!("Could not read backup size: {e}"))?
        .len();
    if !(1..=SAFE_INTEGER).contains(&byte_length) {
        return Err("Unsupported backup file size".into());
    }
    let mut archive =
        ZipArchive::new(BufReader::new(file)).map_err(|e| format!("Invalid backup ZIP: {e}"))?;
    if archive.has_duplicate_names() {
        return Err("Duplicate backup ZIP entry".into());
    }
    archive
        .validate_header_consistency()
        .map_err(|e| format!("Invalid backup ZIP headers: {e}"))?;
    let mut entries = BTreeMap::new();
    for index in 0..archive.len() {
        check_cancelled(cancelled)?;
        let entry = archive
            .by_index_with_options(
                index,
                ZipReadOptions::new().enforce_header_consistency(true),
            )
            .map_err(|e| format!("Invalid backup entry: {e}"))?;
        let name =
            std::str::from_utf8(entry.name_raw()).map_err(|_| "Invalid backup entry name")?;
        let audio = name
            .strip_prefix("audio/")
            .and_then(|name| name.strip_suffix(".ogg"));
        let valid_name = matches!(name, "manifest.json" | "snapshot.json")
            || audio.is_some_and(|hash| {
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            });
        let file_mode = entry.unix_mode().unwrap_or(0) & 0o170000;
        if !valid_name
            || entry.compression() != CompressionMethod::Stored
            || entry.encrypted()
            || !entry.is_file()
            || entry.is_symlink()
            || !matches!(file_mode, 0 | 0o100000)
            || entry.size() != entry.compressed_size()
        {
            return Err("Unsupported backup ZIP entry".into());
        }
        if entries.insert(name.to_owned(), index).is_some() {
            return Err("Duplicate backup ZIP entry".into());
        }
    }
    let manifest_index = *entries
        .get("manifest.json")
        .ok_or("Backup manifest is missing")?;
    let manifest_file = archive
        .by_index_with_options(
            manifest_index,
            ZipReadOptions::new().enforce_header_consistency(true),
        )
        .map_err(|e| e.to_string())?;
    if manifest_file.size() > 64 * 1024 {
        return Err("Backup manifest is too large".into());
    }
    let manifest: BackupManifest = serde_json::from_reader(CancelledReader {
        reader: manifest_file,
        cancelled,
    })
    .map_err(|e| format!("Invalid backup manifest: {e}"))?;
    if manifest.kind != "lexique-backup"
        || manifest.format_version != 1
        || manifest.snapshot_version != 1
        || !(0..=SAFE_INTEGER as i64).contains(&manifest.created_at)
        || manifest.application_version.trim().is_empty()
        || manifest.application_version.len() > 200
    {
        return Err("Unsupported backup format or manifest".into());
    }
    let snapshot_index = *entries
        .get("snapshot.json")
        .ok_or("Backup snapshot is missing")?;
    let snapshot_file = archive
        .by_index_with_options(
            snapshot_index,
            ZipReadOptions::new().enforce_header_consistency(true),
        )
        .map_err(|e| e.to_string())?;
    let snapshot = storage::read_learning_snapshot(
        CancelledReader {
            reader: snapshot_file,
            cancelled,
        },
        manifest.created_at,
    )
    .map_err(|e| format!("Invalid backup snapshot: {e}"))?;
    if manifest.counts != BackupCounts::from_snapshot(&snapshot) {
        return Err("Backup counts do not match the snapshot".into());
    }
    let hashes = included_hashes(&snapshot);
    if entries.len() != 2 + hashes.len()
        || hashes
            .iter()
            .any(|hash| !entries.contains_key(&format!("audio/{hash}.ogg")))
    {
        return Err("Backup audio entries do not match the snapshot".into());
    }
    let audio_directory = staging.join("audio");
    fs::create_dir(&audio_directory)
        .map_err(|e| format!("Could not create validated audio directory: {e}"))?;
    let mut audio_lengths = BTreeMap::new();
    for hash in hashes {
        check_cancelled(cancelled)?;
        let entry = archive
            .by_index_with_options(
                entries[&format!("audio/{hash}.ogg")],
                ZipReadOptions::new().enforce_header_consistency(true),
            )
            .map_err(|e| e.to_string())?;
        if !(4..=10_000_000).contains(&entry.size()) {
            return Err("Invalid included audio size".into());
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        CancelledReader {
            reader: entry,
            cancelled,
        }
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Invalid included audio CRC or bytes: {e}"))?;
        if !bytes.starts_with(b"OggS")
            || storage::content_hash(&bytes) != hash
            || snapshot
                .audio_versions
                .iter()
                .filter(|audio| {
                    audio.payload_status == AudioPayloadStatus::Included
                        && audio.content_hash == hash
                })
                .any(|audio| audio.byte_length != Some(bytes.len() as i64))
        {
            return Err("Included audio content identity or length mismatch".into());
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(audio_directory.join(format!("{hash}.ogg")))
            .map_err(|e| format!("Could not stage validated audio: {e}"))?;
        CancelledWriter {
            writer: &mut file,
            cancelled,
        }
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("Could not write validated audio: {e}"))?;
        audio_lengths.insert(hash.to_owned(), bytes.len() as u64);
    }
    check_cancelled(cancelled)?;
    Ok(ValidatedBackup {
        byte_length,
        manifest,
        snapshot,
        audio_directory,
        audio_lengths,
    })
}

fn package(
    directory: &Path,
    manifest: &BackupManifest,
    snapshot: &LearningSnapshot,
    cancelled: &AtomicBool,
) -> Result<PathBuf, String> {
    let path = directory.join("archive.pending");
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("Could not stage backup file: {e}"))?;
    let mut archive = ZipWriter::new(CancelledWriter {
        writer: BufWriter::new(file),
        cancelled,
    });
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true)
        .unix_permissions(0o600);
    archive
        .start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut archive, manifest)
        .map_err(|e| format!("Could not write backup manifest: {e}"))?;
    check_cancelled(cancelled)?;
    archive
        .start_file("snapshot.json", options)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut archive, snapshot)
        .map_err(|e| format!("Could not write backup snapshot: {e}"))?;
    for hash in included_hashes(snapshot) {
        check_cancelled(cancelled)?;
        archive
            .start_file(format!("audio/{hash}.ogg"), options)
            .map_err(|e| e.to_string())?;
        let source = File::open(directory.join("audio").join(format!("{hash}.ogg")))
            .map_err(|e| format!("Could not read protected audio copy: {e}"))?;
        io::copy(
            &mut CancelledReader {
                reader: source,
                cancelled,
            },
            &mut archive,
        )
        .map_err(|e| format!("Could not package protected audio: {e}"))?;
    }
    let mut file = archive
        .finish()
        .map_err(|e| format!("Could not finish backup ZIP: {e}"))?;
    file.flush()
        .and_then(|()| file.writer.get_ref().sync_all())
        .map_err(|e| format!("Could not sync backup file: {e}"))?;
    Ok(path)
}

#[cfg(windows)]
fn publish_new(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(source: *const u16, destination: *const u16, flags: u32) -> i32;
    }
    let source = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    // MOVEFILE_WRITE_THROUGH, without MOVEFILE_REPLACE_EXISTING. Both paths are on the destination filesystem.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0x8) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn publish_new(source: &Path, destination: &Path) -> io::Result<()> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    unsafe extern "C" {
        fn renamex_np(
            source: *const std::ffi::c_char,
            destination: *const std::ffi::c_char,
            flags: u32,
        ) -> i32;
    }
    let parent = File::open(
        destination
            .parent()
            .ok_or_else(|| io::Error::other("Backup destination has no parent"))?,
    )?;
    let source_c = CString::new(source.as_os_str().as_bytes())?;
    let destination_c = CString::new(destination.as_os_str().as_bytes())?;
    // Darwin RENAME_EXCL, as used by the BSD filesystem boundary: existing names are never replaced.
    if unsafe { renamex_np(source_c.as_ptr(), destination_c.as_ptr(), 0x4) } != 0 {
        return Err(io::Error::last_os_error());
    }
    parent.sync_all().map_err(|error| io::Error::new(error.kind(), format!("A complete validated backup exists at {}; its directory could not be synced: {error}", destination.display())))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn publish_new(source: &Path, destination: &Path) -> io::Result<()> {
    let parent = File::open(
        destination
            .parent()
            .ok_or_else(|| io::Error::other("Backup destination has no parent"))?,
    )?;
    fs::hard_link(source, destination)?;
    parent.sync_all().map_err(|error| io::Error::new(error.kind(), format!("A complete validated backup exists at {}; its directory could not be synced: {error}", destination.display())))?;
    if let Err(error) = fs::remove_file(source) {
        eprintln!("Could not remove published backup staging link: {error}");
    }
    Ok(())
}

pub fn export_backup_to_new_file(
    database: &DatabasePath,
    workers: &SpeechTasks,
    maintenance: &MaintenanceCoordinator,
    destination: &Path,
    _operation_id: &str,
    cancelled: &AtomicBool,
    phase: impl Fn(BackupPhase),
) -> Result<BackupResult, String> {
    if !destination.is_absolute() || destination.file_name().is_none() {
        return Err("Backup destination must be an absolute new file path".into());
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => return Err("Backup destination already exists; choose a new file".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(format!("Could not inspect backup destination: {error}")),
    }
    let directory = OwnedDirectory::create(
        destination
            .parent()
            .ok_or("Backup destination has no parent")?,
    )?;
    phase(BackupPhase::Draining);
    let admission = maintenance.begin(cancelled)?;
    let pause = workers.pause(cancelled)?;
    phase(BackupPhase::Collecting);
    check_cancelled(cancelled)?;
    let captured_at = crate::speech_tasks::now()?;
    crate::dictionaries::portable_catalog(database)
        .map_err(|error| format!("Could not normalize portable dictionary catalog: {error}"))?;
    let mut connection = crate::vocabulary::open(database)?;
    let mut snapshot = storage::export_learning_snapshot(&mut connection, captured_at)
        .map_err(|e| format!("Could not collect learning state: {e}"))?;
    collect_audio(
        &connection,
        database,
        &mut snapshot,
        &directory.0,
        captured_at,
        cancelled,
    )?;
    storage::validate_learning_snapshot(&snapshot, captured_at)
        .map_err(|e| format!("Invalid collected learning state: {e}"))?;
    drop(connection);
    drop(pause);
    drop(admission);
    phase(BackupPhase::Packaging);
    let manifest = BackupManifest {
        kind: "lexique-backup".into(),
        format_version: 1,
        snapshot_version: 1,
        created_at: captured_at,
        application_version: env!("CARGO_PKG_VERSION").into(),
        counts: BackupCounts::from_snapshot(&snapshot),
    };
    let path = package(&directory.0, &manifest, &snapshot, cancelled)?;
    phase(BackupPhase::Validating);
    let validation = directory.0.join("validation");
    fs::create_dir(&validation).map_err(|e| format!("Could not stage backup readback: {e}"))?;
    validate_backup(&path, &validation, cancelled)?;
    let byte_length = fs::metadata(&path).map_err(|e| e.to_string())?.len();
    phase(BackupPhase::Publishing);
    check_cancelled(cancelled)?;
    publish_new(&path, destination)
        .map_err(|e| format!("Could not publish new backup file: {e}"))?;
    Ok(BackupResult {
        destination: destination.to_string_lossy().into_owned(),
        byte_length,
        created_at: captured_at,
        counts: manifest.counts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        maintenance::MaintenanceCoordinator, speech_tasks::SpeechTasks, vocabulary::DatabasePath,
    };
    use std::{
        path::PathBuf,
        sync::{Arc, atomic::AtomicBool, mpsc},
        thread,
        time::{Duration, Instant},
    };

    struct Directory(PathBuf);
    struct Workers(Option<SpeechTasks>);
    impl std::ops::Deref for Workers {
        type Target = SpeechTasks;
        fn deref(&self) -> &SpeechTasks {
            self.0.as_ref().unwrap()
        }
    }
    impl Workers {
        fn shutdown(mut self) -> Result<(), String> {
            self.0.take().unwrap().shutdown()
        }
    }
    impl Drop for Workers {
        fn drop(&mut self) {
            if let Some(workers) = self.0.take() {
                let _ = workers.shutdown();
            }
        }
    }
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "lexique-backup-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(path.join("Lexique-data/data")).unwrap();
            Self(path)
        }
        fn database(&self) -> DatabasePath {
            DatabasePath(self.0.join("Lexique-data/data/lexique.sqlite3"))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn save_word(connection: &mut rusqlite::Connection, headword: &str) -> i64 {
        let result = storage::save(
            connection,
            &storage::SaveRequest {
                operation: format!("fixture-{}", NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)),
                headword: headword.into(),
                notes: "Mes notes\n个人笔记".into(),
                example_fr: "Une école.".into(),
                example_zh: "学校".into(),
                example_source: None,
                example_sense: Some(0),
                more_examples: vec![],
                id: None,
                revision: None,
            },
            Some(storage::Snapshot {
                reference: storage::Reference {
                    dictionary_id: None,
                    dictionary: "Synthetic dictionary".into(),
                    lookup_key: headword.into(),
                    version: "1".into(),
                    attribution: None,
                    license: Some("MIT".into()),
                    source_url: None,
                    license_url: None,
                    sha256: "a".repeat(64),
                },
                digest: vec![storage::DigestEntry {
                    pronunciations: vec![],
                    grammar: vec!["n".into()],
                    translation: Some("学校".into()),
                }],
            }),
        )
        .unwrap();
        result.record.id
    }

    fn selected_fixture(
        connection: &rusqlite::Connection,
        inspector: &crate::speech_cache::Inspector,
        id: i64,
        word: &str,
        bytes: &[u8],
        time: i64,
    ) -> storage::AudioVersion {
        let key = storage::register_speech(
            connection,
            &storage::SpeechSpec::word(word, storage::SpeechVoice::Female),
        )
        .unwrap();
        let generation = if storage::selected_audio(connection, id).unwrap().is_some() {
            storage::begin_audio_change(connection, id, &key).unwrap()
        } else {
            0
        };
        let hash = storage::content_hash(bytes);
        let version = storage::prepare_audio_version(
            connection,
            id,
            Some(&key),
            storage::AudioPublicationContent {
                hash: &hash,
                bytes: bytes.len() as i64,
                generated_at: Some(time),
            },
            generation,
            time,
        )
        .unwrap();
        inspector
            .publish_version(&version.physical_locator, &hash, bytes)
            .unwrap();
        storage::verify_audio_content(connection, &version.locator, bytes.len() as i64).unwrap();
        assert!(storage::complete_audio_version(connection, &version.locator, time).unwrap());
        storage::attach_word_audio_at(connection, id, time).unwrap();
        version
    }

    fn source_fixture(
        connection: &rusqlite::Connection,
        inspector: &crate::speech_cache::Inspector,
        spec: &storage::SpeechSpec,
        bytes: &[u8],
        time: i64,
    ) -> storage::AudioVersion {
        let key = storage::register_speech(connection, spec).unwrap();
        let hash = storage::content_hash(bytes);
        let version =
            storage::prepare_source_audio(connection, &key, &hash, bytes.len() as i64, time)
                .unwrap();
        inspector
            .publish_version(&version.physical_locator, &hash, bytes)
            .unwrap();
        storage::verify_audio_content(connection, &version.locator, bytes.len() as i64).unwrap();
        assert!(storage::complete_source_audio(connection, &version.locator).unwrap());
        version
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !condition() {
            assert!(
                Instant::now() < deadline,
                "native worker did not reach the expected state"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn staging_directory(parent: &Path) -> PathBuf {
        fs::read_dir(parent)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".lexique-backup-")
            })
            .unwrap()
    }

    fn assert_no_staging(parent: &Path) {
        assert!(!fs::read_dir(parent).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".lexique-backup-")
        }));
    }

    #[test]
    fn preview_stages_empty_and_saved_learning_in_an_owned_root_without_live_changes() {
        for vocabulary_count in [0, 1] {
            let directory = Directory::new();
            let database = directory.database();
            let mut connection = rusqlite::Connection::open(&database.0).unwrap();
            storage::initialize(&connection).unwrap();
            let id = (vocabulary_count == 1).then(|| save_word(&mut connection, "école"));
            let captured_at = 1_794_564_000;
            let mut snapshot =
                storage::export_learning_snapshot(&mut connection, captured_at).unwrap();
            if vocabulary_count == 1 {
                snapshot.dictionary_catalog.next_id = 2;
                snapshot.dictionary_catalog.entries.insert(
                    0,
                    storage::DictionaryCatalogEntry {
                        id: "mdx-1".into(),
                        title: "Dictionnaire personnel".into(),
                        content_hash: None,
                    },
                );
            }
            let manifest = BackupManifest {
                kind: "lexique-backup".into(),
                format_version: 1,
                snapshot_version: 1,
                created_at: captured_at,
                application_version: "0.1.0".into(),
                counts: BackupCounts::from_snapshot(&snapshot),
            };
            let source = directory.0.join("selected.lexique-backup");
            fixture_archive(
                &source,
                &[
                    (
                        "manifest.json".into(),
                        serde_json::to_vec(&manifest).unwrap(),
                    ),
                    (
                        "snapshot.json".into(),
                        serde_json::to_vec(&snapshot).unwrap(),
                    ),
                ],
            );
            let app_data = directory.0.join("app-data");
            fs::create_dir(&app_data).unwrap();
            let stage = crate::backup_preview::prepare_preview(
                &source,
                &app_data,
                &AtomicBool::new(false),
                |_| {},
            )
            .unwrap();
            assert!(stage.root().starts_with(&app_data));
            assert_eq!(
                stage.database(),
                stage.root().join("learning/lexique.sqlite3")
            );
            assert_eq!(
                stage.audio_directory(),
                crate::speech_cache::managed_audio_directory(stage.database()).unwrap()
            );
            assert_ne!(stage.audio_directory(), stage.database().parent().unwrap());
            assert_eq!(stage.preview().filename, "selected.lexique-backup");
            assert_eq!(
                stage.preview().byte_length,
                fs::metadata(&source).unwrap().len()
            );
            assert_eq!(stage.preview().created_at, captured_at);
            assert_eq!(stage.preview().counts.vocabulary, vocabulary_count);
            assert!(stage.preview().requires_restart);
            assert_eq!(stage.preview().counts.custom_dictionaries, vocabulary_count);
            assert_eq!(
                stage.preview().pending_dictionaries.len(),
                vocabulary_count as usize
            );
            if vocabulary_count == 1 {
                assert_eq!(stage.preview().pending_dictionaries[0].id, "mdx-1");
                assert_eq!(
                    stage.preview().pending_dictionaries[0].title,
                    "Dictionnaire personnel"
                );
            }
            let restored = rusqlite::Connection::open_with_flags(
                stage.database(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            let count: i64 = restored
                .query_row("SELECT count(*) FROM vocabulary", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, vocabulary_count as i64);
            if let Some(id) = id {
                let record = storage::get(&restored, id).unwrap();
                assert_eq!(record.headword, "école");
                assert_eq!(record.notes, "Mes notes\n个人笔记");
                assert_eq!(record.example_fr, "Une école.");
                assert_eq!(record.example_zh, "学校");
                assert!(record.example_source.is_none());
                assert_eq!(storage::get(&connection, id).unwrap().notes, record.notes);
            }
            let integrity: String = restored
                .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                .unwrap();
            assert_eq!(integrity, "ok");
            drop(restored);
            let root = stage.root().to_path_buf();
            drop(stage);
            assert!(!root.exists());
            assert!(source.is_file() && database.0.is_file());
        }
    }

    #[test]
    fn preview_file_staging_failures_and_phase_cancellation_leave_learning_and_source_unchanged() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        save_word(&mut connection, "école");
        let captured_at = 1_794_564_000;
        let snapshot = storage::export_learning_snapshot(&mut connection, captured_at).unwrap();
        connection.close().unwrap();
        let learning_bytes = fs::read(&database.0).unwrap();
        let manifest = BackupManifest {
            kind: "lexique-backup".into(),
            format_version: 1,
            snapshot_version: 1,
            created_at: captured_at,
            application_version: "0.1.0".into(),
            counts: BackupCounts::from_snapshot(&snapshot),
        };
        let source = directory.0.join("selected.lexique-backup");
        fixture_archive(
            &source,
            &[
                (
                    "manifest.json".into(),
                    serde_json::to_vec(&manifest).unwrap(),
                ),
                (
                    "snapshot.json".into(),
                    serde_json::to_vec(&snapshot).unwrap(),
                ),
            ],
        );
        let source_bytes = fs::read(&source).unwrap();
        let app_data = directory.0.join("app-data");
        fs::create_dir(&app_data).unwrap();
        let corrupt = directory.0.join("corrupt.lexique-backup");
        fs::write(&corrupt, b"truncated corrupt backup").unwrap();
        for invalid in [directory.0.join("missing.lexique-backup"), corrupt.clone()] {
            assert!(
                crate::backup_preview::prepare_preview(
                    &invalid,
                    &app_data,
                    &AtomicBool::new(false),
                    |_| {}
                )
                .is_err()
            );
            assert!(fs::read_dir(&app_data).unwrap().next().is_none());
        }
        assert_eq!(fs::read(&corrupt).unwrap(), b"truncated corrupt backup");
        let unusable_parent = directory.0.join("occupied-app-data");
        fs::write(&unusable_parent, b"existing parent file").unwrap();
        assert!(
            crate::backup_preview::prepare_preview(
                &source,
                &unusable_parent,
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert_eq!(fs::read(&unusable_parent).unwrap(), b"existing parent file");
        for cancel_at in [
            crate::backup_preview::BackupPreviewPhase::Validating,
            crate::backup_preview::BackupPreviewPhase::Staging,
        ] {
            let cancelled = AtomicBool::new(false);
            let result =
                crate::backup_preview::prepare_preview(&source, &app_data, &cancelled, |phase| {
                    if phase == cancel_at {
                        cancelled.store(true, Ordering::Release);
                    }
                });
            assert_eq!(result.err().unwrap(), "backup_cancelled");
            assert!(fs::read_dir(&app_data).unwrap().next().is_none());
        }
        assert_eq!(fs::read(&source).unwrap(), source_bytes);
        assert_eq!(fs::read(&database.0).unwrap(), learning_bytes);
    }

    #[test]
    fn legacy_staging_publication_checks_content_and_preserves_existing_managed_files() {
        let directory = Directory::new();
        let database = directory.database();
        let connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        connection.close().unwrap();
        let bytes = include_bytes!(
            "../../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus"
        );
        let replacement = include_bytes!(
            "../../../../docs/evidence/issue-14/supertonic-opus/female-normal-02-48k.opus"
        );
        let hash = storage::content_hash(bytes);
        let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
        inspector.publish_legacy(6, &hash, bytes).unwrap();
        assert!(inspector.publish_legacy(0, &hash, bytes).is_err());
        assert!(inspector.publish_legacy(7, &"0".repeat(64), bytes).is_err());
        assert!(
            inspector
                .publish_legacy(
                    8,
                    &storage::content_hash(b"invalid audio"),
                    b"invalid audio"
                )
                .is_err()
        );
        inspector
            .publish_legacy(6, &storage::content_hash(replacement), replacement)
            .unwrap();
        drop(inspector);
        let managed = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        assert!(managed.join("6.ogg").is_file());
        assert!(!database.0.parent().unwrap().join("6.ogg").exists());
        let readback = crate::speech_cache::Inspector::new(&database.0).unwrap();
        let restored = readback.legacy(6).unwrap().unwrap();
        assert_eq!(restored, bytes);
        assert_eq!(storage::content_hash(&restored), hash);
        for id in [0, 7, 8] {
            assert!(readback.legacy(id).unwrap().is_none());
        }
    }

    #[test]
    fn preview_materializes_individual_included_audio_and_preserves_fileless_shared_metadata() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let words = [
            "école", "or", "oui", "abîme", "chat", "livre", "fleur", "table",
        ];
        let ids: Vec<_> = words
            .iter()
            .map(|word| save_word(&mut connection, word))
            .collect();
        let shared: &[u8] = include_bytes!(
            "../../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus"
        );
        let other: &[u8] = include_bytes!(
            "../../../../docs/evidence/issue-14/supertonic-opus/female-normal-02-48k.opus"
        );
        let shared_hash = storage::content_hash(shared);
        let captured_at = 1_794_564_000;
        let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
        let initial = selected_fixture(
            &connection,
            &inspector,
            ids[0],
            words[0],
            shared,
            captured_at - 50,
        );
        selected_fixture(
            &connection,
            &inspector,
            ids[1],
            words[1],
            shared,
            captured_at - 50,
        );
        let missing = selected_fixture(
            &connection,
            &inspector,
            ids[2],
            words[2],
            shared,
            captured_at - 50,
        );
        let invalid = selected_fixture(
            &connection,
            &inspector,
            ids[3],
            words[3],
            shared,
            captured_at - 50,
        );
        selected_fixture(
            &connection,
            &inspector,
            ids[4],
            words[4],
            shared,
            captured_at - 2_000_000,
        );
        selected_fixture(
            &connection,
            &inspector,
            ids[4],
            words[4],
            other,
            captured_at - 1_300_000,
        );
        let source_audio = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        fs::remove_file(source_audio.join(format!(
            "s-{}-{}.ogg",
            missing.physical_locator, shared_hash
        )))
        .unwrap();
        fs::write(
            source_audio.join(format!(
                "s-{}-{}.ogg",
                invalid.physical_locator, shared_hash
            )),
            b"invalid original selected audio",
        )
        .unwrap();
        for id in &ids[5..] {
            storage::register_legacy_speech(&connection, *id, &shared_hash).unwrap();
        }
        fs::write(source_audio.join(format!("{}.ogg", ids[5])), shared).unwrap();
        fs::write(
            source_audio.join(format!("{}.ogg", ids[7])),
            b"invalid original legacy",
        )
        .unwrap();
        let cancelled = AtomicBool::new(false);
        let protected = OwnedDirectory::create(&directory.0).unwrap();
        let mut snapshot = storage::export_learning_snapshot(&mut connection, captured_at).unwrap();
        collect_audio(
            &connection,
            &database,
            &mut snapshot,
            &protected.0,
            captured_at,
            &cancelled,
        )
        .unwrap();
        let unknown = snapshot
            .audio_versions
            .iter_mut()
            .find(|version| version.locator == initial.locator)
            .unwrap();
        unknown.generated_at = None;
        unknown.verification = "unverified".into();
        let unknown = snapshot
            .audio_versions
            .iter_mut()
            .find(|version| version.locator == missing.locator)
            .unwrap();
        unknown.byte_length = None;
        unknown.generated_at = None;
        storage::validate_learning_snapshot(&snapshot, captured_at).unwrap();
        let counts = BackupCounts::from_snapshot(&snapshot);
        assert_eq!(counts.included_audio_versions, 3);
        assert_eq!(counts.included_legacy_audio, 1);
        assert_eq!(counts.audio_files, 2);
        assert_eq!(counts.source_missing_audio, 2);
        assert_eq!(counts.source_invalid_audio, 2);
        let manifest = BackupManifest {
            kind: "lexique-backup".into(),
            format_version: 1,
            snapshot_version: 1,
            created_at: captured_at,
            application_version: "0.1.0".into(),
            counts: counts.clone(),
        };
        let mut entries = vec![
            (
                "manifest.json".into(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
            (
                "snapshot.json".into(),
                serde_json::to_vec(&snapshot).unwrap(),
            ),
        ];
        for hash in included_hashes(&snapshot) {
            entries.push((
                format!("audio/{hash}.ogg"),
                fs::read(protected.0.join(format!("audio/{hash}.ogg"))).unwrap(),
            ));
        }
        let source = directory.0.join("scope-preview.lexique-backup");
        fixture_archive(&source, &entries);
        drop(inspector);
        connection.close().unwrap();
        let source_database_bytes = fs::read(&database.0).unwrap();
        let source_archive_bytes = fs::read(&source).unwrap();
        let app_data = directory.0.join("app-data");
        fs::create_dir(&app_data).unwrap();
        let stage =
            crate::backup_preview::prepare_preview(&source, &app_data, &cancelled, |_| {}).unwrap();
        let inspector = crate::speech_cache::Inspector::new(stage.database()).unwrap();
        let mut readback = rusqlite::Connection::open_with_flags(
            stage.database(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        for version in &snapshot.audio_versions {
            let restored = storage::audio_version(&readback, &version.locator)
                .unwrap()
                .unwrap();
            assert_eq!(restored.physical_locator, version.locator);
            let bytes = if version.content_hash == shared_hash {
                shared
            } else {
                other
            };
            let inspection = inspector
                .collect_version(
                    &restored.physical_locator,
                    &version.content_hash,
                    Some(bytes.len() as i64),
                )
                .unwrap();
            if version.payload_status == AudioPayloadStatus::Included {
                let crate::speech_cache::AudioInspection::Present(actual) = inspection else {
                    panic!("Included reference did not receive managed audio");
                };
                assert_eq!(actual, bytes);
            } else {
                assert!(matches!(
                    inspection,
                    crate::speech_cache::AudioInspection::Missing
                ));
            }
        }
        for legacy in &snapshot.legacy_audio {
            let inspection = inspector.collect_legacy(legacy.vocabulary_id).unwrap();
            if legacy.payload_status == AudioPayloadStatus::Included {
                let crate::speech_cache::AudioInspection::Present(actual) = inspection else {
                    panic!("Included legacy reference did not receive managed audio");
                };
                assert_eq!(actual, shared);
            } else {
                assert!(matches!(
                    inspection,
                    crate::speech_cache::AudioInspection::Missing
                ));
            }
        }
        let raw: (Option<i64>, Option<i64>, bool, String) = readback
            .query_row(
                "SELECT byte_length,generated_at,sealed,verification FROM speech_versions WHERE locator=?1",
                [&initial.locator],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            raw,
            (Some(shared.len() as i64), None, true, "unverified".into())
        );
        let raw: (Option<i64>, Option<i64>, bool, String) = readback
            .query_row(
                "SELECT byte_length,generated_at,sealed,verification FROM speech_versions WHERE locator=?1",
                [&missing.locator],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(raw, (None, None, true, "missing".into()));
        let selected = storage::selected_audio(&readback, ids[2]).unwrap().unwrap();
        assert_eq!(selected.byte_length, 0);
        assert!(selected.generated_at.is_none());
        let exported = storage::export_learning_snapshot(&mut readback, captured_at).unwrap();
        assert_eq!(
            serde_json::to_value(&exported.audio_selections).unwrap(),
            serde_json::to_value(&snapshot.audio_selections).unwrap()
        );
        assert_eq!(stage.preview().counts, counts);
        assert!(!stage.root().join("validated").exists());
        let managed_files = fs::read_dir(stage.audio_directory())
            .unwrap()
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "ogg")
            })
            .count();
        assert_eq!(managed_files, 4);
        assert_eq!(fs::read(&database.0).unwrap(), source_database_bytes);
        assert_eq!(fs::read(&source).unwrap(), source_archive_bytes);
        drop(inspector);
        readback.close().unwrap();
        let root = stage.root().to_path_buf();
        drop(stage);
        assert!(!root.exists());
    }

    #[test]
    fn collection_drains_the_complete_accepted_chunked_batch_and_its_callbacks() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let ids: Vec<_> = (0..101)
            .map(|index| save_word(&mut connection, &format!("mot-{index}")))
            .collect();
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let maintenance = MaintenanceCoordinator::default();
        let admission = maintenance.admit().unwrap();
        let cancelled = AtomicBool::new(false);
        let destination = directory.0.join("drained.lexique-backup");
        let (committed, observed) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        let (draining, started) = mpsc::sync_channel(1);
        thread::scope(|scope| {
            let database = &database;
            let writer = scope.spawn(move || {
                let _admission = admission;
                let mut connection = crate::vocabulary::open(database).unwrap();
                let mut chunks = 0;
                let result = storage::add_review_batch_observed(
                    &mut connection,
                    "accepted-whole-batch",
                    &ids,
                    crate::speech_tasks::now().unwrap(),
                    || {
                        chunks += 1;
                        if chunks == 1 {
                            committed.send(()).unwrap();
                            released.recv_timeout(Duration::from_secs(5)).unwrap();
                        }
                    },
                )
                .unwrap();
                assert_eq!(result.items.len(), 101);
                assert_eq!(chunks, 2);
            });
            observed.recv_timeout(Duration::from_secs(5)).unwrap();
            let exporter = scope.spawn(|| {
                export_backup_to_new_file(
                    database,
                    &workers,
                    &maintenance,
                    &destination,
                    "test-drain",
                    &cancelled,
                    |phase| {
                        if phase == BackupPhase::Draining {
                            draining.send(()).unwrap();
                        }
                    },
                )
            });
            started.recv_timeout(Duration::from_secs(5)).unwrap();
            wait_until(|| maintenance.admit().is_err());
            assert!(!destination.exists());
            release.send(()).unwrap();
            writer.join().unwrap();
            let result = exporter.join().unwrap().unwrap();
            assert_eq!(result.counts.batches, 1);
            assert_eq!(result.counts.cards, 202);
        });
        let validation = directory.0.join("drain-readback");
        fs::create_dir(&validation).unwrap();
        let snapshot = validate_backup(&destination, &validation, &cancelled)
            .unwrap()
            .snapshot;
        assert_eq!(snapshot.batches[0].items.len(), 101);
        assert!(
            snapshot.batches[0]
                .items
                .iter()
                .all(|item| item.created_cards == 2 && item.error.is_none())
        );
        assert!(maintenance.admit().is_ok());
        workers.shutdown().unwrap();
    }

    #[test]
    fn cancellation_at_every_export_phase_releases_admission_and_real_speech_work() {
        for cancelled_phase in [
            BackupPhase::Draining,
            BackupPhase::Collecting,
            BackupPhase::Packaging,
            BackupPhase::Validating,
            BackupPhase::Publishing,
        ] {
            let directory = Directory::new();
            let database = directory.database();
            let mut connection = rusqlite::Connection::open(&database.0).unwrap();
            storage::initialize(&connection).unwrap();
            let id = save_word(&mut connection, "école");
            drop(connection);
            let workers = Workers(Some(SpeechTasks::start_with_notifications(
                database.0.clone(),
                directory.0.join("absent-resources"),
                Arc::new(|| Ok(())),
            )));
            let maintenance = MaintenanceCoordinator::default();
            let cancelled = AtomicBool::new(false);
            let destination = directory.0.join("cancelled.lexique-backup");
            let error = export_backup_to_new_file(
                &database,
                &workers,
                &maintenance,
                &destination,
                "test-cancel",
                &cancelled,
                |phase| {
                    if phase == cancelled_phase {
                        cancelled.store(true, Ordering::Release);
                    }
                },
            )
            .unwrap_err();
            assert!(
                error.contains("backup_cancelled"),
                "{cancelled_phase:?}: {error}"
            );
            assert!(!destination.exists());
            assert_no_staging(&directory.0);
            let _admission = maintenance.admit().unwrap();
            let mut connection = crate::vocabulary::open(&database).unwrap();
            storage::add_review_batch(
                &mut connection,
                "after-cancellation",
                &[id],
                crate::speech_tasks::now().unwrap(),
            )
            .unwrap();
            workers.wake().unwrap();
            wait_until(|| {
                connection
                    .query_row("SELECT state <> 'queued' FROM speech_tasks", [], |row| {
                        row.get::<_, bool>(0)
                    })
                    .unwrap()
            });
            assert_eq!(
                storage::batch_status(&connection, "after-cancellation")
                    .unwrap()
                    .items
                    .len(),
                1
            );
            drop(connection);
            workers.shutdown().unwrap();
        }
    }

    #[test]
    fn publication_race_never_overwrites_an_existing_destination() {
        let directory = Directory::new();
        let database = directory.database();
        let connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let maintenance = MaintenanceCoordinator::default();
        let destination = directory.0.join("race.lexique-backup");
        let error = export_backup_to_new_file(
            &database,
            &workers,
            &maintenance,
            &destination,
            "test-race",
            &AtomicBool::new(false),
            |phase| {
                if phase == BackupPhase::Publishing {
                    fs::write(&destination, b"an independently created file").unwrap();
                }
            },
        )
        .unwrap_err();
        assert!(error.contains("publish new backup"), "{error}");
        assert_eq!(
            fs::read(&destination).unwrap(),
            b"an independently created file"
        );
        assert!(maintenance.admit().is_ok());
        assert_no_staging(&directory.0);
        workers.shutdown().unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn genuine_source_read_errors_fail_export_and_release_the_maintenance_guards() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let maintenance = MaintenanceCoordinator::default();
        let locked = std::sync::Mutex::new(None);
        let destination = directory.0.join("read-error.lexique-backup");
        let error = export_backup_to_new_file(
            &database,
            &workers,
            &maintenance,
            &destination,
            "test-read-error",
            &AtomicBool::new(false),
            |phase| {
                if phase != BackupPhase::Collecting {
                    return;
                }
                let connection = crate::vocabulary::open(&database).unwrap();
                let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
                let version = selected_fixture(
                    &connection,
                    &inspector,
                    id,
                    "école",
                    b"OggSsource-read-error",
                    crate::speech_tasks::now().unwrap(),
                );
                let audio = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
                *locked.lock().unwrap() = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(0)
                        .open(audio.join(format!(
                            "s-{}-{}.ogg",
                            version.physical_locator, version.content_hash
                        )))
                        .unwrap(),
                );
            },
        )
        .unwrap_err();
        assert!(
            error.contains("Could not inspect selected audio"),
            "{error}"
        );
        locked.lock().unwrap().take();
        assert!(!destination.exists());
        assert_no_staging(&directory.0);
        assert!(maintenance.admit().is_ok());
        let connection = crate::vocabulary::open(&database).unwrap();
        assert_eq!(storage::get(&connection, id).unwrap().headword, "école");
        drop(connection);
        workers.shutdown().unwrap();
    }

    #[test]
    fn an_unusable_destination_parent_fails_without_changing_learning_data() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let maintenance = MaintenanceCoordinator::default();
        let parent = directory.0.join("occupied-parent");
        fs::write(&parent, b"original parent file").unwrap();
        let destination = parent.join("unusable.lexique-backup");
        assert!(
            export_backup_to_new_file(
                &database,
                &workers,
                &maintenance,
                &destination,
                "test-unusable-parent",
                &AtomicBool::new(false),
                |_| {}
            )
            .is_err()
        );
        assert_eq!(fs::read(&parent).unwrap(), b"original parent file");
        assert_no_staging(&directory.0);
        assert!(maintenance.admit().is_ok());
        let connection = crate::vocabulary::open(&database).unwrap();
        assert_eq!(storage::get(&connection, id).unwrap().headword, "école");
        drop(connection);
        workers.shutdown().unwrap();
    }

    #[test]
    fn protected_copy_corruption_fails_the_whole_backup_without_reclassifying_the_source() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let maintenance = MaintenanceCoordinator::default();
        let bytes = b"OggSoriginal-protected-audio";
        let hash = storage::content_hash(bytes);
        let destination = directory.0.join("corrupted.lexique-backup");
        let error = export_backup_to_new_file(
            &database,
            &workers,
            &maintenance,
            &destination,
            "test-protected-copy",
            &AtomicBool::new(false),
            |phase| {
                if phase == BackupPhase::Collecting {
                    let connection = crate::vocabulary::open(&database).unwrap();
                    let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
                    selected_fixture(
                        &connection,
                        &inspector,
                        id,
                        "école",
                        bytes,
                        crate::speech_tasks::now().unwrap(),
                    );
                }
                if phase == BackupPhase::Packaging {
                    fs::write(
                        staging_directory(&directory.0)
                            .join("audio")
                            .join(format!("{hash}.ogg")),
                        b"OggStampered-protected-audio",
                    )
                    .unwrap();
                }
            },
        )
        .unwrap_err();
        assert!(
            error.contains("content identity or length mismatch"),
            "{error}"
        );
        assert!(!destination.exists());
        assert_no_staging(&directory.0);
        let connection = crate::vocabulary::open(&database).unwrap();
        let selected = storage::selected_audio(&connection, id).unwrap().unwrap();
        assert_eq!(selected.content_hash, hash);
        assert_eq!(selected.verification, "verified");
        assert!(maintenance.admit().is_ok());
        drop(connection);
        workers.shutdown().unwrap();
    }

    #[test]
    fn sustained_io_checks_cancellation_between_bounded_chunks_and_preserves_io_errors() {
        struct CancellingWrite<'a> {
            cancelled: &'a AtomicBool,
            written: usize,
        }
        impl Write for CancellingWrite<'_> {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.written += bytes.len();
                self.cancelled.store(true, Ordering::Release);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let cancelled = AtomicBool::new(false);
        let mut writer = CancelledWriter {
            writer: CancellingWrite {
                cancelled: &cancelled,
                written: 0,
            },
            cancelled: &cancelled,
        };
        let error = writer.write_all(&vec![0; 256 * 1024]).unwrap_err();
        assert_eq!(writer.writer.written, 64 * 1024);
        assert!(error.to_string().contains("backup_cancelled"));
        assert!(writer.flush().is_err());
        let mut reader = CancelledReader {
            reader: io::Cursor::new(vec![0; 256 * 1024]),
            cancelled: &cancelled,
        };
        assert!(reader.read(&mut [0; 4]).is_err());
        cancelled.store(false, Ordering::Release);
        assert_eq!(reader.read(&mut [0; 128 * 1024]).unwrap(), 64 * 1024);
        struct FailedWrite;
        impl Write for FailedWrite {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::StorageFull,
                    "synthetic disk full",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let error = CancelledWriter {
            writer: FailedWrite,
            cancelled: &cancelled,
        }
        .write_all(b"required audio")
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::StorageFull);
    }

    #[cfg(windows)]
    #[test]
    fn separate_audio_gc_drains_its_notification_and_really_resumes_after_pause() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let first = save_word(&mut connection, "école");
        let second = save_word(&mut connection, "livre");
        let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
        let time = crate::speech_tasks::now().unwrap();
        let expired = selected_fixture(
            &connection,
            &inspector,
            first,
            "école",
            b"OggSexpired-first",
            time - 2_000_000,
        );
        selected_fixture(
            &connection,
            &inspector,
            first,
            "école",
            b"OggScurrent-first",
            time - 1_300_000,
        );
        let audio = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        let first_path = audio.join(format!(
            "s-{}-{}.ogg",
            expired.physical_locator, expired.content_hash
        ));
        assert!(first_path.exists());
        let (notified, notification) = mpsc::sync_channel(1);
        let (release, released) = mpsc::sync_channel(1);
        let released = std::sync::Mutex::new(released);
        let once = AtomicBool::new(false);
        let notifications = Arc::new(move || {
            if !once.swap(true, Ordering::AcqRel) {
                notified.send(()).map_err(|error| error.to_string())?;
                released
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        });
        struct Gc(Arc<crate::audio_maintenance::AudioMaintenance>);
        impl Drop for Gc {
            fn drop(&mut self) {
                let _ = self.0.shutdown();
            }
        }
        let gc = Gc(Arc::new(crate::audio_maintenance::AudioMaintenance::start(
            notifications,
            database.0.clone(),
        )));
        notification.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!first_path.exists());
        let worker = gc.0.clone();
        let pausing = thread::spawn(move || worker.pause(&AtomicBool::new(false)));
        thread::sleep(Duration::from_millis(100));
        assert!(
            !pausing.is_finished(),
            "pause acknowledged before the complete GC notification finished"
        );
        release.send(()).unwrap();
        let pause = pausing.join().unwrap().unwrap();
        let expired = selected_fixture(
            &connection,
            &inspector,
            second,
            "livre",
            b"OggSexpired-second",
            time - 2_000_000,
        );
        selected_fixture(
            &connection,
            &inspector,
            second,
            "livre",
            b"OggScurrent-second",
            time - 1_300_000,
        );
        let second_path = audio.join(format!(
            "s-{}-{}.ogg",
            expired.physical_locator, expired.content_hash
        ));
        gc.0.wake();
        thread::sleep(Duration::from_millis(100));
        assert!(
            second_path.exists(),
            "GC ran while its reversible pause was held"
        );
        drop(pause);
        gc.0.wake();
        wait_until(|| !second_path.exists());
        drop(connection);
        gc.0.shutdown().unwrap();
    }

    fn fixture_archive(path: &Path, entries: &[(String, Vec<u8>)]) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .large_file(true)
            .unix_permissions(0o600);
        for (name, bytes) in entries {
            zip.start_file(name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
        zip.finish().unwrap().sync_all().unwrap();
    }

    fn header_offsets(bytes: &[u8], signature: &[u8; 4]) -> Vec<usize> {
        bytes
            .windows(4)
            .enumerate()
            .filter_map(|(index, bytes)| (bytes == signature).then_some(index))
            .collect()
    }

    fn assert_archive_rejected(
        directory: &Directory,
        name: &str,
        entries: &[(String, Vec<u8>)],
        mutation: impl FnOnce(&mut Vec<u8>),
    ) {
        let path = directory.0.join(format!("{name}.lexique-backup"));
        fixture_archive(&path, entries);
        let mut bytes = fs::read(&path).unwrap();
        mutation(&mut bytes);
        fs::write(&path, bytes).unwrap();
        let staging = OwnedDirectory::create(&directory.0).unwrap();
        assert!(
            validate_backup(&path, &staging.0, &AtomicBool::new(false)).is_err(),
            "malformed archive was admitted: {name}"
        );
    }

    #[test]
    fn malformed_containers_manifests_and_included_audio_are_rejected_before_use() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        let time = crate::speech_tasks::now().unwrap();
        let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
        let audio = b"OggSarchive-contract-audio";
        let version = selected_fixture(&connection, &inspector, id, "école", audio, time);
        let snapshot = storage::export_learning_snapshot(&mut connection, time).unwrap();
        let manifest = BackupManifest {
            kind: "lexique-backup".into(),
            format_version: 1,
            snapshot_version: 1,
            created_at: time,
            application_version: "0.1.0".into(),
            counts: BackupCounts::from_snapshot(&snapshot),
        };
        let entries = vec![
            (
                "manifest.json".into(),
                serde_json::to_vec(&manifest).unwrap(),
            ),
            (
                "snapshot.json".into(),
                serde_json::to_vec(&snapshot).unwrap(),
            ),
            (
                format!("audio/{}.ogg", version.content_hash),
                audio.to_vec(),
            ),
        ];
        let valid = directory.0.join("control.lexique-backup");
        fixture_archive(&valid, &entries);
        let staging = OwnedDirectory::create(&directory.0).unwrap();
        let admitted = validate_backup(&valid, &staging.0, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            fs::read(
                admitted
                    .audio_directory
                    .join(format!("{}.ogg", version.content_hash))
            )
            .unwrap(),
            audio
        );
        drop(staging);
        for removed in 0..3 {
            let mut invalid = entries.clone();
            invalid.remove(removed);
            assert_archive_rejected(&directory, &format!("missing-{removed}"), &invalid, |_| {});
        }
        for name in [
            "credentials.json",
            "../manifest.json",
            "audio/../escape.ogg",
            "/absolute",
            "audio/",
            "audio/not-a-content-hash.ogg",
        ] {
            let mut invalid = entries.clone();
            invalid.push((name.into(), b"extra bytes".to_vec()));
            assert_archive_rejected(
                &directory,
                &format!("extra-{}", NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)),
                &invalid,
                |_| {},
            );
        }
        assert_archive_rejected(&directory, "duplicate-name", &entries, |bytes| {
            for index in bytes
                .windows(13)
                .enumerate()
                .filter_map(|(index, bytes)| (bytes == b"snapshot.json").then_some(index))
                .collect::<Vec<_>>()
            {
                bytes[index..index + 13].copy_from_slice(b"manifest.json");
            }
        });
        assert_archive_rejected(&directory, "local-name-disagreement", &entries, |bytes| {
            let local = header_offsets(bytes, b"PK\x03\x04")[0];
            bytes[local + 30] = b'n';
        });
        for (name, flags, method) in [("encrypted", 1_u16, 0_u16), ("compressed", 0, 8)] {
            assert_archive_rejected(&directory, name, &entries, |bytes| {
                for local in header_offsets(bytes, b"PK\x03\x04") {
                    let previous =
                        u16::from_le_bytes(bytes[local + 6..local + 8].try_into().unwrap());
                    bytes[local + 6..local + 8].copy_from_slice(&(previous | flags).to_le_bytes());
                    bytes[local + 8..local + 10].copy_from_slice(&method.to_le_bytes());
                }
                for central in header_offsets(bytes, b"PK\x01\x02") {
                    let previous =
                        u16::from_le_bytes(bytes[central + 8..central + 10].try_into().unwrap());
                    bytes[central + 8..central + 10]
                        .copy_from_slice(&(previous | flags).to_le_bytes());
                    bytes[central + 10..central + 12].copy_from_slice(&method.to_le_bytes());
                }
            });
        }
        for mode in [0o120600_u32, 0o040700, 0o020600, 0o060600] {
            assert_archive_rejected(
                &directory,
                &format!("special-mode-{mode}"),
                &entries,
                |bytes| {
                    let central = header_offsets(bytes, b"PK\x01\x02")[0];
                    bytes[central + 5] = 3;
                    bytes[central + 38..central + 42].copy_from_slice(&(mode << 16).to_le_bytes());
                },
            );
        }
        assert_archive_rejected(&directory, "size-disagreement", &entries, |bytes| {
            let central = header_offsets(bytes, b"PK\x01\x02")[2];
            bytes[central + 20..central + 24].copy_from_slice(&1_u32.to_le_bytes());
        });
        assert_archive_rejected(&directory, "audio-crc", &entries, |bytes| {
            let local = header_offsets(bytes, b"PK\x03\x04")[2];
            let name =
                u16::from_le_bytes(bytes[local + 26..local + 28].try_into().unwrap()) as usize;
            let extra =
                u16::from_le_bytes(bytes[local + 28..local + 30].try_into().unwrap()) as usize;
            bytes[local + 30 + name + extra + audio.len() - 1] ^= 1;
        });
        let mut invalid = entries.clone();
        invalid[2].1 = b"OggSdifferent-valid-crc".to_vec();
        assert_archive_rejected(&directory, "audio-content-hash", &invalid, |_| {});
        let mut invalid = entries.clone();
        let mut json = serde_json::to_value(&snapshot).unwrap();
        json["audio_versions"][0]["byte_length"] = (audio.len() + 1).into();
        invalid[1].1 = serde_json::to_vec(&json).unwrap();
        assert_archive_rejected(&directory, "audio-length", &invalid, |_| {});
        for (field, value) in [
            ("kind", serde_json::json!("other")),
            ("format_version", serde_json::json!(2)),
            ("snapshot_version", serde_json::json!(2)),
            ("created_at", serde_json::json!(9_007_199_254_740_992_i64)),
            ("application_version", serde_json::json!("")),
            ("unknown", serde_json::json!(true)),
        ] {
            let mut invalid = entries.clone();
            let mut json = serde_json::to_value(&manifest).unwrap();
            json[field] = value;
            invalid[0].1 = serde_json::to_vec(&json).unwrap();
            assert_archive_rejected(&directory, &format!("manifest-{field}"), &invalid, |_| {});
        }
        let mut invalid = entries.clone();
        let mut json = serde_json::to_value(&manifest).unwrap();
        json["counts"]["vocabulary"] = 0.into();
        invalid[0].1 = serde_json::to_vec(&json).unwrap();
        assert_archive_rejected(&directory, "counts", &invalid, |_| {});
        let mut invalid = entries.clone();
        invalid[0].1 = vec![b' '; 65 * 1024];
        assert_archive_rejected(&directory, "large-manifest", &invalid, |_| {});
        for (name, addition) in [
            (
                "duplicate-json-field",
                b",\"snapshot_version\":1".as_slice(),
            ),
            ("unknown-json-field", b",\"unexpected\":true"),
        ] {
            let mut invalid = entries.clone();
            invalid[1].1.pop();
            invalid[1].1.extend_from_slice(addition);
            invalid[1].1.push(b'}');
            assert_archive_rejected(&directory, name, &invalid, |_| {});
        }
        let mut invalid = entries.clone();
        invalid[1].1.extend_from_slice(b"{}");
        assert_archive_rejected(&directory, "trailing-json", &invalid, |_| {});
        drop(connection);
    }

    #[test]
    fn selected_previous_shared_missing_invalid_and_unpublished_audio_keep_their_scope() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let ids: Vec<_> = ["école", "or", "oui", "abîme", "chat", "livre"]
            .into_iter()
            .map(|word| save_word(&mut connection, word))
            .collect();
        drop(connection);
        let audio_directory = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        fs::create_dir_all(&audio_directory).unwrap();
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let cancelled = AtomicBool::new(false);
        let destination = directory.0.join("scope.lexique-backup");
        let preview = storage::SpeechSpec::word("temporary lookup", storage::SpeechVoice::Female);
        let result = export_backup_to_new_file(
            &database,
            &workers,
            &MaintenanceCoordinator::default(),
            &destination,
            "test-scope",
            &cancelled,
            |phase| {
                if phase != BackupPhase::Collecting {
                    return;
                }
                let connection = crate::vocabulary::open(&database).unwrap();
                let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
                let time = crate::speech_tasks::now().unwrap();
                selected_fixture(
                    &connection,
                    &inspector,
                    ids[0],
                    "école",
                    b"OggSprevious",
                    time - 100,
                );
                selected_fixture(
                    &connection,
                    &inspector,
                    ids[0],
                    "école",
                    b"OggSshared",
                    time - 50,
                );
                selected_fixture(
                    &connection,
                    &inspector,
                    ids[1],
                    "or",
                    b"OggSshared",
                    time - 50,
                );
                selected_fixture(
                    &connection,
                    &inspector,
                    ids[2],
                    "oui",
                    b"OggSexpired",
                    time - 2_000_000,
                );
                selected_fixture(
                    &connection,
                    &inspector,
                    ids[2],
                    "oui",
                    b"OggScurrent-oui",
                    time - 1_300_000,
                );
                let invalid = selected_fixture(
                    &connection,
                    &inspector,
                    ids[3],
                    "abîme",
                    b"OggSoriginal-abime",
                    time - 50,
                );
                fs::write(
                    audio_directory.join(format!(
                        "s-{}-{}.ogg",
                        invalid.physical_locator, invalid.content_hash
                    )),
                    b"invalid original audio",
                )
                .unwrap();
                let missing = selected_fixture(
                    &connection,
                    &inspector,
                    ids[4],
                    "chat",
                    b"OggSoriginal-chat",
                    time - 50,
                );
                fs::remove_file(audio_directory.join(format!(
                    "s-{}-{}.ogg",
                    missing.physical_locator, missing.content_hash
                )))
                .unwrap();
                let key = storage::register_speech(
                    &connection,
                    &storage::SpeechSpec::word("livre", storage::SpeechVoice::Female),
                )
                .unwrap();
                let bytes = b"OggSunfinished-publication";
                let hash = storage::content_hash(bytes);
                let pending = storage::prepare_audio_version(
                    &connection,
                    ids[5],
                    Some(&key),
                    storage::AudioPublicationContent {
                        hash: &hash,
                        bytes: bytes.len() as i64,
                        generated_at: Some(time),
                    },
                    0,
                    time,
                )
                .unwrap();
                inspector
                    .publish_version(&pending.physical_locator, &hash, bytes)
                    .unwrap();
                let legacy = b"OggSsuperseded-legacy";
                fs::write(audio_directory.join(format!("{}.ogg", ids[5])), legacy).unwrap();
                storage::register_legacy_speech(
                    &connection,
                    ids[5],
                    &storage::content_hash(legacy),
                )
                .unwrap();
                source_fixture(&connection, &inspector, &preview, b"OggStemporary", time);
            },
        )
        .unwrap();
        assert_eq!(result.counts.included_audio_versions, 4);
        assert_eq!(result.counts.audio_files, 3);
        assert_eq!(result.counts.source_missing_audio, 1);
        assert_eq!(result.counts.source_invalid_audio, 1);
        assert_eq!(result.counts.included_legacy_audio, 0);
        let staging = directory.0.join("scope-readback");
        fs::create_dir(&staging).unwrap();
        let validated = validate_backup(&destination, &staging, &cancelled).unwrap();
        assert!(
            !validated
                .snapshot
                .speech_sources
                .iter()
                .any(|source| source.semantic_key == preview.key().unwrap())
        );
        assert_eq!(validated.snapshot.audio_publications.len(), 1);
        let pending = &validated.snapshot.audio_publications[0];
        let version = validated
            .snapshot
            .audio_versions
            .iter()
            .find(|version| version.locator == pending.locator)
            .unwrap();
        assert_eq!(version.payload_status, AudioPayloadStatus::Excluded);
        assert_eq!(
            version.exclusion_reason,
            Some(storage::AudioExclusionReason::Unpublished)
        );
        let expired = validated
            .snapshot
            .audio_selections
            .iter()
            .find(|selection| selection.vocabulary_id == ids[2])
            .unwrap()
            .previous_version
            .as_ref()
            .unwrap();
        let version = validated
            .snapshot
            .audio_versions
            .iter()
            .find(|version| &version.locator == expired)
            .unwrap();
        assert_eq!(
            version.exclusion_reason,
            Some(storage::AudioExclusionReason::Expired)
        );
        assert!(
            validated
                .snapshot
                .speech_owners
                .iter()
                .any(|owner| owner.vocabulary_id == ids[0])
        );
        assert!(
            validated
                .snapshot
                .speech_owners
                .iter()
                .any(|owner| owner.vocabulary_id == ids[1])
        );
        assert_eq!(
            fs::read(
                validated
                    .audio_directory
                    .join(format!("{}.ogg", storage::content_hash(b"OggSshared")))
            )
            .unwrap(),
            b"OggSshared"
        );
        workers.shutdown().unwrap();
    }

    #[test]
    fn initial_owned_audio_uses_playback_order_and_never_packages_every_old_candidate() {
        for mode in ["present", "missing", "invalid"] {
            let directory = Directory::new();
            let database = directory.database();
            let mut connection = rusqlite::Connection::open(&database.0).unwrap();
            storage::initialize(&connection).unwrap();
            let id = save_word(&mut connection, "école");
            drop(connection);
            let audio_directory =
                crate::speech_cache::managed_audio_directory(&database.0).unwrap();
            fs::create_dir_all(&audio_directory).unwrap();
            let workers = Workers(Some(SpeechTasks::start_with_notifications(
                database.0.clone(),
                directory.0.join("absent-resources"),
                Arc::new(|| Ok(())),
            )));
            let cancelled = AtomicBool::new(false);
            let destination = directory.0.join("initial.lexique-backup");
            let result = export_backup_to_new_file(&database, &workers, &MaintenanceCoordinator::default(), &destination, "test-initial", &cancelled, |phase| {
                if phase != BackupPhase::Collecting { return; }
                let connection = crate::vocabulary::open(&database).unwrap();
                let inspector = crate::speech_cache::Inspector::new(&database.0).unwrap();
                let time = crate::speech_tasks::now().unwrap();
                for (voice, bytes, generated_at) in [(storage::SpeechVoice::Female, b"OggSolder".as_slice(), time - 100), (storage::SpeechVoice::Male, b"OggSnewest".as_slice(), time - 50)] {
                    let spec = storage::SpeechSpec::word("école", voice);
                    let version = source_fixture(&connection, &inspector, &spec, bytes, generated_at);
                    // Supported historical ownership before per-word versions were registered.
                    connection.execute("INSERT OR IGNORE INTO speech_owners(semantic_key,vocabulary_id) VALUES(?1,?2)", rusqlite::params![spec.key().unwrap(), id]).unwrap();
                    if voice == storage::SpeechVoice::Male {
                        let path = audio_directory.join(format!("s-{}-{}.ogg", version.physical_locator, version.content_hash));
                        if mode == "missing" { fs::remove_file(&path).unwrap(); }
                        if mode == "invalid" { fs::write(&path, b"OggSwrong-checksum").unwrap(); }
                    }
                }
                let bytes = b"OggSlegacy-alternative";
                fs::write(audio_directory.join(format!("{id}.ogg")), bytes).unwrap();
                storage::register_legacy_speech(&connection, id, &storage::content_hash(bytes)).unwrap();
            }).unwrap();
            let staging = directory.0.join("initial-readback");
            fs::create_dir(&staging).unwrap();
            let validated = validate_backup(&destination, &staging, &cancelled).unwrap();
            assert!(validated.snapshot.audio_selections.is_empty());
            assert_eq!(validated.snapshot.speech_owners.len(), 2);
            assert_eq!(result.counts.included_legacy_audio, 0);
            assert_eq!(result.counts.source_missing_audio, 0);
            assert_eq!(result.counts.audio_files, u64::from(mode != "invalid"));
            assert_eq!(
                result.counts.source_invalid_audio,
                u64::from(mode == "invalid")
            );
            if mode != "invalid" {
                let expected = if mode == "present" {
                    b"OggSnewest".as_slice()
                } else {
                    b"OggSolder".as_slice()
                };
                assert_eq!(
                    fs::read(
                        validated
                            .audio_directory
                            .join(format!("{}.ogg", storage::content_hash(expected)))
                    )
                    .unwrap(),
                    expected
                );
            }
            workers.shutdown().unwrap();
        }
    }

    #[test]
    fn original_unregistered_invalid_bytes_are_not_packaged_or_given_a_fake_identity() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        drop(connection);
        let audio_directory = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        fs::create_dir_all(&audio_directory).unwrap();
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let cancelled = AtomicBool::new(false);
        let bytes = b"original invalid bytes";
        let destination = directory.0.join("invalid-legacy.lexique-backup");
        let result = export_backup_to_new_file(
            &database,
            &workers,
            &MaintenanceCoordinator::default(),
            &destination,
            "test-invalid-legacy",
            &cancelled,
            |phase| {
                if phase == BackupPhase::Collecting {
                    fs::write(audio_directory.join(format!("{id}.ogg")), bytes).unwrap();
                }
            },
        )
        .unwrap();
        assert_eq!(result.counts.source_invalid_audio, 1);
        assert_eq!(result.counts.audio_files, 0);
        let staging = directory.0.join("invalid-readback");
        fs::create_dir(&staging).unwrap();
        let validated = validate_backup(&destination, &staging, &cancelled).unwrap();
        assert_eq!(
            validated.snapshot.legacy_audio[0].content_hash,
            storage::content_hash(bytes)
        );
        assert_eq!(
            validated.snapshot.legacy_audio[0].payload_status,
            AudioPayloadStatus::SourceInvalid
        );
        workers.shutdown().unwrap();
    }

    #[test]
    fn a_native_empty_backup_is_validated_before_publication_and_writers_resume_for_packaging() {
        let directory = Directory::new();
        let path = directory.database();
        let connection = rusqlite::Connection::open(&path.0).unwrap();
        lexique_storage::initialize(&connection).unwrap();
        lexique_storage::set_sort_preference(&connection, "headword-asc").unwrap();
        drop(connection);
        let notification: Arc<dyn Fn() -> Result<(), String> + Send + Sync> = Arc::new(|| Ok(()));
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            path.0.clone(),
            directory.0.join("absent-resources"),
            notification,
        )));
        let maintenance = MaintenanceCoordinator::default();
        let cancelled = AtomicBool::new(false);
        let destination = directory.0.join("empty.lexique-backup");
        let result = export_backup_to_new_file(
            &path,
            &workers,
            &maintenance,
            &destination,
            "test-empty",
            &cancelled,
            |phase| {
                if phase == BackupPhase::Collecting {
                    assert!(
                        maintenance
                            .admit()
                            .unwrap_err()
                            .contains("maintenance_busy")
                    );
                }
                if phase == BackupPhase::Packaging {
                    let _write = maintenance.admit().unwrap();
                    let connection = crate::vocabulary::open(&path).unwrap();
                    lexique_storage::set_sort_preference(&connection, "updated-desc").unwrap();
                }
            },
        )
        .unwrap();
        assert_eq!(result.destination, destination.to_string_lossy());
        assert_eq!(
            result.byte_length,
            std::fs::metadata(&destination).unwrap().len()
        );
        let staging = directory.0.join("validation");
        std::fs::create_dir(&staging).unwrap();
        let validated = validate_backup(&destination, &staging, &cancelled).unwrap();
        assert_eq!(validated.manifest.kind, "lexique-backup");
        assert_eq!(validated.manifest.counts.vocabulary, 0);
        assert_eq!(validated.manifest.counts.audio_files, 0);
        assert_eq!(
            validated.snapshot.preferences.vocabulary_sort,
            "headword-asc"
        );
        assert!(maintenance.admit().is_ok());
        workers.shutdown().unwrap();
    }

    #[test]
    fn unregistered_current_legacy_bytes_are_discovered_without_mutating_selection() {
        let directory = Directory::new();
        let database = directory.database();
        let mut connection = rusqlite::Connection::open(&database.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        drop(connection);
        let audio_directory = crate::speech_cache::managed_audio_directory(&database.0).unwrap();
        fs::create_dir_all(&audio_directory).unwrap();
        let source = audio_directory.join(format!("{id}.ogg"));
        let bytes = b"OggSsynthetic-current-legacy";
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            database.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let cancelled = AtomicBool::new(false);
        let destination = directory.0.join("legacy.lexique-backup");
        let result = export_backup_to_new_file(
            &database,
            &workers,
            &MaintenanceCoordinator::default(),
            &destination,
            "test-legacy",
            &cancelled,
            |phase| {
                // The genuine workers are paused: discovery belongs to the protected collector,
                // not to a lucky migration/reconciliation run before collection.
                if phase == BackupPhase::Collecting {
                    fs::write(&source, bytes).unwrap();
                }
                if phase == BackupPhase::Packaging {
                    fs::remove_file(&source).unwrap();
                }
            },
        )
        .unwrap();
        assert_eq!(result.counts.included_legacy_audio, 1);
        assert_eq!(result.counts.audio_files, 1);
        let staging = directory.0.join("legacy-readback");
        fs::create_dir(&staging).unwrap();
        let validated = validate_backup(&destination, &staging, &cancelled).unwrap();
        assert_eq!(validated.snapshot.legacy_audio[0].vocabulary_id, id);
        let hash = storage::content_hash(bytes);
        assert_eq!(validated.snapshot.legacy_audio[0].content_hash, hash);
        assert_eq!(
            fs::read(validated.audio_directory.join(format!("{hash}.ogg"))).unwrap(),
            bytes
        );
        let connection = crate::vocabulary::open(&database).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM word_audio_selections", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
        drop(connection);
        workers.shutdown().unwrap();
    }

    #[test]
    fn missing_selected_audio_preserves_learning_and_its_real_missing_relationship() {
        let directory = Directory::new();
        let path = directory.database();
        let mut connection = rusqlite::Connection::open(&path.0).unwrap();
        storage::initialize(&connection).unwrap();
        let id = save_word(&mut connection, "école");
        storage::add_review(&mut connection, id).unwrap();
        let hash = storage::content_hash(b"OggSexpected");
        let version = storage::prepare_audio_version(
            &connection,
            id,
            None,
            storage::AudioPublicationContent {
                hash: &hash,
                bytes: 12,
                generated_at: None,
            },
            0,
            1_800_000_000,
        )
        .unwrap();
        storage::verify_audio_content(&connection, &version.locator, 12).unwrap();
        storage::complete_audio_version(&connection, &version.locator, 1_800_000_000).unwrap();
        drop(connection);
        let workers = Workers(Some(SpeechTasks::start_with_notifications(
            path.0.clone(),
            directory.0.join("absent-resources"),
            Arc::new(|| Ok(())),
        )));
        let cancelled = AtomicBool::new(false);
        let destination = directory.0.join("missing.lexique-backup");
        let result = export_backup_to_new_file(
            &path,
            &workers,
            &MaintenanceCoordinator::default(),
            &destination,
            "missing",
            &cancelled,
            |_| {},
        )
        .unwrap();
        assert_eq!(result.counts.source_missing_audio, 1);
        assert_eq!(result.counts.audio_files, 0);
        let staging = directory.0.join("validation");
        fs::create_dir(&staging).unwrap();
        let read = validate_backup(&destination, &staging, &cancelled).unwrap();
        assert_eq!(read.snapshot.records[0].notes, "Mes notes\n个人笔记");
        assert_eq!(read.snapshot.cards.len(), 2);
        assert_eq!(
            read.snapshot.audio_selections[0].current_version.as_deref(),
            Some(version.locator.as_str())
        );
        assert_eq!(
            read.snapshot.audio_versions[0].payload_status,
            AudioPayloadStatus::SourceMissing
        );
        assert_eq!(read.snapshot.audio_versions[0].verification, "missing");
        workers.shutdown().unwrap();
    }
}
