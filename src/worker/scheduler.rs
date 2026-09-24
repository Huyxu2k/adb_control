use std::sync::Arc;

use tokio::{
    sync::mpsc,
    time::{
        sleep,
        Duration,
    },
};

use crate::{
    action::{
        DeviceSelector,
        Task,
    },
    device::DeviceManager,
};

use super::WorkerManager;

pub struct Scheduler {
    task_receiver: mpsc::Receiver<Task>,

    device_manager:
        Arc<DeviceManager>,

    worker_manager:
        Arc<WorkerManager>,
}

impl Scheduler {
    pub fn new(
        task_receiver: mpsc::Receiver<Task>,

        device_manager:
            Arc<DeviceManager>,

        worker_manager:
            Arc<WorkerManager>,
    ) -> Self {
        Self {
            task_receiver,

            device_manager,

            worker_manager,
        }
    }

    pub async fn run(
        mut self,
    ) {
        tracing::info!(
            "scheduler started"
        );

        while let Some(task) =
            self.task_receiver.recv().await
        {
            tracing::info!(
                task = %task.name,
                task_id = %task.id,
                "scheduler received task"
            );

            self.dispatch(task).await;
        }

        tracing::info!(
            "scheduler stopped"
        );
    }

    async fn dispatch(
        &self,
        task: Task,
    ) {
        loop {
            let serials =
                self.find_candidate_devices(
                    &task.selector
                )
                .await;

            if let Some(
                worker
            ) = self.worker_manager
                .find_idle(&serials)
                .await
            {
                match worker
                    .sender
                    .send(task.clone())
                    .await
                {
                    Ok(_) => {
                        tracing::info!(
                            task = %task.name,
                            device = %worker.serial,
                            "task dispatched"
                        );

                        return;
                    }

                    Err(error) => {
                        tracing::error!(
                            task = %task.name,
                            device = %worker.serial,
                            error = %error,
                            "failed to dispatch task"
                        );

                        sleep(
                            Duration::from_millis(
                                500
                            )
                        )
                        .await;
                    }
                }
            } else {
                tracing::debug!(
                    task = %task.name,
                    "no idle worker available"
                );

                sleep(
                    Duration::from_millis(
                        200
                    )
                )
                .await;
            }
        }
    }

    async fn find_candidate_devices(
        &self,
        selector: &DeviceSelector,
    ) -> Vec<String> {
        let devices =
            self.device_manager
                .online_devices()
                .await;

        devices
            .into_iter()
            .filter(|device| {
                selector.matches(
                    &device.serial
                )
            })
            .map(|device| device.serial)
            .collect()
    }
}