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
    if !cfg!(target_os = "windows") {
        return Err("Speech cache is not available on this platform".into());
    }
    let data = database
        .parent()
        .and_then(Path::parent)
        .ok_or("Invalid data path")?;
    let parent = data.parent().ok_or("Invalid portable data path")?;
    let name = data.file_name().ok_or("Invalid portable data path")?;
    let portable = Dir::open_ambient_dir(parent, ambient_authority()).map_err(|e| e.to_string())?;
    open_child(&portable, Path::new(name))
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

pub struct Inspector(Dir);

impl Inspector {
    pub fn new(database: &Path) -> Result<Self, String> {
        Ok(Self(directory(database)?))
    }
    pub fn legacy(&self, id: i64) -> Result<Option<Vec<u8>>, String> {
        read_named(&self.0, &format!("{id}.ogg"), false)
    }
    pub fn read_asset(&self, key: &str, hash: &str) -> Result<Option<Vec<u8>>, String> {
        read_named(&self.0, &asset_name(key, hash)?, false)
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
    match dir.symlink_metadata(&name) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
        Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() => {
            return Err("Invalid cached speech file".into());
        }
        #[cfg(windows)]
        Ok(meta) if meta.file_attributes() & 0x400 != 0 => {
            return Err("Invalid cached speech file".into());
        }
        Ok(meta) if meta.len() > 10_000_000 => return Err("Invalid cached speech audio".into()),
        Ok(_) => (),
    }
    let mut bytes = Vec::new();
    let mut options = OpenOptions::new();
    options.read(true).write(true).follow(FollowSymlinks::No);
    #[cfg(windows)]
    options.share_mode(3);
    let file = dir
        .open_with(name, &options)
        .map_err(|e| e.to_string())?
        .into_std();
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
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
            return Err("Invalid cached speech file".into());
        }
    }
    (&file)
        .take(10_000_001)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 10_000_000 || !bytes.starts_with(b"OggS") {
        return Err("Invalid cached speech audio".into());
    }
    if touch {
        file.set_times(std::fs::FileTimes::new().set_accessed(std::time::SystemTime::now()))
            .map_err(|e| format!("Could not update speech access time: {e}"))?;
    }
    Ok(Some(bytes))
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
        file.sync_all().map_err(|e| e.to_string())?;
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
