//! Token usage accounting.

use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

/// Prompt-side token details.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PromptTokensDetails {
    /// Cached prompt tokens.
    #[serde(default)]
    pub cached_tokens: u32,
    /// Audio input tokens.
    #[serde(default)]
    pub audio_tokens: u32,
}

/// Completion-side token details.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompletionTokensDetails {
    /// Reasoning tokens.
    #[serde(default)]
    pub reasoning_tokens: u32,
    /// Audio output tokens.
    #[serde(default)]
    pub audio_tokens: u32,
}

/// Aggregated token usage for a sample or turn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Usage {
    /// Input / prompt tokens.
    #[serde(default, alias = "prompt_tokens")]
    pub input_tokens: u32,
    /// Output / completion tokens.
    #[serde(default, alias = "completion_tokens")]
    pub output_tokens: u32,
    /// Total tokens when provided by the provider.
    #[serde(default)]
    pub total_tokens: u32,
    /// Prompt details.
    #[serde(default, alias = "prompt_tokens_details")]
    pub prompt_details: PromptTokensDetails,
    /// Completion details.
    #[serde(default, alias = "completion_tokens_details")]
    pub completion_details: CompletionTokensDetails,
}

impl Usage {
    /// Zero usage.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            prompt_details: PromptTokensDetails {
                cached_tokens: 0,
                audio_tokens: 0,
            },
            completion_details: CompletionTokensDetails {
                reasoning_tokens: 0,
                audio_tokens: 0,
            },
        }
    }

    /// Construct from input/output token counts.
    #[must_use]
    pub const fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
            prompt_details: PromptTokensDetails {
                cached_tokens: 0,
                audio_tokens: 0,
            },
            completion_details: CompletionTokensDetails {
                reasoning_tokens: 0,
                audio_tokens: 0,
            },
        }
    }

    /// Recompute `total_tokens` as input + output when total is zero.
    #[must_use]
    pub const fn normalized(mut self) -> Self {
        if self.total_tokens == 0 {
            self.total_tokens = self.input_tokens.saturating_add(self.output_tokens);
        }
        self
    }
}

impl Add for Usage {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            input_tokens: self.input_tokens.saturating_add(rhs.input_tokens),
            output_tokens: self.output_tokens.saturating_add(rhs.output_tokens),
            total_tokens: self.total_tokens.saturating_add(rhs.total_tokens),
            prompt_details: PromptTokensDetails {
                cached_tokens: self
                    .prompt_details
                    .cached_tokens
                    .saturating_add(rhs.prompt_details.cached_tokens),
                audio_tokens: self
                    .prompt_details
                    .audio_tokens
                    .saturating_add(rhs.prompt_details.audio_tokens),
            },
            completion_details: CompletionTokensDetails {
                reasoning_tokens: self
                    .completion_details
                    .reasoning_tokens
                    .saturating_add(rhs.completion_details.reasoning_tokens),
                audio_tokens: self
                    .completion_details
                    .audio_tokens
                    .saturating_add(rhs.completion_details.audio_tokens),
            },
        }
        .normalized()
    }
}

impl AddAssign for Usage {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_normalizes_total() {
        let a = Usage {
            input_tokens: 10,
            output_tokens: 5,
            ..Usage::zero()
        };
        let b = Usage {
            input_tokens: 1,
            output_tokens: 1,
            ..Usage::zero()
        };
        let sum = (a + b).normalized();
        assert_eq!(sum.input_tokens, 11);
        assert_eq!(sum.output_tokens, 6);
        assert_eq!(sum.total_tokens, 17);
    }

    #[test]
    fn serde_aliases() {
        let raw = r#"{"prompt_tokens":3,"completion_tokens":4}"#;
        let u: Usage = serde_json::from_str(raw).expect("parse");
        assert_eq!(u.input_tokens, 3);
        assert_eq!(u.output_tokens, 4);
    }
}
