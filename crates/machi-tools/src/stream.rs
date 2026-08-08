//! Streaming tool execution protocol.
//!
//! Invariant: a tool stream yields zero or more [`ToolStreamItem::Progress`]
//! items followed by **exactly one** [`ToolStreamItem::Terminal`].

use std::future::Future;
use std::pin::Pin;

use futures::Stream;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ToolError, codes};
use crate::tool::ToolResult;

/// Opaque pinned stream of tool items.
pub type ToolStream = Pin<Box<dyn Stream<Item = ToolStreamItem> + Send>>;

/// One item in a tool stream.
#[derive(Debug)]
pub enum ToolStreamItem {
    /// Intermediate progress (logs, partial stdout, custom payloads).
    Progress(ToolProgress),
    /// Terminal result — always last.
    Terminal(Result<ToolResult, ToolError>),
}

impl ToolStreamItem {
    /// Whether this is the terminal item.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal(_))
    }
}

/// Progress payload shapes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolProgress {
    /// Free-form text chunk.
    Text {
        /// Chunk body.
        text: String,
    },
    /// Tool-defined progress.
    Custom {
        /// Stable producer discriminator.
        subkind: String,
        /// Arbitrary payload.
        payload: Value,
    },
}

impl ToolProgress {
    /// Text progress helper.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

/// Single-item terminal stream from a completed result.
#[must_use]
pub fn terminal_only(result: Result<ToolResult, ToolError>) -> ToolStream {
    Box::pin(stream::once(
        async move { ToolStreamItem::Terminal(result) },
    ))
}

/// Progress items then a terminal future.
pub fn with_progress<I, F, Fut>(progress: I, terminal: F) -> ToolStream
where
    I: IntoIterator<Item = ToolProgress> + Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<ToolResult, ToolError>> + Send + 'static,
{
    let items: Vec<ToolStreamItem> = progress.into_iter().map(ToolStreamItem::Progress).collect();
    let prog = stream::iter(items);
    let term = stream::once(async move { ToolStreamItem::Terminal(terminal().await) });
    Box::pin(prog.chain(term))
}

/// Drain a stream to the terminal result, discarding progress.
///
/// # Errors
///
/// Returns stream protocol error when the stream ends without a terminal item.
pub async fn drain_terminal(stream: ToolStream) -> Result<ToolResult, ToolError> {
    let (_progress, result) = drain_with_progress(stream).await;
    result
}

/// Drain a stream, collecting progress items and the terminal result.
///
/// # Errors
///
/// The terminal `Result` carries tool failures. When the stream ends without a
/// terminal item, returns protocol error in the terminal slot.
pub async fn drain_with_progress(
    mut stream: ToolStream,
) -> (Vec<ToolProgress>, Result<ToolResult, ToolError>) {
    let mut progress = Vec::new();
    while let Some(item) = stream.next().await {
        match item {
            ToolStreamItem::Progress(p) => progress.push(p),
            ToolStreamItem::Terminal(result) => return (progress, result),
        }
    }
    (
        progress,
        Err(codes::stream_protocol(
            "tool stream ended without a terminal item",
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn terminal_only_drains() {
        let s = terminal_only(Ok(ToolResult::text("ok")));
        let r = drain_terminal(s).await.expect("drain");
        assert_eq!(r.content, "ok");
    }

    #[tokio::test]
    async fn progress_then_terminal() {
        let s = with_progress(vec![ToolProgress::text("working")], || async {
            Ok(ToolResult::text("done"))
        });
        let r = drain_terminal(s).await.expect("drain");
        assert_eq!(r.content, "done");
    }

    #[tokio::test]
    async fn drain_with_progress_captures_chunks() {
        let s = with_progress(
            vec![ToolProgress::text("a"), ToolProgress::text("b")],
            || async { Ok(ToolResult::text("done")) },
        );
        let (progress, result) = drain_with_progress(s).await;
        assert_eq!(progress.len(), 2);
        assert_eq!(result.expect("ok").content, "done");
    }

    #[tokio::test]
    async fn empty_stream_is_protocol_error() {
        let s: ToolStream = Box::pin(stream::empty());
        let err = drain_terminal(s).await.expect_err("proto");
        assert_eq!(err.code(), machi_types::ErrorCode::ToolStreamProtocol);
    }
}
