//! Restore assertions use existing domain reads and real isolated files.
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use lexique_storage as storage;
use rusqlite::Connection;

use crate::{
    backup::{OwnedDirectory, export_backup_to_new_file},
    backup_preview::prepare_preview,
    maintenance::MaintenanceCoordinator,
    speech_tasks::SpeechTasks,
    vocabulary::DatabasePath,
};

struct Fixture {
    root: OwnedDirectory,
    database: DatabasePath,
}

struct TestWorkers {
    tasks: SpeechTasks,
    stopped: AtomicBool,
}

impl TestWorkers {
    fn new(tasks: SpeechTasks) -> Self {
        Self {
            tasks,
            stopped: AtomicBool::new(false),
        }
    }

    fn shutdown(&self) -> Result<(), String> {
        if self.stopped.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }
        self.tasks.shutdown()?;
        self.stopped
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }
}

impl std::ops::Deref for TestWorkers {
    type Target = SpeechTasks;
    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

impl Drop for TestWorkers {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            eprintln!("Could not shut down restore test workers: {error}");
        }
    }
}

#[test]
fn formal_preparation_waits_for_accepted_readers_and_closes_new_reader_admission() {
    let fixture = Fixture::new();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("reader.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "reader-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let reader = maintenance.admit_reader().unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    let (closing_send, closing_receive) = std::sync::mpsc::channel();
    let (closed_send, closed_receive) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let database = &fixture.database;
        let workers = &workers;
        let maintenance_ref = &maintenance;
        let preparation = scope.spawn(move || {
            crate::restore::prepare_normal(
                database,
                workers,
                maintenance_ref,
                stage,
                "restore-reader",
                || {
                    workers.shutdown()?;
                    closed_send.send(()).unwrap();
                    Ok(())
                },
                |phase| {
                    if phase == "closing" {
                        closing_send.send(()).unwrap();
                    }
                    Ok(())
                },
            )
        });
        closing_receive
            .recv_timeout(Duration::from_secs(3))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if maintenance.admit_reader().is_err() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "formal closure never rejected new readers"
            );
            std::thread::yield_now();
        }
        assert!(
            closed_receive
                .recv_timeout(Duration::from_millis(30))
                .is_err()
        );
        assert!(
            !fixture
                .database
                .0
                .parent()
                .unwrap()
                .join(".lexique-restore.json")
                .exists()
        );
        connection.close().unwrap();
        drop(reader);
        preparation.join().unwrap().unwrap();
        closed_receive.recv_timeout(Duration::from_secs(1)).unwrap();
    });
    assert!(maintenance.admit().is_err());
    assert!(maintenance.admit_reader().is_err());
}

impl Fixture {
    fn new() -> Self {
        let root = OwnedDirectory::create(&std::env::temp_dir()).unwrap();
        let directory = root.0.join("Lexique-data/data");
        fs::create_dir_all(&directory).unwrap();
        let database = DatabasePath(directory.join("lexique.sqlite3"));
        storage::initialize(&Connection::open(&database.0).unwrap()).unwrap();
        Self { root, database }
    }

    fn workers(&self) -> TestWorkers {
        TestWorkers::new(SpeechTasks::start_with_notifications(
            self.database.0.clone(),
            self.root.0.join("no-model"),
            Arc::new(|| Ok(())),
        ))
    }
}

fn save(connection: &mut Connection, notes: &str) -> storage::SaveResult {
    storage::save(
        connection,
        &storage::SaveRequest {
            operation: "save-A".into(),
            headword: "école".into(),
            notes: notes.into(),
            example_fr: "Une école.".into(),
            example_zh: "一所学校".into(),
            example_source: None,
            example_sense: Some(0),
            more_examples: vec![storage::Example {
                fr: "Je vais à l'école.".into(),
                zh: "我去学校。".into(),
                source: None,
            }],
            id: None,
            revision: None,
        },
        Some(storage::Snapshot {
            reference: storage::Reference {
                dictionary_id: Some("mdx-6".into()),
                dictionary: "Retained dictionary".into(),
                lookup_key: "école".into(),
                version: "A".into(),
                attribution: Some("MIT fixture".into()),
                license: Some("MIT".into()),
                source_url: None,
                license_url: None,
                sha256: "a".repeat(64),
            },
            digest: vec![storage::DigestEntry {
                pronunciations: vec!["ekɔl".into()],
                grammar: vec!["n.f.".into()],
                translation: Some("学校".into()),
            }],
        }),
    )
    .unwrap()
}

