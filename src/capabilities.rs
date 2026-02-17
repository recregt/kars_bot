use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub is_systemd: bool,
    pub has_sensors: bool,
    pub has_free: bool,
    pub has_top: bool,
    pub has_ip: bool,
    pub has_ss: bool,
    pub has_uptime: bool,
}

impl Capabilities {
    pub fn detect() -> Self {
        let has_systemctl = command_exists("systemctl");

        Self {
            is_systemd: has_systemctl && Path::new("/run/systemd/system").exists(),
            has_sensors: command_exists("sensors"),
            has_free: command_exists("free"),
            has_top: command_exists("top"),
            has_ip: command_exists("ip"),
            has_ss: command_exists("ss"),
            has_uptime: command_exists("uptime"),
        }
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {} >/dev/null 2>&1", command))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
