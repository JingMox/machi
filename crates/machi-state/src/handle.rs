//! Actor-backed conversation handle.

use machi_types::{MachiError, Message};
use tokio::sync::{mpsc, oneshot};

use crate::ledger::UsageLedger;
use crate::strict::{StrictAppendError, check_append};

/// Serializable conversation snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatStateSnapshot {
    /// Ordered messages.
    pub messages: Vec<Message>,
    /// Usage ledger.
    pub usage: UsageLedger,
}

enum Command {
    Append {
        message: Message,
        strict: bool,
        reply: oneshot::Sender<Result<(), MachiError>>,
    },
    Replace {
        messages: Vec<Message>,
        reply: oneshot::Sender<()>,
    },
    Snapshot {
        reply: oneshot::Sender<ChatStateSnapshot>,
    },
    RecordUsage {
        usage: machi_types::Usage,
        subagent: bool,
        reply: oneshot::Sender<()>,
    },
    MarkIncomplete {
        reply: oneshot::Sender<()>,
    },
    Shutdown {
        reply: oneshot::Sender<()>,
    },
}

/// Cloneable handle to a single-writer conversation actor.
#[derive(Clone)]
pub struct ChatStateHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl std::fmt::Debug for ChatStateHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatStateHandle").finish_non_exhaustive()
    }
}

impl ChatStateHandle {
    /// Spawn an actor with optional seed messages.
    #[must_use]
    pub fn spawn(seed: Vec<Message>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(actor_loop(rx, seed, UsageLedger::new()));
        Self { tx }
    }

    /// Append a message (strict tool pairing enforced).
    ///
    /// # Errors
    ///
    /// Returns invariant or actor-channel failures.
    pub async fn append(&self, message: Message) -> Result<(), MachiError> {
        self.append_inner(message, true).await
    }

    /// Append without strict pairing (escape hatch for repair paths).
    ///
    /// # Errors
    ///
    /// Channel failures only.
    pub async fn append_unchecked(&self, message: Message) -> Result<(), MachiError> {
        self.append_inner(message, false).await
    }

