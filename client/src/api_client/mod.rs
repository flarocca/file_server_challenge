// This ApiClient is a wrapper around the Server API to hide all implementation details
// and give semantics. Ideally, no business logic should be written here other than
// translation of responses from the server
// Additionally, this module could be a separate crate for better reusability and maintenance.
mod errors;
mod models;
mod retryable;

use std::time::Duration;

use chrono::Utc;
use file_server_library::models::Proof;
use hmac::{Hmac, Mac};
use reqwest::{Client as HttpClient, StatusCode, Url};
use sha2::Sha256;
use tracing::instrument;
use uuid::Uuid;

use crate::api_client::{
    errors::ApiClientError,
    models::{FileMetadataResponse, FinalUploadResponse, InitiateUploadResponse},
    retryable::{RetrySettings, Retryable},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct ApiClientArgs {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: Url,
    pub correlation_id: Uuid,
}

impl ApiClientArgs {
    fn create_authentication_headers(&self) -> (String, String, String) {
        let ts = Utc::now().timestamp_millis().to_string();

        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).unwrap();
        mac.update(ts.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        (self.api_key.clone(), ts, sig)
    }
}

#[derive(Clone)]
pub struct ApiClient {
    http: HttpClient,
    args: ApiClientArgs,
    retry_settings: RetrySettings,
}

impl ApiClient {
    pub fn new(args: ApiClientArgs) -> Result<Self, ApiClientError> {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiClientError::Unexpected(e.to_string()))?;

        Ok(Self {
            http,
            args,
            retry_settings: RetrySettings::default(),
        })
    }

    fn sign_request(&self, req_builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let (key, ts, sig) = self.args.create_authentication_headers();

        req_builder
            .header("X-AUTH-KEY", key)
            .header("X-AUTH-TS", ts)
            .header("X-AUTH-SIGNATURE", sig)
            .header("X-CORRELATION-ID", self.args.correlation_id.to_string())
    }

    #[instrument(skip(self), fields(correlation_id = %self.args.correlation_id))]
    pub async fn initiate(&self) -> Result<Uuid, ApiClientError> {
        let url = format!("{}api/v1/initiate", self.args.base_url);
        let resp = self.send_with_retries(self.http.post(url)).await?;

        if resp.status() != StatusCode::CREATED {
            return Err(ApiClientError::from_response(resp).await);
        }

        let body: InitiateUploadResponse = resp.json().await?;

        Ok(body.id)
    }

    #[instrument(skip(self), fields(correlation_id = %self.args.correlation_id, id = %id, name = name, index = index))]
    pub async fn upload_file(
        &self,
        id: Uuid,
        name: &str,
        index: usize,
        bytes: Vec<u8>,
    ) -> Result<FileMetadataResponse, ApiClientError> {
        let url = format!(
            "{}api/v1/{}/upload?name={}&index={}",
            self.args.base_url,
            id,
            urlencoding::encode(name),
            index
        );

        let resp = self
            .send_with_retries(self.http.post(url).body(bytes))
            .await?;

        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(ApiClientError::from_response(resp).await),
        }
    }

    #[instrument(skip(self), fields(correlation_id = %self.args.correlation_id, id = %id))]
    pub async fn complete(&self, id: Uuid) -> Result<String, ApiClientError> {
        let url = format!("{}api/v1/{}/complete", self.args.base_url, id);
        let resp = self.send_with_retries(self.http.post(url)).await?;

        if !resp.status().is_success() {
            return Err(ApiClientError::from_response(resp).await);
        }

        let body: FinalUploadResponse = resp.json().await?;
        Ok(body.root_hex)
    }

    #[instrument(skip(self), fields(correlation_id = %self.args.correlation_id, id = %id, index = index))]
    pub async fn get_proof(&self, id: Uuid, index: usize) -> Result<Proof, ApiClientError> {
        let url = format!("{}api/v1/{}/proof/{}", self.args.base_url, id, index);
        let resp = self.send_with_retries(self.http.get(url)).await?;

        match resp.status() {
            StatusCode::OK => Ok(resp.json().await?),
            _ => Err(ApiClientError::from_response(resp).await),
        }
    }

    #[instrument(skip(self), fields(correlation_id = %self.args.correlation_id, id = %id, index = index))]
    pub async fn download_file(&self, id: Uuid, index: usize) -> Result<Vec<u8>, ApiClientError> {
        let url = format!("{}api/v1/{}/file/{}", self.args.base_url, id, index);
        let resp = self.send_with_retries(self.http.get(url)).await?;

        if !resp.status().is_success() {
            return Err(ApiClientError::from_response(resp).await);
        }

        Ok(resp.bytes().await?.to_vec())
    }
}
