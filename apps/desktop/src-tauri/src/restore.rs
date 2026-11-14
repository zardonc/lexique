//! Native-owned switch decisions are resolved before SQLite or workers can start.
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::{
    backup::{export_paused, publish_new},
    backup_preview::{BackupPreview, VerifiedBackupStage},
    maintenance::MaintenanceCoordinator,
    speech_cache::managed_audio_directory,
    speech_tasks::SpeechTasks,
    vocabulary::DatabasePath,
};

const JOURNAL: &str = ".lexique-restore.json";
const RETAINED: &str = "retained-restores";
static NEXT_WRITE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreservationKind {
    LogicalBackup,
    OriginalDatabase,
    OriginalAudio,
    RawFiles,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawPendingRequest {
    pub key: String,
    pub value: String,
}

pub(crate) fn validate_pending_requests(requests: &[RawPendingRequest]) -> Result<(), String> {
    let mut keys = HashSet::new();
    for request in requests {
        if !(request.key.starts_with("lexique-pending-rating-")
            || request.key == "lexique-pending-undo"
            || request.key == "lexique-pending-review-batch")
            || !keys.insert(&request.key)
        {
            return Err("Raw pending requests must use distinct known application keys".into());
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedCopy {
    pub id: String,
    pub kind: PreservationKind,
    pub path: String,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupPhase {
    Ready,
    CleanupRequired,
    RecoveryRequired,
}

#[derive(Clone, Debug, Serialize)]
pub struct StartupStatus {
    pub state: StartupPhase,
    pub operation: Option<String>,
    pub restored: Option<BackupPreview>,
    pub retained_copies: Vec<RetainedCopy>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct PreparedRestore {
    pub retained_copies: Vec<RetainedCopy>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Prepared,
    Swapping,
    Committed,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ComponentKind {
    Database,
    Wal,
    Shm,
    Journal,
    Audio,
}

const COMPONENTS: [ComponentKind; 5] = [
    ComponentKind::Database,
    ComponentKind::Wal,
    ComponentKind::Shm,
    ComponentKind::Journal,
    ComponentKind::Audio,
];

impl ComponentKind {
    fn name(self) -> &'static str {
        match self {
            Self::Database => "database",
            Self::Wal => "wal",
            Self::Shm => "shm",
            Self::Journal => "journal",
            Self::Audio => "audio",
        }
    }

    fn path(self, database: &Path) -> Result<PathBuf, String> {
        match self {
            Self::Database => Ok(database.to_owned()),
            Self::Audio => managed_audio_directory(database),
            Self::Wal | Self::Shm | Self::Journal => {
                let mut path = database.as_os_str().to_owned();
                path.push(match self {
                    Self::Wal => "-wal",
                    Self::Shm => "-shm",
                    _ => "-journal",
                });
                Ok(path.into())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Component {
    kind: ComponentKind,
    original_present: bool,
    staged_present: bool,
    isolate_started: bool,
    isolated: bool,
    install_started: bool,
    installed: bool,
    remove_new_started: bool,
    new_removed: bool,
    restore_old_started: bool,
    old_restored: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RestoreJournal {
    restore_id: String,
    phase: Phase,
    preservation_kind: PreservationKind,
    stage_root: PathBuf,
    preview: BackupPreview,
    components: Vec<Component>,
    cleanup_started: bool,
    keys_cleared: bool,
    retained_copies: Vec<RetainedCopy>,
    rolling_back: bool,
    failure: Option<String>,
}

fn directory(database: &DatabasePath) -> Result<&Path, String> {
    database
        .0
        .parent()
        .ok_or_else(|| "Learning database has no application directory".into())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn retention(database: &DatabasePath, id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("Invalid native restore identity".into());
    }
    Ok(directory(database)?.join(RETAINED).join(id))
}

fn original_path(
    database: &DatabasePath,
    journal: &RestoreJournal,
    kind: ComponentKind,
) -> Result<PathBuf, String> {
    let root = retention(database, &journal.restore_id)?;
    if kind == ComponentKind::Audio {
        Ok(root.join("original-audio"))
    } else {
        Ok(root.join("original-database").join(
            kind.path(&database.0)?
                .file_name()
                .ok_or("Invalid database filename")?,
        ))
    }
}

fn staged_database(journal: &RestoreJournal) -> PathBuf {
    journal.stage_root.join("learning/lexique.sqlite3")
}

fn present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("Restore component is a link: {}", path.display()))
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "Could not inspect restore component {}: {error}",
            path.display()
        )),
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn fsync(fd: i32) -> i32;
    }
    let file = File::open(path).map_err(|error| {
        format!(
            "Could not open restore directory {}: {error}",
            path.display()
        )
    })?;
    if unsafe { fsync(file.as_raw_fd()) } != 0 {
        return Err(format!(
            "Could not sync restore directory {}: {}",
            path.display(),
            io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    // Windows namespace publication uses MoveFileExW WRITE_THROUGH; directory fsync is unavailable.
    Ok(())
}

fn sync_tree(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "Could not inspect retained state {}: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(format!("Retained state is a link: {}", path.display()));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
            sync_tree(&entry.map_err(|error| error.to_string())?.path())?;
        }
        sync_directory(path)
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .and_then(|file| file.sync_all())
            .map_err(|error| format!("Could not sync restore file {}: {error}", path.display()))
    }
}

#[cfg(windows)]
fn replace_record(source: &Path, destination: &Path) -> io::Result<()> {
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
    // Replace only the coordinator's record, with a write-through namespace update.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0x9) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_record(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let parent = path.parent().ok_or("Restore record has no parent")?;
    let temporary = parent.join(format!(
        ".restore-{}-{}-{}.tmp",
        std::process::id(),
        crate::speech_tasks::now()?,
        NEXT_WRITE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("Could not create restore record: {error}"))?;
        serde_json::to_writer(&mut file, value)
            .map_err(|error| format!("Could not write restore record: {error}"))?;
        file.flush()
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("Could not sync restore record: {error}"))?;
        drop(file);
        replace_record(&temporary, path)
            .map_err(|error| format!("Could not publish restore record: {error}"))?;
        sync_directory(parent)
    })();
    if present(&temporary).unwrap_or(false)
        && let Err(error) = fs::remove_file(&temporary)
    {
        eprintln!("Could not remove restore record temporary: {error}");
    }
    result
}

fn write_journal(database: &DatabasePath, journal: &RestoreJournal) -> Result<(), String> {
    write_json(&directory(database)?.join(JOURNAL), journal)
}

fn read_journal(database: &DatabasePath) -> Result<Option<RestoreJournal>, String> {
    let path = directory(database)?.join(JOURNAL);
    if !present(&path)? {
        return Ok(None);
    }
    let journal: RestoreJournal = serde_json::from_reader(
        File::open(&path).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("Could not read restore record {}: {error}", path.display()))?;
    if !valid_id(&journal.restore_id)
        || journal.stage_root.parent() != Some(directory(database)?)
        || !journal
            .stage_root
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".lexique-backup-"))
        || journal.components.len() != COMPONENTS.len()
        || !journal
            .components
            .iter()
            .zip(COMPONENTS)
            .all(|(component, kind)| component.kind == kind)
    {
        return Err(format!(
            "Restore record has inconsistent native paths or components: {}",
            path.display()
        ));
    }
    Ok(Some(journal))
}

fn byte_length(path: &Path) -> Result<u64, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Retained copy is a link".into());
    }
    if !metadata.is_dir() {
        return Ok(metadata.len());
    }
    let mut length = 0_u64;
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        length = length
            .checked_add(byte_length(
                &entry.map_err(|error| error.to_string())?.path(),
            )?)
            .ok_or("Retained copy is too large")?;
    }
    Ok(length)
}

