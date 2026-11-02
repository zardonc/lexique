#![cfg(windows)]

#[path = "../src/speech_cache.rs"]
#[allow(dead_code)]
mod speech_cache;

use std::path::PathBuf;

struct Cache(PathBuf);

impl Cache {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/speech-cache-tests")
            .join(format!("{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(root.join("Lexique-data/data")).unwrap();
        Self(root.canonicalize().unwrap())
    }

    fn database(&self) -> PathBuf {
        self.0.join("Lexique-data/data/test.sqlite3")
    }

    fn speech(&self) -> PathBuf {
        self.0.join("Lexique-data/temp/speech")
    }

    fn assert_no_temporary(&self) {
        assert!(std::fs::read_dir(self.speech()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        }));
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        let owned = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../target/speech-cache-tests")
            .canonicalize()
            .unwrap();
        assert!(self.0.starts_with(owned));
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn immutable_asset_versions_and_legacy_files_survive_targeted_gc() {
    let cache = Cache::new();
    let database = cache.database();
    let key = "a".repeat(64);
    let first = b"OggSfirst";
    let second = b"OggSsecond";
    let one = lexique_storage::content_hash(first);
    let two = lexique_storage::content_hash(second);
    speech_cache::write(&database, 1, b"OggSlegacy", false).unwrap();
    speech_cache::write_asset(&database, &key, &one, first).unwrap();
    speech_cache::write_asset(&database, &key, &two, second).unwrap();
    let inspector = speech_cache::Inspector::new(&database).unwrap();
    assert_eq!(inspector.read_asset(&key, &one).unwrap().unwrap(), first);
    assert_eq!(inspector.read_asset(&key, &two).unwrap().unwrap(), second);
    assert!(speech_cache::remove_asset(&database, &key, &one).unwrap());
    assert!(!speech_cache::remove_asset(&database, &key, &one).unwrap());
    assert!(inspector.read_asset(&key, &one).unwrap().is_none());
    assert_eq!(inspector.read_asset(&key, &two).unwrap().unwrap(), second);
    assert_eq!(inspector.legacy(1).unwrap().unwrap(), b"OggSlegacy");
    assert!(speech_cache::write_asset(&database, &key, &one, second).is_err());
    assert!(inspector.read_asset("../../escape", &two).is_err());
    cache.assert_no_temporary();
}

#[test]
fn publishes_new_audio_without_overwriting_an_existing_file() {
    let cache = Cache::new();
    let database = cache.database();
    assert_eq!(speech_cache::read(&database, 1).unwrap(), None);
    speech_cache::write(&database, 1, b"OggSfirst", false).unwrap();
    speech_cache::write(&database, 1, b"OggSsecond", false).unwrap();
    assert_eq!(
        speech_cache::read(&database, 1).unwrap().unwrap(),
        b"OggSfirst"
    );
    speech_cache::write(&database, 1, b"OggSreplacement", true).unwrap();
    assert_eq!(
        speech_cache::read(&database, 1).unwrap().unwrap(),
        b"OggSreplacement"
    );
    cache.assert_no_temporary();
}

#[test]
fn concurrent_publishers_preserve_one_complete_audio_file() {
    let cache = Cache::new();
    let database = cache.database();
    let writers = 8;
    let barrier = std::sync::Barrier::new(writers);
    let outputs: Vec<_> = (0..writers)
        .map(|index| format!("OggS{index}{}", "x".repeat(65536)).into_bytes())
        .collect();
    std::thread::scope(|scope| {
        for bytes in &outputs {
            scope.spawn(|| {
                barrier.wait();
                speech_cache::write(&database, 1, bytes, false).unwrap();
            });
        }
    });
    let published = speech_cache::read(&database, 1).unwrap().unwrap();
    assert!(outputs.contains(&published));
    cache.assert_no_temporary();
}

#[test]
fn failed_replacement_preserves_audio_and_removes_temporary_file() {
    use std::os::windows::fs::OpenOptionsExt;
    let cache = Cache::new();
    let database = cache.database();
    speech_cache::write(&database, 1, b"OggSoriginal", false).unwrap();
    let occupied = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(cache.speech().join("1.ogg"))
        .unwrap();
    assert!(speech_cache::write(&database, 1, b"OggSreplacement", true).is_err());
    assert_eq!(
        std::fs::read(cache.speech().join("1.ogg")).unwrap(),
        b"OggSoriginal"
    );
    drop(occupied);
    cache.assert_no_temporary();
}

#[test]
fn existing_directory_is_not_replaced_or_followed() {
    let cache = Cache::new();
    let database = cache.database();
    speech_cache::read(&database, 1).unwrap();
    let target = cache.speech().join("1.ogg");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("protected"), b"preserve").unwrap();
    let _ = speech_cache::write(&database, 1, b"OggSnew", false);
    assert_eq!(
        std::fs::read(target.join("protected")).unwrap(),
        b"preserve"
    );
    assert!(speech_cache::read(&database, 1).is_err());
    cache.assert_no_temporary();
}

#[test]
fn pinned_cache_directory_prevents_ancestor_retargeting() {
    let cache = Cache::new();
    speech_cache::read(&cache.database(), 1).unwrap();
    let directory =
        cap_std::fs::Dir::open_ambient_dir(cache.speech(), cap_std::ambient_authority()).unwrap();
    for target in [
        cache.speech(),
        cache.0.join("Lexique-data/temp"),
        cache.0.join("Lexique-data"),
    ] {
        let error = std::fs::rename(&target, cache.0.join("retargeted")).unwrap_err();
        assert!(
            matches!(error.raw_os_error(), Some(5 | 32)),
            "{target:?}: {error}"
        );
        assert!(target.is_dir());
    }
    drop(directory);
    let moved = cache.0.join("retargeted");
    std::fs::rename(cache.speech(), &moved).unwrap();
    std::fs::rename(moved, cache.speech()).unwrap();
}

#[test]
fn cache_directory_reparse_point_is_rejected() {
    let cache = Cache::new();
    let protected = cache.0.join("outside");
    std::fs::create_dir(&protected).unwrap();
    std::fs::write(protected.join("1.ogg"), b"OggSprotected").unwrap();
    std::fs::create_dir_all(cache.0.join("Lexique-data/temp")).unwrap();
    let result = std::process::Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "New-Item -ItemType Junction -Path $env:LEXIQUE_TEST_JUNCTION -Target $env:LEXIQUE_TEST_TARGET -ErrorAction Stop | Out-Null"])
        .env("LEXIQUE_TEST_JUNCTION", cache.speech().to_string_lossy().trim_start_matches(r"\\?\"))
        .env("LEXIQUE_TEST_TARGET", protected.to_string_lossy().trim_start_matches(r"\\?\"))
        .output().unwrap();
    if !result.status.success() {
        eprintln!(
            "SKIPPED: cache junction creation unavailable: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    assert!(speech_cache::read(&cache.database(), 1).is_err());
    assert!(speech_cache::write(&cache.database(), 1, b"OggSnew", false).is_err());
    assert!(speech_cache::write(&cache.database(), 1, b"OggSnew", true).is_err());
    assert_eq!(
        std::fs::read(protected.join("1.ogg")).unwrap(),
        b"OggSprotected"
    );
    std::fs::remove_dir(cache.speech()).unwrap();
}

#[test]
fn uncoordinated_writer_cannot_be_overwritten_during_publication() {
    let cache = Cache::new();
    let database = cache.database();
    speech_cache::read(&database, 1).unwrap();
    for id in 1..=32 {
        let barrier = std::sync::Barrier::new(2);
        let target = cache.speech().join(format!("{id}.ogg"));
        let outsider_created = std::thread::scope(|scope| {
            scope.spawn(|| {
                barrier.wait();
                speech_cache::write(&database, id, b"OggSpublisher", false).unwrap();
            });
            barrier.wait();
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
            {
                Ok(mut file) => {
                    use std::io::Write;
                    file.write_all(b"OggSoutsider").unwrap();
                    drop(file);
                    true
                }
                Err(error) => {
                    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
                    false
                }
            }
        });
        let bytes = std::fs::read(&target).unwrap();
        assert_eq!(
            bytes,
            if outsider_created {
                &b"OggSoutsider"[..]
            } else {
                &b"OggSpublisher"[..]
            }
        );
    }
    cache.assert_no_temporary();
}
