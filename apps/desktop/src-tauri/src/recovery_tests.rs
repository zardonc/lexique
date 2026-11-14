//! Recovery uses verified staging and real isolated originals without learning workers.
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use lexique_storage as storage;
use rusqlite::Connection;

use crate::{
    backup::export_paused,
    backup_preview::prepare_preview,
    restore::{PreservationKind, RawPendingRequest, StartupPhase},
    restore_tests::{Fixture, interrupt_child_at, save},
    speech_cache::managed_audio_directory,
};

const AUDIO: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/female-1.ogg");
const MDX: &[u8] =
    include_bytes!("../../../../resources/freedict-fra-eng-mdx-0.4.1/freedict-fra-eng-0.4.1.mdx");

#[test]
fn failed_startup_preserves_the_real_wal_database_and_sidecars_on_every_retry() {
    let fixture = Fixture::new();
    interrupt_child_at(
        &fixture,
        "future-wal-ready",
        "seed-future-wal",
        "recovery_tests::recovery_interruption_child",
    );
    let originals: Vec<_> = ["", "-wal", "-shm"]
        .into_iter()
        .map(|suffix| {
            let path = PathBuf::from(format!("{}{suffix}", fixture.database.0.display()));
            let bytes = fs::read(&path).unwrap();
            assert!(!bytes.is_empty());
            (path, bytes)
        })
        .collect();
    for _ in 0..2 {
        let status = crate::startup::load(
            &fixture.database,
            |_| Ok(()),
            || panic!("unsupported WAL source admitted learning"),
        );
        assert_eq!(status.state, StartupPhase::RecoveryRequired, "{status:?}");
        assert!(
            status
                .error
                .unwrap()
                .contains("Unsupported database schema 99")
        );
        for (path, bytes) in &originals {
            assert!(
                fs::read(path).unwrap() == *bytes,
                "failed startup changed {}",
                path.display()
            );
        }
    }
}

#[test]
#[ignore = "owned child entry point; invoked only by the interruption parent"]
fn recovery_interruption_child() {
    use std::io::Write;

    let database = crate::vocabulary::DatabasePath(PathBuf::from(
        std::env::var_os("LEXIQUE_RESTORE_TEST_DATABASE").unwrap(),
    ));
    let root = PathBuf::from(std::env::var_os("LEXIQUE_RESTORE_TEST_ROOT").unwrap());
    let checkpoint = std::env::var("LEXIQUE_RESTORE_TEST_CHECKPOINT").unwrap();
    let action = std::env::var("LEXIQUE_RESTORE_TEST_ACTION").unwrap();
    let observe = |point: &str| -> Result<(), String> {
        if point == checkpoint {
            let mut marker = fs::File::create(root.join("interruption-ready")).unwrap();
            marker.write_all(point.as_bytes()).unwrap();
            marker.sync_all().unwrap();
            loop {
                std::thread::park();
            }
        }
        if action == "raw-rollback" && point == "after-install-audio" {
            return Err("Owned raw restore installation failed before commitment".into());
        }
        Ok(())
    };
    if action == "seed-future-wal" {
        let connection = Connection::open(&database.0).unwrap();
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; PRAGMA user_version=99; CREATE TABLE recovery_sentinel(value TEXT); INSERT INTO recovery_sentinel VALUES ('old WAL timeline')").unwrap();
        observe("future-wal-ready").unwrap();
    } else if action == "raw-prepare" {
        let stage = prepare_preview(
            &root.join("recovery.lexique-backup"),
            database.0.parent().unwrap(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let result = crate::restore::prepare_raw(
            &database,
            stage,
            "raw-interruption",
            &raw_requests(),
            || Ok(()),
            observe,
        );
        panic!("Raw preparation missed checkpoint {checkpoint}: {result:?}");
    } else if action == "raw-cleanup" {
        let status = crate::startup::load(
            &database,
            |_| Ok(()),
            || panic!("cleanup admitted learning before acknowledgement"),
        );
        assert_eq!(status.state, StartupPhase::CleanupRequired);
        crate::restore::cleanup_begin(&database, "raw-interruption", observe).unwrap();
        let status =
            crate::restore::cleanup_complete(&database, "raw-interruption", observe).unwrap();
        panic!("Raw cleanup missed checkpoint {checkpoint}: {status:?}");
    } else {
        assert!(action == "raw-startup" || action == "raw-rollback");
        let status = crate::startup::load(&database, observe, || Ok(()));
        panic!("Raw startup missed checkpoint {checkpoint}: {status:?}");
    }
}

#[test]
fn ordinary_startup_rejects_current_schema_with_missing_learning_tables() {
    let fixture = Fixture::new();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection.execute_batch("DROP TABLE vocabulary").unwrap();
    connection.close().unwrap();
    let initialized = AtomicBool::new(false);
    let status = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, Ordering::Release);
            Ok(())
        },
    );
    assert_eq!(status.state, StartupPhase::RecoveryRequired);
    assert!(status.error.unwrap().contains("vocabulary"));
    assert!(!initialized.load(Ordering::Acquire));
}