fn copies_for(database: &DatabasePath, id: &str) -> Result<Vec<RetainedCopy>, String> {
    let root = retention(database, id)?;
    let mut copies = Vec::new();
    for (name, kind, path) in [
        (
            "backup",
            PreservationKind::LogicalBackup,
            root.join("pre-restore.lexique-backup"),
        ),
        (
            "database",
            PreservationKind::OriginalDatabase,
            root.join("original-database"),
        ),
        (
            "audio",
            PreservationKind::OriginalAudio,
            root.join("original-audio"),
        ),
        ("raw", PreservationKind::RawFiles, root.join("raw-files")),
    ] {
        if present(&path)? {
            copies.push(RetainedCopy {
                id: format!("{id}-{name}"),
                kind,
                path: path.to_string_lossy().into_owned(),
                byte_length: byte_length(&path)?,
            });
        }
    }
    Ok(copies)
}

fn update_copies(database: &DatabasePath, journal: &mut RestoreJournal) -> Result<(), String> {
    journal.retained_copies = copies_for(database, &journal.restore_id)?;
    write_json(
        &retention(database, &journal.restore_id)?.join("copies.json"),
        &journal.retained_copies,
    )
}

pub(crate) fn retained_copies(database: &DatabasePath) -> Result<Vec<RetainedCopy>, String> {
    let root = directory(database)?.join(RETAINED);
    if !present(&root)? {
        return Ok(Vec::new());
    }
    let mut copies = Vec::new();
    for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let id = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid retained restore identity")?;
        if !valid_id(id) || !present(&path)? || !path.is_dir() {
            continue;
        }
        copies.extend(copies_for(database, id)?);
    }
    Ok(copies)
}

