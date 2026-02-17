use chrono::{Datelike, Days, TimeZone, Utc};
use teloxide::{prelude::*, types::InputFile};
use tokio::time::{interval, sleep, Duration};

use crate::anomaly_db::run_maintenance;
use crate::app_context::AppContext;
use crate::commands::build_weekly_cpu_report;
use crate::monitor::{take_daily_summary_report, DailySummaryReport};

pub(super) fn start_daily_summary_job(bot: Bot, app_context: AppContext) {
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

pub(super) fn start_maintenance_job(app_context: AppContext) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(3600));

        loop {
            ticker.tick().await;
            run_maintenance(&app_context.config);
        }
    });
}

pub(super) fn start_weekly_report_job(bot: Bot, app_context: AppContext) {
    tokio::spawn(async move {
        loop {
            let wait = duration_until_next_weekly_report(
                app_context.config.weekly_report.weekday_utc,
                app_context.config.weekly_report.hour_utc,
                app_context.config.weekly_report.minute_utc,
            );
            sleep(wait).await;

            let owner_chat_id = match app_context.config.owner_chat_id() {
                Ok(chat_id) => chat_id,
                Err(error) => {
                    log::error!("weekly report skipped: invalid owner chat id: {}", error);
                    continue;
                }
            };

            match build_weekly_cpu_report(&app_context).await {
                Ok(report) => {
                    if let Err(error) = bot
                        .send_photo(
                            owner_chat_id,
                            InputFile::memory(report.png_bytes).file_name(report.file_name),
                        )
                        .caption(report.caption)
                        .await
                    {
                        log::error!("failed to send weekly report chart: {}", error);
                    }
                }
                Err(error) => {
                    log::warn!("weekly report skipped: {}", error);
                    if let Err(send_error) = bot
                        .send_message(
                            owner_chat_id,
                            format!(
                                "📈 Weekly Report\n\nCould not generate chart this cycle: {}",
                                error
                            ),
                        )
                        .await
                    {
                        log::error!("failed to send weekly report fallback: {}", send_error);
                    }
                }
            }
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

fn duration_until_next_weekly_report(weekday_utc: u8, hour_utc: u8, minute_utc: u8) -> Duration {
    let now = Utc::now();
    let today = now.date_naive();

    let current_weekday = now.weekday().num_days_from_monday() as i64;
    let target_weekday = (weekday_utc.saturating_sub(1)) as i64;

    let mut days_ahead = (target_weekday - current_weekday).rem_euclid(7);
    let scheduled_today = match today.and_hms_opt(hour_utc as u32, minute_utc as u32, 0) {
        Some(value) => Utc.from_utc_datetime(&value),
        None => return Duration::from_secs(60),
    };

    if days_ahead == 0 && scheduled_today <= now {
        days_ahead = 7;
    }

    let target_date = match today.checked_add_days(Days::new(days_ahead as u64)) {
        Some(value) => value,
        None => return Duration::from_secs(60),
    };

    let scheduled = match target_date.and_hms_opt(hour_utc as u32, minute_utc as u32, 0) {
        Some(value) => Utc.from_utc_datetime(&value),
        None => return Duration::from_secs(60),
    };

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
