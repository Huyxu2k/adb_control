use tokio::sync::{mpsc::Receiver, oneshot};

use crate::{
    action::{ActionExecutor, Task},
    device::DeviceController,
};

use crate::error::Result;

pub struct WorkerJob {
    pub task: Task,
    pub completion: oneshot::Sender<Result<()>>,
}

pub struct DeviceWorker {
    controller: DeviceController,

    receiver: Receiver<WorkerJob>,
}

impl DeviceWorker {
    pub fn new(controller: DeviceController, receiver: Receiver<WorkerJob>) -> Self {
        Self {
            controller,

            receiver,
        }
    }

    pub async fn run(mut self) {
        let executor = ActionExecutor::new(self.controller.clone());

        tracing::info!(device = self.controller.serial(), "worker started");

        while let Some(job) = self.receiver.recv().await {
            let task = job.task;

            tracing::info!(
                device = self.controller.serial(),
                task = %task.name,
                task_id = %task.id,
                "worker executing task"
            );
            let result = executor.execute_task(&task).await;
            match &result {
                Ok(_) => {
                    tracing::info!(
                        device = self.controller.serial(),
                        task = %task.name,
                        task_id = %task.id,
                        "task completed"
                    );
                }

                Err(error) => {
                    tracing::error!(
                        device = self.controller.serial(),
                        task = %task.name,
                        task_id = %task.id,
                        error = %error,
                        "task failed"
                    );
                }
            }
            let _ = job.completion.send(result);
        }

        tracing::info!(device = self.controller.serial(), "worker stopped");
    }
}
