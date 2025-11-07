use tokio::signal;

// Graceful shutdown signal tells `axum` to stop accepting requests
// and gives an opportunity to complete ongoing requests instead of just
// killing connenctions along with the process.
pub async fn graceful_shutdown_signal() {
    // `ctrl_c` makes sense only in dev environment
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // ChatGPT catch:
    // Take into account environments not unix-based, such as Windows.
    // Without this flag, this code would not have even compiled in Windows,
    // which I wouldn't have realized as I am not a windows user.
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

