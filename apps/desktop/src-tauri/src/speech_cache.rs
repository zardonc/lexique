#[cfg(windows)]
use std::time::{Duration, SystemTime};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cap_fs_ext::OpenOptionsFollowExt;
use cap_primitives::fs::FollowSymlinks;
#[cfg(windows)]
use cap_std::fs::{MetadataExt, OpenOptionsExt};
use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};

fn open_child(parent: &Dir, name: &Path) -> Result<Dir, String> {
    let parent = parent
        .try_clone()
        .map_err(|e| e.to_string())?
        .into_std_file();
    cap_primitives::fs::open_dir_nofollow(&parent, name)
        .map(Dir::from_std_file)
        .map_err(|e| e.to_string())
}

fn data_directory(database: &Path) -> Result<Dir, String> {
    let data = data_directory_path(database)?;
    let parent = data.parent().ok_or("Invalid portable data path")?;
    let name = data.file_name().ok_or("Invalid portable data path")?;
    let portable = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|e| e.to_string())?;
    open_child(&portable, Path::new(name))
}

fn data_directory_path(database: &Path) -> Result<&Path, String> {
    if cfg!(windows) {
        database
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "Invalid portable data path".into())
    } else if cfg!(target_os = "macos") {
        database
            .parent()
            .ok_or_else(|| "Invalid application data path".into())
    } else {
        Err("Speech cache is not available on this platform".into())
    }
}

/// Only this dedicated component contains managed audio; the database parent may also contain dictionaries.
pub fn managed_audio_directory(database: &Path) -> Result<PathBuf, String> {
    Ok(data_directory_path(database)?.join("temp").join("speech"))
}

fn create_child(parent: &Dir, name: &str) -> Result<Dir, String> {
    if let Err(error) = parent.create_dir(name)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(error.to_string());
    }
    open_child(parent, Path::new(name))
}

fn directory(database: &Path) -> Result<Dir, String> {
    let temp = create_child(&data_directory(database)?, "temp")?;
    create_child(&temp, "speech")
}

pub fn report_error(_database: &Path, message: &str) {
    eprintln!("{message}");
    #[cfg(windows)]
    {
        let result = (|| {
            let logs = create_child(&data_directory(_database)?, "logs")?;
            let mut options = OpenOptions::new();
            options.append(true).create(true).follow(FollowSymlinks::No);
            let mut file = logs
                .open_with("speech-cache.log", &options)
                .map_err(|e| e.to_string())?;
            let metadata = file.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
                return Err("Invalid speech diagnostic file".to_string());
            }
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            writeln!(file, "{time}: {message}").map_err(|e| e.to_string())
        })();
        if let Err(error) = result {
            eprintln!("Could not preserve speech cache diagnostic: {error}");
        }
    }
}

fn lock(dir: &Dir) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .create(true)
        .follow(FollowSymlinks::No);
    let file = dir
        .open_with(".lock", &options)
        .map_err(|e| e.to_string())?
        .into_std();
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
            return Err("Invalid speech cache lock".into());
        }
    }
    let started = std::time::Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock)
                if started.elapsed() < std::time::Duration::from_secs(2) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(10))
            }
            Err(std::fs::TryLockError::WouldBlock) => return Err("Speech cache is busy".into()),
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(file)
}

#[cfg(test)]
pub fn read(database: &Path, id: i64) -> Result<Option<Vec<u8>>, String> {
    read_inner(database, id, false)
}

#[cfg(test)]
pub fn inspect(database: &Path, id: i64) -> Result<bool, String> {
    Ok(read_inner(database, id, false)?.is_some())
}

#[cfg(test)]
fn read_inner(database: &Path, id: i64, touch: bool) -> Result<Option<Vec<u8>>, String> {
    let dir = directory(database)?;
    let name = format!("{id}.ogg");
    read_named(&dir, &name, touch)
}

pub struct Inspector(Dir, PathBuf);

pub enum AudioInspection {
    Present(Vec<u8>),
    Missing,
    Invalid {
        error: String,
        observed_hash: Option<String>,
    },
}

