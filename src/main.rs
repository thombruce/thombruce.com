use std::net::SocketAddr;

use axum::{
    Router,
    routing::get,
};
use tokio::signal;

mod handlers;
use handlers::{errors, pages};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(pages::home))
        .route("/about", get(pages::about))
        .fallback(errors::not_found);

    // Render (and most PaaS) inject the port via $PORT; fall back for local dev.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // run our app with hyper, listening on all interfaces (required in a container)
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// Resolves when the process receives Ctrl+C or SIGTERM (sent by Render on deploy/stop).
async fn shutdown_signal() {
    let ctrl_c = async {
        // On error installing the handler, never resolve so the other branch still works.
        if signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    // unix-only; Render and local dev are both Linux. Add a #[cfg(not(unix))]
    // fallback if a Windows target ever appears.
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
