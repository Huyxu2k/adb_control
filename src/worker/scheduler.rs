use std::sync::Arc;

use tokio::{ sync::{ Semaphore, mpsc, oneshot }, time::{ Duration, sleep } };

use crate::{ error::{ AppError, Result }, worker::manager::WorkerState };
use crate::{ action::{ DeviceSelector, Task }, device::DeviceManager, worker::worker::WorkerJob };

use super::WorkerManager;

pub struct Scheduler {
    task_receiver: mpsc::Receiver<Task>,
    inner: Arc<SchedulerInner>,
}

struct SchedulerInner {
    device_manager: Arc<DeviceManager>,

    worker_manager: Arc<WorkerManager>,

    concurrency: Arc<Semaphore>,
}

impl Scheduler {
    pub fn new(
        task_receiver: mpsc::Receiver<Task>,

        device_manager: Arc<DeviceManager>,

        worker_manager: Arc<WorkerManager>,

        max_concurrent_devices: usize
    ) -> Self {
        Self {
            task_receiver,
            inner: Arc::new(SchedulerInner {
                device_manager,
                worker_manager,
                concurrency: Arc::new(Semaphore::new(max_concurrent_devices)),
            }),
        }
    }

    pub async fn run(mut self) {
        tracing::info!(
            max_concurrent_devices = self.inner.concurrency.available_permits(),
            "scheduler started"
        );

        while let Some(task) = self.task_receiver.recv().await {
            let inner = Arc::clone(&self.inner);
            tokio::spawn(async move {
                if let Err(error) = inner.dispatch(task).await {
                    tracing::error!(error = %error, "task dispatch failed");
                }
            });
        }

        tracing::info!("scheduler stopped");
    }

    async fn find_candidate_devices(&self, selector: &DeviceSelector) -> Vec<String> {
        let devices = self.inner.device_manager.online_devices().await;

        devices
            .into_iter()
            .filter(|device| selector.matches(&device.serial))
            .map(|device| device.serial)
            .collect()
    }
}

impl SchedulerInner {
    async fn dispatch(&self, task: Task) -> Result<()> {
        let permit = Arc::clone(&self.concurrency)
            .acquire_owned().await
            .map_err(|_| AppError::ActionFailed("scheduler semaphore closed".to_string()))?;

        tracing::info!(
            task = %task.name,
            task_id = %task.id,
            "concurrency slot acquired"
        );

        let devices = self.device_manager.list().await;
        let candidate_serials: Vec<String> = devices
            .into_iter()
            .filter(|device| device.is_online())
            .map(|device| device.serial)
            .collect();

        if candidate_serials.is_empty() {
            drop(permit);
            return Err(AppError::DeviceNotFound("no online device available".to_string()));
        }

        let worker = self.worker_manager
            .find_available(&candidate_serials).await
            .ok_or_else(|| AppError::DeviceNotFound("no idle worker available".to_string()))?;
        tracing::info!(task = %task.name, task_id = %task.id, device = %worker.serial, "task assigned to worker");

        let (completion_tx, completion_rx) = oneshot::channel();
        let job = WorkerJob {
            task,
            completion: completion_tx,
        };

        if let Err(error) = worker.sender.send(job).await {
            tracing::error!(device = %worker.serial, error = %error, "failed to send task to worker");

            self.worker_manager.release(&worker.serial).await;

            drop(permit);

            return Err(AppError::ActionFailed(format!("failed to send task to worker: {error}")));
        }

        let result = completion_rx.await;
        match result {
            Ok(task_result) => {
                match &task_result {
                    Ok(_) => {
                        tracing::info!(device = %worker.serial, "task completed successfully");
                    }
                    Err(error) => {
                        tracing::error!(device = %worker.serial, error = %error, "task failed");
                    }
                }
                self.worker_manager.release(&worker.serial).await;
                task_result
            }
            Err(_) => {
                tracing::error!(device = %worker.serial, "worker dropped completion channel");
                self.worker_manager.set_state(&worker.serial, WorkerState::Error).await;
                Err(
                    AppError::ActionFailed(
                        format!("worker {} dropped completion channel", worker.serial)
                    )
                )
            }
        }
    }
}