#[test]
fn preparation_preserves_b_and_ordinary_startup_installs_verified_a() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A notes — accents é").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "export-A",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B notes',revision=revision+1 WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();

    let prepared = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-A",
        || workers.shutdown(),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B notes"
    );
    let outcome = crate::restore::recover_startup(&fixture.database, |_| Ok(()));
    assert_eq!(
        outcome.state,
        crate::restore::StartupPhase::CleanupRequired,
        "{outcome:?}"
    );
    let restored =
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(&original).unwrap()
    );

    let before = prepared
        .retained_copies
        .iter()
        .find(|copy| copy.kind == crate::restore::PreservationKind::LogicalBackup)
        .unwrap();
    assert!(before.byte_length > 0);
    let preserved = prepare_preview(
        &PathBuf::from(&before.path),
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    assert_eq!(
        storage::get(
            &Connection::open(preserved.database()).unwrap(),
            original.id
        )
        .unwrap()
        .notes,
        "B notes"
    );
}

#[test]
fn uncommitted_install_failure_restores_complete_b_and_keeps_the_preserved_copies() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A before rollback").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("rollback-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "rollback-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B before rollback' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();
    crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-rollback",
        || workers.shutdown(),
        |_| Ok(()),
    )
    .unwrap();
    let failed = crate::restore::recover_startup(&fixture.database, |point| {
        if point == "after-install-database" {
            Err("Installation interrupted".into())
        } else {
            Ok(())
        }
    });
    assert_eq!(
        failed.state,
        crate::restore::StartupPhase::Ready,
        "{failed:?}"
    );
    assert!(failed.restored.is_none());
    assert!(
        failed
            .error
            .as_deref()
            .unwrap()
            .contains("previous learning state")
    );
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before rollback"
    );
    assert!(
        failed
            .retained_copies
            .iter()
            .any(|copy| copy.kind == crate::restore::PreservationKind::LogicalBackup)
    );
    assert!(failed.retained_copies.iter().any(|copy| copy.kind
        == crate::restore::PreservationKind::OriginalDatabase
        && copy.byte_length > 0));
    let repeated = crate::restore::recover_startup(&fixture.database, |_| Ok(()));
    assert_eq!(
        repeated.state,
        crate::restore::StartupPhase::Ready,
        "{repeated:?}"
    );
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before rollback"
    );
}

#[test]
fn committed_restore_repeats_cleanup_without_replaying_the_import() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A with pending cleanup").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("cleanup-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "cleanup-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-cleanup",
        || workers.shutdown(),
        |_| Ok(()),
    )
    .unwrap();
    let committed = crate::restore::recover_startup(&fixture.database, |_| Ok(()));
    assert_eq!(
        committed.state,
        crate::restore::StartupPhase::CleanupRequired,
        "{committed:?}"
    );
    assert!(
        crate::restore::cleanup_complete(&fixture.database, "restore-cleanup", |_| Ok(())).is_err()
    );
    assert!(
        crate::restore::cleanup_begin(&fixture.database, "another-operation", |_| Ok(())).is_err()
    );
    crate::restore::cleanup_begin(&fixture.database, "restore-cleanup", |_| Ok(())).unwrap();
    assert!(
        crate::restore::cleanup_complete(&fixture.database, "restore-cleanup", |point| {
            if point == "keys-cleared" {
                Err("Cleanup acknowledgement interrupted".into())
            } else {
                Ok(())
            }
        })
        .is_err()
    );
    let resumed = crate::restore::recover_startup(&fixture.database, |_| Ok(()));
    assert_eq!(
        resumed.state,
        crate::restore::StartupPhase::Ready,
        "{resumed:?}"
    );
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "A with pending cleanup"
    );
    let repeated =
        crate::restore::cleanup_complete(&fixture.database, "restore-cleanup", |_| Ok(())).unwrap();
    assert_eq!(repeated.state, crate::restore::StartupPhase::Ready);
    assert_eq!(
        crate::restore::recover_startup(&fixture.database, |_| Ok(())).state,
        crate::restore::StartupPhase::Ready
    );
}

#[test]
fn ordinary_startup_blocks_initialization_and_workers_when_the_restore_record_is_unreadable() {
    let fixture = Fixture::new();
    fs::remove_file(&fixture.database.0).unwrap();
    fs::write(
        fixture
            .database
            .0
            .parent()
            .unwrap()
            .join(".lexique-restore.json"),
        b"{interrupted",
    )
    .unwrap();
    let initialized = AtomicBool::new(false);
    let status = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            storage::initialize(&Connection::open(&fixture.database.0).unwrap()).unwrap();
            let workers = fixture.workers();
            workers.shutdown()
        },
    );
    assert_eq!(status.state, crate::restore::StartupPhase::RecoveryRequired);
    assert!(status.error.as_deref().unwrap().contains("restore record"));
    assert!(!initialized.load(std::sync::atomic::Ordering::SeqCst));
    assert!(
        !fixture.database.0.exists(),
        "recovery initialized an empty learning database"
    );
}