    async fn append_inner(&self, message: Message, strict: bool) -> Result<(), MachiError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Command::Append {
                message,
                strict,
                reply,
            })
            .map_err(|_| actor_gone())?;
        rx.await.map_err(|_| actor_gone())?
    }

    /// Replace full history (compaction).
    pub async fn replace(&self, messages: Vec<Message>) {
        let (reply, rx) = oneshot::channel();
        if self.tx.send(Command::Replace { messages, reply }).is_ok() {
            let _ = rx.await;
        }
    }

    /// Snapshot messages + usage.
    #[must_use]
    pub async fn snapshot(&self) -> ChatStateSnapshot {
        let (reply, rx) = oneshot::channel();
        if self.tx.send(Command::Snapshot { reply }).is_err() {
            return ChatStateSnapshot {
                messages: Vec::new(),
                usage: UsageLedger::default(),
            };
        }
        rx.await.unwrap_or(ChatStateSnapshot {
            messages: Vec::new(),
            usage: UsageLedger::default(),
        })
    }

    /// Messages only (convenience).
    #[must_use]
    pub async fn messages(&self) -> Vec<Message> {
        self.snapshot().await.messages
    }

    /// Message count.
    #[must_use]
    pub async fn len(&self) -> usize {
        self.messages().await.len()
    }

    /// True when conversation is empty.
    #[must_use]
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Usage ledger snapshot.
    #[must_use]
    pub async fn usage(&self) -> UsageLedger {
        self.snapshot().await.usage
    }

    /// Persist via a [`crate::persistence::ChatPersistence`] backend.
    ///
    /// # Errors
    ///
    /// Backend I/O failures.
    pub async fn save_to(
        &self,
        store: &dyn crate::persistence::ChatPersistence,
    ) -> Result<(), MachiError> {
        let snap = self.snapshot().await;
        store.save(&snap).await
    }

    /// Replace state from a persistence backend when present.
    ///
    /// # Errors
    ///
    /// Backend I/O failures.
    pub async fn load_from(
        &self,
        store: &dyn crate::persistence::ChatPersistence,
    ) -> Result<bool, MachiError> {
        match store.load().await? {
            Some(snap) => {
                self.replace(snap.messages).await;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Record main-loop usage.
    pub async fn record_main_usage(&self, usage: machi_types::Usage) {
        let (reply, rx) = oneshot::channel();
        if self
            .tx
            .send(Command::RecordUsage {
                usage,
                subagent: false,
                reply,
            })
            .is_ok()
        {
            let _ = rx.await;
        }
    }

    /// Record nested agent usage.
    pub async fn record_subagent_usage(&self, usage: machi_types::Usage) {
        let (reply, rx) = oneshot::channel();
        if self
            .tx
            .send(Command::RecordUsage {
                usage,
                subagent: true,
                reply,
            })
            .is_ok()
        {
            let _ = rx.await;
        }
    }

    /// Mark usage incomplete.
    pub async fn mark_incomplete(&self) {
        let (reply, rx) = oneshot::channel();
        if self.tx.send(Command::MarkIncomplete { reply }).is_ok() {
            let _ = rx.await;
        }
    }

    /// Stop the actor.
    pub async fn shutdown(self) {
        let (reply, rx) = oneshot::channel();
        if self.tx.send(Command::Shutdown { reply }).is_ok() {
            let _ = rx.await;
        }
    }
}

fn actor_gone() -> MachiError {
    MachiError::new(
        machi_types::ErrorCode::StatePersistence,
        "chat state actor is gone",
    )
}

fn map_strict(err: StrictAppendError) -> MachiError {
    MachiError::new(
        machi_types::ErrorCode::StateInvariant,
        err.to_string(),
    )
}

async fn actor_loop(
    mut rx: mpsc::UnboundedReceiver<Command>,
    mut messages: Vec<Message>,
    mut usage: UsageLedger,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Append {
                message,
                strict,
                reply,
            } => {
                let result = if strict {
                    match check_append(&messages, &message) {
                        Ok(()) => {
                            messages.push(message);
                            Ok(())
                        }
                        Err(e) => Err(map_strict(e)),
                    }
                } else {
                    messages.push(message);
                    Ok(())
                };
                let _ = reply.send(result);
            }
            Command::Replace {
                messages: next,
                reply,
            } => {
                messages = next;
                let _ = reply.send(());
            }
            Command::Snapshot { reply } => {
                let _ = reply.send(ChatStateSnapshot {
                    messages: messages.clone(),
                    usage,
                });
            }
            Command::RecordUsage {
                usage: u,
                subagent,
                reply,
            } => {
                if subagent {
                    usage.record_subagent(u);
                } else {
                    usage.record_main(u);
                }
                let _ = reply.send(());
            }
            Command::MarkIncomplete { reply } => {
                usage.mark_incomplete();
                let _ = reply.send(());
            }
            Command::Shutdown { reply } => {
                let _ = reply.send(());
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use machi_types::{ToolCall, ToolCallId};
    use serde_json::json;

    #[tokio::test]
    async fn strict_blocks_dangling_result() {
        let h = ChatStateHandle::spawn(vec![]);
        let id = ToolCallId::new("x").expect("id");
        let err = h
            .append(Message::tool_result(id, "t", "nope"))
            .await
            .expect_err("dangling");
        assert_eq!(err.code(), machi_types::ErrorCode::StateInvariant);
        h.shutdown().await;
    }

    #[tokio::test]
    async fn accepts_paired_flow() {
        let h = ChatStateHandle::spawn(vec![Message::user("hi")]);
        let id = ToolCallId::new("c1").expect("id");
        h.append(Message::assistant_tools(vec![ToolCall {
            id: id.clone(),
            name: "t".into(),
            arguments: json!({}),
        }]))
        .await
        .expect("assistant");
        h.append(Message::tool_result(id, "t", "ok"))
            .await
            .expect("result");
        assert_eq!(h.messages().await.len(), 3);
        h.shutdown().await;
    }
}
