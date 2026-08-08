//! Token usage ledger for sessions and nested agents.

use machi_types::Usage;
use serde::{Deserialize, Serialize};

/// Accumulated usage with incomplete markers (fail-closed reporting).
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UsageLedger {
    /// Main-loop / session totals.
    pub main: Usage,
    /// Nested agent fold-in totals.
    pub subagents: Usage,
    /// True when some usage could not be attributed.
    pub incomplete: bool,
}

impl UsageLedger {
    /// Empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a main-loop sample/turn usage.
    pub fn record_main(&mut self, usage: Usage) {
        self.main += usage;
    }

    /// Fold nested agent usage.
    pub fn record_subagent(&mut self, usage: Usage) {
        self.subagents += usage;
    }

    /// Mark ledger incomplete (cancel mid-flight, missing child drain).
    pub fn mark_incomplete(&mut self) {
        self.incomplete = true;
    }

    /// Combined totals.
    #[must_use]
    pub fn total(self) -> Usage {
        self.main + self.subagents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds() {
        let mut l = UsageLedger::new();
        l.record_main(Usage::new(3, 2));
        l.record_subagent(Usage::new(1, 1));
        assert_eq!(l.total().total_tokens, 7);
    }
}
