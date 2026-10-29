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

fn report_error(_database: &Path, message: &str) {
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
    file.lock().map_err(|e| e.to_string())?;
    Ok(file)
}

pub fn read(database: &Path, id: i64) -> Result<Option<Vec<u8>>, String> {
    read_inner(database, id, true)
}

pub fn inspect(database: &Path, id: i64) -> Result<bool, String> {
    Ok(read_inner(database, id, false)?.is_some())
}

fn read_inner(database: &Path, id: i64, touch: bool) -> Result<Option<Vec<u8>>, String> {
    let dir = directory(database)?;
    let name = format!("{id}.ogg");
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
        .open_with(&name, &options)
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
fn cleanup(database: &Path) -> Result<(), String> {
    let dir = directory(database)?;
    let _lock = lock(&dir)?;
    for entry in dir.entries().map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !managed_name(name) {
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
pub fn start_cleanup(database: PathBuf) {
    std::thread::spawn(move || {
        loop {
            if let Err(error) = cleanup(&database) {
                report_error(&database, &format!("Could not clean speech cache: {error}"));
            }
            std::thread::sleep(Duration::from_secs(24 * 60 * 60));
        }
    });
}

pub fn write(database: &Path, id: i64, bytes: &[u8], replace: bool) -> Result<(), String> {
    let dir = directory(database)?;
    let _lock = lock(&dir)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temporary = format!(".{id}-{}-{nonce}.tmp", std::process::id());
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = dir
        .open_with(&temporary, &options)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        if replace {
            return dir
                .rename(&temporary, &dir, format!("{id}.ogg"))
                .map_err(|e| e.to_string());
        }
        match dir.hard_link(&temporary, &dir, format!("{id}.ogg")) {
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