#[cfg(windows)]
#[test]
fn unusable_portable_temp_blocks_learning_without_changing_the_original_database() {
    let fixture = Fixture::new();
    let original = fs::read(&fixture.database.0).unwrap();
    let temporary = fixture.root.0.join("Lexique-data/temp");
    fs::write(&temporary, b"the intended temporary directory is blocked").unwrap();
    let initialized = AtomicBool::new(false);
    let status = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, Ordering::Release);
            Ok(())
        },
    );
    assert_eq!(status.state, StartupPhase::RecoveryRequired, "{status:?}");
    assert!(status.error.unwrap().contains("temp"));
    assert!(!initialized.load(Ordering::Acquire));
    assert!(fs::read(&fixture.database.0).unwrap() == original);
    fs::remove_file(&temporary).unwrap();
    let retry = crate::startup::load(&fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(retry.state, StartupPhase::Ready, "{retry:?}");
}

fn raw_requests() -> Vec<RawPendingRequest> {
    vec![
        RawPendingRequest {
            key: "lexique-pending-rating-not-a-card".into(),
            value: "{malformed".into(),
        },
        RawPendingRequest {
            key: "lexique-pending-undo".into(),
            value: String::new(),
        },
        RawPendingRequest {
            key: "lexique-pending-review-batch".into(),
            value: "null".into(),
        },
    ]
}

fn broken_originals(fixture: &Fixture) -> Vec<(PathBuf, Vec<u8>, PathBuf)> {
    let directory = fixture.database.0.parent().unwrap();
    let mut files = vec![(
        fixture.database.0.clone(),
        b"unreadable SQLite header".to_vec(),
        PathBuf::from("database/lexique.sqlite3"),
    )];
    for (suffix, bytes) in [
        ("-wal", b"raw WAL".as_slice()),
        ("-shm", b"raw SHM".as_slice()),
        ("-journal", b"raw rollback journal".as_slice()),
    ] {
        files.push((
            PathBuf::from(format!("{}{suffix}", fixture.database.0.display())),
            bytes.to_vec(),
            PathBuf::from(format!("database/lexique.sqlite3{suffix}")),
        ));
    }
    files.push((
        managed_audio_directory(&fixture.database.0)
            .unwrap()
            .join("nested/original.ogg"),
        AUDIO.to_vec(),
        PathBuf::from("audio/nested/original.ogg"),
    ));
    for id in [1, 9] {
        files.push((
            directory.join(format!("dictionaries/mdx-{id}.mdx")),
            MDX.to_vec(),
            PathBuf::from(format!("dictionaries/mdx-{id}.mdx")),
        ));
    }
    for (path, bytes, _) in &files {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    fs::create_dir(directory.join("dictionaries/empty-retained")).unwrap();
    files
}

#[test]
fn raw_restore_retains_every_original_byte_and_pending_string_before_prepared() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let record = save(&mut connection, "A from the validated backup");
    connection.close().unwrap();
    let archive = fixture.root.0.join("A.lexique-backup");
    let cancelled = AtomicBool::new(false);
    export_paused(&fixture.database, &archive, &cancelled).unwrap();
    let originals = broken_originals(&fixture);
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let raw = fixture
        .database
        .0
        .parent()
        .unwrap()
        .join("retained-restores/recovery-A/raw-files");
    let closed = AtomicBool::new(false);
    let prepared = crate::restore::prepare_raw(
        &fixture.database,
        stage,
        "recovery-A",
        &raw_requests(),
        || {
            closed.store(true, Ordering::Release);
            Ok(())
        },
        |point| {
            if point == "prepared" {
                assert!(closed.load(Ordering::Acquire));
                for (path, bytes, relative) in &originals {
                    assert_eq!(fs::read(path).unwrap(), *bytes);
                    assert_eq!(fs::read(raw.join(relative)).unwrap(), *bytes);
                }
                let saved: Vec<RawPendingRequest> =
                    serde_json::from_slice(&fs::read(raw.join("pending-requests.json")).unwrap())
                        .unwrap();
                assert_eq!(saved, raw_requests());
            }
            Ok(())
        },
    )
    .unwrap();
    let copy = prepared
        .retained_copies
        .iter()
        .find(|copy| copy.kind == PreservationKind::RawFiles)
        .unwrap();
    assert_eq!(PathBuf::from(&copy.path), raw);
    assert!(
        prepare_preview(
            &raw.join("database/lexique.sqlite3"),
            fixture.database.0.parent().unwrap(),
            &cancelled,
            |_| {}
        )
        .is_err(),
        "raw SQLite files were admitted by the logical importer"
    );
    assert!(raw.join("dictionaries/empty-retained").is_dir());
    assert_eq!(
        copy.byte_length,
        originals
            .iter()
            .map(|(_, bytes, _)| bytes.len() as u64)
            .sum::<u64>()
            + fs::metadata(raw.join("pending-requests.json"))
                .unwrap()
                .len()
    );
    let initialized = AtomicBool::new(false);
    let startup = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, Ordering::Release);
            Ok(())
        },
    );
    assert_eq!(startup.state, StartupPhase::CleanupRequired);
    assert!(!initialized.load(Ordering::Acquire));
    crate::restore::cleanup_begin(&fixture.database, "recovery-A", |_| Ok(())).unwrap();
    crate::restore::cleanup_complete(&fixture.database, "recovery-A", |_| Ok(())).unwrap();
    let ready = crate::startup::load(&fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(ready.state, StartupPhase::Ready);
    assert_eq!(
        storage::get(
            &Connection::open(&fixture.database.0).unwrap(),
            record.record.id
        )
        .unwrap()
        .notes,
        "A from the validated backup"
    );
    for (_, bytes, relative) in &originals {
        assert_eq!(fs::read(raw.join(relative)).unwrap(), *bytes);
    }
    for suffix in ["-wal", "-shm", "-journal"] {
        assert!(!PathBuf::from(format!("{}{suffix}", fixture.database.0.display())).exists());
    }
}