pub(crate) fn retained_directory(database: &DatabasePath, id: &str) -> Result<PathBuf, String> {
    let copy = retained_copies(database)?
        .into_iter()
        .find(|copy| copy.id == id)
        .ok_or("Retained copy identity is unavailable")?;
    let path = PathBuf::from(copy.path);
    let directory = if copy.kind == PreservationKind::LogicalBackup {
        path.parent()
            .ok_or("Retained backup has no directory")?
            .to_owned()
    } else {
        path
    };
    if !present(&directory)? || !directory.is_dir() {
        return Err("Retained copy directory is unavailable".into());
    }
    Ok(directory)
}

pub(crate) fn prepare_normal(
    database: &DatabasePath,
    workers: &SpeechTasks,
    maintenance: &MaintenanceCoordinator,
    stage: VerifiedBackupStage,
    id: &str,
    close: impl FnOnce() -> Result<(), String>,
    mut observe: impl FnMut(&str) -> Result<(), String>,
) -> Result<PreparedRestore, String> {
    if present(&directory(database)?.join(JOURNAL))? {
        return Err("A restore record already requires startup recovery".into());
    }
    if stage.root().parent() != Some(directory(database)?) {
        return Err("Verified restore staging is outside the application filesystem".into());
    }
    let root = retention(database, id)?;
    fs::create_dir_all(root.parent().ok_or("Retained copies have no parent")?)
        .map_err(|error| error.to_string())?;
    fs::create_dir(&root)
        .map_err(|error| format!("Could not retain pre-restore state: {error}"))?;
    let cancelled = AtomicBool::new(false);
    let admission = maintenance.begin(&cancelled)?;
    let pause = workers.pause(&cancelled)?;
    observe("preserving")?;
    export_paused(
        database,
        &root.join("pre-restore.lexique-backup"),
        &cancelled,
    )?;
    observe("preparing")?;
    let old = crate::vocabulary::open(database)?;
    let mut target = crate::vocabulary::open(&DatabasePath(stage.database().to_owned()))?;
    let catalog =
        lexique_storage::read_dictionary_catalog(&target).map_err(|error| error.to_string())?;
    crate::dictionaries::restore_catalog(&mut target, database, &catalog)
        .map_err(|error| error.to_string())?;
    crate::selection_shortcut::restore_device_preferences(&old, &target)?;
    old.close().map_err(|(_, error)| error.to_string())?;
    target.close().map_err(|(_, error)| error.to_string())?;
    observe("closing")?;
    admission.close_readers(&cancelled)?;
    admission.commit();
    close()?;
    drop(pause);
    publish_prepared(
        database,
        stage,
        id,
        PreservationKind::LogicalBackup,
        &mut observe,
    )
}