#[cfg(windows)]
#[test]
fn sync_failure_after_formal_closure_keeps_the_old_runtime_closed() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A before failed closure").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("closure-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "closure-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let staged_database = stage.database().to_owned();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B before failed closure' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();

    let mut locked = None;
    let failed = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-failed-closure",
        || workers.shutdown(),
        |point| {
            if point == "closing" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(0)
                        .open(&staged_database)
                        .map_err(|error| error.to_string())?,
                );
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(failed.contains("Could not sync restore file"), "{failed}");
    assert!(
        maintenance.admit().is_err(),
        "closed restore runtime reopened writer admission"
    );
    assert!(
        maintenance.admit_reader().is_err(),
        "closed restore runtime reopened reader admission"
    );
    drop(locked);
    let initialized = AtomicBool::new(false);
    let restarted = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        },
    );
    assert_eq!(restarted.state, crate::restore::StartupPhase::Ready);
    assert!(initialized.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before failed closure"
    );
}

#[test]
fn failed_prepared_handoff_keeps_b_on_ordinary_startup() {
    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A before failed prepared handoff").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("handoff-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "handoff-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B before failed prepared handoff' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();
    let error = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-failed-handoff",
        || workers.shutdown(),
        |point| {
            if point == "prepared" {
                Err("Prepared publication acknowledgement failed".into())
            } else {
                Ok(())
            }
        },
    )
    .unwrap_err();
    assert!(
        error.contains("Prepared publication acknowledgement failed"),
        "{error}"
    );
    let initialized = AtomicBool::new(false);
    let restarted = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || {
            initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        },
    );
    assert_eq!(
        restarted.state,
        crate::restore::StartupPhase::Ready,
        "{restarted:?}"
    );
    assert!(
        restarted.restored.is_none(),
        "a failed prepared handoff installed the new timeline"
    );
    assert!(initialized.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before failed prepared handoff"
    );
    assert!(
        restarted
            .retained_copies
            .iter()
            .any(|copy| copy.kind == crate::restore::PreservationKind::LogicalBackup)
    );
}

#[cfg(windows)]
#[test]
fn failed_preparation_with_a_locked_journal_keeps_b_after_the_lock_is_released() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A before locked preparation").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("locked-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "locked-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B before locked preparation' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();
    let journal = fixture
        .database
        .0
        .parent()
        .unwrap()
        .join(".lexique-restore.json");
    let mut locked = None;
    let failed = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-locked-preparation",
        || workers.shutdown(),
        |point| {
            if point == "prepared" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(3)
                        .open(&journal)
                        .map_err(|error| error.to_string())?,
                );
                return Err("Prepared handoff failed with journal replacement blocked".into());
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(failed.contains("preparation rollback failed"), "{failed}");
    assert!(maintenance.admit().is_err());
    assert!(maintenance.admit_reader().is_err());
    let blocked = crate::startup::load(
        &fixture.database,
        |_| Ok(()),
        || panic!("journal replacement failure admitted learning initialization"),
    );
    assert_eq!(
        blocked.state,
        crate::restore::StartupPhase::RecoveryRequired,
        "{blocked:?}"
    );
    drop(locked);
    let restarted = crate::startup::load(&fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(
        restarted.state,
        crate::restore::StartupPhase::Ready,
        "{restarted:?}"
    );
    assert!(
        restarted.restored.is_none(),
        "failed preparation later installed A"
    );
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before locked preparation"
    );
    let repeated = crate::startup::load(&fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(repeated.state, crate::restore::StartupPhase::Ready);
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before locked preparation"
    );
}

struct CheckpointChild(std::process::Child);

impl Drop for CheckpointChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none()
            && let Err(error) = self.0.kill()
        {
            eprintln!("Could not stop restore checkpoint child: {error}");
        }
        if let Err(error) = self.0.wait() {
            eprintln!("Could not reap restore checkpoint child: {error}");
        }
    }
}

