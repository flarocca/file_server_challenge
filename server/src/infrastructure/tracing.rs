// This file is part of the template, usually, this does not need to be modified.
// Additionally, in a microservice architecture and in order to enforce certain architectural
// standards, this code should better be moved to a separate crate or library.
// This ensures consistency accross services and also eliminates the temptation to modify it

use axum::{
    Router,
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Span;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt, layer::SubscriberExt};
use uuid::Uuid;

const CORRELATION_ID_HEADER: &str = "x-correlation-id";
const REQUEST_ID_HEADER: &str = "x-request-id";

// This kind of extension trait pattern is a bias inherited from when I was a C# developer,
// which literally has the concept of extension methods.
pub trait TracingExtensions {
    fn with_tracing(self) -> Self;
    fn with_request_id(self) -> Self;
    fn with_correlation_id(self) -> Self;
}

pub fn init_tracing() {
    LogTracer::init().expect("Failed to set logger");
    let fmt_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .json()
        .boxed();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    let subscriber = Registry::default().with(filter_layer).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

impl<S> TracingExtensions for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn with_tracing(self) -> Self {
        self.layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
    }

    fn with_request_id(self) -> Self {
        self.layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<_>| {
                let id = Uuid::new_v4().to_string();
                let request_id = req
                    .headers()
                    .get(REQUEST_ID_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or(&id);

                tracing::info_span!(
                    "request",
                    %request_id,
                    method = %req.method(),
                    uri = %req.uri(),
                    version = ?req.version()
                )
            }),
        )
        .layer(middleware::from_fn(request_id_middleware))
    }

    fn with_correlation_id(self) -> Self {
        self.layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    let correlation_id = req
                        .headers()
                        .get(CORRELATION_ID_HEADER)
                        .and_then(|v| v.to_str().ok());

                    if let Some(cid) = correlation_id {
                        tracing::info_span!(
                            "correlation_id",
                            correlation_id = %cid
                        )
                    } else {
                        Span::none()
                    }
                })
                .on_request(DefaultOnRequest::new())
                .on_response(
                    DefaultOnResponse::new().latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
        .layer(middleware::from_fn(correlation_id_middleware))
    }
}

/// Correlation ID is used to track cross-service requests or e2e executions.
/// Common practice is to have the client including it (where client is the one
/// initiating the flow, not neccesarily the end user app as in this challenge).
/// The client generates an ID (usually a UUID) and includes it in the request as metadata
/// (e.g., HTTP header). All services handling the request should propagate the ID
/// in their requests to downstream services, allowing tracing the entire flow.
/// Another example can be checked out in this [C# Serilog demo](https://github.com/flarocca/tmenos3.netcore.serilog)
async fn correlation_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    let header_name = HeaderName::from_static(CORRELATION_ID_HEADER);
    let correlation_id = req.headers().get(&header_name).cloned();

    if let Some(cid) = correlation_id {
        req.headers_mut().insert(header_name.clone(), cid.clone());
        req.extensions_mut().insert(cid.clone());

        let mut res = next.run(req).await;

        res.headers_mut().insert(header_name, cid);

        res
    } else {
        next.run(req).await
    }
}

/// Request ID is used to uniquely identify all logs for each request.
/// It is always generated by the server and returned back to the client in
/// the response as metadata (e.g., HTTP header).
async fn request_id_middleware(mut req: Request<Body>, next: Next) -> Response {
    let header_name = HeaderName::from_static(REQUEST_ID_HEADER);

    let request_id = req
        .headers()
        .get(&header_name)
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_str(&Uuid::new_v4().to_string()).unwrap());

    req.headers_mut()
        .insert(header_name.clone(), request_id.clone());
    req.extensions_mut().insert(request_id.clone());

    let mut res = next.run(req).await;

    res.headers_mut().insert(header_name, request_id);
    res
}
