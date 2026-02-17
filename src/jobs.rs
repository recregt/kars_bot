use chrono::{Days, TimeZone, Utc};
use teloxide::prelude::*;
use tokio::time::{interval, sleep, Duration};

use crate::anomaly_journal::run_maintenance;
use crate::app_context::AppContext;
use crate::monitor::{check_alerts, take_daily_summary_report, DailySummaryReport, RealMetricsProvider};

pub fn start_background_jobs(bot: Bot, app_context: AppContext) {
    start_monitor_job(bot.clone(), app_context.clone());

    if app_context.config.anomaly_journal.enabled {
        start_maintenance_job(app_context.clone());
    }

    if app_context.config.daily_summary.enabled {
        start_daily_summary_job(bot, app_context);
    }
}

fn start_monitor_job(bot: Bot, app_context: AppContext) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(app_context.config.monitor_interval));
        let mut metrics_provider = RealMetricsProvider::new();

        loop {
            ticker.tick().await;
            {
                let mut tick = app_context.last_monitor_tick.lock().await;
                *tick = Some(Utc::now());
            }

            check_alerts(
                &bot,
                &app_context.config,
                &app_context.alert_state,
                &mut metrics_provider,
            )
            .await;
        }
    });
}

fn start_daily_summary_job(bot: Bot, app_context: AppContext) {
    tokio::spawn(async move {
        loop {
            let wait = duration_until_next_daily_summary(
                app_context.config.daily_summary.hour_utc,
                app_context.config.daily_summary.minute_utc,
            );
            sleep(wait).await;

            let report = take_daily_summary_report(&app_context.alert_state).await;
            let message = format_daily_summary_message(report);
            let owner_chat_id = match app_context.config.owner_chat_id() {
                Ok(chat_id) => chat_id,
                Err(error) => {
                    log::error!("daily summary skipped: invalid owner chat id: {}", error);
                    continue;
                }
            };

            if let Err(error) = bot.send_message(owner_chat_id, message).await {
                log::error!("failed to send daily summary: {}", error);
            }
        }
    });
}

fn start_maintenance_job(app_context: AppContext) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(3600));

        loop {
            ticker.tick().await;
            run_maintenance(&app_context.config);
        }
    });
}

fn duration_until_next_daily_summary(hour_utc: u8, minute_utc: u8) -> Duration {
    let now = Utc::now();

    let today = now.date_naive();
    let Some(scheduled_today_naive) = today.and_hms_opt(hour_utc as u32, minute_utc as u32, 0)
    else {
        return Duration::from_secs(60);
    };

    let mut scheduled = Utc.from_utc_datetime(&scheduled_today_naive);
    if scheduled <= now {
        let tomorrow = today.checked_add_days(Days::new(1)).unwrap_or(today);
        let Some(scheduled_tomorrow_naive) = tomorrow.and_hms_opt(hour_utc as u32, minute_utc as u32, 0)
        else {
            return Duration::from_secs(60);
        };
        scheduled = Utc.from_utc_datetime(&scheduled_tomorrow_naive);
    }

    (scheduled - now)
        .to_std()
        .unwrap_or_else(|_| Duration::from_secs(60))
}

fn format_daily_summary_message(report: Option<DailySummaryReport>) -> String {
    match report {
        Some(report) => format!(
            "📅 Daily Summary\n\nSamples: {}\nAlerts triggered: {}\n\nCPU avg/min/max: {:.1}% / {:.1}% / {:.1}%\nRAM avg/min/max: {:.1}% / {:.1}% / {:.1}%\nDisk avg/min/max: {:.1}% / {:.1}% / {:.1}%\n\nGenerated at (UTC): {}",
            report.sample_count,
            report.alert_count,
            report.cpu_avg,
            report.cpu_min,
            report.cpu_max,
            report.ram_avg,
            report.ram_min,
            report.ram_max,
            report.disk_avg,
            report.disk_min,
            report.disk_max,
            report.generated_at.to_rfc3339(),
        ),
        None => format!(
            "📅 Daily Summary\n\nNo monitoring samples were collected since the last summary window.\nGenerated at (UTC): {}",
            Utc::now().to_rfc3339()
        ),
    }
}