pub(crate) fn prepare_raw(
    database: &DatabasePath,
    stage: VerifiedBackupStage,
    id: &str,
    requests: &[RawPendingRequest],
    close: impl FnOnce() -> Result<(), String>,
    mut observe: impl FnMut(&str) -> Result<(), String>,
) -> Result<PreparedRestore, String> {
    validate_pending_requests(requests)?;
    if present(&directory(database)?.join(JOURNAL))? {
        return Err("A restore record already requires startup recovery".into());
    }
    if stage.root().parent() != Some(directory(database)?) {
        return Err("Verified restore staging is outside the application filesystem".into());
    }
    let root = retention(database, id)?;
    let raw = root.join("raw-files");
    let preserve = (|| {
        fs::create_dir_all(root.parent().ok_or("Retained copies have no parent")?)
            .map_err(|error| error.to_string())?;
        fs::create_dir(&root).map_err(|error| error.to_string())?;
        observe("preserving")?;
        close()?;
        fs::create_dir(&raw).map_err(|error| error.to_string())?;
        observe("raw-preservation-created")?;
        for kind in COMPONENTS {
            let source = kind.path(&database.0)?;
            if !present(&source)? {
                continue;
            }
            let destination = if kind == ComponentKind::Audio {
                raw.join("audio")
            } else {
                let files = raw.join("database");
                fs::create_dir_all(&files).map_err(|error| error.to_string())?;
                files.join(
                    source
                        .file_name()
                        .ok_or("Invalid original database filename")?,
                )
            };
            copy_tree(&source, &destination).map_err(|error| {
                format!("Could not preserve original {}: {error}", source.display())
            })?;
        }
        let dictionaries = directory(database)?.join("dictionaries");
        if present(&dictionaries)? {
            copy_tree(&dictionaries, &raw.join("dictionaries")).map_err(|error| {
                format!(
                    "Could not preserve original {}: {error}",
                    dictionaries.display()
                )
            })?;
        }
        observe("raw-files-copied")?;
        write_json(&raw.join("pending-requests.json"), &requests)?;
        observe("raw-requests-written")?;
        sync_tree(&raw)?;
        observe("preparing")?;
        observe("closing")?;
        publish_prepared(
            database,
            stage,
            id,
            PreservationKind::RawFiles,
            &mut observe,
        )
    })();
    preserve.map_err(|error| format!("Raw-file preservation at {} failed: {error}", raw.display()))
}

