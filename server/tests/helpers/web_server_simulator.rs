use axum::Router;
use file_server_server::{
    handlers, infrastructure::AuthenticationExtensions, server::ServerState, services::FileService,
};
use std::sync::Arc;
use tokio::{net::TcpListener, task::JoinHandle};

use crate::helpers::mocks::MockFileServiceImpl;

type FileServiceType = Arc<dyn FileService + Send + Sync>;

pub struct WebServerSimulator {
    api_base_url: String,
    listener: TcpListener,
    file_service: MockFileServiceImpl,
}

impl WebServerSimulator {
    pub async fn new() -> anyhow::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let host = listener.local_addr().unwrap();
        let api_base_url = format!("http://{}/api/v1", host);

        Ok(Self {
            api_base_url,
            listener,
            file_service: MockFileServiceImpl::new(),
        })
    }

    pub fn url(&self) -> String {
        self.api_base_url.clone()
    }

    pub fn configure_file_service(&mut self, mut callback: impl FnMut(&mut MockFileServiceImpl)) {
        callback(&mut self.file_service);
    }

    pub async fn start(self) -> JoinHandle<()> {
        let state = Arc::new(ServerState::new(
            Arc::new(self.file_service) as FileServiceType
        ));

        let routes = handlers::router(state.clone());

        // For testing purposes, I am taking the router only, which skips some additions
        // like OpenAPI docs, tracing, etc. Depending on the project, that might o r might not be
        // desired. For that reason I need to add authentication manually here.
        let server = Router::new()
            .nest("/api/v1", routes.into())
            .with_authentication();
        let server = axum::serve(self.listener, server.into_make_service());

        tokio::spawn(async move {
            server.await.unwrap();
        })
    }
}
