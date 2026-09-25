#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unknown,
    Offline,
    Online,
    Busy,
    Error,
}

impl DeviceState {
    pub fn from_adb_state(state: &str) -> Self {
        match state {
            "device" => Self::Online,
            "offline" => Self::Offline,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AndroidDevice {
    pub serial: String,
    pub state: DeviceState,
}

impl AndroidDevice {
    pub fn new(serial: impl Into<String>, state: DeviceState) -> Self {
        Self {
            serial: serial.into(),
            state,
        }
    }

    pub fn is_online(&self) -> bool {
        self.state == DeviceState::Online
    }
}