fn publish_prepared(
    database: &DatabasePath,
    stage: VerifiedBackupStage,
    id: &str,
    preservation_kind: PreservationKind,
    observe: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<PreparedRestore, String> {
    let root = retention(database, id)?;
    sync_tree(stage.root())?;
    sync_tree(&root)?;
    let mut components = Vec::new();
    for kind in COMPONENTS {
        components.push(Component {
            kind,
            original_present: present(&kind.path(&database.0)?)?,
            staged_present: present(&kind.path(stage.database())?)?,
            isolate_started: false,
            isolated: false,
            install_started: false,
            installed: false,
            remove_new_started: false,
            new_removed: false,
            restore_old_started: false,
            old_restored: false,
        });
    }
    let mut journal = RestoreJournal {
        restore_id: id.into(),
        phase: Phase::Prepared,
        preservation_kind,
        stage_root: stage.root().to_owned(),
        preview: stage.preview().clone(),
        components,
        cleanup_started: false,
        keys_cleared: false,
        retained_copies: Vec::new(),
        rolling_back: false,
        failure: None,
    };
    update_copies(database, &mut journal)?;
    let published = write_journal(database, &journal);
    if published.is_ok() || !matches!(read_journal(database), Ok(None)) {
        let _ = stage.keep();
    }
    if let Err(error) = published.and_then(|()| observe("prepared")) {
        return Err(match abort_preparation(database, id, &error, observe) {
            Ok(()) => error,
            Err(recovery) => format!("{error}; preparation rollback failed: {recovery}"),
        });
    }
    Ok(PreparedRestore {
        retained_copies: journal.retained_copies,
    })
}

fn abort_preparation(
    database: &DatabasePath,
    id: &str,
    error: &str,
    observe: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let Some(mut journal) = read_journal(database)? else {
        return Ok(());
    };
    if journal.restore_id != id || journal.phase == Phase::Committed {
        return Err(
            "The visible restore decision cannot be rolled back as this preparation".into(),
        );
    }
    journal.failure = Some(error.into());
    rollback(database, &mut journal, observe)?;
    remove_partial_rollback(&journal.stage_root)
}

fn move_component(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(
        destination
            .parent()
            .ok_or("Restore destination has no parent")?,
    )
    .map_err(|error| error.to_string())?;
    publish_new(source, destination).map_err(|error| {
        format!(
            "Could not move restore component {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    sync_directory(source.parent().ok_or("Restore source has no parent")?)?;
    sync_directory(
        destination
            .parent()
            .ok_or("Restore destination has no parent")?,
    )
}

fn rollback_path(journal: &RestoreJournal, kind: ComponentKind) -> Result<PathBuf, String> {
    let root = journal.stage_root.join("rollback-originals");
    if kind == ComponentKind::Audio {
        Ok(root.join("audio"))
    } else {
        Ok(root.join(kind.name()))
    }
}

fn rejected_path(
    database: &DatabasePath,
    journal: &RestoreJournal,
    kind: ComponentKind,
) -> Result<PathBuf, String> {
    Ok(retention(database, &journal.restore_id)?
        .join("rejected-new")
        .join(kind.name()))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|error| {
        format!(
            "Could not inspect original restore component {}: {error}",
            source.display()
        )
    })?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "Preserved restore component is a link: {}",
            source.display()
        ));
    }
    if metadata.is_dir() {
        fs::create_dir(destination).map_err(|error| {
            format!(
                "Could not create restore copy directory {}: {error}",
                destination.display()
            )
        })?;
        for entry in fs::read_dir(source).map_err(|error| {
            format!(
                "Could not read original restore directory {}: {error}",
                source.display()
            )
        })? {
            let entry = entry.map_err(|error| error.to_string())?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
        sync_directory(destination)
    } else {
        let mut input = File::open(source).map_err(|error| {
            format!(
                "Could not read original restore file {}: {error}",
                source.display()
            )
        })?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .map_err(|error| {
                format!(
                    "Could not create restore copy file {}: {error}",
                    destination.display()
                )
            })?;
        io::copy(&mut input, &mut output)
            .and_then(|_| output.sync_all())
            .map_err(|error| {
                format!(
                    "Could not copy and sync restore file {} to {}: {error}",
                    source.display(),
                    destination.display()
                )
            })
    }
}

fn remove_partial_rollback(path: &Path) -> Result<(), String> {
    if !present(path)? {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|error| format!("Could not remove incomplete rollback staging: {error}"))
}

fn archive_journal(database: &DatabasePath, journal: &RestoreJournal) -> Result<(), String> {
    write_json(
        &retention(database, &journal.restore_id)?.join("decision.json"),
        journal,
    )?;
    fs::remove_file(directory(database)?.join(JOURNAL))
        .map_err(|error| format!("Could not finish restore decision: {error}"))?;
    sync_directory(directory(database)?)
}

fn rollback(
    database: &DatabasePath,
    journal: &mut RestoreJournal,
    observe: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    journal.phase = Phase::Swapping;
    journal.rolling_back = true;
    write_json(
        &retention(database, &journal.restore_id)?.join("decision.json"),
        journal,
    )?;
    write_journal(database, journal)?;
    observe("rollback")?;
    for index in (0..journal.components.len()).rev() {
        let component = &journal.components[index];
        if component.new_removed {
            continue;
        }
        let kind = component.kind;
        if component.install_started {
            let active = kind.path(&database.0)?;
            let staged = kind.path(&staged_database(journal))?;
            let rejected = rejected_path(database, journal, kind)?;
            let active_present = present(&active)?;
            let staged_present = present(&staged)?;
            let rejected_present = present(&rejected)?;
            if active_present && (staged_present || rejected_present) {
                return Err(format!(
                    "Rollback cannot identify the active {} component",
                    kind.name()
                ));
            }
            if active_present {
                journal.components[index].remove_new_started = true;
                write_journal(database, journal)?;
                observe(&format!("before-rollback-remove-{}", kind.name()))?;
                move_component(&active, &rejected)?;
                observe(&format!("after-rollback-remove-{}", kind.name()))?;
            }
        }
        journal.components[index].new_removed = true;
        write_journal(database, journal)?;
    }
    for index in 0..journal.components.len() {
        let component = &journal.components[index];
        let kind = component.kind;
        let active = kind.path(&database.0)?;
        let original = original_path(database, journal, kind)?;
        let staged = rollback_path(journal, kind)?;
        let active_present = present(&active)?;
        if component.old_restored {
            if active_present != component.original_present {
                return Err(format!(
                    "Completed rollback {} component is unavailable",
                    kind.name()
                ));
            }
            continue;
        }
        if !component.original_present {
            if active_present || present(&original)? {
                return Err(format!(
                    "Originally absent {} component is not absent",
                    kind.name()
                ));
            }
        } else if present(&original)? {
            if active_present {
                if !component.restore_old_started || present(&staged)? {
                    return Err(format!(
                        "Rollback {} installation is ambiguous",
                        kind.name()
                    ));
                }
            } else {
                journal.components[index].restore_old_started = true;
                write_journal(database, journal)?;
                fs::create_dir_all(staged.parent().ok_or("Rollback staging has no parent")?)
                    .map_err(|error| error.to_string())?;
                remove_partial_rollback(&staged)?;
                copy_tree(&original, &staged)?;
                observe(&format!("before-rollback-install-{}", kind.name()))?;
                move_component(&staged, &active)?;
                observe(&format!("after-rollback-install-{}", kind.name()))?;
            }
        } else if !active_present || component.isolated || component.install_started {
            return Err(format!(
                "Preserved {} component is unavailable for rollback",
                kind.name()
            ));
        }
        journal.components[index].old_restored = true;
        write_journal(database, journal)?;
    }
    if !journal.components[0].original_present {
        return Err("Rollback preserved the original absence of the learning database; recovery is still required".into());
    }
    sync_tree(&database.0)?;
    if present(&managed_audio_directory(&database.0)?)? {
        sync_tree(&managed_audio_directory(&database.0)?)?;
    }
    if journal.preservation_kind != PreservationKind::RawFiles {
        validate_current(database)?;
    }
    update_copies(database, journal)?;
    write_journal(database, journal)?;
    observe("rollback-complete")?;
    archive_journal(database, journal)
}

fn validate_current(database: &DatabasePath) -> Result<(), String> {
    let connection = Connection::open_with_flags(&database.0, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("Could not validate restored database: {error}"))?;
    let schema: u32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if schema != lexique_storage::DATABASE_SCHEMA_VERSION {
        return Err("Restored database schema is not supported".into());
    }
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if integrity != "ok" {
        return Err(format!(
            "Restored database integrity check failed: {integrity}"
        ));
    }
    if connection
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut statement| statement.exists([]))
        .map_err(|error| error.to_string())?
    {
        return Err("Restored database relationships are invalid".into());
    }
    connection.close().map_err(|(_, error)| error.to_string())
}

