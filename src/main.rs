mod anomaly_db;
mod app_builder;
mod app_context;
mod architecture;
mod bot_runtime;
mod capabilities;
mod commands;
mod config;
mod contracts;
mod jobs;
mod monitor;
mod monitor_context;
mod release_notes;
mod reporting_store;
mod system;
#[cfg(test)]
mod test_utils;

use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use tokio::signal;
use tracing_subscriber::EnvFilter;

use crate::app_builder::build_runtime;
use crate::commands::{MyCommands, answer, answer_callback};

fn init_json_logging() {
    if let Err(error) = tracing_log::LogTracer::init() {
        eprintln!(
            "logging bridge initialization failed (continuing with existing logger): {error}"
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
        eprintln!("global logger initialization failed: {error}");
    }
}

const CONFIG_PATH: &str = "config.toml";

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal as unix_signal};

        let mut terminate = match unix_signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(error) => {
                log::warn!("failed_to_bind_sigterm_handler error={error}");
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

#[tokio::main]
async fn main() {
    init_json_logging();
    log::info!("Kars Server Bot is starting...");

    let runtime = match build_runtime(CONFIG_PATH, 2).await {
        Ok(runtime) => runtime,
        Err(error) => {
            log::error!("{error}");
            return;
        }
    };
    let bot = runtime.bot;
    let app_context = runtime.app_context;

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<MyCommands>()
                .endpoint({
                    let app_context = app_context.clone();
                    move |bot: Bot, msg: Message, cmd: MyCommands| {
                        let app_context = app_context.clone();
                        async move { answer(bot, msg, cmd, &app_context).await }
                    }
                }),
        )
        .branch(Update::filter_callback_query().endpoint({
            let app_context = app_context.clone();
            move |bot: Bot, q: CallbackQuery| {
                let app_context = app_context.clone();
                async move { answer_callback(bot, q, app_context).await }
            }
        }));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build();

    tokio::select! {
        () = dispatcher.dispatch() => {
            log::info!("bot_dispatcher_stopped");
        }
        () = wait_for_shutdown_signal() => {
            log::warn!("bot_shutdown_sequence_started reason=signal");
        }
    }
}
