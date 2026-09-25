use std::{
    collections::HashMap,
    sync::Arc
};

use tokio::sync::{RwLock, mpsc};

use crate::{
    adb::AdbClient,
    device::{AndroidDevice, DeviceController, DeviceState},
    worker::worker::WorkerJob,
};

use super::DeviceWorker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    Idle,
    Busy,
    Offline,
    Stopped,
    Error,
}

#[derive(Clone)]
pub struct WorkerHandle {
    pub serial: String,

    pub sender: mpsc::Sender<WorkerJob>,

    state: Arc<RwLock<WorkerState>>,
}

impl WorkerHandle {
    pub fn new(
        serial: String,
        sender: mpsc::Sender<WorkerJob>,
        state: Arc<RwLock<WorkerState>>,
    ) -> Self {
        Self {
            serial,
            sender,
            state,
        }
    }
    pub async fn state(&self) -> WorkerState {
        *self.state.read().await
    }
    pub async fn is_idle(&self) -> bool {
        self.state().await == WorkerState::Idle
    }
    pub async fn is_busy(&self) -> bool {
        self.state().await == WorkerState::Busy
    }
    pub async fn is_offline(&self) -> bool {
        self.state().await == WorkerState::Offline
    }
    pub async fn set_state(&self, state: WorkerState) {
        let mut current = self.state.write().await;
        *current = state;
    }
}

struct WorkerRecord {
    handle: WorkerHandle,
}

pub struct WorkerManager {
    adb: AdbClient,

    queue_size: usize,

    workers: RwLock<HashMap<String, WorkerRecord>>,
}

impl WorkerManager {
    pub fn new(adb: AdbClient, queue_size: usize) -> Self {
        Self {
            adb,

            queue_size,

            workers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn sync_devices(&self, devices: &[AndroidDevice]) {
        for device in devices {
            match device.state {
                DeviceState::Online => {
                    self.ensure_worker(&device.serial).await;
                    self.handle_device_online(&device.serial).await;
                }
                DeviceState::Offline => {
                    self.handle_device_offline(&device.serial).await;
                }
                _ => {}
            }
        }
    }

    async fn ensure_worker(&self, serial: &str) {
        {
            let workers = self.workers.read().await;
            if workers.contains_key(serial) {
                return;
            }
        }
        self.create_worker(serial).await;
    }

    async fn handle_device_online(&self, serial: &str) {
        let worker = {
            let workers = self.workers.read().await;
            workers.get(serial).map(|record| record.handle.clone())
        };
        let Some(worker) = worker else {
            return;
        };
        let current_state = worker.state().await;
        if current_state == WorkerState::Offline {
            worker.set_state(WorkerState::Idle).await;
            tracing::info!(device = serial, "worker device reconnected");
        }
    }

    async fn handle_device_offline(&self, serial: &str) {
        let worker = {
            let workers = self.workers.read().await;
            workers.get(serial).map(|record| record.handle.clone())
        };
        let Some(worker) = worker else {
            return;
        };
        let current_state = worker.state().await;
        if current_state != WorkerState::Stopped {
            worker.set_state(WorkerState::Offline).await;
            tracing::warn!( device = serial, previous_state = ?current_state, "worker device went offline" );
        }
    }

    async fn create_worker(&self, serial: &str) {
        let mut workers = self.workers.write().await;

        if workers.contains_key(serial) {
            return;
        }

        let (sender, receiver) = mpsc::channel::<WorkerJob>(self.queue_size);

        let state = Arc::new(RwLock::new(WorkerState::Idle));

        let controller = DeviceController::new(self.adb.clone(), serial.to_string());

        let worker = DeviceWorker::new(controller, receiver);

        tokio::spawn(worker.run());

        let handle = WorkerHandle::new(serial.to_string(), sender, state);

        workers.insert(serial.to_string(), WorkerRecord { handle });

        tracing::info!(device = serial, "worker created");
    }

    pub async fn get(&self, serial: &str) -> Option<WorkerHandle> {
        let workers = self.workers.read().await;

        workers.get(serial).map(|record| record.handle.clone())
    }

    pub async fn list(&self) -> Vec<WorkerHandle> {
        let workers = self.workers.read().await;

        workers
            .values()
            .map(|record| record.handle.clone())
            .collect()
    }

    //
    //  WORK STATE
    //

    pub async fn state(&self, serial: &str) -> Option<WorkerState> {
        let worker = self.get(serial).await?;
        Some(worker.state().await)
    }

    pub async fn set_state(&self, serial: &str, state: WorkerState) {
        let worker = self.get(serial).await;

        let Some(worker) = worker else {
            return;
        };
        worker.set_state(state).await;
        tracing::debug!(device = serial, ?state, "worker state changed");
    }

    //
    //  WORKER RESERVATION
    //

    pub async fn find_available(&self, serials: &[String]) -> Option<WorkerHandle> {
        let workers = self.workers.write().await;

        for serial in serials {
            let Some(record) = workers.get(serial) else {
                continue;
            };

            let worker = &record.handle; 
            let current_state = worker.state().await;

            if current_state != WorkerState::Idle { continue; }

            worker .set_state( WorkerState::Busy ) .await; 
            tracing::debug!( device = serial, "worker reserved" ); 

            return Some( worker.clone() );
        }
        None
    }

    pub async fn release(&self, serial: &str) {
        let worker = self.get(serial).await; 
        let Some(worker) = worker else { return; }; 
        let current_state = worker.state().await;

        if current_state == WorkerState::Busy { 
            worker .set_state(WorkerState::Idle).await; 
            tracing::debug!( device = serial, "worker released" ); 
        }
    }

    pub async fn mark_offline(&self, serial: &str) {
        self.set_state(serial, WorkerState::Offline).await;
    }

    pub async fn mark_idle(&self, serial: &str) {
        self.set_state(serial, WorkerState::Idle).await;
    }


    pub async fn busy_count(&self) -> usize {
        let workers = self.list().await;
        let mut count = 0;
        for worker in workers {
            if worker.is_busy().await {
                count += 1;
            }
        }
        count
    }

    pub async fn idle_count(&self) -> usize {
        let workers = self.list().await;
        let mut count = 0;
        for worker in workers {
            if worker.is_idle().await {
                count += 1;
            }
        }
        count
    }

    pub async fn offline_count(&self) -> usize {
       let workers = self.list().await;
        let mut count = 0;
        for worker in workers {
            if worker.is_offline().await {
                count += 1;
            }
        }
        count
    }
}
