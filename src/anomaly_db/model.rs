use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnomalyEvent {
    pub timestamp: String,
    pub cpu: f32,
    pub ram: f32,
    pub disk: f32,
    pub cpu_threshold: f32,
    pub ram_threshold: f32,
    pub disk_threshold: f32,
    pub cpu_over: bool,
    pub ram_over: bool,
    pub disk_over: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AnomalyIndexEntry {
    pub(crate) timestamp: String,
    pub(crate) cpu: f32,
    pub(crate) ram: f32,
    pub(crate) disk: f32,
    pub(crate) cpu_threshold: f32,
    pub(crate) ram_threshold: f32,
    pub(crate) disk_threshold: f32,
    pub(crate) cpu_over: bool,
    pub(crate) ram_over: bool,
    pub(crate) disk_over: bool,
}