impl AudioInspection {
    fn invalid(error: &str) -> Self {
        Self::Invalid {
            error: error.into(),
            observed_hash: None,
        }
    }
}

impl Inspector {
    pub fn new(database: &Path) -> Result<Self, String> {
        Ok(Self(directory(database)?, database.to_path_buf()))
    }
    pub fn legacy(&self, id: i64) -> Result<Option<Vec<u8>>, String> {
        read_named(&self.0, &format!("{id}.ogg"), false)
    }
    pub fn read_asset(&self, key: &str, hash: &str) -> Result<Option<Vec<u8>>, String> {
        read_named(&self.0, &asset_name(key, hash)?, false)
    }
    pub fn collect_legacy(&self, id: i64) -> Result<AudioInspection, String> {
        read_named_audio(&self.0, &format!("{id}.ogg"), false)
    }
    pub fn collect_version(
        &self,
        key: &str,
        hash: &str,
        length: Option<i64>,
    ) -> Result<AudioInspection, String> {
        let result = read_named_audio(&self.0, &asset_name(key, hash)?, false)?;
        match result {
            AudioInspection::Present(bytes)
                if lexique_storage::content_hash(&bytes) != hash
                    || length.is_some_and(|length| length != bytes.len() as i64) =>
            {
                Ok(AudioInspection::Invalid {
                    error: "Selected audio integrity check failed".into(),
                    observed_hash: Some(lexique_storage::content_hash(&bytes)),
                })
            }
            result => Ok(result),
        }
    }
    pub fn publish_version(&self, locator: &str, hash: &str, bytes: &[u8]) -> Result<(), String> {
        if lexique_storage::content_hash(bytes) != hash {
            return Err("Audio version checksum mismatch".into());
        }
        write_named(
            &self.1,
            &asset_name(locator, hash)?,
            &format!("s-{locator}-{hash}"),
            bytes,
            false,
        )
    }
    pub fn publish_legacy(&self, id: i64, hash: &str, bytes: &[u8]) -> Result<(), String> {
        if id <= 0 {
            return Err("Invalid legacy audio owner".into());
        }
        if lexique_storage::content_hash(bytes) != hash {
            return Err("Legacy audio checksum mismatch".into());
        }
        write_named(&self.1, &format!("{id}.ogg"), &id.to_string(), bytes, false)
    }
}

fn validate_hash(key: &str) -> Result<(), String> {
    if key.len() != 64
        || !key
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err("Invalid speech asset identity".into());
    }
    Ok(())
}

fn asset_name(key: &str, hash: &str) -> Result<String, String> {
    validate_hash(key)?;
    validate_hash(hash)?;
    Ok(format!("s-{key}-{hash}.ogg"))
}

fn read_named(dir: &Dir, name: &str, touch: bool) -> Result<Option<Vec<u8>>, String> {
    match read_named_audio(dir, name, touch)? {
        AudioInspection::Present(bytes) => Ok(Some(bytes)),
        AudioInspection::Missing => Ok(None),
        AudioInspection::Invalid { error, .. } => Err(error),
    }
}

