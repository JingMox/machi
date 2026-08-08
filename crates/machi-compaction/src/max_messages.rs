//! Keep at most N messages, preserving leading system and tail.

use machi_types::{ErrorCode, MachiError, Message, Role};

use crate::strategy::{CompactionOutcome, CompactionStrategy};

/// Drop oldest non-system messages until `max` remains.
#[derive(Debug, Clone, Copy)]
pub struct MaxMessages {
    /// Maximum messages retained (including system).
    pub max: usize,
}

impl MaxMessages {
    /// Construct with a positive max.
    ///
    /// # Errors
    ///
    /// Returns error when `max == 0`.
    pub fn new(max: usize) -> Result<Self, MachiError> {
        if max == 0 {
            return Err(MachiError::new(
                ErrorCode::CompactionFailed,
                "MaxMessages max must be >= 1",
            ));
        }
        Ok(Self { max })
    }
}

impl CompactionStrategy for MaxMessages {
    fn name(&self) -> &'static str {
        "max_messages"
    }

    fn should_compact(&self, messages: &[Message], _token_estimate: u64) -> bool {
        messages.len() > self.max
    }

    fn compact(&self, messages: Vec<Message>) -> Result<CompactionOutcome, MachiError> {
        if messages.len() <= self.max {
            return Ok(CompactionOutcome {
                messages,
                changed: false,
                strategy: self.name(),
            });
        }
        let compacted = compact_max_messages(messages, self.max);
        Ok(CompactionOutcome {
            messages: compacted,
            changed: true,
            strategy: self.name(),
        })
    }
}

/// Shared algorithm used by runtime `VecConversationState` and this strategy.
#[must_use]
pub fn compact_max_messages(messages: Vec<Message>, max_messages: usize) -> Vec<Message> {
    if max_messages == 0 || messages.len() <= max_messages {
        return messages;
    }
    let (system, rest) = split_leading_system(&messages);
    let keep_rest = max_messages.saturating_sub(usize::from(system.is_some()));
    if rest.len() <= keep_rest {
        return messages;
    }
    let mut start = rest.len().saturating_sub(keep_rest);
    while start < rest.len() {
        let Some(msg) = rest.get(start) else {
            break;
        };
        if msg.role == Role::Tool && start > 0 {
            start = start.saturating_sub(1);
            continue;
        }
        break;
    }
    let mut out = Vec::with_capacity(max_messages);
    if let Some(sys) = system {
        out.push(sys);
    }
    out.extend(rest.into_iter().skip(start));
    out
}

fn split_leading_system(messages: &[Message]) -> (Option<Message>, Vec<Message>) {
    match messages.split_first() {
        Some((first, rest)) if first.role == Role::System => (Some(first.clone()), rest.to_vec()),
        _ => (None, messages.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_system_and_tail() {
        let s = MaxMessages::new(3).expect("max");
        let out = s
            .compact(vec![
                Message::system("sys"),
                Message::user("1"),
                Message::user("2"),
                Message::user("3"),
                Message::user("4"),
            ])
            .expect("compact");
        assert!(out.changed);
        assert_eq!(out.messages.len(), 3);
        assert_eq!(
            out.messages.first().map(Message::text).as_deref(),
            Some("sys")
        );
        assert_eq!(out.messages.get(2).map(Message::text).as_deref(), Some("4"));
    }
}
