use uuid::Uuid;

use super::Action;

#[derive(Debug, Clone)]
pub enum DeviceSelector {
    Any,

    Device(String),

    Devices(Vec<String>),
}

impl DeviceSelector {
    pub fn matches(
        &self,
        serial: &str,
    ) -> bool {
        match self {
            Self::Any => true,

            Self::Device(target) => {
                target == serial
            }

            Self::Devices(targets) => {
                targets
                    .iter()
                    .any(|target| target == serial)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,

    pub name: String,

    pub selector: DeviceSelector,

    pub actions: Vec<Action>,
}

impl Task {
    pub fn new(
        name: impl Into<String>,
        selector: DeviceSelector,
        actions: Vec<Action>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),

            name: name.into(),

            selector,

            actions,
        }
    }
}