fn read_named_audio(dir: &Dir, name: &str, touch: bool) -> Result<AudioInspection, String> {
    let path_metadata = match dir.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AudioInspection::Missing);
        }
        Err(error) => return Err(error.to_string()),
    };
    if !path_metadata.is_file() || path_metadata.file_type().is_symlink() {
        return Ok(AudioInspection::invalid("Invalid cached speech file"));
    }
    #[cfg(windows)]
    if path_metadata.file_attributes() & 0x400 != 0 {
        return Ok(AudioInspection::invalid("Invalid cached speech file"));
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    #[cfg(windows)]
    options.share_mode(3);
    let file = match dir.open_with(name, &options) {
        Ok(file) => file.into_std(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(AudioInspection::Missing);
        }
        Err(error) => return Err(error.to_string()),
    };
    // Validate the nofollow handle we actually read, avoiding a second path open and its race.
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Ok(AudioInspection::invalid("Invalid cached speech file"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        if metadata.file_attributes() & 0x400 != 0 {
            return Ok(AudioInspection::invalid("Invalid cached speech file"));
        }
    }
    if metadata.len() > 10_000_000 {
        return Ok(AudioInspection::invalid("Invalid cached speech audio"));
    }
    // Take<File> cannot use File's length hint; reserve it to avoid many tiny reads.
    let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
    #[cfg(windows)]
    if !touch {
        use std::os::windows::io::AsRawHandle;
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn SetFileTime(
                handle: *mut std::ffi::c_void,
                creation: *const u64,
                access: *const u64,
                write: *const u64,
            ) -> i32;
        }
        // Windows treats an all-ones access time as disabling updates for this handle.
        let disabled = u64::MAX;
        let success = unsafe {
            SetFileTime(
                file.as_raw_handle(),
                std::ptr::null(),
                &disabled,
                std::ptr::null(),
            )
        };
        if success == 0 {
            return Err(format!(
                "Could not preserve speech access time: {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    (&file)
        .take(10_000_001)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 10_000_000 {
        return Ok(AudioInspection::invalid("Invalid cached speech audio"));
    }
    if !bytes.starts_with(b"OggS") {
        return Ok(AudioInspection::Invalid {
            error: "Invalid cached speech audio".into(),
            observed_hash: Some(lexique_storage::content_hash(&bytes)),
        });
    }
    if touch {
        file.set_times(std::fs::FileTimes::new().set_accessed(std::time::SystemTime::now()))
            .map_err(|e| format!("Could not update speech access time: {e}"))?;
    }
    Ok(AudioInspection::Present(bytes))
}

#[cfg(windows)]
fn managed_name(name: &str) -> bool {
    if let Some(stem) = name
        .strip_prefix(".cpu-")
        .and_then(|s| s.strip_suffix(".config"))
    {
        let parts: Vec<_> = stem.split('-').collect();
        return parts.len() == 2
            && parts[0]
                .parse::<u32>()
                .is_ok_and(|pid| pid > 0 && pid.to_string() == parts[0])
            && parts[1]
                .parse::<u128>()
                .is_ok_and(|nonce| nonce.to_string() == parts[1]);
    }
    if let Some(stem) = name
        .strip_prefix(".s-")
        .and_then(|s| s.strip_suffix(".tmp"))
    {
        let parts: Vec<_> = stem.split('-').collect();
        return parts.len() == 4
            && validate_hash(parts[0]).is_ok()
            && validate_hash(parts[1]).is_ok()
            && parts[2]
                .parse::<u32>()
                .is_ok_and(|pid| pid > 0 && pid.to_string() == parts[2])
            && parts[3]
                .parse::<u128>()
                .is_ok_and(|nonce| nonce.to_string() == parts[3]);
    }
    if let Some(id) = name
        .strip_suffix(".ogg")
        .and_then(|s| s.parse::<i64>().ok())
    {
        return id > 0 && name == format!("{id}.ogg");
    }
    let Some(stem) = name.strip_prefix('.').and_then(|s| s.strip_suffix(".tmp")) else {
        return false;
    };
    let mut parts = stem.split('-');
    let (Some(id), Some(pid), Some(nonce), None) = (
        parts.next().and_then(|s| s.parse::<i64>().ok()),
        parts.next().and_then(|s| s.parse::<u32>().ok()),
        parts.next().and_then(|s| s.parse::<u128>().ok()),
        parts.next(),
    ) else {
        return false;
    };
    id > 0 && pid > 0 && name == format!(".{id}-{pid}-{nonce}.tmp")
}

#[cfg(windows)]
#[cfg(test)]
pub fn cleanup(database: &Path) -> Result<(), String> {
    let dir = directory(database)?;
    let _lock = lock(&dir)?;
    for entry in dir.entries().map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !managed_name(name) || !(name.ends_with(".tmp") || name.ends_with(".config")) {
            continue;
        }
        let metadata = dir.symlink_metadata(name).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
            continue;
        }
        let accessed = metadata.accessed().map_err(|e| e.to_string())?.into_std();
        if SystemTime::now()
            .duration_since(accessed)
            .unwrap_or_default()
            < Duration::from_secs(7 * 24 * 60 * 60)
        {
            continue;
        }
        if let Err(error) = dir.remove_file(name)
            && error.raw_os_error() != Some(32)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            report_error(
                database,
                &format!("Could not clean speech cache {name}: {error}"),
            );
        }
    }
    Ok(())
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CacheReport {
    pub file_bytes: u64,
    pub allocated_bytes: Option<u64>,
    pub unknown_files: u32,
    pub unknown_bytes: u64,
    pub unsafe_files: u32,
    pub scan_failures: u32,
    pub scanned_entries: u64,
    pub in_progress: bool,
    pub last_reconciled_at: Option<i64>,
    pub error: Option<String>,
}

impl Default for CacheReport {
    fn default() -> Self {
        Self {
            file_bytes: 0,
            allocated_bytes: Some(0),
            unknown_files: 0,
            unknown_bytes: 0,
            unsafe_files: 0,
            scan_failures: 0,
            scanned_entries: 0,
            in_progress: true,
            last_reconciled_at: None,
            error: None,
        }
    }
}

#[cfg(windows)]
pub struct Reconciler {
    dir: Dir,
    entries: cap_std::fs::ReadDir,
    pub report: CacheReport,
}

#[cfg(windows)]
impl Reconciler {
    pub fn new(database: &Path) -> Result<Self, String> {
        let dir = directory(database)?;
        let entries = dir.entries().map_err(|e| e.to_string())?;
        Ok(Self {
            dir,
            entries,
            report: CacheReport::default(),
        })
    }

    /// Retain the pinned iterator; do not restart a full directory scan for each chunk.
    pub fn step(
        &mut self,
        connection: &rusqlite::Connection,
        database: &Path,
        now: i64,
        should_stop: impl Fn() -> bool,
    ) -> Result<bool, String> {
        let mut names = Vec::with_capacity(200);
        for _ in 0..200 {
            let Some(entry) = self.entries.next() else {
                self.report.in_progress = false;
                self.report.last_reconciled_at = Some(now);
                break;
            };
            names.push(entry.map_err(|e| e.to_string())?.file_name());
            self.report.scanned_entries += 1;
        }
        let mut assets = Vec::new();
        let mut legacy = Vec::new();
        for name in &names {
            if let Some((physical, hash)) = name.to_str().and_then(asset_identity) {
                assets.push((physical.to_owned(), hash.to_owned()));
            } else if let Some(id) = name
                .to_str()
                .and_then(|s| s.strip_suffix(".ogg"))
                .and_then(|s| s.parse::<i64>().ok())
                .filter(|id| *id > 0)
            {
                legacy.push(id);
            }
        }
        let mut registered = std::collections::HashMap::new();
        if !assets.is_empty() {
            let values = vec!["(?,?)"; assets.len()].join(",");
            let mut query = connection.prepare(&format!("SELECT physical_locator,content_hash,locator,COALESCE(byte_length,0),gc_state FROM speech_versions WHERE (physical_locator,content_hash) IN (VALUES {values})")).map_err(|e| e.to_string())?;
            let rows = query
                .query_map(
                    rusqlite::params_from_iter(assets.iter().flat_map(|(p, h)| [p, h])),
                    |r| {
                        Ok((
                            (r.get::<_, String>(0)?, r.get::<_, String>(1)?),
                            (
                                r.get::<_, String>(2)?,
                                r.get::<_, i64>(3)?,
                                r.get::<_, String>(4)?,
                            ),
                        ))
                    },
                )
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (key, value) = row.map_err(|e| e.to_string())?;
                registered.insert(key, value);
            }
        }
        let mut legacy_words = std::collections::HashSet::new();
        if !legacy.is_empty() {
            let values = vec!["?"; legacy.len()].join(",");
            let mut query = connection
                .prepare(&format!("SELECT id FROM vocabulary WHERE id IN ({values})"))
                .map_err(|e| e.to_string())?;
            for row in query
                .query_map(rusqlite::params_from_iter(legacy), |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())?
            {
                legacy_words.insert(row.map_err(|e| e.to_string())?);
            }
        }
        let mut changed = false;
        for name in names {
            if should_stop() {
                break;
            }
            let result = (|| {
                let metadata = match self.dir.symlink_metadata(&name) {
                    Ok(metadata) => metadata,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(e) => return Err(e.to_string()),
                };
                if metadata.file_attributes() & 0x400 != 0 {
                    self.report.unsafe_files += 1;
                    return Ok(());
                }
                if !metadata.is_file() {
                    return Ok(());
                }
                let mut options = OpenOptions::new();
                options.read(true).follow(FollowSymlinks::No).share_mode(3);
                let file = match self.dir.open_with(&name, &options) {
                    Ok(file) => file.into_std(),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    Err(e) => return Err(e.to_string()),
                };
                let metadata = file.metadata().map_err(|e| e.to_string())?;
                use std::os::windows::fs::MetadataExt as _;
                if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
                    self.report.unsafe_files += 1;
                    return Ok(());
                }
                let text = name.to_str().unwrap_or("");
                let registration = asset_identity(text)
                    .and_then(|(p, h)| registered.get(&(p.to_owned(), h.to_owned())));
                if registration.is_some_and(|(_, _, state)| state == "deleted") {
                    drop(file);
                    let (physical, hash) =
                        asset_identity(text).ok_or("Invalid retired audio name")?;
                    changed |= remove_asset(database, physical, hash)?;
                    return Ok(());
                }
                if managed_name(text)
                    && (text.ends_with(".tmp") || text.ends_with(".config"))
                    && SystemTime::now()
                        .duration_since(metadata.modified().map_err(|e| e.to_string())?)
                        .unwrap_or_default()
                        >= Duration::from_secs(604800)
                {
                    drop(file);
                    let _guard = lock(&self.dir)?;
                    self.dir.remove_file(&name).map_err(|e| e.to_string())?;
                    return Ok(());
                }
                self.report.file_bytes = self.report.file_bytes.saturating_add(metadata.len());
                match allocated_size(&file) {
                    Ok(bytes) => {
                        self.report.allocated_bytes = self
                            .report
                            .allocated_bytes
                            .map(|total| total.saturating_add(bytes))
                    }
                    Err(error) => {
                        self.report.allocated_bytes = None;
                        self.failure(&error);
                    }
                }
                let known = text == ".lock"
                    || registration.is_some()
                    || (managed_name(text)
                        && (text.ends_with(".tmp") || text.ends_with(".config")))
                    || text
                        .strip_suffix(".ogg")
                        .and_then(|s| s.parse::<i64>().ok())
                        .is_some_and(|id| {
                            legacy_words.contains(&id) && text == format!("{id}.ogg")
                        });
                if !known {
                    self.report.unknown_files += 1;
                    self.report.unknown_bytes =
                        self.report.unknown_bytes.saturating_add(metadata.len());
                }
                if let Some((locator, expected, state)) = registration {
                    if *expected > 0 && *expected as u64 != metadata.len() && state == "live" {
                        changed |= connection.execute("UPDATE speech_versions SET verification='invalid' WHERE locator=?1 AND gc_state='live' AND verification<>'invalid'",[locator]).map_err(|e|e.to_string())?==1;
                    }
                }
                Ok::<_, String>(())
            })();
            if let Err(error) = result {
                self.failure(&error);
            }
        }
        Ok(changed)
    }

    fn failure(&mut self, error: &str) {
        self.report.scan_failures += 1;
        self.report.allocated_bytes = None;
        self.report.error = Some(format!("Speech cache reconciliation failed: {error}"));
    }
}

