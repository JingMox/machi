// W6: independent HTTP status → class/code contract (not a clone of the SUT match).
#[cfg(test)]
#[allow(clippy::missing_assert_message, reason = "table cases named by status")]
mod http_status_matrix {
    use machi_types::ErrorCode;

    use super::{HttpRetryClass, classify_http_status, error_code_for_http};
    use crate::openai_compat::http_status_error;

    /// Documented policy table — independent of production match arms.
    /// When policy changes, update this table deliberately (not by copying SUT).
    const CLASS_CASES: &[(u16, HttpRetryClass)] = &[
        (100, HttpRetryClass::Fatal),
        (200, HttpRetryClass::Fatal),
        (301, HttpRetryClass::Fatal),
        (400, HttpRetryClass::Fatal),
        (401, HttpRetryClass::Fatal),
        (403, HttpRetryClass::Fatal),
        (404, HttpRetryClass::Fatal),
        (418, HttpRetryClass::Fatal),
        (422, HttpRetryClass::Fatal),
        (429, HttpRetryClass::RateLimited),
        (450, HttpRetryClass::Fatal),
        (499, HttpRetryClass::Fatal),
        (500, HttpRetryClass::Retry),
        (501, HttpRetryClass::Retry),
        (502, HttpRetryClass::Retry),
        (503, HttpRetryClass::Retry),
        (504, HttpRetryClass::Retry),
        (507, HttpRetryClass::Retry),
        (524, HttpRetryClass::Retry),
        (525, HttpRetryClass::Fatal),
        (526, HttpRetryClass::Fatal),
        (527, HttpRetryClass::Retry),
        (599, HttpRetryClass::Retry),
    ];

    #[test]
    fn classify_matches_policy_table() {
        for &(status, want) in CLASS_CASES {
            assert_eq!(
                classify_http_status(status, None),
                want,
                "status {status}"
            );
        }
    }

    #[test]
    fn x_should_retry_overrides() {
        assert_eq!(
            classify_http_status(418, Some(true)),
            HttpRetryClass::Retry
        );
        assert_eq!(
            classify_http_status(500, Some(false)),
            HttpRetryClass::Fatal
        );
        assert_eq!(
            classify_http_status(200, Some(true)),
            HttpRetryClass::Retry
        );
    }

    #[test]
    fn live_http_status_error_uses_classifier() {
        // Production path (OpenAI/Ollama) must emit codes from error_code_for_http.
        let e429 = http_status_error(429, "rate");
        assert_eq!(e429.code(), ErrorCode::LlmRateLimit);
        assert_eq!(
            error_code_for_http(429, HttpRetryClass::RateLimited),
            ErrorCode::LlmRateLimit
        );

        let e401 = http_status_error(401, "nope");
        assert_eq!(e401.code(), ErrorCode::LlmAuth);

        let e500 = http_status_error(500, "boom");
        assert_eq!(e500.code(), ErrorCode::LlmProvider);
        assert_eq!(e500.retry_class(), machi_types::RetryClass::Backoff);

        let e525 = http_status_error(525, "tls");
        assert_eq!(e525.code(), ErrorCode::LlmProvider);
        assert_eq!(e525.retry_class(), machi_types::RetryClass::Never);

        let e404 = http_status_error(404, "missing");
        assert_eq!(e404.retry_class(), machi_types::RetryClass::Never);
    }

    #[test]
    fn all_5xx_except_tls_are_retry_class() {
        for status in 500_u16..600 {
            let class = classify_http_status(status, None);
            if status == 525 || status == 526 {
                assert_eq!(class, HttpRetryClass::Fatal, "status {status}");
            } else {
                assert_eq!(class, HttpRetryClass::Retry, "status {status}");
            }
        }
    }

    #[test]
    fn all_4xx_except_429_are_fatal() {
        for status in 400_u16..500 {
            let class = classify_http_status(status, None);
            if status == 429 {
                assert_eq!(class, HttpRetryClass::RateLimited);
            } else {
                assert_eq!(class, HttpRetryClass::Fatal, "status {status}");
            }
        }
    }
}
