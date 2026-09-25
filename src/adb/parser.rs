#[derive(Debug, Clone)]
pub struct AdbDeviceInfo {
    pub serial: String,
    pub state: String,
}

pub fn parse_devices_output(output: &str) -> Vec<AdbDeviceInfo> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let line = line.trim();

            if line.is_empty() {
                return None;
            }

            let mut parts = line.split_whitespace();

            let serial = parts.next()?.to_string();

            let state = parts.next()?.to_string();

            Some(AdbDeviceInfo { serial, state })
        })
        .collect()
}