fn switch(
    database: &DatabasePath,
    journal: &mut RestoreJournal,
    observe: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    journal.phase = Phase::Swapping;
    write_journal(database, journal)?;
    observe("swapping")?;
    for index in 0..journal.components.len() {
        if !journal.components[index].original_present {
            continue;
        }
        let kind = journal.components[index].kind;
        journal.components[index].isolate_started = true;
        write_journal(database, journal)?;
        observe(&format!("before-isolate-{}", kind.name()))?;
        move_component(
            &kind.path(&database.0)?,
            &original_path(database, journal, kind)?,
        )?;
        observe(&format!("after-isolate-{}", kind.name()))?;
        journal.components[index].isolated = true;
        write_journal(database, journal)?;
    }
    update_copies(database, journal)?;
    write_journal(database, journal)?;
    for index in 0..journal.components.len() {
        if !journal.components[index].staged_present {
            continue;
        }
        let kind = journal.components[index].kind;
        journal.components[index].install_started = true;
        write_journal(database, journal)?;
        observe(&format!("before-install-{}", kind.name()))?;
        move_component(
            &kind.path(&staged_database(journal))?,
            &kind.path(&database.0)?,
        )?;
        observe(&format!("after-install-{}", kind.name()))?;
        journal.components[index].installed = true;
        write_journal(database, journal)?;
    }
    sync_tree(&database.0)?;
    sync_tree(&managed_audio_directory(&database.0)?)?;
    validate_current(database)?;
    observe("new-state-synced")?;
    journal.phase = Phase::Committed;
    write_journal(database, journal)?;
    observe("committed")
}