#[cfg(windows)]
fn asset_identity(name: &str) -> Option<(&str, &str)> {
    let (physical, hash) = name
        .strip_prefix("s-")?
        .strip_suffix(".ogg")?
        .split_once('-')?;
    validate_hash(physical).ok()?;
    validate_hash(hash).ok()?;
    Some((physical, hash))
}

#[cfg(windows)]
fn allocated_size(file: &File) -> Result<u64, String> {
    use std::{
        ffi::c_void,
        os::windows::{fs::MetadataExt as _, io::AsRawHandle},
    };
    #[repr(C)]
    #[derive(Default)]
    struct Standard {
        allocation: i64,
        end: i64,
        links: u32,
        deleting: u8,
        directory: u8,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Compression {
        size: i64,
        format: u16,
        unit: u8,
        chunk: u8,
        cluster: u8,
        reserved: [u8; 3],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandleEx(
            handle: *mut c_void,
            class: i32,
            info: *mut c_void,
            length: u32,
        ) -> i32;
    }
    let mut standard = Standard::default();
    // SAFETY: the nofollow file handle is pinned; the C layout and buffer size match FILE_STANDARD_INFO.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            1,
            std::ptr::from_mut(&mut standard).cast(),
            std::mem::size_of::<Standard>() as u32,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let mut bytes = standard.allocation;
    if file
        .metadata()
        .map_err(|e| e.to_string())?
        .file_attributes()
        & (0x800 | 0x200)
        != 0
    {
        let mut compression = Compression::default();
        // SAFETY: the same pinned handle and FILE_COMPRESSION_INFO layout cover compressed/sparse allocation.
        if unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                8,
                std::ptr::from_mut(&mut compression).cast(),
                std::mem::size_of::<Compression>() as u32,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
        bytes = compression.size;
    }
    u64::try_from(bytes).map_err(|_| "Invalid allocated speech cache size".into())
}

#[cfg(windows)]
fn pinned_path(dir: &Dir) -> std::io::Result<PathBuf> {
    use std::{
        ffi::c_void,
        os::windows::{ffi::OsStringExt, io::AsRawHandle},
    };

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFinalPathNameByHandleW(
            handle: *mut c_void,
            path: *mut u16,
            size: u32,
            flags: u32,
        ) -> u32;
    }
    // SAFETY: The directory handle remains open; a null buffer requests its required size.
    let size =
        unsafe { GetFinalPathNameByHandleW(dir.as_raw_handle(), std::ptr::null_mut(), 0, 0) };
    if size == 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut buffer = vec![0; size as usize];
    // SAFETY: The writable buffer has the advertised size and the directory remains pinned.
    let length =
        unsafe { GetFinalPathNameByHandleW(dir.as_raw_handle(), buffer.as_mut_ptr(), size, 0) };
    if length == 0 {
        return Err(std::io::Error::last_os_error());
    }
    if length >= size {
        return Err(std::io::Error::other(
            "Pinned speech directory path changed",
        ));
    }
    Ok(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..length as usize],
    )))
}

