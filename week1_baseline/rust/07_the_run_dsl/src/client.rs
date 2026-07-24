use std::thread;
use std::time::Duration;

use ureq::config::Config as TlsAgentConfig;
use ureq::tls::{TlsConfig, TlsProvider};
use ureq::Agent;

use crate::context::Context;
use crate::errors::ApiError;
use crate::prompt_builder::PromptBuilder;
use crate::tasks::Task;

const RETRYABLE_STATUS_CODES: [u16; 7] = [408, 409, 429, 500, 502, 503, 504];
const MAX_RETRIES: u32 = 3;
const BASE_RETRY_DELAY_SECS: f64 = 0.5;

// Rust has no default-argument syntax; this names the value Ruby/Python's
// `max_output_tokens: 1024` kwarg default falls back to.
pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 1024;

pub struct Client<'a, T: Task> {
    builder: &'a PromptBuilder<T>,
    agent: Agent,
}

impl<'a, T: Task> Client<'a, T> {
    pub fn new(builder: &'a PromptBuilder<T>) -> Self {
        let config = TlsAgentConfig::builder()
            .tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build())
            .http_status_as_error(false)
            .build();

        Self { builder, agent: config.new_agent() }
    }

    pub fn call(
        &self,
        context: &Context<T>,
        max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> Result<serde_json::Value, ApiError> {
        let url = self.builder.url();
        let headers = self.builder.headers();
        let body = serde_json::to_string(&self.builder.to_api_payload(context, max_output_tokens, tools))
            .expect("payload must serialize");

        let mut attempts = 0u32;

        loop {
            attempts += 1;

            let mut request = self.agent.post(&url);
            for (key, value) in &headers {
                request = request.header(key.as_str(), value.as_str());
            }

            match request.send(&body) {
                Ok(mut response) => {
                    let status = response.status().as_u16();

                    if Self::is_retryable_status(status) && attempts <= MAX_RETRIES {
                        thread::sleep(Self::retry_delay(attempts));
                        continue;
                    }

                    if !(200..300).contains(&status) {
                        let response_body = response.body_mut().read_to_string().unwrap_or_default();
                        let suffix = if attempts == 1 { "" } else { "s" };
                        return Err(ApiError(format!(
                            "API request failed after {attempts} attempt{suffix} ({status}): {response_body}"
                        )));
                    }

                    let response_body = response
                        .body_mut()
                        .read_to_string()
                        .map_err(|e| ApiError(format!("failed to read response body: {e}")))?;

                    return serde_json::from_str(&response_body)
                        .map_err(|e| ApiError(format!("failed to parse response JSON: {e}")));
                }
                Err(e) if Self::is_transient(&e) => {
                    if attempts > MAX_RETRIES {
                        return Err(ApiError(format!("API request failed after {attempts} attempts: {e}")));
                    }

                    thread::sleep(Self::retry_delay(attempts));
                }
                Err(e) => return Err(ApiError(format!("API request failed: {e}"))),
            }
        }
    }

    fn is_retryable_status(status: u16) -> bool {
        RETRYABLE_STATUS_CODES.contains(&status)
    }

    fn is_transient(err: &ureq::Error) -> bool {
        matches!(
            err,
            ureq::Error::Timeout(_)
                | ureq::Error::HostNotFound
                | ureq::Error::ConnectionFailed
                | ureq::Error::Io(_)
                | ureq::Error::Protocol(_)
                | ureq::Error::Tls(_)
                | ureq::Error::Rustls(_)
                | ureq::Error::NativeTls(_)
        )
    }

    fn retry_delay(attempt: u32) -> Duration {
        Duration::from_secs_f64(BASE_RETRY_DELAY_SECS * 2f64.powi(attempt as i32 - 1))
    }
}