fn cleanup_journal(database: &DatabasePath, id: &str) -> Result<RestoreJournal, String> {
    let journal =
        read_journal(database)?.ok_or("No committed restore requires pending-request cleanup")?;
    if journal.restore_id != id || journal.phase != Phase::Committed || journal.rolling_back {
        return Err("Pending-request cleanup identity is no longer available".into());
    }
    Ok(journal)
}

pub(crate) fn cleanup_begin(
    database: &DatabasePath,
    id: &str,
    mut observe: impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let mut journal = cleanup_journal(database, id)?;
    journal.cleanup_started = true;
    write_journal(database, &journal)?;
    observe("cleanup-started")
}

fn finish_cleanup(
    database: &DatabasePath,
    journal: &RestoreJournal,
    observe: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    if !journal.cleanup_started || !journal.keys_cleared {
        return Err("Committed restore still requires pending-request cleanup".into());
    }
    observe("before-cleanup-stage")?;
    if present(&journal.stage_root)? {
        fs::remove_dir_all(&journal.stage_root)
            .map_err(|error| format!("Could not clean verified restore staging: {error}"))?;
        sync_directory(directory(database)?)?;
    }
    observe("after-cleanup-stage")?;
    archive_journal(database, journal)?;
    observe("cleanup-complete")
}

pub(crate) fn cleanup_complete(
    database: &DatabasePath,
    id: &str,
    mut observe: impl FnMut(&str) -> Result<(), String>,
) -> Result<StartupStatus, String> {
    let journal = if read_journal(database)?.is_some() {
        let mut journal = cleanup_journal(database, id)?;
        if !journal.cleanup_started {
            return Err("Pending-request cleanup has not begun".into());
        }
        journal.keys_cleared = true;
        write_journal(database, &journal)?;
        observe("keys-cleared")?;
        validate_current(database)?;
        finish_cleanup(database, &journal, &mut observe)?;
        journal
    } else {
        let path = retention(database, id)?.join("decision.json");
        let journal: RestoreJournal = serde_json::from_reader(
            File::open(&path)
                .map_err(|error| format!("Restore cleanup result is unavailable: {error}"))?,
        )
        .map_err(|error| format!("Could not read restore cleanup result: {error}"))?;
        if journal.restore_id != id || journal.phase != Phase::Committed || !journal.keys_cleared {
            return Err("Restore cleanup result has a different identity".into());
        }
        journal
    };
    Ok(StartupStatus {
        state: StartupPhase::Ready,
        operation: Some(id.into()),
        restored: Some(journal.preview),
        retained_copies: retained_copies(database)?,
        error: None,
    })
}

