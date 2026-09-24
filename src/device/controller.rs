use crate::{
    adb::AdbClient,
    error::Result,
};

#[derive(Clone)]
pub struct DeviceController {
    adb: AdbClient,
    serial: String,
}

impl DeviceController {
    pub fn new(
        adb: AdbClient,
        serial: impl Into<String>,
    ) -> Self {
        Self {
            adb,
            serial: serial.into(),
        }
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    // --------------------------------------------------
    // BASIC SHELL
    // --------------------------------------------------

    pub async fn shell(
        &self,
        command: &str,
    ) -> Result<String> {
        self.adb
            .shell(&self.serial, command)
            .await
    }

    // --------------------------------------------------
    // TOUCH
    // --------------------------------------------------

    pub async fn tap(
        &self,
        x: i32,
        y: i32,
    ) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "tap",
                    &x.to_string(),
                    &y.to_string(),
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // SWIPE
    // --------------------------------------------------

    pub async fn swipe(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration_ms: u64,
    ) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "swipe",
                    &x1.to_string(),
                    &y1.to_string(),
                    &x2.to_string(),
                    &y2.to_string(),
                    &duration_ms.to_string(),
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // TEXT
    // --------------------------------------------------

    pub async fn input_text(
        &self,
        text: &str,
    ) -> Result<()> {
        let escaped = escape_adb_input_text(text);

        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "text",
                    &escaped,
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // KEY EVENTS
    // --------------------------------------------------

    pub async fn back(&self) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "keyevent",
                    "4",
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn home(&self) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "keyevent",
                    "3",
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // APP
    // --------------------------------------------------

    pub async fn launch_app(
        &self,
        package: &str,
    ) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "monkey",
                    "-p",
                    package,
                    "-c",
                    "android.intent.category.LAUNCHER",
                    "1",
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // SCREENSHOT
    // --------------------------------------------------

    pub async fn screenshot(
        &self,
    ) -> Result<Vec<u8>> {
        self.adb
            .exec(
                &self.serial,
                &[
                    "exec-out",
                    "screencap",
                    "-p",
                ],
            )
            .await
    }

    // --------------------------------------------------
    // UI DUMP
    // --------------------------------------------------

    pub async fn dump_ui(
        &self,
    ) -> Result<String> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "uiautomator",
                    "dump",
                    "/sdcard/window.xml",
                ],
            )
            .await?;

        let xml = self.adb
            .shell_args(
                &self.serial,
                &[
                    "cat",
                    "/sdcard/window.xml",
                ],
            )
            .await?;

        Ok(xml)
    }

    // --------------------------------------------------
    // VOLUME
    // --------------------------------------------------

    pub async fn set_volume(
        &self,
        value: u8,
    ) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "cmd",
                    "media_session",
                    "volume",
                    "--stream",
                    "3",
                    "--set",
                    &value.to_string(),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn mute(&self) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "input",
                    "keyevent",
                    "KEYCODE_VOLUME_MUTE",
                ],
            )
            .await?;

        Ok(())
    }

    // --------------------------------------------------
    // WIFI
    // --------------------------------------------------

    pub async fn wifi_enable(&self) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "svc",
                    "wifi",
                    "enable",
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn wifi_disable(&self) -> Result<()> {
        self.adb
            .shell_args(
                &self.serial,
                &[
                    "svc",
                    "wifi",
                    "disable",
                ],
            )
            .await?;

        Ok(())
    }
}

fn escape_adb_input_text(
    text: &str,
) -> String {
    text
        .replace(' ', "%s")
        .replace('&', "\\&")
        .replace('|', "\\|")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace(';', "\\;")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
}