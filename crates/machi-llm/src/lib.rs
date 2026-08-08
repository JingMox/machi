//! LLM sampling contracts for the Machi kernel.
//!
//! - Always: [`LlmSampler`], [`MockSampler`], wire helpers in [`openai_compat`].
//! - Feature `openai`: [`OpenAiCompatSampler`] HTTP client.
//! - Feature `ollama`: [`OllamaSampler`] HTTP client.

#![forbid(unsafe_code)]

pub mod mock;
pub mod openai_compat;
pub mod sample;
pub mod sampler;
pub mod stream;

#[cfg(feature = "ollama")]
pub mod ollama;

pub use mock::MockSampler;
#[cfg(feature = "ollama")]
pub use ollama::{OllamaConfig, OllamaSampler, build_ollama_chat_body, parse_ollama_chat_response};
#[cfg(feature = "openai")]
pub use openai_compat::OpenAiCompatSampler;
pub use openai_compat::{
    OpenAiCompatConfig, build_chat_completions_body, parse_chat_completions_response,
};
pub use sample::{SampleRequest, SampleResponse, ToolChoice};
pub use sampler::{LlmSampler, response_to_stream};
pub use stream::{SampleEvent, SampleStream};
