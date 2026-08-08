//! Streaming sample events.

use std::pin::Pin;

use futures::Stream;
use machi_types::{Message, Usage};

/// One event in a streaming sample.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SampleEvent {
    /// Incremental assistant text.
    TextDelta {
        /// Delta text.
        text: String,
    },
    /// Full tool-call snapshot (v1: emitted once when known).
    ToolCalls {
        /// Complete assistant message carrying tool calls.
        message: Message,
    },
    /// Usage totals (may arrive at end).
    Usage(Usage),
    /// Stream completed successfully with a final message.
    Completed {
        /// Final assistant message.
        message: Message,
        /// Provider stop reason.
        stop_reason: Option<String>,
    },
    /// Stream failed after partial progress (terminal).
    Failed {
        /// Error message.
        message: String,
    },
}

/// Opaque pinned sample event stream.
pub type SampleStream = Pin<Box<dyn Stream<Item = SampleEvent> + Send>>;
