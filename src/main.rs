mod action;
mod adb;
mod config;
mod device;
mod error;
mod ui;
mod worker;

use std::sync::Arc;

use tokio::{
    sync::mpsc,
    time::{Duration, Sleep, sleep},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    action::{Action, DeviceSelector, Task},
    adb::AdbClient,
    config::AppConfig,
    device::DeviceManager,
    worker::{Scheduler, WorkerManager},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "android_controller=debug",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("android controller starting");

    let config = AppConfig::load("config/config.toml").await?;

    tracing::info!(adb = %config.system.adb_path,max_devices = config.system.max_concurrent_devices,
        "configuration loaded"
    );
    let adb = AdbClient::new(config.system.adb_path.clone());

    let device_manager = Arc::new(DeviceManager::new(adb.clone()));

    let worker_manager = Arc::new(WorkerManager::new(
        adb.clone(),
        config.system.worker_queue_size,
    ));

    let (task_sender, task_receiver) = mpsc::channel::<Task>(100);

    if config.system.auto_discovery {
        let device_manager = device_manager.clone();

        let worker_manager = worker_manager.clone();

        let poll_interval = config.system.device_poll_interval_ms;

        tokio::spawn(async move {
            loop {
                match device_manager.refresh().await {
                    Ok(_) => {
                        let devices = device_manager.list().await;

                        tracing::info!(count = devices.len(), "device discovery updated");

                        for device in &devices {
                            tracing::debug!(
                                device = %device.serial,
                                state = ?device.state,
                                "device state"
                            );
                        }

                        worker_manager.sync_devices(&devices).await;
                    }

                    Err(error) => {
                        tracing::error!(
                            error = %error,
                            "device discovery failed"
                        );
                    }
                }

                sleep(Duration::from_millis(poll_interval)).await;
            }
        });
    }

    let scheduler = Scheduler::new(
        task_receiver,
        device_manager.clone(),
        worker_manager.clone(),
        config.system.max_concurrent_devices
    );

    tokio::spawn(scheduler.run());

    let demo_task = Task::new(
        "open-example-app",
        DeviceSelector::Any,
        vec![
            Action::LaunchApp {
                package: "com.example.app".to_string(),
            },
            Action::Wait { duration_ms: 1000 },
            Action::TapText {
                text: "Login".to_string(),
            },
            Action::InputText {
                text: "hello@example.com".to_string(),
            },
            Action::Wait { duration_ms: 500 },
            Action::Screenshot,
            Action::DumpUi,
        ],
    );

    task_sender.send(demo_task).await?;

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
