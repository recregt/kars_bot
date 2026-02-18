use crate::{config::Alerts, monitor::MetricSample};

#[derive(Clone, Copy)]
pub(super) enum GraphMetric {
    Cpu,
    Ram,
    Disk,
}

impl GraphMetric {
    pub(super) fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "ram" => Some(Self::Ram),
            "disk" => Some(Self::Disk),
            _ => None,
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Ram => "RAM",
            Self::Disk => "Disk",
        }
    }

    pub(super) fn caption(self) -> &'static str {
        match self {
            Self::Cpu => "CPU usage",
            Self::Ram => "RAM usage",
            Self::Disk => "Disk usage",
        }
    }

    pub(super) fn file_name(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Ram => "ram",
            Self::Disk => "disk",
        }
    }

    pub(super) fn value(self, sample: &MetricSample) -> f32 {
        match self {
            Self::Cpu => sample.cpu,
            Self::Ram => sample.ram,
            Self::Disk => sample.disk,
        }
    }

    pub(super) fn threshold(self, alerts: &Alerts) -> f32 {
        match self {
            Self::Cpu => alerts.cpu,
            Self::Ram => alerts.ram,
            Self::Disk => alerts.disk,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct GraphWindow {
    minutes: i64,
}

impl GraphWindow {
    pub(super) fn from_minutes(minutes: i64) -> Option<Self> {
        if minutes <= 0 {
            return None;
        }

        Some(Self { minutes })
    }

    pub(super) fn minutes(self) -> i64 {
        self.minutes
    }

    pub(super) fn suffix(self) -> String {
        if self.minutes % 60 == 0 {
            format!("{}h", self.minutes / 60)
        } else {
            format!("{}m", self.minutes)
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct GraphRequest {
    pub(super) metric: GraphMetric,
    pub(super) window: GraphWindow,
}
