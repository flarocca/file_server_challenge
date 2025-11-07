use crate::infrastructure::{AuthenticationExtensions, TracingExtensions};
use crate::services::FileService;
use crate::{
    apidoc::ApiDoc,
    handlers, repositories,
    services::{self},
};
use axum::Router;
use config::Config;
use serde::Deserialize;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct ServerState {
    file_service: Arc<dyn FileService>,
}

impl ServerState {
    pub fn new(file_service: Arc<dyn FileService>) -> Self {
        Self { file_service }
    }

    pub fn file_service(&self) -> Arc<dyn FileService> {
        Arc::clone(&self.file_service)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
}

impl ServerConfig {
    const CONFIG_PREFIX: &'static str = "SERVER_CONFIG";

    pub fn load_from_env() -> anyhow::Result<Self> {
        let config = Config::builder()
            .add_source(config::Environment::with_prefix(Self::CONFIG_PREFIX).separator("__"))
            .build()?
            .try_deserialize::<ServerConfig>()?;

        Ok(config)
    }
}

pub async fn init_server() -> anyhow::Result<(Router, TcpListener)> {
    let config = ServerConfig::load_from_env()?;

    let (file_repository, file_storage) = repositories::init_repositories().await?;
    let file_service = services::init_services(file_repository, file_storage).await?;
    let state = ServerState::new(file_service);
    let server_routes = handlers::router(Arc::new(state.clone()));

    let (server, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", server_routes)
        .split_for_parts();

    let server = server
        .with_tracing()
        .with_request_id()
        .with_correlation_id()
        .with_authentication()
        .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)).await?;

    Ok((server, listener))
}
