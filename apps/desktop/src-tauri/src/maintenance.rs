//! Admission covers an entire accepted operation, including its blocking job and callbacks.
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Debug, Default)]
struct Admission {
    busy: bool,
    active: usize,
    readers_closed: bool,
    readers: usize,
}

#[derive(Debug, Default)]
struct Gate {
    state: Mutex<Admission>,
    changed: Condvar,
}

#[derive(Clone, Debug, Default)]
pub struct MaintenanceCoordinator {
    gate: Arc<Gate>,
}

#[derive(Debug)]
pub struct WriteAdmission {
    gate: Arc<Gate>,
}

#[derive(Debug)]
pub struct ReadAdmission {
    gate: Arc<Gate>,
}

impl Drop for ReadAdmission {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.readers -= 1;
        self.gate.changed.notify_all();
    }
}

impl Drop for WriteAdmission {
    fn drop(&mut self) {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.active -= 1;
        self.gate.changed.notify_all();
    }
}

pub struct MaintenanceGuard {
    gate: Arc<Gate>,
    committed: bool,
}

impl MaintenanceGuard {
    pub fn close_readers(&self, cancelled: &AtomicBool) -> Result<(), String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        state.readers_closed = true;
        while state.readers != 0 {
            if cancelled.load(Ordering::Acquire) {
                return Err("backup_cancelled".into());
            }
            state = self
                .gate
                .changed
                .wait_timeout(state, Duration::from_millis(50))
                .map_err(|_| "Maintenance reader drain unavailable")?
                .0;
        }
        Ok(())
    }

    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for MaintenanceGuard {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.busy = false;
        state.readers_closed = false;
        self.gate.changed.notify_all();
    }
}

impl MaintenanceCoordinator {
    pub(crate) fn suspend_startup(&self) -> Result<(), String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        if state.active != 0 || state.readers != 0 {
            return Err("Learning operations have not closed".into());
        }
        state.busy = true;
        state.readers_closed = true;
        Ok(())
    }

    pub(crate) fn resume_startup(&self) -> Result<(), String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        state.busy = false;
        state.readers_closed = false;
        self.gate.changed.notify_all();
        Ok(())
    }

    pub fn admit_reader(&self) -> Result<ReadAdmission, String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        if state.readers_closed {
            return Err("maintenance_busy: Learning data is restarting".into());
        }
        state.readers += 1;
        Ok(ReadAdmission {
            gate: self.gate.clone(),
        })
    }

    pub fn admit(&self) -> Result<WriteAdmission, String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        if state.busy {
            return Err("maintenance_busy: Learning data is being collected".into());
        }
        state.active += 1;
        Ok(WriteAdmission {
            gate: self.gate.clone(),
        })
    }

    pub fn begin(&self, cancelled: &AtomicBool) -> Result<MaintenanceGuard, String> {
        let mut state = self
            .gate
            .state
            .lock()
            .map_err(|_| "Maintenance admission unavailable")?;
        if state.busy {
            return Err("maintenance_busy: Another maintenance operation is active".into());
        }
        state.busy = true;
        let guard = MaintenanceGuard {
            gate: self.gate.clone(),
            committed: false,
        };
        while state.active != 0 {
            if cancelled.load(Ordering::Acquire) {
                drop(state);
                drop(guard);
                return Err("backup_cancelled".into());
            }
            state = self
                .gate
                .changed
                .wait_timeout(state, Duration::from_millis(50))
                .map_err(|_| "Maintenance drain unavailable")?
                .0;
        }
        drop(state);
        if cancelled.load(Ordering::Acquire) {
            return Err("backup_cancelled".into());
        }
        Ok(guard)
    }
}

pub fn admit_app(app: &tauri::AppHandle) -> Result<WriteAdmission, String> {
    use tauri::Manager;
    app.try_state::<MaintenanceCoordinator>()
        .ok_or("Maintenance admission unavailable")?
        .admit()
}

pub fn admit_reader_app(app: &tauri::AppHandle) -> Result<ReadAdmission, String> {
    use tauri::Manager;
    app.try_state::<MaintenanceCoordinator>()
        .ok_or("Maintenance admission unavailable")?
        .admit_reader()
}

/// The filesystem worker enters once per bounded GC/reconciliation unit.
#[derive(Clone, Default)]
pub struct WorkerGate {
    gate: Arc<Gate>,
}

impl WorkerGate {
    pub fn enter(&self, stopping: &AtomicBool) -> Option<WriteAdmission> {
        let mut state = self
            .gate
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while state.busy && !stopping.load(Ordering::Acquire) {
            state = self
                .gate
                .changed
                .wait_timeout(state, Duration::from_millis(50))
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
        if stopping.load(Ordering::Acquire) {
            return None;
        }
        state.active += 1;
        Some(WriteAdmission {
            gate: self.gate.clone(),
        })
    }

    pub fn pause(&self, cancelled: &AtomicBool) -> Result<MaintenanceGuard, String> {
        MaintenanceCoordinator {
            gate: self.gate.clone(),
        }
        .begin(cancelled)
    }
}
