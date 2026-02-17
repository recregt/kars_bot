use teloxide::prelude::*;

use crate::app_context::AppContext;

mod config_reload;
mod monitor;
mod release_notify;
mod schedules;

pub fn start_background_jobs(bot: Bot, app_context: AppContext) {
    monitor::start_monitor_job(bot.clone(), app_context.clone());
    config_reload::start_config_hot_reload_job(app_context.clone());
    release_notify::start_release_notify_job(bot.clone(), app_context.clone());

    if app_context.config.anomaly_db.enabled {
        schedules::start_maintenance_job(app_context.clone());
    }

    if app_context.config.daily_summary.enabled {
        schedules::start_daily_summary_job(bot.clone(), app_context.clone());
    }

    if app_context.config.weekly_report.enabled {
        schedules::start_weekly_report_job(bot, app_context);
    }
}
