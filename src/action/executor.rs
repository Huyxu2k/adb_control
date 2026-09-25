use tokio::time::{Duration, Instant, sleep};

use crate::{
    device::DeviceController,
    error::{AppError, Result},
    ui::{Selector, UiParser},
};

use super::{Action, ActionResult, Task};

pub struct ActionExecutor {
    controller: DeviceController,
}

impl ActionExecutor {
    pub fn new(controller: DeviceController) -> Self {
        Self { controller }
    }

    pub async fn execute_task(&self, task: &Task) -> Result<()> {
        tracing::info!(
            device = self.controller.serial(),
            task = %task.name,
            task_id = %task.id,
            "starting task"
        );

        for (index, action) in task.actions.iter().enumerate() {
            tracing::debug!(
                device = self.controller.serial(),
                task = %task.name,
                action_index = index,
                "executing action"
            );

            self.execute(action).await?;
        }

        tracing::info!(
            device = self.controller.serial(),
            task = %task.name,
            "task completed"
        );

        Ok(())
    }

    pub async fn execute(&self, action: &Action) -> Result<ActionResult> {
        match action {
            Action::LaunchApp { package } => {
                self.controller.launch_app(package).await?;

                Ok(ActionResult::Success)
            }

            Action::Tap { x, y } => {
                self.controller.tap(*x, *y).await?;

                Ok(ActionResult::Success)
            }

            Action::TapText { text } => {
                self.tap_selector(Selector::Text(text.clone())).await?;

                Ok(ActionResult::Success)
            }

            Action::TapId { id } => {
                self.tap_selector(Selector::ResourceId(id.clone())).await?;

                Ok(ActionResult::Success)
            }

            Action::TapClass { class_name } => {
                self.tap_selector(Selector::Class(class_name.clone()))
                    .await?;

                Ok(ActionResult::Success)
            }

            Action::TapContentDescription { content_desc } => {
                self.tap_selector(Selector::ContentDescription(content_desc.clone()))
                    .await?;

                Ok(ActionResult::Success)
            }

            Action::InputText { text } => {
                self.controller.input_text(text).await?;

                Ok(ActionResult::Success)
            }

            Action::Swipe {
                x1,
                y1,
                x2,
                y2,
                duration_ms,
            } => {
                self.controller
                    .swipe(*x1, *y1, *x2, *y2, *duration_ms)
                    .await?;

                Ok(ActionResult::Success)
            }

            Action::Back => {
                self.controller.back().await?;

                Ok(ActionResult::Success)
            }

            Action::Home => {
                self.controller.home().await?;

                Ok(ActionResult::Success)
            }

            Action::Screenshot => {
                let data = self.controller.screenshot().await?;

                Ok(ActionResult::Screenshot(data))
            }

            Action::DumpUi => {
                let xml = self.controller.dump_ui().await?;

                Ok(ActionResult::UiDump(xml))
            }

            Action::Wait { duration_ms } => {
                sleep(Duration::from_millis(*duration_ms)).await;

                Ok(ActionResult::Success)
            }

            Action::WaitFor {
                selector,
                timeout_ms,
                interval_ms,
            } => {
                self.wait_for(selector, *timeout_ms, *interval_ms).await?;

                Ok(ActionResult::Success)
            }

            Action::SetVolume { value } => {
                self.controller.set_volume(*value).await?;

                Ok(ActionResult::Success)
            }

            Action::Mute => {
                self.controller.mute().await?;

                Ok(ActionResult::Success)
            }

            Action::WifiEnable => {
                self.controller.wifi_enable().await?;

                Ok(ActionResult::Success)
            }

            Action::WifiDisable => {
                self.controller.wifi_disable().await?;

                Ok(ActionResult::Success)
            }

            Action::Shell { command } => {
                self.controller.shell(command).await?;

                Ok(ActionResult::Success)
            }
        }
    }

    async fn tap_selector(&self, selector: Selector) -> Result<()> {
        let xml = self.controller.dump_ui().await?;

        let tree = UiParser::parse(&xml)?;

        let node = tree.find(&selector).ok_or(AppError::ElementNotFound)?;

        if !node.enabled {
            return Err(AppError::ActionFailed(format!(
                "element is disabled: {:?}",
                selector
            )));
        }

        let (x, y) = node.center();

        self.controller.tap(x, y).await?;

        Ok(())
    }

    async fn wait_for(&self, selector: &Selector, timeout_ms: u64, interval_ms: u64) -> Result<()> {
        let start = Instant::now();

        let timeout = Duration::from_millis(timeout_ms);

        let interval = Duration::from_millis(interval_ms);

        loop {
            if start.elapsed() >= timeout {
                return Err(AppError::Timeout);
            }

            let xml = self.controller.dump_ui().await?;

            let tree = UiParser::parse(&xml)?;

            if tree.find(selector).is_some() {
                return Ok(());
            }

            sleep(interval).await;
        }
    }
}
