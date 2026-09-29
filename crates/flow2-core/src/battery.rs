//! Battery level. Only macOS is implemented, see the plan for Windows and Linux.
use serde_json::Value;

/// Keys observed in `system_profiler SPBluetoothDataType -json` (reference project, real hardware).
const LEVEL_KEYS: [&str; 2] = ["device_batteryLevelMain", "device_batteryLevel"];

/// Finds the first connected device whose name contains `name_filter` and reports its battery percent.
pub fn parse_system_profiler(json: &str, name_filter: &str) -> Option<u8> {
    let root: Value = serde_json::from_str(json).ok()?;
    let connected = root
        .pointer("/SPBluetoothDataType/0/device_connected")?
        .as_array()?;
    let needle = name_filter.to_lowercase();
    connected.iter().find_map(|wrapper| {
        let (name, info) = wrapper.as_object()?.iter().next()?;
        if !name.to_lowercase().contains(&needle) {
            return None;
        }
        LEVEL_KEYS.iter().find_map(|k| {
            info.get(k)?
                .as_str()?
                .trim_end_matches('%')
                .trim()
                .parse::<u8>()
                .ok()
        })
    })
}

#[cfg(target_os = "macos")]
pub fn read() -> Option<u8> {
    // ponytail: no timeout, Command::output drains both pipes so it cannot deadlock. Add one if system_profiler hangs.
    let out = std::process::Command::new("/usr/sbin/system_profiler")
        .args(["SPBluetoothDataType", "-json"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| parse_system_profiler(&String::from_utf8_lossy(&out.stdout), "flow"))
        .flatten()
}

#[cfg(not(target_os = "macos"))]
pub fn read() -> Option<u8> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"SPBluetoothDataType":[{"device_connected":[
        {"Magic Mouse":{"device_batteryLevelMain":"55%"}},
        {"Lofree Flow2":{"device_address":"AA","device_batteryLevelMain":"87%"}}
    ]}]}"#;

    #[test]
    fn finds_matching_device_case_insensitively() {
        assert_eq!(parse_system_profiler(SAMPLE, "flow"), Some(87));
        assert_eq!(parse_system_profiler(SAMPLE, "FLOW"), Some(87));
    }

    #[test]
    fn falls_back_to_second_key() {
        let json = r#"{"SPBluetoothDataType":[{"device_connected":[{"Flow":{"device_batteryLevel":"12%"}}]}]}"#;
        assert_eq!(parse_system_profiler(json, "flow"), Some(12));
    }

    #[test]
    fn skips_matching_device_without_battery_and_continues() {
        let json = r#"{"SPBluetoothDataType":[{"device_connected":[
            {"Flow A":{"device_address":"1"}},
            {"Flow B":{"device_batteryLevelMain":"40%"}}]}]}"#;
        assert_eq!(parse_system_profiler(json, "flow"), Some(40));
    }

    #[test]
    fn returns_none_for_missing_or_invalid_data() {
        assert_eq!(parse_system_profiler(SAMPLE, "keychron"), None);
        assert_eq!(parse_system_profiler("not json", "flow"), None);
        assert_eq!(parse_system_profiler("{}", "flow"), None);
    }
}
