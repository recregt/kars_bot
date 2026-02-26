use async_trait::async_trait;
use teloxide::prelude::*;
use teloxide::types::InputFile;

/// General messaging interface for the bot.  Provides both text and
/// photo/graphic delivery so that scheduled reports, release notices and
/// numeric alerts all share the same abstraction.
#[async_trait]
pub trait Notifier: Send + Sync {
    /// send a plain-text message
    async fn send_message(&self, chat_id: ChatId, text: String) -> Result<(), String>;

    /// send a photo with given bytes, filename and caption
    async fn send_photo(
        &self,
        chat_id: ChatId,
        bytes: Vec<u8>,
        file_name: String,
        caption: String,
    ) -> Result<(), String>;
}

/// adapter that delegates to a live `teloxide::Bot`.
pub struct TeloxideNotifier(pub Bot);

#[async_trait]
impl Notifier for TeloxideNotifier {
    async fn send_message(&self, chat_id: ChatId, text: String) -> Result<(), String> {
        self.0
            .send_message(chat_id, text)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn send_photo(
        &self,
        chat_id: ChatId,
        bytes: Vec<u8>,
        file_name: String,
        caption: String,
    ) -> Result<(), String> {
        self.0
            .send_photo(chat_id, InputFile::memory(bytes).file_name(file_name))
            .caption(caption)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
/// Spy implementation records all outgoing messages/photos for assertions.
pub struct SpyNotifier {
    pub sent: tokio::sync::Mutex<Vec<SentItem>>,
}

#[cfg(test)]
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SentItem {
    Message(ChatId, String),
    Photo(ChatId, Vec<u8>, String, String),
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
impl Notifier for SpyNotifier {
    async fn send_message(&self, chat_id: ChatId, text: String) -> Result<(), String> {
        let mut guard = self.sent.lock().await;
        guard.push(SentItem::Message(chat_id, text));
        Ok(())
    }

    async fn send_photo(
        &self,
        chat_id: ChatId,
        bytes: Vec<u8>,
        file_name: String,
        caption: String,
    ) -> Result<(), String> {
        let mut guard = self.sent.lock().await;
        guard.push(SentItem::Photo(chat_id, bytes, file_name, caption));
        Ok(())
    }
}
