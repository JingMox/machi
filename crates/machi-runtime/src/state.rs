//! Conversation state abstractions.

use machi_types::Message;

/// Mutable conversation backing a turn or session.
pub trait ConversationState: Send {
    /// Immutable view of messages.
    fn messages(&self) -> &[Message];
    /// Append a message.
    fn append(&mut self, message: Message);
    /// Rough token estimate for compaction triggers (bytes/4 heuristic by default).
    fn token_estimate(&self) -> u64 {
        self.messages()
            .iter()
            .map(|m| (m.text().len() as u64) / 4 + 1)
            .sum()
    }
}

/// In-memory conversation state.
#[derive(Debug, Clone, Default)]
pub struct VecConversationState {
    messages: Vec<Message>,
}

impl VecConversationState {
    /// Empty state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed with messages.
    #[must_use]
    pub fn from_messages(messages: Vec<Message>) -> Self {
        Self { messages }
    }
}

impl ConversationState for VecConversationState {
    fn messages(&self) -> &[Message] {
        &self.messages
    }

    fn append(&mut self, message: Message) {
        self.messages.push(message);
    }
}
