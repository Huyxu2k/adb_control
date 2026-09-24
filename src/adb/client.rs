use std::process::Stdio;

use tokio::process::Command;

use crate::error::{
    AppError,
    Result,
};

use super::parser::{
    parse_devices_output,
    AdbDeviceInfo,
};

#[derive(Clone)]
pub struct AdbClient {
    adb_path: String,
}

impl AdbClient {
    pub fn new(adb_path: impl Into<String>) -> Self {
        Self {
            adb_path: adb_path.into(),
        }
    }

    pub fn adb_path(&self) -> &str {
        &self.adb_path
    }

    pub async fn devices(&self) -> Result<Vec<AdbDeviceInfo>> {
        let output = Command::new(&self.adb_path)
            .arg("devices")
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::AdbCommand(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(parse_devices_output(&stdout))
    }

    pub async fn shell(
        &self,
        serial: &str,
        command: &str,
    ) -> Result<String> {
        let output = Command::new(&self.adb_path)
            .arg("-s")
            .arg(serial)
            .arg("shell")
            .arg(command)
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::AdbCommand(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn shell_args(
        &self,
        serial: &str,
        args: &[&str],
    ) -> Result<String> {
        let mut command = Command::new(&self.adb_path);

        command
            .arg("-s")
            .arg(serial)
            .arg("shell");

        for arg in args {
            command.arg(arg);
        }

        let output = command.output().await?;

        if !output.status.success() {
            return Err(AppError::AdbCommand(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn exec(
        &self,
        serial: &str,
        args: &[&str],
    ) -> Result<Vec<u8>> {
        let mut command = Command::new(&self.adb_path);

        command
            .arg("-s")
            .arg(serial);

        for arg in args {
            command.arg(arg);
        }

        command.stdout(Stdio::piped());

        let output = command.output().await?;

        if !output.status.success() {
            return Err(AppError::AdbCommand(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(output.stdout)
    }

    pub async fn is_device_online(
        &self,
        serial: &str,
    ) -> Result<bool> {
        let devices = self.devices().await?;

        Ok(devices
            .iter()
            .any(|device| {
                device.serial == serial &&
                device.state == "device"
            }))
    }
}