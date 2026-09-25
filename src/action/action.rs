use crate::ui::Selector;

#[derive(Debug, Clone)]
pub enum Action {
    LaunchApp {
        package: String,
    },

    Tap {
        x: i32,
        y: i32,
    },

    TapText {
        text: String,
    },

    TapId {
        id: String,
    },

    TapClass {
        class_name: String,
    },

    TapContentDescription {
        content_desc: String,
    },

    InputText {
        text: String,
    },

    Swipe {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration_ms: u64,
    },

    Back,

    Home,

    Screenshot,

    DumpUi,

    Wait {
        duration_ms: u64,
    },

    WaitFor {
        selector: Selector,
        timeout_ms: u64,
        interval_ms: u64,
    },

    SetVolume {
        value: u8,
    },

    Mute,

    WifiEnable,

    WifiDisable,

    Shell {
        command: String,
    },
}
