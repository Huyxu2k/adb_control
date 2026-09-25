use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::{adb::AdbClient, error::Result};

use super::{AndroidDevice, DeviceState};

pub struct DeviceManager {
    adb: AdbClient,

    devices: RwLock<HashMap<String, AndroidDevice>>,
}

impl DeviceManager {
    pub fn new(adb: AdbClient) -> Self {
        Self {
            adb,

            devices: RwLock::new(HashMap::new()),
        }
    }

    pub async fn refresh(&self) -> Result<()> {
        let adb_devices = self.adb.devices().await?;

        let mut devices = self.devices.write().await;

        let current_serials = adb_devices
            .iter()
            .map(|device| device.serial.clone())
            .collect::<std::collections::HashSet<_>>();

        for info in adb_devices {
            let state = DeviceState::from_adb_state(&info.state);

            devices
                .entry(info.serial.clone())
                .and_modify(|device| {
                    device.state = state;
                })
                .or_insert_with(|| AndroidDevice::new(info.serial, state));
        }

        devices.retain(|serial, _| current_serials.contains(serial));

        Ok(())
    }

    pub async fn list(&self) -> Vec<AndroidDevice> {
        self.devices.read().await.values().cloned().collect()
    }

    pub async fn online_devices(&self) -> Vec<AndroidDevice> {
        self.devices
            .read()
            .await
            .values()
            .filter(|device| device.state == DeviceState::Online)
            .cloned()
            .collect()
    }

    pub async fn get(&self, serial: &str) -> Option<AndroidDevice> {
        self.devices.read().await.get(serial).cloned()
    }

    pub async fn is_online(&self, serial: &str) -> bool {
        self.devices
            .read()
            .await
            .get(serial)
            .map(|device| device.is_online())
            .unwrap_or(false)
    }
}