pub(crate) fn recover_startup(
    database: &DatabasePath,
    mut observe: impl FnMut(&str) -> Result<(), String>,
) -> StartupStatus {
    let mut status = StartupStatus {
        state: StartupPhase::Ready,
        operation: None,
        restored: None,
        retained_copies: Vec::new(),
        error: None,
    };
    let result = (|| {
        let Some(mut journal) = read_journal(database)? else {
            status.retained_copies = retained_copies(database)?;
            return Ok(());
        };
        status.operation = Some(journal.restore_id.clone());
        status.restored = Some(journal.preview.clone());
        status.retained_copies = journal.retained_copies.clone();
        if journal.phase == Phase::Prepared {
            let decision = retention(database, &journal.restore_id)?.join("decision.json");
            if present(&decision)? {
                let aborted: RestoreJournal = serde_json::from_reader(
                    File::open(&decision).map_err(|error| error.to_string())?,
                )
                .map_err(|error| format!("Could not read retained restore decision: {error}"))?;
                if aborted.restore_id != journal.restore_id
                    || aborted.phase != Phase::Swapping
                    || !aborted.rolling_back
                {
                    return Err(
                        "Retained restore decision is inconsistent with the prepared handoff"
                            .into(),
                    );
                }
                journal.phase = Phase::Swapping;
                journal.rolling_back = true;
                journal.failure = aborted.failure;
            }
        }
        if journal.phase == Phase::Swapping {
            let failure = journal
                .failure
                .clone()
                .unwrap_or_else(|| "Restore was interrupted before commitment".into());
            rollback(database, &mut journal, &mut observe)?;
            status.restored = None;
            if journal.preservation_kind == PreservationKind::RawFiles {
                status.state = StartupPhase::RecoveryRequired;
                status.error = Some(format!(
                    "The original raw learning files were restored; recovery is still required: {failure}"
                ));
            } else {
                status.error = Some(format!(
                    "The previous learning state was restored: {failure}"
                ));
            }
            status.retained_copies = retained_copies(database)?;
            return Ok(());
        }
        if journal.phase == Phase::Prepared
            && let Err(error) = switch(database, &mut journal, &mut observe)
        {
            let mut visible = read_journal(database)?
                .ok_or("Restore commitment is unavailable; preserved copies require recovery")?;
            if visible.phase != Phase::Committed {
                visible.failure = Some(error.clone());
                rollback(database, &mut visible, &mut observe).map_err(|rollback_error| {
                    format!("{error}; rollback failed: {rollback_error}")
                })?;
                status.restored = None;
                if visible.preservation_kind == PreservationKind::RawFiles {
                    status.state = StartupPhase::RecoveryRequired;
                    status.error = Some(format!(
                        "The original raw learning files were restored; recovery is still required: {error}"
                    ));
                } else {
                    status.error =
                        Some(format!("The previous learning state was restored: {error}"));
                }
                status.retained_copies = retained_copies(database)?;
                return Ok(());
            }
            journal = visible;
            write_journal(database, &journal)?;
        }
        validate_current(database)?;
        status.retained_copies = retained_copies(database)?;
        if journal.keys_cleared {
            finish_cleanup(database, &journal, &mut observe)?;
        } else {
            status.state = StartupPhase::CleanupRequired;
        }
        Ok::<_, String>(())
    })();
    if let Err(error) = result {
        status.state = StartupPhase::RecoveryRequired;
        status.error = Some(error);
        if let Ok(copies) = retained_copies(database) {
            status.retained_copies = copies;
        }
    }
    status
}
