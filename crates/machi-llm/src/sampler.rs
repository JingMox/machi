//! [`LlmSampler`] trait.

use async_trait::async_trait;
use machi_types::MachiError;

use crate::sample::{SampleRequest, SampleResponse};

/// Abstraction over model providers.
#[async_trait]
pub trait LlmSampler: Send + Sync {
    /// Perform one non-streaming sample.
    async fn sample(&self, request: SampleRequest) -> Result<SampleResponse, MachiError>;
}
