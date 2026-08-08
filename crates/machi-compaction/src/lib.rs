//! Conversation compaction strategies.
//!
//! Strategies are pure transforms over message lists. Runtime hosts decide
//! *when* to compact; this crate decides *how*.

#![forbid(unsafe_code)]

pub mod max_messages;
pub mod strategy;
pub mod token_threshold;

pub use max_messages::MaxMessages;
pub use strategy::{CompactionOutcome, CompactionStrategy};
pub use token_threshold::TokenThreshold;
