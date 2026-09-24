use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
};

use tokio::sync::mpsc::{
    Receiver,
};

use crate::{
    action::{
        ActionExecutor,
        Task,
    },
    device::DeviceController,
};

pub struct DeviceWorker {
    controller: DeviceController,

    receiver: Receiver<Task>,

    busy: Arc<AtomicBool>,
}

impl DeviceWorker {
    pub fn new(
        controller: DeviceController,
        receiver: Receiver<Task>,
        busy: Arc<AtomicBool>,
    ) -> Self {
        Self {
            controller,

            receiver,

            busy,
        }
    }

    pub async fn run(
        mut self,
    ) {
        let executor =
            ActionExecutor::new(
                self.controller.clone()
            );

        tracing::info!(
            device = self.controller.serial(),
            "worker started"
        );

        while let Some(task) =
            self.receiver.recv().await
        {
            self.busy
                .store(
                    true,
                    Ordering::SeqCst
                );

            let task_name =
                task.name.clone();

            let task_id =
                task.id.clone();

            tracing::info!(
                device = self.controller.serial(),
                task = %task_name,
                task_id = %task_id,
                "worker executing task"
            );

            let result =
                executor
                    .execute_task(&task)
                    .await;

            match result {
                Ok(_) => {
                    tracing::info!(
                        device = self.controller.serial(),
                        task = %task_name,
                        "task completed"
                    );
                }

                Err(error) => {
                    tracing::error!(
                        device = self.controller.serial(),
                        task = %task_name,
                        error = %error,
                        "task failed"
                    );
                }
            }

            self.busy
                .store(
                    false,
                    Ordering::SeqCst
                );
        }

        tracing::info!(
            device = self.controller.serial(),
            "worker stopped"
        );
    }
}