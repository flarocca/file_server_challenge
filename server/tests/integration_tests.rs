// This tests are testing the server as close as possible to a real client.
// This was completely written by myself. As a general policy, never delegate tests
// to ChatGPT

mod helpers;

use crate::helpers::web_server_simulator::WebServerSimulator;
use bytes::Bytes;
use chrono::{Duration, Utc};
use file_server_server::{
    handlers::responses::{FileMetadataResponse, FinalUploadResponse, InitiateUploadResponse},
    services::FileServiceError,
};
use hmac::{Hmac, Mac};
use mockall::predicate::{always, eq};
use reqwest::StatusCode;
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const TEST_KEY: &str = "client-1";
const TEST_SECRET: &str = "secret-1";

fn create_valid_signature() -> (String, String) {
    let timestamp = Utc::now().timestamp_millis().to_string();
    let signature = create_signature(TEST_SECRET, &timestamp);

    (signature, timestamp)
}

fn create_signature(secret: &str, timestamp: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(timestamp.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::test]
async fn test_auth_missing_headers_returns_401() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    simulator.configure_file_service(|srv| {
        srv.expect_initiate().times(0);
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/initiate", base_url);
    let resp = reqwest::Client::new().post(&url).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server_handle.abort();
}

#[tokio::test]
async fn test_auth_bad_signature_returns_401() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    simulator.configure_file_service(|srv| {
        srv.expect_initiate().times(0);
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/initiate", base_url);
    let timestamp = Utc::now().timestamp_millis().to_string();

    let signature = create_signature("wrong-secret", &timestamp);

    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server_handle.abort();
}

#[tokio::test]
async fn test_auth_old_timestamp_returns_401() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    simulator.configure_file_service(|srv| {
        srv.expect_initiate().times(0);
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/initiate", base_url);

    let old_timestamp = (Utc::now() - Duration::milliseconds(6000))
        .timestamp_millis()
        .to_string();
    let signature = create_signature("wrong-secret", &old_timestamp);

    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", old_timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server_handle.abort();
}

#[tokio::test]
async fn test_auth_unknown_api_key_returns_401() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    simulator.configure_file_service(|srv| {
        srv.expect_initiate().times(0);
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/initiate", base_url);
    let timestamp = Utc::now().timestamp_millis().to_string();

    let signature = create_signature("unknown-secret", &timestamp);

    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", "unknown-key")
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server_handle.abort();
}

#[tokio::test]
async fn test_can_initiate_upload() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let expected_id = Uuid::new_v4();

    simulator.configure_file_service(|srv| {
        srv.expect_initiate()
            .times(1)
            .returning(move || Ok(expected_id));
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/initiate", base_url);

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);

    let actual_response: InitiateUploadResponse = resp.json().await.unwrap();
    assert_eq!(actual_response.id, expected_id);

    server_handle.abort();
}

#[tokio::test]
async fn test_unload_without_initialization_returns_not_found() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let invalid_id = Uuid::new_v4();

    simulator.configure_file_service(|srv| {
        srv.expect_upload_file()
            .with(eq(invalid_id), always(), always())
            .times(1)
            .returning(move |_, _, _| Err(FileServiceError::FileNotFound));
    });
    let server_handle = simulator.start().await;

    let url = format!("{}/{}/upload?name=test.txt&index=0", base_url, invalid_id);

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .body(Bytes::from_static(b"fake data"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    server_handle.abort();
}

#[tokio::test]
async fn test_unload_with_duplicated_file_returns_conflict() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let expected_id = Uuid::new_v4();
    let expected_filename = "test.txt";
    let expected_index = 0;

    simulator.configure_file_service(|srv| {
        srv.expect_upload_file()
            .withf(move |id, metadata, _| {
                *id == expected_id
                    && metadata.name == expected_filename
                    && metadata.index == expected_index
            })
            .times(1)
            .returning(move |_, _, _| Err(FileServiceError::FileAlreadyExists));
    });
    let server_handle = simulator.start().await;

    let url = format!(
        "{}/{}/upload?name={}&index={}",
        base_url, expected_id, expected_filename, expected_index
    );

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .body(Bytes::from_static(b"fake data"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);

    server_handle.abort();
}

#[tokio::test]
async fn test_upload_with_valid_request_returns_ok() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let expected_id = Uuid::new_v4();
    let expected_filename = "test.txt";
    let expected_index = 0;
    let expected_filecontent = b"fake data";
    let expected_hash = "abcd1234deadbeef";

    simulator.configure_file_service(|srv| {
        srv.expect_upload_file()
            .withf(move |id, metadata, body| {
                *id == expected_id
                    && metadata.name == expected_filename
                    && metadata.index == expected_index
                    && body == &Bytes::from_static(expected_filecontent)
            })
            .times(1)
            .returning(move |_, _, _| Ok(expected_hash.to_string()));
    });

    let server_handle = simulator.start().await;

    let url = format!(
        "{}/{}/upload?name={}&index={}",
        base_url, expected_id, expected_filename, expected_index
    );

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .body(Bytes::from_static(expected_filecontent))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: FileMetadataResponse = resp.json().await.unwrap();
    assert_eq!(body.name, expected_filename);
    assert_eq!(body.index, expected_index);
    assert_eq!(body.encoded_hash, expected_hash);

    server_handle.abort();
}

#[tokio::test]
async fn test_complete_returns_ok() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let expected_id = Uuid::new_v4();
    let expected_root = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    simulator.configure_file_service(|srv| {
        srv.expect_complete()
            .withf(move |id| *id == expected_id)
            .times(1)
            .returning(move |_| Ok(expected_root.to_string()));
    });

    let server_handle = simulator.start().await;

    let url = format!("{}/{}/complete", base_url, expected_id);

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: FinalUploadResponse = resp.json().await.unwrap();
    assert_eq!(body.root_hex, expected_root);

    server_handle.abort();
}

#[tokio::test]
async fn test_complete_with_invalid_id_returns_not_found() {
    let mut simulator = WebServerSimulator::new().await.unwrap();
    let base_url = simulator.url();

    let id = Uuid::new_v4();

    simulator.configure_file_service(|srv| {
        srv.expect_complete()
            .withf(move |got_id| *got_id == id)
            .times(1)
            .returning(|_| Err(FileServiceError::FileNotFound));
    });

    let server_handle = simulator.start().await;

    let url = format!("{}/{}/complete", base_url, id);

    let (signature, timestamp) = create_valid_signature();
    let resp = reqwest::Client::new()
        .post(&url)
        .header("X-AUTH-KEY", TEST_KEY)
        .header("X-AUTH-TS", timestamp)
        .header("X-AUTH-SIGNATURE", signature)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    server_handle.abort();
}
