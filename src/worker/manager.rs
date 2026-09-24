use std::{
    collections::HashMap,
    sync::{
        atomic::{
            AtomicBool,
            Ordering,
        },
        Arc,
    },
};

use tokio::sync::{
    mpsc,
    RwLock,
};

use crate::{
    action::Task,
    adb::AdbClient,
    device::{
        AndroidDevice,
        DeviceController,
        DeviceState,
    },
};

use super::DeviceWorker;

pub struct WorkerHandle {
    pub serial: String,

    pub sender: mpsc::Sender<Task>,

    pub busy: Arc<AtomicBool>,
}

impl WorkerHandle {
    pub fn is_busy(&self) -> bool {
        self.busy
            .load(Ordering::SeqCst)
    }
}

pub struct WorkerManager {
    adb: AdbClient,

    queue_size: usize,

    workers:
        RwLock<HashMap<String, WorkerHandle>>,
}

impl WorkerManager {
    pub fn new(
        adb: AdbClient,
        queue_size: usize,
    ) -> Self {
        Self {
            adb,

            queue_size,

            workers: RwLock::new(
                HashMap::new()
            ),
        }
    }

    pub async fn sync_devices(
        &self,
        devices: &[AndroidDevice],
    ) {
        for device in devices {
            if device.state != DeviceState::Online {
                continue;
            }

            let exists = {
                let workers =
                    self.workers.read().await;

                workers.contains_key(
                    &device.serial
                )
            };

            if !exists {
                self.create_worker(
                    &device.serial
                )
                .await;
            }
        }
    }

    async fn create_worker(
        &self,
        serial: &str,
    ) {
        let mut workers =
            self.workers.write().await;

        if workers.contains_key(serial) {
            return;
        }

        let (
            sender,
            receiver,
        ) = mpsc::channel::<Task>(
            self.queue_size
        );

        let busy =
            Arc::new(
                AtomicBool::new(false)
            );

        let controller =
            DeviceController::new(
                self.adb.clone(),
                serial.to_string()
            );

        let worker =
            DeviceWorker::new(
                controller,
                receiver,
                busy.clone()
            );

        tokio::spawn(
            worker.run()
        );

        let handle =
            WorkerHandle {
                serial: serial.to_string(),

                sender,

                busy,
            };

        workers.insert(
            serial.to_string(),
            handle
        );

        tracing::info!(
            device = serial,
            "worker created"
        );
    }

    pub async fn get(
        &self,
        serial: &str,
    ) -> Option<WorkerHandle> {
        let workers =
            self.workers.read().await;

        workers
            .get(serial)
            .map(|worker| WorkerHandle {
                serial: worker.serial.clone(),
                sender: worker.sender.clone(),
                busy: worker.busy.clone(),
            })
    }

    pub async fn find_idle(
        &self,
        serials: &[String],
    ) -> Option<WorkerHandle> {
        let workers =
            self.workers.read().await;

        for serial in serials {
            let Some(worker) =
                workers.get(serial)
            else {
                continue;
            };

            if !worker.is_busy() {
                return Some(
                    WorkerHandle {
                        serial: worker.serial.clone(),
                        sender: worker.sender.clone(),
                        busy: worker.busy.clone(),
                    }
                );
            }
        }

        None
    }

    pub async fn list(
        &self,
    ) -> Vec<WorkerHandle> {
        let workers =
            self.workers.read().await;

        workers
            .values()
            .map(|worker| {
                WorkerHandle {
                    serial: worker.serial.clone(),
                    sender: worker.sender.clone(),
                    busy: worker.busy.clone(),
                }
            })
            .collect()
    }
}