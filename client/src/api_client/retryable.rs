use std::time::Duration;

use reqwest::{RequestBuilder, Response, StatusCode};
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::Instrument;
use uuid::Uuid;

use crate::{ApiClient, api_client::errors::ApiClientError};

fn is_transient_reqwest(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect() || e.is_request()
}

fn is_transient_http(status: StatusCode) -> bool {
    status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
}

fn retry_after(resp: &Response) -> Option<Duration> {
    resp.headers()
        .get("retry-after")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}

#[derive(Clone, Debug)]
pub struct RetrySettings {
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub respect_retry_after: bool,
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            max_retries: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(5),
            respect_retry_after: true,
        }
    }
}

pub trait Retryable {
    async fn send_with_retries(
        &self,
        req_builder: RequestBuilder,
    ) -> Result<Response, ApiClientError>;
}

impl Retryable for ApiClient {
    async fn send_with_retries(
        &self,
        req_builder: RequestBuilder,
    ) -> Result<Response, ApiClientError> {
        let retry_settings = self.retry_settings.clone();
        let request = req_builder.try_clone().unwrap().build().unwrap();
        let method = request.method().as_str().to_string();
        let path = request.url().path().to_string();

        let request_id = Uuid::new_v4().to_string();
        let url_for_log = format!("{}{}", self.args.base_url, path);

        let strategy =
            ExponentialBackoff::from_millis(retry_settings.base_delay.as_millis() as u64)
                .max_delay(retry_settings.max_delay)
                .map(jitter)
                .take(retry_settings.max_retries);

        Retry::spawn(strategy, move || {
            let req_builder = self.sign_request(req_builder.try_clone().unwrap());

            let span = tracing::info_span!(
                "http.client",
                %method,
                %path,
                %request_id,
                url = %url_for_log,
            );

            async move {
                match req_builder
                    .try_clone()
                    .unwrap()
                    .send()
                    .instrument(span.clone())
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();

                        if is_transient_http(status) {
                            if retry_settings.respect_retry_after
                                && let Some(wait) = retry_after(&resp)
                            {
                                tracing::warn!(
                                    parent: &span,
                                    status = %status,
                                    ?wait,
                                    "transient HTTP; honoring Retry-After"
                                );
                                tokio::time::sleep(wait.min(retry_settings.max_delay)).await;
                            }
                            Err(ApiClientError::Other(
                                status,
                                format!("transient http: {status}"),
                            ))
                        } else {
                            Ok(resp)
                        }
                    }
                    Err(e) if is_transient_reqwest(&e) => {
                        tracing::warn!(parent: &span, error=%e, "transient network; retrying");
                        Err(ApiClientError::from(e))
                    }
                    Err(e) => {
                        tracing::error!(parent: &span, error=%e, "non-retryable error");
                        Err(ApiClientError::from(e))
                    }
                }
            }
        })
        .await
    }
}
