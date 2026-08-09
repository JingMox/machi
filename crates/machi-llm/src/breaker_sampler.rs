//! [`BreakerSampler`]: decorator that gates samples through a [`CircuitBreaker`].
//!
//! Maturity: **core**

use std::sync::Arc;

use async_trait::async_trait;
use machi_types::{ErrorCode, MachiError};

use crate::breaker::{Admission, BreakerOutcome, CircuitBreaker};
use crate::sample::{SampleRequest, SampleResponse};
use crate::sampler::LlmSampler;
use crate::stream::SampleStream;

/// Sampler wrapper that refuses traffic while the breaker is open.
#[derive(Debug, Clone)]
pub struct BreakerSampler<S> {
    inner: Arc<S>,
    breaker: Arc<CircuitBreaker>,
    /// Stable key for multi-endpoint registries (metrics / logs).
    endpoint: String,
}

impl<S> BreakerSampler<S> {
    /// Wrap `inner` with a shared breaker.
    #[must_use]
    pub fn new(inner: Arc<S>, breaker: Arc<CircuitBreaker>, endpoint: impl Into<String>) -> Self {
        Self {
            inner,
            breaker,
            endpoint: endpoint.into(),
        }
    }

    /// Endpoint label.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Shared breaker.
    #[must_use]
    pub fn breaker(&self) -> &Arc<CircuitBreaker> {
        &self.breaker
    }
}

#[async_trait]
impl<S: LlmSampler + 'static> LlmSampler for BreakerSampler<S> {
    async fn sample(&self, request: SampleRequest) -> Result<SampleResponse, MachiError> {
        self.admit()?;
        match self.inner.sample(request).await {
            Ok(r) => {
                self.breaker.record(BreakerOutcome::Success);
                Ok(r)
            }
            Err(e) => {
                self.breaker.record(BreakerOutcome::Failure);
                Err(e)
            }
        }
    }

    async fn sample_stream(&self, request: SampleRequest) -> Result<SampleStream, MachiError> {
        self.admit()?;
        match self.inner.sample_stream(request).await {
            Ok(s) => {
                self.breaker.record(BreakerOutcome::Success);
                Ok(s)
            }
            Err(e) => {
                self.breaker.record(BreakerOutcome::Failure);
                Err(e)
            }
        }
    }
}

impl<S> BreakerSampler<S> {
    fn admit(&self) -> Result<(), MachiError> {
        match self.breaker.check() {
            Admission::Allow => Ok(()),
            Admission::Reject { retry_after } => Err(MachiError::new(
                ErrorCode::LlmProvider,
                format!(
                    "circuit breaker open for endpoint '{}'; retry after {}ms",
                    self.endpoint,
                    retry_after.as_millis()
                ),
            )
            .with_retry(machi_types::RetryClass::Backoff)),
        }
    }
}