#[cfg(windows)]
fn publish_new(dir: &Dir, temporary: &str, name: &str) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileW(from: *const u16, to: *const u16) -> i32;
    }
    let path = pinned_path(dir)?;
    let wide = |name: &str| {
        path.join(name)
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>()
    };
    let from = wide(temporary);
    let to = wide(name);
    // SAFETY: Names are internal, buffers are NUL-terminated, and cap-std pins the directory
    // without FILE_SHARE_DELETE, just as in its Windows rename implementation.
    if unsafe { MoveFileW(from.as_ptr(), to.as_ptr()) } != 0 {
        return Ok(());
    }
    let error = std::io::Error::last_os_error();
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        return Ok(());
    }
    Err(error)
}

#[cfg(windows)]
pub struct WorkerConfiguration {
    directory: Dir,
    file: Option<cap_std::fs::File>,
    pub path: PathBuf,
    name: String,
}

#[cfg(windows)]
impl Drop for WorkerConfiguration {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Err(error) = self.directory.remove_file(&self.name) {
            eprintln!("Could not remove CPU worker configuration: {error}");
        }
    }
}

#[cfg(windows)]
pub fn worker_configuration(database: &Path) -> Result<WorkerConfiguration, String> {
    let directory = directory(database)?;
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let name = format!(".cpu-{}-{nonce}.config", std::process::id());
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No)
        .share_mode(1);
    let mut file = directory
        .open_with(&name, &options)
        .map_err(|e| e.to_string())?;
    file.write_all(b"SessionConfig.session.intra_op.allow_spinning=0\nSessionConfig.session.inter_op.allow_spinning=0\n").map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    let path = pinned_path(&directory)
        .map_err(|e| e.to_string())?
        .join(&name);
    Ok(WorkerConfiguration {
        directory,
        file: Some(file),
        path,
        name,
    })
}

