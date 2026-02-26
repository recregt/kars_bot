use chrono::Utc;
use teloxide::prelude::*;
use tokio::time::{Duration, sleep};

use crate::app_context::AppContext;
use crate::architecture::{
    adapters::{TeloxideNotifier, new_metrics_provider},
    use_cases::{CheckAlertsContext, check_alerts_use_case},
};

pub(super) fn start_monitor_job(bot: Bot, app_context: AppContext) {
    tokio::spawn(async move {
        let notifier = TeloxideNotifier(bot.clone());
        let mut metrics_provider = new_metrics_provider(app_context.config.simulation.enabled);
        if app_context.config.simulation.enabled {
            log::warn!(
                "simulation_mode_enabled profile={} source=monitor_provider",
                app_context.config.simulation.profile
            );
        }
        let mut previous_tick = None;
        let reporting_store = app_context.reporting_store.clone();

        loop {
            let runtime_config = app_context.runtime_config.read().await.clone();
            let now = Utc::now();

            if let Some(previous) = previous_tick {
                let elapsed_secs = now.signed_duration_since(previous).num_seconds().max(0);
                let threshold_secs = (runtime_config.monitor_interval * 2) as i64;
                if elapsed_secs > threshold_secs {
                    log::warn!(
                        "monitor_loop_delayed elapsed_secs={elapsed_secs} threshold_secs={threshold_secs}"
                    );
                }
            }

            previous_tick = Some(now);

            {
                let mut tick = app_context.monitor.last_monitor_tick.lock().await;
                *tick = Some(now);
            }

            check_alerts_use_case(
                CheckAlertsContext {
                    notifier: &notifier,
                    config: &app_context.config,
                    runtime_config: &runtime_config,
                    reporting_store: reporting_store.as_ref(),
                    anomaly_storage: app_context.anomaly_storage.as_ref(),
                    state: &app_context.monitor.alert_state,
                    metric_history: &app_context.monitor.metric_history,
                },
                &mut metrics_provider,
            )
            .await;

            let sleep_duration = Duration::from_secs(runtime_config.monitor_interval);
            tokio::select! {
                () = sleep(sleep_duration) => {}
                () = app_context.runtime_update_notify.notified() => {
                    log::info!(
                        "monitor_interval_change_interrupt_applied previous_sleep_secs={}",
                        runtime_config.monitor_interval
                    );
                }
            }
        }
    });
}
