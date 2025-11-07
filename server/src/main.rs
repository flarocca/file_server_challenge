use file_server_server::{infrastructure, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::init_infrastructure().await;
    let (server, listener) = server::init_server().await?;

    axum::serve(listener, server)
        .with_graceful_shutdown(infrastructure::graceful_shutdown_signal())
        .await?;

    Ok(())
}
