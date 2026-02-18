use chrono::Utc;
use teloxide::prelude::*;
use tokio::time::{Duration, sleep};

use crate::app_context::AppContext;
use crate::monitor::ActiveMetricsProvider;
use crate::monitor::check_alerts;

pub(super) fn start_monitor_job(bot: Bot, app_context: AppContext) {
    tokio::spawn(async move {
        let mut metrics_provider =
            ActiveMetricsProvider::new(app_context.config.simulation.enabled);
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
                        "monitor_loop_delayed elapsed_secs={} threshold_secs={}",
                        elapsed_secs,
                        threshold_secs
                    );
                }
            }

            previous_tick = Some(now);

            {
                let mut tick = app_context.last_monitor_tick.lock().await;
                *tick = Some(now);
            }

            check_alerts(
                &bot,
                &app_context.config,
                &runtime_config,
                reporting_store.as_ref(),
                &app_context.alert_state,
                &app_context.metric_history,
                &mut metrics_provider,
            )
            .await;

            let sleep_duration = Duration::from_secs(runtime_config.monitor_interval);
            tokio::select! {
                _ = sleep(sleep_duration) => {}
                _ = app_context.runtime_update_notify.notified() => {
                    log::info!(
                        "monitor_interval_change_interrupt_applied previous_sleep_secs={}",
                        runtime_config.monitor_interval
                    );
                }
            }
        }
    });
}