#[test]
fn corrupt_unsupported_integrity_and_migration_failures_preserve_originals_on_retry() {
    for fault in ["header", "future", "integrity", "migration"] {
        let fixture = Fixture::new();
        if fault == "header" {
            fs::write(&fixture.database.0, b"invalid SQLite header").unwrap();
        } else if fault == "migration" {
            fs::remove_file(&fixture.database.0).unwrap();
            let connection = Connection::open(&fixture.database.0).unwrap();
            connection
                .execute_batch(include_str!(
                    "../../../../crates/storage/migrations/0001_initial.sql"
                ))
                .unwrap();
            connection.execute_batch("INSERT INTO app_metadata VALUES ('keep', 'original'); CREATE TABLE vocabulary (unexpected TEXT)").unwrap();
            connection.close().unwrap();
        } else {
            let connection = Connection::open(&fixture.database.0).unwrap();
            let sql = if fault == "future" {
                "PRAGMA user_version=99"
            } else {
                "PRAGMA writable_schema=ON; UPDATE sqlite_schema SET rootpage=99999 WHERE name='vocabulary'; PRAGMA writable_schema=OFF"
            };
            connection.execute_batch(sql).unwrap();
            connection.close().unwrap();
        }
        let original = fs::read(&fixture.database.0).unwrap();
        for _ in 0..2 {
            let status = crate::startup::load(
                &fixture.database,
                |_| Ok(()),
                || panic!("{fault} initialized learning"),
            );
            assert_eq!(
                status.state,
                StartupPhase::RecoveryRequired,
                "{fault}: {status:?}"
            );
            assert!(status.error.is_some(), "{fault}: missing native error");
            assert!(
                fs::read(&fixture.database.0).unwrap() == original,
                "{fault}: failed startup rewrote the source"
            );
        }
    }
}

#[test]
fn unusable_intended_data_directory_never_creates_a_fallback_database() {
    let fixture = Fixture::new();
    let blocked = fixture.root.0.join("blocked-data");
    fs::write(&blocked, b"preserve this path obstruction").unwrap();
    let database = crate::vocabulary::DatabasePath(blocked.join("lexique.sqlite3"));
    let status = crate::startup::load(
        &database,
        |_| Ok(()),
        || panic!("blocked data path admitted learning"),
    );
    assert_eq!(status.state, StartupPhase::RecoveryRequired);
    assert!(status.error.unwrap().contains("blocked-data"));
    assert_eq!(
        fs::read(&blocked).unwrap(),
        b"preserve this path obstruction"
    );
    assert!(!fixture.root.0.join("lexique.sqlite3").exists());
}

