//! Conversation state abstractions.

use machi_compaction::max_messages::compact_max_messages;
use machi_types::Message;

/// Mutable conversation backing a turn or session.
pub trait ConversationState: Send {
    /// Immutable view of messages.
    fn messages(&self) -> &[Message];
    /// Append a message.
    fn append(&mut self, message: Message);
    /// Replace the entire message list (compaction / restore).
    fn replace(&mut self, messages: Vec<Message>);
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

    /// Drop oldest non-system messages until `max_messages` remains.
    ///
    /// Delegates to [`machi_compaction::max_messages::compact_max_messages`].
    pub fn compact_max_messages(&mut self, max_messages: usize) {
        self.messages = compact_max_messages(std::mem::take(&mut self.messages), max_messages);
    }
}

impl ConversationState for VecConversationState {
    fn messages(&self) -> &[Message] {
        &self.messages
    }

    fn append(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn replace(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }
}

#[cfg(test)]
mod tests {
    use machi_types::Message;

    use super::*;

    #[test]
    fn max_messages_keeps_system_and_tail() {
        let mut state = VecConversationState::from_messages(vec![
            Message::system("sys"),
            Message::user("1"),
            Message::user("2"),
            Message::user("3"),
            Message::user("4"),
        ]);
        state.compact_max_messages(3);
        let msgs = state.messages();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs.first().map(Message::text).as_deref(), Some("sys"));
        assert_eq!(msgs.get(1).map(Message::text).as_deref(), Some("3"));
        assert_eq!(msgs.get(2).map(Message::text).as_deref(), Some("4"));
    }
}