#[cfg(test)]
pub fn write(database: &Path, id: i64, bytes: &[u8], replace: bool) -> Result<(), String> {
    write_named(
        database,
        &format!("{id}.ogg"),
        &id.to_string(),
        bytes,
        replace,
    )
}

#[cfg(test)]
pub fn write_asset(database: &Path, key: &str, hash: &str, bytes: &[u8]) -> Result<(), String> {
    if lexique_storage::content_hash(bytes) != hash {
        return Err("Generated speech content hash mismatch".into());
    }
    write_named(
        database,
        &asset_name(key, hash)?,
        &format!("s-{key}-{hash}"),
        bytes,
        true,
    )
}

pub fn remove_asset(database: &Path, key: &str, hash: &str) -> Result<bool, String> {
    let dir = directory(database)?;
    let _guard = lock(&dir)?;
    let name = asset_name(key, hash)?;
    let metadata = match dir.symlink_metadata(&name) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Invalid speech asset file".into());
    }
    #[cfg(windows)]
    if metadata.file_attributes() & 0x400 != 0 {
        return Err("Invalid speech asset file".into());
    }
    dir.remove_file(name).map_err(|error| error.to_string())?;
    Ok(true)
}

fn write_named(
    database: &Path,
    name: &str,
    stem: &str,
    bytes: &[u8],
    replace: bool,
) -> Result<(), String> {
    if bytes.len() > 10_000_000 || !bytes.starts_with(b"OggS") {
        return Err("Invalid generated speech audio".into());
    }
    let dir = directory(database)?;
    let _lock = lock(&dir)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temporary = format!(".{stem}-{}-{nonce}.tmp", std::process::id());
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = dir
        .open_with(&temporary, &options)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(bytes).map_err(|e| e.to_string())?;
        #[cfg(test)]
        crash_boundary("tmp_written");
        file.sync_all().map_err(|e| e.to_string())?;
        #[cfg(test)]
        crash_boundary("tmp_synced");
        drop(file);
        if replace {
            return dir
                .rename(&temporary, &dir, name)
                .map_err(|e| e.to_string());
        }
        #[cfg(windows)]
        return publish_new(&dir, &temporary, name)
            .map_err(|error| format!("Could not publish cached speech audio: {error}"));
        #[cfg(not(windows))]
        match dir.hard_link(&temporary, &dir, name) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    })();
    #[cfg(test)]
    if result.is_ok() {
        crash_boundary("renamed");
    }
    if let Err(error) = dir.remove_file(&temporary)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        report_error(
            database,
            &format!("Could not remove temporary speech file: {error}"),
        );
    }
    result
}

// Available only in test binaries, including the subprocess used by the native
// crash matrix. No environment-controlled pause exists in the shipped application.
#[cfg(test)]
pub fn crash_boundary(point: &str) {
    if std::env::var("LEXIQUE_TEST_CRASH_POINT").as_deref() != Ok(point) {
        return;
    }
    let marker = std::env::var_os("LEXIQUE_TEST_CRASH_MARKER").expect("crash marker path");
    std::fs::write(marker, point).expect("write crash marker");
    // The supervising harness kills this process after independently inspecting
    // the durable boundary. EOF is an error, never simulated crash success.
    let mut release = [0];
    std::io::stdin()
        .read_exact(&mut release)
        .expect("crash barrier released");
    panic!("crash boundary was released instead of killed");
}
