use async_trait::async_trait;
use teloxide::prelude::*;

/// Abstraction over whatever mechanism is used to deliver alert messages.
/// Keeping this separate from `Bot` itself allows tests to inject a spy or
/// fake notifier instead of hitting network.
#[async_trait]
pub trait AlertNotifier: Send + Sync {
    /// send a textual message to given chat id. returns Err(String) on failure.
    async fn send(&self, chat_id: ChatId, text: String) -> Result<(), String>;
}

/// adapter that uses a `teloxide::Bot` under the hood.
pub struct TeloxideNotifier(pub Bot);

#[async_trait]
impl AlertNotifier for TeloxideNotifier {
    async fn send(&self, chat_id: ChatId, text: String) -> Result<(), String> {
        self.0
            .send_message(chat_id, text)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
/// simple spy implementation that records messages instead of sending them.
pub struct SpyNotifier {
    pub sent: tokio::sync::Mutex<Vec<(ChatId, String)>>,
}

#[cfg(test)]
impl SpyNotifier {
    pub fn new() -> Self {
        Self {
            sent: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl AlertNotifier for SpyNotifier {
    async fn send(&self, chat_id: ChatId, text: String) -> Result<(), String> {
        let mut guard = self.sent.lock().await;
        guard.push((chat_id, text));
        Ok(())
    }
}
