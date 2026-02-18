mod anomaly_db;
mod app_context;
mod capabilities;
mod commands;
mod config;
mod jobs;
mod monitor;
mod release_notes;
mod reporting_store;
mod system;

use teloxide::prelude::*;
use tokio::net::lookup_host;
use tokio::signal;
use tracing_subscriber::EnvFilter;

use crate::app_context::AppContext;
use crate::capabilities::Capabilities;
use crate::commands::{MyCommands, answer, check_graph_render_readiness};
use crate::config::{Config, load_config};
use crate::jobs::start_background_jobs;

fn init_json_logging() {
    if let Err(error) = tracing_log::LogTracer::init() {
        eprintln!(
            "logging bridge initialization failed (continuing with existing logger): {}",
            error
        );
    }

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .finish();

    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("global logger initialization failed: {}", error);
    }
}

const CONFIG_PATH: &str = "config.toml";

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
                log::info!("dns_probe_ok host=api.telegram.org address={}", address);
            } else {
                log::warn!("dns_probe_degraded host=api.telegram.org reason=no_records");
            }
        }
        Err(error) => {
            log::warn!(
                "dns_probe_degraded host=api.telegram.org reason=lookup_failed error={}",
                error
            );
        }
    }
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal as unix_signal};

        let mut terminate = match unix_signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(error) => {
                log::warn!("failed_to_bind_sigterm_handler error={}", error);
                let _ = signal::ctrl_c().await;
                return;
            }
        };

        tokio::select! {
            _ = signal::ctrl_c() => {
                log::warn!("shutdown_signal_received signal=SIGINT");
            }
            _ = terminate.recv() => {
                log::warn!("shutdown_signal_received signal=SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = signal::ctrl_c().await;
        log::warn!("shutdown_signal_received signal=CTRL_C");
    }
}

// Main (cache benchmark touch #1)
#[tokio::main]
async fn main() {
    init_json_logging();

    let config: Config = match load_config(CONFIG_PATH) {
        Ok(config) => config,
        Err(error) => {
            log::error!("Configuration error: {}", error);
            return;
        }
    };

    if let Err(error) = config.validate() {
        log::error!("Configuration validation failed: {}", error);
        return;
    }

    log::info!("Kars Server Bot is starting...");
    let capabilities = Capabilities::detect();
    log_capability_warnings(&capabilities);
    log_dns_probe().await;

    let bot = Bot::new(&config.bot_token);

    let app_context = AppContext::new(config.clone(), 2, CONFIG_PATH, capabilities);

    if app_context.graph_runtime.read().await.enabled
        && let Err(error) = check_graph_render_readiness()
    {
        log::warn!(
            "graph_startup_degraded action=disable_graph_feature reason={}",
            error
        );

        {
            let mut graph_runtime = app_context.graph_runtime.write().await;
            graph_runtime.enabled = false;
        }

        {
            let mut runtime_config = app_context.runtime_config.write().await;
            runtime_config.graph.enabled = false;
        }
    }

    start_background_jobs(bot.clone(), app_context.clone());

    let repl = MyCommands::repl(bot, move |bot, msg, cmd| {
        let app_context = app_context.clone();
        async move { answer(bot, msg, cmd, &app_context).await }
    });

    tokio::pin!(repl);
    tokio::select! {
        _ = &mut repl => {
            log::info!("bot_repl_stopped reason=polling_ended");
        }
        _ = wait_for_shutdown_signal() => {
            log::warn!("bot_shutdown_sequence_started reason=signal");
        }
    }
}
