// Except for the extension trait, the rest was delegated to ChatGPT
// This is not part of the Template Repository, as in our current ecosystem
// we use AWS API Gateway with some Lambda authorizers
use axum::{
    Router,
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use sha2::Sha256;
use std::{collections::HashMap, sync::Arc};

const AUTH_TS_HEADER_NAME: &str = "X-AUTH-TS";
const AUTH_SIGNATURE_HEADER_NAME: &str = "X-AUTH-SIGNATURE";
const AUTH_KEY_HEADER_NAME: &str = "X-AUTH-KEY";
const FIVE_SECONDS_IN_MILLIS: i64 = 5000;

type HmacSha256 = Hmac<Sha256>;

pub trait AuthenticationExtensions {
    fn with_authentication(self) -> Self;
}

impl<S> AuthenticationExtensions for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_authentication(self) -> Self {
        let auth_state = AuthState::new();
        self.layer(axum::middleware::from_fn_with_state(
            auth_state,
            authentication_middleware,
        ))
    }
}

#[derive(Clone)]
struct AuthState {
    clients: Arc<HashMap<String, String>>,
    max_skew: i64,
}

impl AuthState {
    pub fn new() -> Self {
        let clients = load_clients();
        Self {
            clients: Arc::new(clients),
            max_skew: FIVE_SECONDS_IN_MILLIS,
        }
    }
}

async fn authentication_middleware(
    State(auth): State<AuthState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    println!("Authentication middleware invoked");
    let Some(ts_header) = req.headers().get(AUTH_TS_HEADER_NAME) else {
        return (StatusCode::UNAUTHORIZED, "Missing X-AUTH-TS").into_response();
    };
    let Some(signature_header) = req.headers().get(AUTH_SIGNATURE_HEADER_NAME) else {
        return (StatusCode::UNAUTHORIZED, "Missing X-AUTH-SIGNATURE").into_response();
    };
    let Some(key_header) = req.headers().get(AUTH_KEY_HEADER_NAME) else {
        return (StatusCode::UNAUTHORIZED, "Missing X-AUTH-KEY").into_response();
    };

    let timestamp = match ts_header.to_str() {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid X-AUTH-TS").into_response(),
    };
    let sigature = match signature_header.to_str() {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid X-AUTH-SIGNATURE").into_response(),
    };
    let key = match key_header.to_str() {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid X-AUTH-KEY").into_response(),
    }
    .to_string();

    let Some(secret) = auth.clients.get(&key) else {
        return (StatusCode::UNAUTHORIZED, "Unknown key").into_response();
    };

    let expected_signature = hmac_hex(secret.as_bytes(), timestamp.as_bytes());
    if !constant_time_eq(&expected_signature, sigature) {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    let now = Utc::now().timestamp_millis();
    let timestamp: i64 = match timestamp.parse::<i64>() {
        Ok(v) => v,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid timestamp").into_response(),
    };
    let age = (now - timestamp).abs();
    if age > auth.max_skew {
        return (StatusCode::UNAUTHORIZED, "Expired timestamp").into_response();
    }

    next.run(req).await
}

fn hmac_hex(secret: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

fn constant_time_eq(a_hex: &str, b_hex: &str) -> bool {
    if a_hex.len() != b_hex.len() {
        return false;
    }
    let (a, b) = (a_hex.as_bytes(), b_hex.as_bytes());
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

// This only makes sense for demo purposes. In real scenarios not only that
// secrets must be stored hashed and encrypted, but also loaded from secure
// storage or environment variables. Ideally, not kept in memory neither.
// Additionally, each client should be able to have more than one secret with
// expiration, rotation, etc.
fn load_clients() -> HashMap<String, String> {
    let mut clients = HashMap::new();

    clients.insert("client-1".to_string(), "secret-1".to_string());
    clients.insert("client-2".to_string(), "secret-2".to_string());

    clients
}
