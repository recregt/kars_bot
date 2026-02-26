use std::{sync::Arc, time::Instant};

use tokio::sync::{Mutex, Semaphore};

#[derive(Clone)]
pub struct BotRuntime {
    pub command_slots: Arc<Semaphore>,
    pub graph_render_slots: Arc<Semaphore>,
    pub last_graph_command_at: Arc<Mutex<Option<Instant>>>,
}

impl BotRuntime {
    pub fn new(command_concurrency: usize) -> Self {
        Self {
            command_slots: Arc::new(Semaphore::new(command_concurrency)),
            graph_render_slots: Arc::new(Semaphore::new(1)),
            last_graph_command_at: Arc::new(Mutex::new(None)),
        }
    }
}