#[test]
fn raw_restore_admission_requires_independent_recovery_and_validates_only_key_identity() {
    let failed = crate::startup::StartupState::new(crate::startup::unavailable(
        "synthetic source cannot be opened",
    ));
    assert!(failed.raw_restore(true).unwrap());
    assert!(failed.raw_restore(false).is_err());
    assert!(!failed.requires_restart().unwrap());
    let fixture = Fixture::new();
    let healthy = crate::startup::StartupState::new(crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || Ok(()),
    ));
    assert!(!healthy.raw_restore(false).unwrap());
    assert!(healthy.raw_restore(true).is_err());
    crate::restore::validate_pending_requests(&raw_requests()).unwrap();
    for requests in [
        vec![RawPendingRequest {
            key: "reader-preference".into(),
            value: "keep".into(),
        }],
        vec![raw_requests()[0].clone(), raw_requests()[0].clone()],
    ] {
        assert!(crate::restore::validate_pending_requests(&requests).is_err());
    }
    assert!(
        serde_json::from_str::<RawPendingRequest>(r#"{"key":"lexique-pending-undo","value":null}"#)
            .is_err()
    );
}

struct RecoveryFixture {
    fixture: Fixture,
    archive: PathBuf,
    originals: Vec<(PathBuf, Vec<u8>, PathBuf)>,
}

impl RecoveryFixture {
    fn new() -> Self {
        let fixture = Fixture::new();
        let mut connection = Connection::open(&fixture.database.0).unwrap();
        save(&mut connection, "A from recovery fixture");
        connection.close().unwrap();
        let archive = fixture.root.0.join("recovery.lexique-backup");
        export_paused(&fixture.database, &archive, &AtomicBool::new(false)).unwrap();
        let originals = broken_originals(&fixture);
        Self {
            fixture,
            archive,
            originals,
        }
    }

