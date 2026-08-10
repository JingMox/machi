//! Persistence ports for conversation snapshots (W4.4 incremental append).

use async_trait::async_trait;
use machi_types::{MachiError, Message};

use crate::handle::ChatStateSnapshot;

/// Host-provided persistence backend.
#[async_trait]
pub trait ChatPersistence: Send + Sync {
    /// Persist a full snapshot.
    async fn save(&self, snapshot: &ChatStateSnapshot) -> Result<(), MachiError>;
    /// Load the latest snapshot when present.
    async fn load(&self) -> Result<Option<ChatStateSnapshot>, MachiError>;
    /// Append a single message (incremental). Default: load → push → save.
    ///
    /// # Errors
    ///
    /// Backend I/O failures.
    async fn persist_message(&self, message: &Message) -> Result<(), MachiError> {
        let mut snap = self.load().await?.unwrap_or_else(|| messages_only(vec![]));
        snap.messages.push(message.clone());
        if message.role == machi_types::Role::User {
            snap.prompt_index
                .push(snap.messages.len().saturating_sub(1));
        }
        self.save(&snap).await
    }
}

/// No-op persistence.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullPersistence;

#[async_trait]
impl ChatPersistence for NullPersistence {
    async fn save(&self, _snapshot: &ChatStateSnapshot) -> Result<(), MachiError> {
        Ok(())
    }
    async fn load(&self) -> Result<Option<ChatStateSnapshot>, MachiError> {
        Ok(None)
    }
    async fn persist_message(&self, _message: &Message) -> Result<(), MachiError> {
        Ok(())
    }
}

/// In-memory persistence for tests.
#[derive(Debug, Default)]
pub struct MemoryPersistence {
    slot: tokio::sync::Mutex<Option<ChatStateSnapshot>>,
}

#[async_trait]
impl ChatPersistence for MemoryPersistence {
    async fn save(&self, snapshot: &ChatStateSnapshot) -> Result<(), MachiError> {
        *self.slot.lock().await = Some(snapshot.clone());
        Ok(())
    }
    async fn load(&self) -> Result<Option<ChatStateSnapshot>, MachiError> {
        Ok(self.slot.lock().await.clone())
    }
}

/// Helper: messages-only snapshot body.
#[must_use]
pub fn messages_only(messages: Vec<Message>) -> ChatStateSnapshot {
    let prompt_index = messages
        .iter()
        .enumerate()
        .filter_map(|(i, m)| (m.role == machi_types::Role::User).then_some(i))
        .collect();
    ChatStateSnapshot {
        messages,
        usage: crate::ledger::UsageLedger::default(),
        prompt_index,
    }
}
