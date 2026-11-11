#![cfg(windows)]

#[path = "../src/speech_cache.rs"]
#[allow(dead_code)]
mod speech_cache;

use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

/// Executed explicitly by native-crash-24.mjs after precompilation. The native
/// application performs the restart/recovery assertions against these real files.
#[test]
#[ignore = "subprocess crash fixture; requires explicit isolated paths"]
fn publication_crash_fixture() {
    use lexique_storage::*;
    let database = PathBuf::from(std::env::var_os("LEXIQUE_TEST_CRASH_DATABASE").unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    initialize(&connection).unwrap();
    let inspector = speech_cache::Inspector::new(&database).unwrap();
    let point = std::env::var("LEXIQUE_TEST_CRASH_POINT").unwrap();
    let bytes = include_bytes!(
        "../../../../docs/evidence/issue-14/supertonic-opus/female-normal-01-48k.opus"
    );
    if point.starts_with("gc_") {
        let key = register_speech(
            &connection,
            &SpeechSpec::word("temporary", SpeechVoice::Female),
        )
        .unwrap();
        let version = prepare_source_audio(
            &connection,
            &key,
            &content_hash(bytes),
            bytes.len() as i64,
            1000,
        )
        .unwrap();
        inspector
            .publish_version(&version.physical_locator, &version.content_hash, bytes)
            .unwrap();
        verify_audio_content(&connection, &version.locator, bytes.len() as i64).unwrap();
        complete_source_audio(&connection, &version.locator).unwrap();
        let gc = mark_audio_gc(&connection, &version.locator, 605800)
            .unwrap()
            .unwrap();
        speech_cache::crash_boundary("gc_marked");
        assert!(
            speech_cache::remove_asset(&database, &version.physical_locator, &version.content_hash)
                .unwrap()
        );
        speech_cache::crash_boundary("gc_deleted");
        assert!(finish_audio_gc(&connection, &version.locator, gc.generation).unwrap());
        speech_cache::crash_boundary("gc_finished");
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let version = prepare_audio_version(
            &connection,
            1,
            None,
            lexique_storage::AudioPublicationContent {
                hash: &content_hash(bytes),
                bytes: bytes.len() as i64,
                generated_at: None,
            },
            0,
            now,
        )
        .unwrap();
        speech_cache::crash_boundary("intent_committed");
        inspector
            .publish_version(&version.physical_locator, &version.content_hash, bytes)
            .unwrap();
        let read = inspector
            .read_asset(&version.physical_locator, &version.content_hash)
            .unwrap()
            .unwrap();
        assert_eq!(read, bytes);
        verify_audio_content(&connection, &version.locator, read.len() as i64).unwrap();
        speech_cache::crash_boundary("readback");
        assert!(complete_audio_version(&connection, &version.locator, now).unwrap());
        speech_cache::crash_boundary("db_complete");
    }
    panic!("unknown/unreached crash point: {point}");
}

fn cache_root(nonce: u128) -> PathBuf {
    static NEXT_CACHE_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/speech-cache-tests")
        .join(format!("{}-{nonce}-{id}", std::process::id()))
}

#[test]
fn cache_roots_stay_unique_when_clock_values_repeat() {
    assert_ne!(cache_root(0), cache_root(0));
}

struct Cache(PathBuf);

impl Cache {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = cache_root(nonce);
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
fn reconciliation_advances_in_bounded_chunks_and_preserves_unregistered_files() {
    let cache = Cache::new();
    let database = cache.database();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    lexique_storage::initialize(&connection).unwrap();
    let inspector = speech_cache::Inspector::new(&database).unwrap();
    let bytes = b"OggSunknown";
    for index in 0..450 {
        std::fs::write(
            cache.speech().join(format!("unregistered-{index}.ogg")),
            bytes,
        )
        .unwrap();
    }
    let unknown = format!(
        "s-{}-{}.ogg",
        "a".repeat(64),
        lexique_storage::content_hash(bytes)
    );
    std::fs::write(cache.speech().join(&unknown), bytes).unwrap();
    let mut scan = speech_cache::Reconciler::new(&database).unwrap();
    assert!(!scan.step(&connection, &database, 1000, || false).unwrap());
    assert!(scan.report.in_progress);
    assert_eq!(scan.report.scanned_entries, 200);
    for _ in 0..2 {
        scan.step(&connection, &database, 1000, || false).unwrap();
    }
    assert!(!scan.report.in_progress);
    assert_eq!(scan.report.unknown_files, 451);
    assert_eq!(scan.report.file_bytes, 451 * bytes.len() as u64);
    assert!(scan.report.allocated_bytes.unwrap() >= scan.report.file_bytes);
    assert_eq!(scan.report.scan_failures, 0);
    assert_eq!(std::fs::read(cache.speech().join(unknown)).unwrap(), bytes);
    drop(inspector);
}

#[test]
fn occupied_gc_file_stays_marked_until_deletion_can_finish() {
    use lexique_storage::*;
    use std::os::windows::fs::OpenOptionsExt;
    let cache = Cache::new();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    initialize(&connection).unwrap();
    let key = register_speech(
        &connection,
        &SpeechSpec::word("temporary", SpeechVoice::Female),
    )
    .unwrap();
    let bytes = b"OggSgc-owned";
    let version = prepare_source_audio(
        &connection,
        &key,
        &content_hash(bytes),
        bytes.len() as i64,
        1000,
    )
    .unwrap();
    let inspector = speech_cache::Inspector::new(&cache.database()).unwrap();
    inspector
        .publish_version(&version.physical_locator, &version.content_hash, bytes)
        .unwrap();
    verify_audio_content(&connection, &version.locator, bytes.len() as i64).unwrap();
    complete_source_audio(&connection, &version.locator).unwrap();
    let gc = mark_audio_gc(&connection, &version.locator, 605800)
        .unwrap()
        .unwrap();
    let path = cache.speech().join(format!(
        "s-{}-{}.ogg",
        version.physical_locator, version.content_hash
    ));
    let occupied = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(&path)
        .unwrap();
    let error = speech_cache::remove_asset(
        &cache.database(),
        &version.physical_locator,
        &version.content_hash,
    )
    .unwrap_err();
    fail_audio_gc(&connection, &version.locator, gc.generation, &error).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(
        connection
            .query_row(
                "SELECT gc_state FROM speech_versions WHERE locator=?1",
                [&version.locator],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "marked"
    );
    drop(occupied);
    assert!(
        speech_cache::remove_asset(
            &cache.database(),
            &version.physical_locator,
            &version.content_hash
        )
        .unwrap()
    );
    assert!(finish_audio_gc(&connection, &version.locator, gc.generation).unwrap());
    assert!(!path.exists());
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
