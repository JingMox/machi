//! Persistence ports for conversation snapshots.

use async_trait::async_trait;
use machi_types::{MachiError, Message};

use crate::handle::ChatStateSnapshot;

/// Host-provided persistence backend.
#[async_trait]
pub trait ChatPersistence: Send + Sync {
    /// Persist a snapshot.
    async fn save(&self, snapshot: &ChatStateSnapshot) -> Result<(), MachiError>;
    /// Load the latest snapshot when present.
    async fn load(&self) -> Result<Option<ChatStateSnapshot>, MachiError>;
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
    ChatStateSnapshot {
        messages,
        usage: crate::ledger::UsageLedger::default(),
    }
}
