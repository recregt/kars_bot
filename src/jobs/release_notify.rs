use std::{fs, path::Path, path::PathBuf};

use crate::monitor::Notifier;

use chrono::Utc;
use serde_json::json;
use teloxide::prelude::*;
use tokio::time::{Duration, sleep};

use crate::app_context::AppContext;
use crate::release_notes::release_notes_for_version;

pub(super) fn start_release_notify_job(bot: Bot, app_context: AppContext) {
    if !app_context.config.release_notifier.enabled {
        return;
    }

    let notifier = crate::monitor::TeloxideNotifier(bot.clone());

    tokio::spawn(async move {
        perform_release_notify(&notifier, &app_context.config).await;
    });
}

// separated logic so it can be called from tests with a spy notifier
async fn perform_release_notify<N: Notifier>(notifier: &N, config: &crate::config::Config) {
    sleep(Duration::from_secs(5)).await;

    let version = env!("CARGO_PKG_VERSION");
    let state_path = PathBuf::from(&config.release_notifier.state_path);

    if read_notified_version(&state_path).as_deref() == Some(version) {
        return;
    }

    let notes = release_notes_for_version(&config.release_notifier.changelog_path, version)
        .unwrap_or_else(|| "No changelog notes found for this release.".to_string());

    let owner_chat_id = match config.owner_chat_id() {
        Ok(chat_id) => chat_id,
        Err(error) => {
            log::error!("release notify skipped: invalid owner chat id: {}", error);
            return;
        }
    };

    let message = format!(
        "🚀 Deploy Notification\n\nVersion: v{}\n\n{}",
        version, notes
    );

    if let Err(error) = notifier.send_message(owner_chat_id, message).await {
        log::warn!("release notify send failed: {}", error);
        return;
    }

    if let Err(error) = write_notified_version(&state_path, version) {
        log::warn!("release notify state write failed: {}", error);
    }
}

fn read_notified_version(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<serde_json::Value>(&content)
        .ok()?
        .get("last_notified_version")?
        .as_str()
        .map(|s| s.to_string())
}

fn write_notified_version(path: &Path, version: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = json!({
        "last_notified_version": version,
        "updated_at_utc": Utc::now().to_rfc3339(),
    });

    fs::write(path, payload.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::{SentItem, SpyNotifier};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn perform_release_notify_writes_state_and_sends_message() {
        // create temporary changelog and state path
        let dir = tempdir().expect("temp dir");
        let changelog = dir.path().join("CHANGELOG.md");
        fs::write(&changelog, "### v1.0.0\n- test").unwrap();

        let config = crate::config::Config {
            bot_token: "tok".to_string(),
            owner_id: 1,
            monitor_interval: 1,
            command_timeout_secs: 1,
            alerts: Default::default(),
            daily_summary: Default::default(),
            weekly_report: Default::default(),
            graph: Default::default(),
            anomaly_db: Default::default(),
            simulation: Default::default(),
            reporting_store: Default::default(),
            release_notifier: crate::config::ReleaseNotifierConfig {
                enabled: true,
                changelog_path: changelog.to_string_lossy().to_string(),
                state_path: dir.path().join("state.json").to_string_lossy().to_string(),
            },
            security: Default::default(),
        };

        let notifier = SpyNotifier::new();
        perform_release_notify(&notifier, &config).await;

        // check that a message was recorded and state file exists
        let sent = notifier.sent.lock().await;
        assert_eq!(sent.len(), 1);
        match &sent[0] {
            SentItem::Message(_, text) => assert!(text.contains("Deploy Notification")),
            _ => panic!("expected message"),
        }
        assert!(fs::metadata(&config.release_notifier.state_path).is_ok());
    }
}