fn interrupt_at(fixture: &Fixture, checkpoint: &str, action: &str) {
    let marker = fixture.root.0.join("interruption-ready");
    let log_path = fixture.root.0.join("interruption-child.log");
    if marker.exists() {
        fs::remove_file(&marker).unwrap();
    }
    let output = fs::File::create(&log_path).unwrap();
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--ignored",
            "--exact",
            "restore_tests::restore_interruption_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("LEXIQUE_RESTORE_TEST_DATABASE", &fixture.database.0)
        .env("LEXIQUE_RESTORE_TEST_ROOT", &fixture.root.0)
        .env("LEXIQUE_RESTORE_TEST_CHECKPOINT", checkpoint)
        .env("LEXIQUE_RESTORE_TEST_ACTION", action)
        .stdout(std::process::Stdio::from(output.try_clone().unwrap()))
        .stderr(std::process::Stdio::from(output));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let mut child = CheckpointChild(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read(&marker).ok().as_deref() != Some(checkpoint.as_bytes()) {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "checkpoint {checkpoint} exited early: {}",
            fs::read_to_string(&log_path).unwrap()
        );
        assert!(
            Instant::now() < deadline,
            "checkpoint {checkpoint} timed out: {}",
            fs::read_to_string(&log_path).unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    child.0.kill().unwrap();
    assert!(
        !child.0.wait().unwrap().success(),
        "checkpoint child exited normally"
    );
    println!("Forced owned subprocess exit at {checkpoint}");
}

#[test]
#[ignore = "owned child entry point; invoked only by the interruption parent"]
fn restore_interruption_child() {
    use std::io::Write;

    let database = DatabasePath(PathBuf::from(
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
        if action == "rollback" && point == "after-install-audio" {
            return Err("Owned subprocess installation failure before commitment".into());
        }
        Ok(())
    };
    if action == "prepare" {
        let stage = prepare_preview(
            &root.join("interruption-A.lexique-backup"),
            database.0.parent().unwrap(),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        let workers = TestWorkers::new(SpeechTasks::start_with_notifications(
            database.0.clone(),
            root.join("no-model"),
            Arc::new(|| Ok(())),
        ));
        let result = crate::restore::prepare_normal(
            &database,
            &workers,
            &MaintenanceCoordinator::default(),
            stage,
            "restore-interruption",
            || workers.shutdown(),
            observe,
        );
        panic!("prepare child did not reach {checkpoint}: {result:?}");
    }
    if action == "cleanup" {
        let status = crate::startup::load(
            &database,
            |_| Ok(()),
            || panic!("committed child initialized learning before pending cleanup"),
        );
        assert_eq!(status.state, crate::restore::StartupPhase::CleanupRequired);
        crate::restore::cleanup_begin(&database, "restore-interruption", observe).unwrap();
        let status =
            crate::restore::cleanup_complete(&database, "restore-interruption", observe).unwrap();
        panic!("cleanup child did not reach {checkpoint}: {status:?}");
    }
    assert!(action == "startup" || action == "rollback");
    let status = crate::startup::load(&database, observe, || Ok(()));
    panic!("restore child did not reach {checkpoint}: {status:?}");
}

const AUDIO_A: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/female-1.ogg");
const AUDIO_B: &[u8] = include_bytes!("../../../../docs/evidence/issue-56/female-2.ogg");

struct InterruptionFixture {
    fixture: Fixture,
    id: i64,
    raw_b: Vec<(PathBuf, Vec<u8>)>,
}

fn select_fixture_audio(
    connection: &Connection,
    database: &DatabasePath,
    id: i64,
    bytes: &[u8],
    time: i64,
) {
    let key = storage::attach_word_audio_at(connection, id, time).unwrap();
    let generation = storage::begin_audio_change(connection, id, &key).unwrap();
    let version = storage::prepare_source_audio(
        connection,
        &key,
        &storage::content_hash(bytes),
        bytes.len() as i64,
        time,
    )
    .unwrap();
    crate::speech_cache::Inspector::new(&database.0)
        .unwrap()
        .publish_version(&version.physical_locator, &version.content_hash, bytes)
        .unwrap();
    storage::verify_audio_content(connection, &version.locator, bytes.len() as i64).unwrap();
    assert!(storage::complete_source_audio(connection, &version.locator).unwrap());
    assert!(
        storage::reuse_audio_version(connection, id, &version.locator, generation, time).unwrap()
    );
}

fn sqlite_images(database: &DatabasePath) -> Vec<(PathBuf, Vec<u8>)> {
    ["", "-wal", "-shm", "-journal"]
        .into_iter()
        .filter_map(|suffix| {
            let mut path = database.0.as_os_str().to_owned();
            path.push(suffix);
            let path = PathBuf::from(path);
            if path.exists() {
                Some((path.clone(), fs::read(path).unwrap()))
            } else {
                None
            }
        })
        .collect()
}

fn interruption_fixture(prepared: bool) -> InterruptionFixture {
    interruption_fixture_with_journal(prepared, "WAL")
}

fn interruption_fixture_with_journal(prepared: bool, journal_mode: &str) -> InterruptionFixture {
    let fixture = Fixture::new();
    let workers = fixture.workers();
    let cancelled = AtomicBool::new(false);
    let pause = workers.pause(&cancelled).unwrap();
    let time = crate::speech_tasks::now().unwrap();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A subprocess timeline").record;
    select_fixture_audio(
        &connection,
        &fixture.database,
        original.id,
        AUDIO_A,
        time - 5,
    );
    let maintenance = MaintenanceCoordinator::default();
    let archive = fixture.root.0.join("interruption-A.lexique-backup");
    crate::backup::export_paused(&fixture.database, &archive, &cancelled).unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    connection
        .pragma_update(None, "journal_mode", journal_mode)
        .unwrap();
    if journal_mode == "WAL" {
        connection
            .pragma_update(None, "wal_autocheckpoint", 0)
            .unwrap();
    }
    connection
        .execute(
            "UPDATE vocabulary SET notes='B subprocess timeline' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    select_fixture_audio(
        &connection,
        &fixture.database,
        original.id,
        AUDIO_B,
        time - 4,
    );
    assert_eq!(
        storage::get(&connection, original.id).unwrap().notes,
        "B subprocess timeline"
    );
    drop(pause);
    let mut raw_b = Vec::new();
    let close = || {
        workers.shutdown()?;
        // Retain a real live SQLite image as an interrupted process leaves it.
        raw_b = sqlite_images(&fixture.database);
        connection.close().map_err(|(_, error)| error.to_string())?;
        for (path, bytes) in &raw_b {
            fs::write(path, bytes).map_err(|error| error.to_string())?;
        }
        Ok(())
    };
    if prepared {
        let staged_database = stage.database().to_owned();
        crate::restore::prepare_normal(
            &fixture.database,
            &workers,
            &maintenance,
            stage,
            "restore-interruption",
            close,
            |point| {
                if journal_mode == "PERSIST" && point == "closing" {
                    let staged =
                        Connection::open(&staged_database).map_err(|error| error.to_string())?;
                    staged
                        .pragma_update(None, "journal_mode", "PERSIST")
                        .map_err(|error| error.to_string())?;
                    let original_notes: String = staged
                        .query_row("SELECT notes FROM vocabulary LIMIT 1", [], |row| row.get(0))
                        .map_err(|error| error.to_string())?;
                    staged
                        .execute(
                            "UPDATE vocabulary SET notes='Persistent journal fixture'",
                            [],
                        )
                        .map_err(|error| error.to_string())?;
                    staged
                        .execute("UPDATE vocabulary SET notes=?1", [original_notes])
                        .map_err(|error| error.to_string())?;
                    staged.close().map_err(|(_, error)| error.to_string())?;
                }
                Ok(())
            },
        )
        .unwrap();
    } else {
        close().unwrap();
        drop(stage);
    }
    if journal_mode == "WAL" {
        assert!(
            raw_b
                .iter()
                .any(|(path, bytes)| path.to_string_lossy().ends_with("-wal") && bytes.len() > 32)
        );
        assert!(
            raw_b
                .iter()
                .any(|(path, bytes)| path.to_string_lossy().ends_with("-shm") && !bytes.is_empty())
        );
    } else {
        assert!(
            raw_b
                .iter()
                .any(|(path, bytes)| path.to_string_lossy().ends_with("-journal")
                    && bytes.len() > 28)
        );
    }
    InterruptionFixture {
        fixture,
        id: original.id,
        raw_b,
    }
}

fn assert_timeline(case: &InterruptionFixture, notes: &str, audio: &[u8]) {
    let connection = Connection::open(&case.fixture.database.0).unwrap();
    assert_eq!(storage::get(&connection, case.id).unwrap().notes, notes);
    let version = storage::selected_audio(&connection, case.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        crate::speech_cache::Inspector::new(&case.fixture.database.0)
            .unwrap()
            .read_asset(&version.physical_locator, &version.content_hash)
            .unwrap()
            .unwrap(),
        audio
    );
    connection.close().unwrap();
}

fn assert_recovered_b(case: &InterruptionFixture) {
    let restarted = crate::startup::load(
        &case.fixture.database,
        |_| Ok(()),
        || {
            for (path, bytes) in &case.raw_b {
                assert!(
                    path.exists(),
                    "recovery lost an original SQLite component: {}",
                    path.display()
                );
                if !path.to_string_lossy().ends_with("-shm") {
                    assert_eq!(
                        fs::read(path).unwrap(),
                        *bytes,
                        "recovery changed the original SQLite image: {}",
                        path.display()
                    );
                }
            }
            for suffix in ["-wal", "-shm", "-journal"] {
                let path = case
                    .fixture
                    .database
                    .0
                    .with_file_name(format!("lexique.sqlite3{suffix}"));
                assert_eq!(
                    path.exists(),
                    case.raw_b.iter().any(|(original, _)| *original == path)
                );
            }
            Ok(())
        },
    );
    assert_eq!(
        restarted.state,
        crate::restore::StartupPhase::Ready,
        "{restarted:?}"
    );
    assert!(restarted.restored.is_none());
    let originals = case
        .fixture
        .database
        .0
        .parent()
        .unwrap()
        .join("retained-restores/restore-interruption/original-database");
    for (path, bytes) in &case.raw_b {
        let retained = originals.join(path.file_name().unwrap());
        if retained.exists() {
            assert_eq!(fs::read(retained).unwrap(), *bytes);
        }
    }
    assert_timeline(case, "B subprocess timeline", AUDIO_B);
    let repeated = crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(repeated.state, crate::restore::StartupPhase::Ready);
    assert_timeline(case, "B subprocess timeline", AUDIO_B);
}

#[test]
fn forced_exit_after_database_isolation_recovers_complete_b_on_ordinary_startup() {
    let case = interruption_fixture(true);
    interrupt_at(&case.fixture, "after-isolate-database", "startup");
    assert!(!case.fixture.database.0.exists());
    assert_recovered_b(&case);
}

#[test]
fn forced_exits_at_uncommitted_database_sidecar_and_audio_boundaries_restore_b() {
    for checkpoint in [
        "swapping",
        "before-isolate-database",
        "before-isolate-wal",
        "after-isolate-wal",
        "before-isolate-shm",
        "after-isolate-shm",
        "before-isolate-audio",
        "after-isolate-audio",
        "before-install-database",
        "after-install-database",
        "before-install-audio",
        "after-install-audio",
        "new-state-synced",
    ] {
        let case = interruption_fixture(true);
        interrupt_at(&case.fixture, checkpoint, "startup");
        assert_recovered_b(&case);
    }
}

#[test]
fn forced_exits_during_rollback_repeat_without_losing_b_or_the_original_copies() {
    for checkpoint in [
        "rollback",
        "before-rollback-remove-audio",
        "after-rollback-remove-audio",
        "before-rollback-remove-database",
        "after-rollback-remove-database",
        "before-rollback-install-database",
        "after-rollback-install-database",
        "before-rollback-install-wal",
        "after-rollback-install-wal",
        "before-rollback-install-shm",
        "after-rollback-install-shm",
        "before-rollback-install-audio",
        "after-rollback-install-audio",
        "rollback-complete",
    ] {
        let case = interruption_fixture(true);
        interrupt_at(&case.fixture, checkpoint, "rollback");
        assert_recovered_b(&case);
    }
}

#[test]
fn forced_exit_at_prepared_keeps_the_confirmed_stage_for_ordinary_startup() {
    let case = interruption_fixture(false);
    interrupt_at(&case.fixture, "prepared", "prepare");
    let status = crate::startup::load(
        &case.fixture.database,
        |_| Ok(()),
        || panic!("prepared handoff enabled learning before pending-key cleanup"),
    );
    assert_eq!(
        status.state,
        crate::restore::StartupPhase::CleanupRequired,
        "{status:?}"
    );
    assert_timeline(&case, "A subprocess timeline", AUDIO_A);
    crate::restore::cleanup_begin(&case.fixture.database, "restore-interruption", |_| Ok(()))
        .unwrap();
    crate::restore::cleanup_complete(&case.fixture.database, "restore-interruption", |_| Ok(()))
        .unwrap();
    assert_eq!(
        crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(())).state,
        crate::restore::StartupPhase::Ready
    );
    assert_timeline(&case, "A subprocess timeline", AUDIO_A);
}

#[test]
fn forced_exits_after_commit_and_during_cleanup_keep_a_and_gate_learning() {
    for checkpoint in [
        "committed",
        "cleanup-started",
        "keys-cleared",
        "before-cleanup-stage",
        "after-cleanup-stage",
        "cleanup-complete",
    ] {
        let case = interruption_fixture(true);
        let action = if checkpoint == "committed" {
            "startup"
        } else {
            "cleanup"
        };
        interrupt_at(&case.fixture, checkpoint, action);
        let initialized = AtomicBool::new(false);
        let status = crate::startup::load(
            &case.fixture.database,
            |_| Ok(()),
            || {
                initialized.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            },
        );
        let needs_cleanup = checkpoint == "committed" || checkpoint == "cleanup-started";
        assert_eq!(
            status.state,
            if needs_cleanup {
                crate::restore::StartupPhase::CleanupRequired
            } else {
                crate::restore::StartupPhase::Ready
            },
            "{checkpoint}: {status:?}"
        );
        assert_eq!(
            initialized.load(std::sync::atomic::Ordering::SeqCst),
            !needs_cleanup
        );
        assert_timeline(&case, "A subprocess timeline", AUDIO_A);
        if needs_cleanup {
            crate::restore::cleanup_begin(&case.fixture.database, "restore-interruption", |_| {
                Ok(())
            })
            .unwrap();
            crate::restore::cleanup_complete(
                &case.fixture.database,
                "restore-interruption",
                |_| Ok(()),
            )
            .unwrap();
        }
        let repeated = crate::restore::cleanup_complete(
            &case.fixture.database,
            "restore-interruption",
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(repeated.state, crate::restore::StartupPhase::Ready);
        assert!(repeated.restored.is_some());
        assert_timeline(&case, "A subprocess timeline", AUDIO_A);
    }
}

#[test]
fn forced_exits_at_persistent_sqlite_journal_moves_and_rollback_restore_b() {
    for checkpoint in [
        "before-isolate-journal",
        "after-isolate-journal",
        "before-install-journal",
        "after-install-journal",
        "before-rollback-remove-journal",
        "after-rollback-remove-journal",
        "before-rollback-install-journal",
        "after-rollback-install-journal",
    ] {
        let case = interruption_fixture_with_journal(true, "PERSIST");
        let action = if checkpoint.contains("rollback") {
            "rollback"
        } else {
            "startup"
        };
        interrupt_at(&case.fixture, checkpoint, action);
        assert_recovered_b(&case);
    }
}

fn mark_original_absent(case: &InterruptionFixture, kinds: &[&str]) {
    let journal_path = case
        .fixture
        .database
        .0
        .parent()
        .unwrap()
        .join(".lexique-restore.json");
    let mut journal: serde_json::Value =
        serde_json::from_slice(&fs::read(&journal_path).unwrap()).unwrap();
    for component in journal["components"].as_array_mut().unwrap() {
        let kind = component["kind"].as_str().unwrap().to_owned();
        if !kinds.contains(&kind.as_str()) {
            continue;
        }
        component["original_present"] = serde_json::Value::Bool(false);
        let path = if kind == "audio" {
            crate::speech_cache::managed_audio_directory(&case.fixture.database.0).unwrap()
        } else {
            let suffix = match kind.as_str() {
                "database" => "",
                "wal" => "-wal",
                "shm" => "-shm",
                _ => "-journal",
            };
            case.fixture
                .database
                .0
                .with_file_name(format!("lexique.sqlite3{suffix}"))
        };
        assert!(path.starts_with(&case.fixture.root.0));
        if path.is_dir() {
            fs::remove_dir_all(path).unwrap();
        } else if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }
    fs::write(journal_path, serde_json::to_vec(&journal).unwrap()).unwrap();
}

#[test]
fn uncommitted_recovery_preserves_original_audio_absence() {
    let case = interruption_fixture(true);
    mark_original_absent(&case, &["audio"]);
    interrupt_at(&case.fixture, "after-install-audio", "startup");
    for _ in 0..2 {
        let status = crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(()));
        assert_eq!(
            status.state,
            crate::restore::StartupPhase::Ready,
            "{status:?}"
        );
        assert!(status.restored.is_none());
        assert!(
            !crate::speech_cache::managed_audio_directory(&case.fixture.database.0)
                .unwrap()
                .exists()
        );
        let connection = Connection::open_with_flags(
            &case.fixture.database.0,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        assert_eq!(
            storage::get(&connection, case.id).unwrap().notes,
            "B subprocess timeline"
        );
        connection.close().unwrap();
    }
}

#[test]
fn uncommitted_recovery_preserves_original_database_absence_without_initializing_an_empty_database()
{
    let case = interruption_fixture(true);
    mark_original_absent(&case, &["database", "wal", "shm", "journal"]);
    interrupt_at(&case.fixture, "after-install-database", "startup");
    for _ in 0..2 {
        let status = crate::startup::load(
            &case.fixture.database,
            |_| Ok(()),
            || panic!("originally absent database recovery created an empty learning timeline"),
        );
        assert_eq!(
            status.state,
            crate::restore::StartupPhase::RecoveryRequired,
            "{status:?}"
        );
        assert!(
            status
                .error
                .as_deref()
                .unwrap()
                .contains("original absence")
        );
        assert!(!case.fixture.database.0.exists());
        assert!(
            status
                .retained_copies
                .iter()
                .any(|copy| copy.kind == crate::restore::PreservationKind::LogicalBackup)
        );
        for suffix in ["-wal", "-shm", "-journal"] {
            assert!(
                !case
                    .fixture
                    .database
                    .0
                    .with_file_name(format!("lexique.sqlite3{suffix}"))
                    .exists()
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn genuine_preservation_read_failure_keeps_b_and_releases_normal_learning_admission() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let fixture = Fixture::new();
    let mut connection = Connection::open(&fixture.database.0).unwrap();
    let original = save(&mut connection, "A before preservation failure").record;
    connection.close().unwrap();
    let workers = fixture.workers();
    let maintenance = MaintenanceCoordinator::default();
    let cancelled = AtomicBool::new(false);
    let archive = fixture.root.0.join("preservation-A.lexique-backup");
    export_backup_to_new_file(
        &fixture.database,
        &workers,
        &maintenance,
        &archive,
        "preservation-export",
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage = prepare_preview(
        &archive,
        fixture.database.0.parent().unwrap(),
        &cancelled,
        |_| {},
    )
    .unwrap();
    let stage_root = stage.root().to_owned();
    let connection = Connection::open(&fixture.database.0).unwrap();
    connection
        .execute(
            "UPDATE vocabulary SET notes='B before preservation failure' WHERE id=?1",
            [original.id],
        )
        .unwrap();
    connection.close().unwrap();
    let mut locked = None;
    let error = crate::restore::prepare_normal(
        &fixture.database,
        &workers,
        &maintenance,
        stage,
        "restore-preservation-failure",
        || workers.shutdown(),
        |point| {
            if point == "preserving" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(0)
                        .open(&fixture.database.0)
                        .map_err(|error| error.to_string())?,
                );
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert!(!error.is_empty());
    drop(locked);
    assert!(!stage_root.exists());
    assert!(
        !fixture
            .database
            .0
            .parent()
            .unwrap()
            .join(".lexique-restore.json")
            .exists()
    );
    assert!(maintenance.admit().is_ok() && maintenance.admit_reader().is_ok());
    let status = crate::startup::load(&fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(status.state, crate::restore::StartupPhase::Ready);
    assert!(status.restored.is_none());
    assert_eq!(
        storage::get(&Connection::open(&fixture.database.0).unwrap(), original.id)
            .unwrap()
            .notes,
        "B before preservation failure"
    );
}

#[cfg(windows)]
#[test]
fn genuine_new_state_sync_failure_rolls_back_complete_b_before_commit() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let case = interruption_fixture(true);
    let mut locked = None;
    let status = crate::startup::load(
        &case.fixture.database,
        |point| {
            if point == "after-install-audio" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(5)
                        .open(&case.fixture.database.0)
                        .map_err(|error| error.to_string())?,
                );
            }
            Ok(())
        },
        || Ok(()),
    );
    assert_eq!(
        status.state,
        crate::restore::StartupPhase::Ready,
        "{status:?}"
    );
    assert!(status.restored.is_none());
    assert!(
        status
            .error
            .as_deref()
            .unwrap()
            .contains("Could not sync restore file"),
        "{status:?}"
    );
    drop(locked);
    assert_recovered_b(&case);
}

#[cfg(windows)]
#[test]
fn genuine_rollback_copy_failure_blocks_learning_until_b_can_be_completed() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let case = interruption_fixture(true);
    let original = case
        .fixture
        .database
        .0
        .parent()
        .unwrap()
        .join("retained-restores/restore-interruption/original-database/lexique.sqlite3");
    let mut locked = None;
    let status = crate::startup::load(
        &case.fixture.database,
        |point| {
            if point == "after-install-audio" {
                return Err("Installation failed before commitment".into());
            }
            if point == "rollback" {
                locked = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(0)
                        .open(&original)
                        .map_err(|error| error.to_string())?,
                );
            }
            Ok(())
        },
        || panic!("partial rollback initialized a learning database"),
    );
    assert_eq!(
        status.state,
        crate::restore::StartupPhase::RecoveryRequired,
        "{status:?}"
    );
    assert!(status.error.as_deref().unwrap().contains("rollback failed"));
    assert!(!case.fixture.database.0.exists());
    assert!(
        status
            .retained_copies
            .iter()
            .any(|copy| copy.kind == crate::restore::PreservationKind::OriginalDatabase)
    );
    drop(locked);
    assert_recovered_b(&case);
}

#[cfg(windows)]
#[test]
fn genuine_cleanup_directory_failure_keeps_committed_a_and_retries_without_reimporting() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let case = interruption_fixture(true);
    let committed = crate::startup::load(
        &case.fixture.database,
        |_| Ok(()),
        || panic!("committed restore initialized before pending cleanup"),
    );
    assert_eq!(
        committed.state,
        crate::restore::StartupPhase::CleanupRequired
    );
    let journal: serde_json::Value = serde_json::from_slice(
        &fs::read(
            case.fixture
                .database
                .0
                .parent()
                .unwrap()
                .join(".lexique-restore.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let stage = PathBuf::from(journal["stage_root"].as_str().unwrap());
    assert!(stage.starts_with(&case.fixture.root.0));
    crate::restore::cleanup_begin(&case.fixture.database, "restore-interruption", |_| Ok(()))
        .unwrap();
    let locked = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .custom_flags(0x0200_0000)
        .open(&stage)
        .unwrap();
    let error = crate::restore::cleanup_complete(
        &case.fixture.database,
        "restore-interruption",
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(
        error.contains("Could not clean verified restore staging"),
        "{error}"
    );
    let blocked = crate::startup::load(
        &case.fixture.database,
        |_| Ok(()),
        || panic!("failed cleanup initialized learning"),
    );
    assert_eq!(
        blocked.state,
        crate::restore::StartupPhase::RecoveryRequired,
        "{blocked:?}"
    );
    assert_timeline(&case, "A subprocess timeline", AUDIO_A);
    drop(locked);
    let resumed = crate::startup::load(&case.fixture.database, |_| Ok(()), || Ok(()));
    assert_eq!(
        resumed.state,
        crate::restore::StartupPhase::Ready,
        "{resumed:?}"
    );
    assert!(!stage.exists());
    assert_timeline(&case, "A subprocess timeline", AUDIO_A);
    assert!(resumed.retained_copies.iter().any(|copy| copy.kind
        == crate::restore::PreservationKind::LogicalBackup
        && copy.byte_length > 0));
    let repeated = crate::restore::cleanup_complete(
        &case.fixture.database,
        "restore-interruption",
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(repeated.state, crate::restore::StartupPhase::Ready);
    assert_timeline(&case, "A subprocess timeline", AUDIO_A);
}
