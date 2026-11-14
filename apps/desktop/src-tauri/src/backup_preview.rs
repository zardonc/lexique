//! Verified app-owned learning staging; selected source paths stay in the native worker.
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use lexique_storage::{
    self as storage, AudioPayloadStatus, LearningSnapshot, SnapshotAudioVersion,
};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::{
    backup::{
        BackupCounts, BackupManifest, CancelledReader, OwnedDirectory, ValidatedBackup,
        validate_backup,
    },
    speech_cache::{AudioInspection, Inspector, managed_audio_directory},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupPreviewPhase {
    Validating,
    Staging,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupPreviewState {
    Choosing,
    Running,
    PreviewReady,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackupPreviewStatus {
    pub operation: String,
    pub state: BackupPreviewState,
    pub phase: Option<BackupPreviewPhase>,
    pub cancel_requested: bool,
    pub preview: Option<BackupPreview>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PendingBackupDictionary {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BackupPreview {
    pub filename: String,
    pub byte_length: u64,
    pub created_at: i64,
    pub application_version: String,
    pub counts: BackupCounts,
    pub pending_dictionaries: Vec<PendingBackupDictionary>,
    pub requires_restart: bool,
}

pub struct VerifiedBackupStage {
    root: OwnedDirectory,
    database: PathBuf,
    audio_directory: PathBuf,
    manifest: BackupManifest,
    preview: BackupPreview,
}

impl VerifiedBackupStage {
    pub fn root(&self) -> &Path {
        &self.root.0
    }

    pub fn database(&self) -> &Path {
        &self.database
    }

    pub fn audio_directory(&self) -> &Path {
        &self.audio_directory
    }

    pub fn manifest(&self) -> &BackupManifest {
        &self.manifest
    }

    pub fn preview(&self) -> &BackupPreview {
        &self.preview
    }

    /// After a durable prepared journal, transfers root cleanup to journal recovery.
    #[must_use]
    pub fn keep(self) -> PathBuf {
        self.root.keep()
    }
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        Err("backup_cancelled".into())
    } else {
        Ok(())
    }
}

fn verify_database(connection: &Connection, snapshot: &LearningSnapshot) -> Result<(), String> {
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| format!("Could not check staged database integrity: {e}"))?;
    if integrity != "ok" {
        return Err(format!(
            "Staged database integrity check failed: {integrity}"
        ));
    }
    if connection
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut statement| statement.exists([]))
        .map_err(|e| format!("Could not check staged database relationships: {e}"))?
    {
        return Err("Staged database foreign-key check failed".into());
    }
    for expected in &snapshot.records {
        let record = storage::get(connection, expected.id)
            .map_err(|e| format!("Could not read staged vocabulary: {e}"))?;
        if serde_json::to_value(record).map_err(|e| e.to_string())?
            != serde_json::to_value(expected).map_err(|e| e.to_string())?
        {
            return Err(format!(
                "Staged vocabulary {} differs from the backup",
                expected.id
            ));
        }
    }
    Ok(())
}

fn payload_length(verified: &ValidatedBackup, hash: &str) -> Result<u64, String> {
    verified
        .audio_lengths
        .get(hash)
        .copied()
        .ok_or_else(|| "Included backup audio has no verified length".into())
}

fn read_payload(
    verified: &ValidatedBackup,
    hash: &str,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, String> {
    check_cancelled(cancelled)?;
    let length = payload_length(verified, hash)?;
    let file = File::open(verified.audio_directory.join(format!("{hash}.ogg")))
        .map_err(|e| format!("Could not open verified backup audio: {e}"))?;
    let mut bytes = Vec::with_capacity(length as usize);
    CancelledReader {
        reader: file,
        cancelled,
    }
    .take(length + 1)
    .read_to_end(&mut bytes)
    .map_err(|e| format!("Could not read verified backup audio: {e}"))?;
    check_cancelled(cancelled)?;
    if bytes.len() as u64 != length
        || !bytes.starts_with(b"OggS")
        || storage::content_hash(&bytes) != hash
    {
        return Err("Verified backup audio integrity check failed".into());
    }
    Ok(bytes)
}

fn staged_version(
    connection: &Connection,
    expected: &SnapshotAudioVersion,
) -> Result<storage::AudioVersion, String> {
    let version = storage::audio_version(connection, &expected.locator)
        .map_err(|e| format!("Could not read staged audio version: {e}"))?
        .ok_or("Staged audio version is missing")?;
    if version.content_hash != expected.content_hash {
        return Err("Staged audio version differs from the backup".into());
    }
    Ok(version)
}

fn materialize_audio(
    connection: &Connection,
    database: &Path,
    verified: &ValidatedBackup,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let inspector = Inspector::new(database)?;
    for expected in &verified.snapshot.audio_versions {
        if expected.payload_status != AudioPayloadStatus::Included {
            continue;
        }
        let version = staged_version(connection, expected)?;
        let bytes = read_payload(verified, &expected.content_hash, cancelled)?;
        inspector
            .publish_version(&version.physical_locator, &expected.content_hash, &bytes)
            .map_err(|e| format!("Could not publish staged audio version: {e}"))?;
    }
    for legacy in &verified.snapshot.legacy_audio {
        if legacy.payload_status != AudioPayloadStatus::Included {
            continue;
        }
        let bytes = read_payload(verified, &legacy.content_hash, cancelled)?;
        inspector
            .publish_legacy(legacy.vocabulary_id, &legacy.content_hash, &bytes)
            .map_err(|e| format!("Could not publish staged legacy audio: {e}"))?;
    }
    check_cancelled(cancelled)
}

fn verify_managed_audio(
    connection: &Connection,
    database: &Path,
    verified: &ValidatedBackup,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let inspector = Inspector::new(database)?;
    for expected in &verified.snapshot.audio_versions {
        check_cancelled(cancelled)?;
        let version = staged_version(connection, expected)?;
        let inspection = inspector.collect_version(
            &version.physical_locator,
            &expected.content_hash,
            expected.byte_length,
        )?;
        match (expected.payload_status, inspection) {
            (AudioPayloadStatus::Included, AudioInspection::Present(bytes))
                if bytes.len() as u64 == payload_length(verified, &expected.content_hash)? => {}
            (status, AudioInspection::Missing) if status != AudioPayloadStatus::Included => {}
            _ => return Err("Staged managed audio differs from the backup".into()),
        }
    }
    for legacy in &verified.snapshot.legacy_audio {
        check_cancelled(cancelled)?;
        let hash: String = connection
            .query_row(
                "SELECT content_hash FROM legacy_speech WHERE vocabulary_id=?1",
                [legacy.vocabulary_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Could not read staged legacy audio relation: {e}"))?;
        if hash != legacy.content_hash {
            return Err("Staged legacy audio relation differs from the backup".into());
        }
        match (
            legacy.payload_status,
            inspector.collect_legacy(legacy.vocabulary_id)?,
        ) {
            (AudioPayloadStatus::Included, AudioInspection::Present(bytes))
                if bytes.len() as u64 == payload_length(verified, &legacy.content_hash)?
                    && storage::content_hash(&bytes) == legacy.content_hash => {}
            (status, AudioInspection::Missing) if status != AudioPayloadStatus::Included => {}
            _ => return Err("Staged managed legacy audio differs from the backup".into()),
        }
    }
    check_cancelled(cancelled)
}

pub fn prepare_preview(
    archive_path: &Path,
    app_data_directory: &Path,
    cancelled: &AtomicBool,
    phase: impl Fn(BackupPreviewPhase),
) -> Result<VerifiedBackupStage, String> {
    check_cancelled(cancelled)?;
    let root = OwnedDirectory::create(app_data_directory)?;
    let validation = root.0.join("validated");
    fs::create_dir(&validation)
        .map_err(|e| format!("Could not create backup validation area: {e}"))?;
    phase(BackupPreviewPhase::Validating);
    let verified = validate_backup(archive_path, &validation, cancelled)?;
    check_cancelled(cancelled)?;
    phase(BackupPreviewPhase::Staging);
    check_cancelled(cancelled)?;
    let learning = root.0.join("learning");
    fs::create_dir(&learning)
        .map_err(|e| format!("Could not create learning staging area: {e}"))?;
    let database = learning.join(crate::DATABASE_FILENAME);
    let mut connection = Connection::open(&database)
        .map_err(|e| format!("Could not create staged learning database: {e}"))?;
    storage::initialize(&connection)
        .map_err(|e| format!("Could not initialize staged learning database: {e}"))?;
    storage::import_learning_snapshot(
        &mut connection,
        &verified.snapshot,
        verified.manifest.created_at,
    )
    .map_err(|e| format!("Could not import staged learning database: {e}"))?;
    check_cancelled(cancelled)?;
    let audio_directory = managed_audio_directory(&database)?;
    materialize_audio(&connection, &database, &verified, cancelled)?;
    connection
        .close()
        .map_err(|(_, e)| format!("Could not close staged learning database: {e}"))?;
    let readback = Connection::open_with_flags(&database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Could not reopen staged learning database for verification: {e}"))?;
    verify_database(&readback, &verified.snapshot)?;
    verify_managed_audio(&readback, &database, &verified, cancelled)?;
    readback
        .close()
        .map_err(|(_, e)| format!("Could not close staged verification database: {e}"))?;
    fs::remove_dir_all(&validation)
        .map_err(|e| format!("Could not remove validated extraction copies: {e}"))?;
    File::options()
        .read(true)
        .write(true)
        .open(&database)
        .and_then(|file| file.sync_all())
        .map_err(|e| format!("Could not sync staged learning database: {e}"))?;
    check_cancelled(cancelled)?;
    let preview = BackupPreview {
        filename: archive_path
            .file_name()
            .ok_or("Backup filename unavailable")?
            .to_string_lossy()
            .into_owned(),
        byte_length: verified.byte_length,
        created_at: verified.manifest.created_at,
        application_version: verified.manifest.application_version.clone(),
        counts: verified.manifest.counts.clone(),
        pending_dictionaries: verified
            .snapshot
            .dictionary_catalog
            .entries
            .iter()
            .filter(|entry| storage::custom_dictionary_number(&entry.id).is_some())
            .map(|entry| PendingBackupDictionary {
                id: entry.id.clone(),
                title: entry.title.clone(),
            })
            .collect(),
        requires_restart: true,
    };
    Ok(VerifiedBackupStage {
        root,
        database,
        audio_directory,
        manifest: verified.manifest,
        preview,
    })
}
