use std::{fs, path::Path, path::PathBuf};

use chrono::Utc;
use serde_json::json;
use teloxide::prelude::*;
use tokio::time::{sleep, Duration};

use crate::app_context::AppContext;
use crate::release_notes::release_notes_for_version;

pub(super) fn start_release_notify_job(bot: Bot, app_context: AppContext) {
    if !app_context.config.release_notifier.enabled {
        return;
    }

    tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;

        let version = env!("CARGO_PKG_VERSION");
        let state_path = PathBuf::from(&app_context.config.release_notifier.state_path);

        if read_notified_version(&state_path).as_deref() == Some(version) {
            return;
        }

        let notes = release_notes_for_version(
            &app_context.config.release_notifier.changelog_path,
            version,
        )
        .unwrap_or_else(|| "No changelog notes found for this release.".to_string());

        let owner_chat_id = match app_context.config.owner_chat_id() {
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

        if let Err(error) = bot.send_message(owner_chat_id, message).await {
            log::warn!("release notify send failed: {}", error);
            return;
        }

        if let Err(error) = write_notified_version(&state_path, version) {
            log::warn!("release notify state write failed: {}", error);
        }
    });
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
