use std::path::Path;

use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::app_context::AppContext;
use crate::config::load_config;

async fn apply_runtime_reload_from_path(
    app_context: &AppContext,
    config_path: &str,
) -> Result<crate::config::RuntimeConfig, String> {
    let new_config = load_config(config_path).map_err(|error| error.to_string())?;
    let runtime_config = crate::config::RuntimeConfig::from_config(&new_config);
    app_context.update_runtime_config(runtime_config.clone()).await;
    Ok(runtime_config)
}

pub(super) fn start_config_hot_reload_job(app_context: AppContext) {
    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let config_path = app_context.config_path.clone();
        let mut watcher = match RecommendedWatcher::new(
            move |result| {
                let _ = tx.send(result);
            },
            NotifyConfig::default(),
        ) {
            Ok(watcher) => watcher,
            Err(error) => {
                log::warn!("config hot-reload disabled: watcher init failed: {}", error);
                return;
            }
        };

        if let Err(error) = watcher.watch(Path::new(config_path.as_str()), RecursiveMode::NonRecursive)
        {
            log::warn!(
                "config hot-reload disabled: failed to watch {}: {}",
                config_path,
                error
            );
            return;
        }

        while let Some(event_result) = rx.recv().await {
            let event = match event_result {
                Ok(event) => event,
                Err(error) => {
                    log::warn!("config hot-reload event error: {}", error);
                    continue;
                }
            };

            let should_reload = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any
            );
            if !should_reload {
                continue;
            }

            match apply_runtime_reload_from_path(&app_context, config_path.as_str()).await {
                Ok(runtime_config) => {

                    let graph = runtime_config.graph;
                    log::info!(
                        "config_hot_reload_applied target=runtime alerts_cpu={} alerts_ram={} alerts_disk={} monitor_interval={} command_timeout_secs={} graph_enabled={} default_window_minutes={} max_window_hours={} max_points={}",
                        runtime_config.alerts.cpu,
                        runtime_config.alerts.ram,
                        runtime_config.alerts.disk,
                        runtime_config.monitor_interval,
                        runtime_config.command_timeout_secs,
                        graph.enabled,
                        graph.default_window_minutes,
                        graph.max_window_hours,
                        graph.max_points,
                    );
                }
                Err(error) => {
                    log::warn!("config hot-reload ignored invalid config: {}", error);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests;
