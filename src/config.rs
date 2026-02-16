use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot_token: String,
    pub owner_id: u64,
    pub authorized_users: Vec<u64>,
    pub alerts: Alerts,
    pub monitor_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Alerts {
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
}

pub fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path).expect("Failed to read config file");
    toml::from_str(&content).expect("Invalid config format")
}