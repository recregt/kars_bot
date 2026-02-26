mod evaluator;
mod history;
mod notify;
mod provider;
mod service;
mod state;

pub use history::{MetricHistory, MetricSample};
pub use provider::new_metrics_provider;
pub use service::{
    MuteActionError, alert_snapshot, check_alerts, mute_alerts_for, take_daily_summary_report,
    unmute_alerts,
};

#[cfg(test)]
pub use notify::SpyNotifier;
pub use notify::{AlertNotifier, TeloxideNotifier};
pub use state::{AlertState, DailySummaryReport};
