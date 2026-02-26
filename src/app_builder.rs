use std::sync::Arc;

use teloxide::prelude::Bot;
use tokio::net::lookup_host;

use crate::app_context::AppContext;
use crate::capabilities::Capabilities;
use crate::config::load_config;
use crate::jobs::start_background_jobs;

pub struct AppRuntime {
    pub bot: Bot,
    pub app_context: Arc<AppContext>,
}

pub async fn build_runtime(
    config_path: &str,
    command_concurrency: usize,
) -> Result<AppRuntime, String> {
    let config =
        load_config(config_path).map_err(|error| format!("Configuration error: {error}"))?;
    config
        .validate()
        .map_err(|error| format!("Configuration validation failed: {error}"))?;

    let capabilities = Capabilities::detect();
    log_capability_warnings(&capabilities);
    log_dns_probe().await;

    let bot = Bot::new(&config.bot_token);
    let app_context = Arc::new(AppContext::new(
        config.clone(),
        command_concurrency,
        config_path,
        capabilities,
    ));

    start_background_jobs(bot.clone(), (*app_context).clone());

    Ok(AppRuntime { bot, app_context })
}

fn log_capability_warnings(capabilities: &Capabilities) {
    if !capabilities.is_systemd {
        log::warn!(
            "capability_degraded feature=systemd_services reason=systemctl_or_systemd_unavailable"
        );
    }
    if !capabilities.has_sensors {
        log::warn!("capability_degraded feature=temperature reason=sensors_unavailable");
    }
    if !capabilities.has_ss {
        log::warn!("capability_degraded feature=ports reason=ss_unavailable");
    }
    if !capabilities.has_ip {
        log::warn!("capability_degraded feature=network reason=ip_unavailable");
    }
    if !capabilities.has_free {
        log::warn!("capability_degraded feature=sysstatus_ram reason=free_unavailable");
    }
    if !capabilities.has_top {
        log::warn!("capability_degraded feature=cpu reason=top_unavailable");
    }
    if !capabilities.has_uptime {
        log::warn!("capability_degraded feature=uptime reason=uptime_unavailable");
    }
}

async fn log_dns_probe() {
    match lookup_host(("api.telegram.org", 443)).await {
        Ok(mut addresses) => {
            if let Some(address) = addresses.next() {
                log::info!("dns_probe_ok host=api.telegram.org address={address}");
            } else {
                log::warn!("dns_probe_degraded host=api.telegram.org reason=no_records");
            }
        }
        Err(error) => {
            log::warn!(
                "dns_probe_degraded host=api.telegram.org reason=lookup_failed error={error}"
            );
        }
    }
}