    fn stage(&self) -> crate::backup_preview::VerifiedBackupStage {
        prepare_preview(
            &self.archive,
            self.fixture.database.0.parent().unwrap(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap()
    }

    fn raw(&self, id: &str) -> PathBuf {
        self.fixture
            .database
            .0
            .parent()
            .unwrap()
            .join("retained-restores")
            .join(id)
            .join("raw-files")
    }

    fn assert_originals(&self) {
        for (path, bytes, _) in &self.originals {
            assert!(
                fs::read(path).unwrap() == *bytes,
                "Original bytes changed at {}",
                path.display()
            );
        }
    }

    fn assert_raw(&self, id: &str) {
        let raw = self.raw(id);
        for (_, bytes, relative) in &self.originals {
            assert!(
                fs::read(raw.join(relative)).unwrap() == *bytes,
                "Preserved bytes changed at {}",
                relative.display()
            );
        }
        let saved: Vec<RawPendingRequest> =
            serde_json::from_slice(&fs::read(raw.join("pending-requests.json")).unwrap()).unwrap();
        assert_eq!(saved, raw_requests());
    }

    fn prepare(&self, id: &str) {
        crate::restore::prepare_raw(
            &self.fixture.database,
            self.stage(),
            id,
            &raw_requests(),
            || Ok(()),
            |_| Ok(()),
        )
        .unwrap();
    }
}

fn assert_recovery_retries(case: &RecoveryFixture) {
    for _ in 0..2 {
        let status = crate::startup::load(
            &case.fixture.database,
            |_| Ok(()),
            || panic!("raw rollback admitted invalid learning state"),
        );
        assert_eq!(status.state, StartupPhase::RecoveryRequired, "{status:?}");
        case.assert_originals();
    }
}

#[test]
fn unusable_retention_directory_stops_raw_restore_before_closing_or_replacing_originals() {
    let case = RecoveryFixture::new();
    let retained = case
        .fixture
        .database
        .0
        .parent()
        .unwrap()
        .join("retained-restores");
    fs::write(&retained, b"preserve the blocked retention path").unwrap();
    let closed = AtomicBool::new(false);
    let error = crate::restore::prepare_raw(
        &case.fixture.database,
        case.stage(),
        "blocked-retention",
        &raw_requests(),
        || {
            closed.store(true, Ordering::Release);
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(
        error.contains("Raw-file preservation") && error.contains("blocked-retention"),
        "{error}"
    );
    assert!(!closed.load(Ordering::Acquire));
    case.assert_originals();
    assert_eq!(
        fs::read(retained).unwrap(),
        b"preserve the blocked retention path"
    );
    assert!(
        !case
            .fixture
            .database
            .0
            .parent()
            .unwrap()
            .join(".lexique-restore.json")
            .exists()
    );
}

#[cfg(windows)]
#[test]
fn real_windows_source_read_and_copy_failures_stop_raw_preservation_before_prepared() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
    for index in 0..7 {
        let case = RecoveryFixture::new();
        let (source, _, _) = &case.originals[index];
        let locked = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(source)
            .unwrap();
        let error = crate::restore::prepare_raw(
            &case.fixture.database,
            case.stage(),
            "locked-source",
            &raw_requests(),
            || Ok(()),
            |_| Ok(()),
        )
        .unwrap_err();
        assert!(
            error.contains("Raw-file preservation")
                && error
                    .replace('\\', "/")
                    .contains(&source.display().to_string().replace('\\', "/")),
            "{error}"
        );
        assert!(
            !case
                .fixture
                .database
                .0
                .parent()
                .unwrap()
                .join(".lexique-restore.json")
                .exists()
        );
        drop(locked);
        case.assert_originals();
        assert_recovery_retries(&case);
    }
}

#[cfg(windows)]
#[test]
fn real_windows_destination_request_publication_and_retained_sync_failures_leave_originals_untouched()
 {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
    for fault in ["destination", "requests", "sync"] {
        let case = RecoveryFixture::new();
        let raw = case.raw("locked-retained");
        let mut locked = None;
        let error = crate::restore::prepare_raw(
            &case.fixture.database,
            case.stage(),
            "locked-retained",
            &raw_requests(),
            || Ok(()),
            |point| {
                let path = match (fault, point) {
                    ("destination", "raw-preservation-created") => {
                        fs::create_dir(raw.join("database")).unwrap();
                        let destination = raw.join("database/lexique.sqlite3");
                        fs::write(&destination, b"locked destination").unwrap();
                        Some(destination)
                    }
                    ("requests", "raw-files-copied") => {
                        let destination = raw.join("pending-requests.json");
                        fs::write(&destination, b"locked native request publication").unwrap();
                        Some(destination)
                    }
                    ("sync", "raw-requests-written") => Some(raw.join("database/lexique.sqlite3")),
                    _ => None,
                };
                if let Some(path) = path {
                    locked = Some(
                        OpenOptions::new()
                            .read(true)
                            .share_mode(0)
                            .open(path)
                            .map_err(|error| error.to_string())?,
                    );
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert!(
            error.contains("Raw-file preservation") && error.contains(&raw.display().to_string()),
            "{fault}: {error}"
        );
        if fault == "sync" {
            assert!(error.contains("Could not sync restore file"), "{error}");
        }
        assert!(
            !case
                .fixture
                .database
                .0
                .parent()
                .unwrap()
                .join(".lexique-restore.json")
                .exists()
        );
        drop(locked);
        case.assert_originals();
        assert_recovery_retries(&case);
        assert!(
            crate::restore::retained_copies(&case.fixture.database)
                .unwrap()
                .iter()
                .any(|copy| copy.kind == PreservationKind::RawFiles)
        );
    }
}

#[cfg(windows)]
#[test]
fn failed_prepared_publication_and_rollback_preserve_invalid_originals_and_block_learning() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
    let case = RecoveryFixture::new();
    let journal = case
        .fixture
        .database
        .0
        .parent()
        .unwrap()
        .join(".lexique-restore.json");
    let mut locked = None;
    let error = crate::restore::prepare_raw(
        &case.fixture.database,
        case.stage(),
        "blocked-prepared",
        &raw_requests(),
        || Ok(()),
        |point| {
            if point == "prepared" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(3)
                        .open(&journal)
                        .map_err(|error| error.to_string())?,
                );
                return Err(
                    "Prepared raw handoff failed while journal replacement was blocked".into(),
                );
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(error.contains("preparation rollback failed"), "{error}");
    case.assert_originals();
    case.assert_raw("blocked-prepared");
    let blocked = crate::startup::load(
        &case.fixture.database,
        |_| Ok(()),
        || panic!("blocked raw decision admitted learning"),
    );
    assert_eq!(blocked.state, StartupPhase::RecoveryRequired, "{blocked:?}");
    drop(locked);
    assert_recovery_retries(&case);
    case.assert_raw("blocked-prepared");
}

#[test]
fn raw_rollback_preserves_an_originally_absent_database_without_initializing_an_empty_one() {
    let mut case = RecoveryFixture::new();
    fs::remove_file(&case.fixture.database.0).unwrap();
    case.originals
        .retain(|(path, _, _)| path != &case.fixture.database.0);
    case.prepare("absent-database");
    for attempt in 0..2 {
        let status = crate::startup::load(
            &case.fixture.database,
            |point| {
                if attempt == 0 && point == "after-install-audio" {
                    return Err("Force rollback before committing an absent original".into());
                }
                Ok(())
            },
            || panic!("absence was replaced by empty learning state"),
        );
        assert_eq!(status.state, StartupPhase::RecoveryRequired, "{status:?}");
        assert!(!case.fixture.database.0.exists());
        case.assert_originals();
        case.assert_raw("absent-database");
    }
}

#[test]
fn raw_subprocess_interruptions_recover_only_complete_original_or_verified_timelines() {
    for checkpoint in [
        "raw-preservation-created",
        "raw-files-copied",
        "raw-requests-written",
        "prepared",
        "swapping",
        "after-isolate-database",
        "after-isolate-wal",
        "after-install-database",
        "new-state-synced",
        "committed",
    ] {
        let case = RecoveryFixture::new();
        let prepare = checkpoint.starts_with("raw-") || checkpoint == "prepared";
        if !prepare {
            case.prepare("raw-interruption");
        }
        interrupt_child_at(
            &case.fixture,
            checkpoint,
            if prepare {
                "raw-prepare"
            } else {
                "raw-startup"
            },
            "recovery_tests::recovery_interruption_child",
        );
        let status = crate::startup::load(
            &case.fixture.database,
            |_| Ok(()),
            || panic!("interruption initialized learning before recovery decision"),
        );
        if checkpoint == "prepared" || checkpoint == "committed" {
            assert_eq!(
                status.state,
                StartupPhase::CleanupRequired,
                "{checkpoint}: {status:?}"
            );
            crate::restore::cleanup_begin(&case.fixture.database, "raw-interruption", |_| Ok(()))
                .unwrap();
            crate::restore::cleanup_complete(
                &case.fixture.database,
                "raw-interruption",
                |_| Ok(()),
            )
            .unwrap();
            let ready = crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(()));
            assert_eq!(ready.state, StartupPhase::Ready, "{checkpoint}: {ready:?}");
            assert_eq!(
                storage::get(&Connection::open(&case.fixture.database.0).unwrap(), 1)
                    .unwrap()
                    .notes,
                "A from recovery fixture"
            );
            for suffix in ["-wal", "-shm", "-journal"] {
                assert!(
                    !PathBuf::from(format!("{}{suffix}", case.fixture.database.0.display()))
                        .exists()
                );
            }
        } else {
            assert_eq!(
                status.state,
                StartupPhase::RecoveryRequired,
                "{checkpoint}: {status:?}"
            );
            assert_recovery_retries(&case);
        }
        if !checkpoint.starts_with("raw-") {
            case.assert_raw("raw-interruption");
        }
    }
}

#[test]
fn raw_subprocess_rollback_interruptions_keep_invalid_originals_and_raw_requests() {
    for checkpoint in [
        "before-rollback-install-database",
        "after-rollback-install-database",
        "rollback-complete",
    ] {
        let case = RecoveryFixture::new();
        case.prepare("raw-interruption");
        interrupt_child_at(
            &case.fixture,
            checkpoint,
            "raw-rollback",
            "recovery_tests::recovery_interruption_child",
        );
        assert_recovery_retries(&case);
        case.assert_raw("raw-interruption");
    }
}

#[test]
fn raw_preserved_requests_survive_interrupted_acknowledged_cleanup() {
    for checkpoint in ["keys-cleared", "after-cleanup-stage"] {
        let case = RecoveryFixture::new();
        case.prepare("raw-interruption");
        interrupt_child_at(
            &case.fixture,
            checkpoint,
            "raw-cleanup",
            "recovery_tests::recovery_interruption_child",
        );
        let status = crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(()));
        assert_eq!(
            status.state,
            StartupPhase::Ready,
            "{checkpoint}: {status:?}"
        );
        case.assert_raw("raw-interruption");
    }
}
