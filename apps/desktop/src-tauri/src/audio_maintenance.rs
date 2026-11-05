//! Filesystem maintenance has its own bounded worker, separate from synthesis dispatch.
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use lexique_storage as storage;
use tauri::Emitter;

use crate::{
    speech_cache::{self, CacheReport, Reconciler},
    speech_tasks::now,
    vocabulary::{self, DatabasePath},
};

pub struct AudioMaintenance {
    sender: mpsc::SyncSender<()>,
    stopping: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
    report: Arc<Mutex<CacheReport>>,
}

impl AudioMaintenance {
    pub fn start(app: tauri::AppHandle, database: PathBuf) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let stopping = Arc::new(AtomicBool::new(false));
        let report = Arc::new(Mutex::new(CacheReport::default()));
        let worker_stopping = stopping.clone();
        let worker_report = report.clone();
        let thread = thread::spawn(move || {
            while !worker_stopping.load(Ordering::Relaxed) {
                let result = maintain(&app, &database, &receiver, &worker_stopping, &worker_report);
                if let Err(error) = result {
                    speech_cache::report_error(
                        &database,
                        &format!("Audio maintenance failed: {error}"),
                    );
                    if let Ok(mut report) = worker_report.lock() {
                        report.error = Some(error);
                        report.in_progress = false;
                    }
                    let _ = receiver.recv_timeout(Duration::from_secs(60));
                } else {
                    break;
                }
            }
        });
        Self {
            sender,
            stopping,
            thread: Mutex::new(Some(thread)),
            report,
        }
    }

    pub fn wake(&self) {
        let _ = self.sender.try_send(());
    }

    pub fn snapshot(&self) -> Result<CacheReport, String> {
        self.report
            .lock()
            .map(|report| report.clone())
            .map_err(|_| "Audio maintenance status unavailable".into())
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.stopping.store(true, Ordering::Relaxed);
        self.wake();
        if let Some(thread) = self.thread.lock().map_err(|e| e.to_string())?.take() {
            thread.join().map_err(|_| "Audio maintenance panicked")?;
        }
        Ok(())
    }
}

fn maintain(
    app: &tauri::AppHandle,
    database: &PathBuf,
    receiver: &mpsc::Receiver<()>,
    stopping: &AtomicBool,
    shared: &Mutex<CacheReport>,
) -> Result<(), String> {
    let connection = vocabulary::open(&DatabasePath(database.clone()))?;
    let mut cursor = String::new();
    let mut gc_pending = true;
    let mut reconciler = Reconciler::new(database)?;
    let mut gc_error = None;
    loop {
        if stopping.load(Ordering::Relaxed) {
            return Ok(());
        }
        let mut changed = false;
        if gc_pending {
            let mut work = storage::pending_audio_gc_after(&connection, &cursor, 100)
                .map_err(|e| e.to_string())?;
            let remaining = 100 - work.len() as u32;
            work.extend(
                storage::mark_audio_gc_batch(&connection, now()?, remaining)
                    .map_err(|e| e.to_string())?,
            );
            gc_pending = work.len() == 100;
            for gc in work {
                if stopping.load(Ordering::Relaxed) {
                    return Ok(());
                }
                if gc.version.locator > cursor {
                    cursor.clone_from(&gc.version.locator);
                }
                match speech_cache::remove_asset(
                    database,
                    &gc.version.physical_locator,
                    &gc.version.content_hash,
                ) {
                    Ok(_) => {
                        changed |= storage::finish_audio_gc(
                            &connection,
                            &gc.version.locator,
                            gc.generation,
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    Err(error) => {
                        storage::fail_audio_gc(
                            &connection,
                            &gc.version.locator,
                            gc.generation,
                            &error,
                        )
                        .map_err(|e| e.to_string())?;
                        let message = format!("Could not reclaim audio version: {error}");
                        if gc_error.as_ref() != Some(&message) {
                            speech_cache::report_error(database, &message);
                        }
                        gc_error = Some(message);
                    }
                }
            }
        }
        if !gc_pending && reconciler.report.in_progress {
            changed |= reconciler.step(&connection, database, now()?, || {
                stopping.load(Ordering::Relaxed)
            })?;
        }
        let mut report = reconciler.report.clone();
        if gc_error.is_some() {
            report.error = gc_error.clone();
        }
        *shared
            .lock()
            .map_err(|_| "Audio maintenance status unavailable")? = report.clone();
        if changed {
            app.emit("lexique-review-changed", ())
                .map_err(|e| format!("Audio maintenance notification failed: {e}"))?;
        }
        let continuing = gc_pending || report.in_progress;
        let delay = if continuing {
            Duration::from_millis(50)
        } else if report.error.is_some() {
            Duration::from_secs(60)
        } else {
            Duration::from_secs(6 * 60 * 60)
        };
        let woke = match receiver.recv_timeout(delay) {
            Ok(()) => true,
            Err(mpsc::RecvTimeoutError::Timeout) => false,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        };
        if stopping.load(Ordering::Relaxed) {
            return Ok(());
        }
        if !continuing {
            cursor.clear();
            gc_pending = true;
            gc_error = None;
            reconciler = Reconciler::new(database)?;
        } else if woke {
            gc_pending = true;
        }
    }
}
