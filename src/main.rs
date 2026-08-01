use std::net::SocketAddr;

use tokio::signal;

mod content;
mod handlers;
mod routes;
mod ssh;
mod view;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = routes::app();

    // Render (and most PaaS) inject the HTTP port via $PORT; fall back for local dev.
    // SSH binds a high port locally (privileged ports need root); a host maps :22 to it.
    let http_addr = SocketAddr::from(([0, 0, 0, 0], env_port("PORT", 3000)));
    let ssh_addr = format!("0.0.0.0:{}", env_port("SSH_PORT", 2222));

    // SSH frontend runs in the background; when the HTTP server returns on
    // shutdown, the process exits and this task is dropped.
    // ponytail: no graceful shutdown for SSH connections yet — add if abrupt
    // disconnects on redeploy become a problem.
    tokio::spawn(async move {
        if let Err(err) = ssh::serve(ssh_addr).await {
            eprintln!("ssh server error: {err}");
        }
    });

    // run our app with hyper, listening on all interfaces (required in a container)
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn env_port(var: &str, default: u16) -> u16 {
    std::env::var(var)
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default)
}

// Resolves when the process receives Ctrl+C or SIGTERM (sent by the host on deploy/stop).
async fn shutdown_signal() {
    let ctrl_c = async {
        // On error installing the handler, never resolve so the other branch still works.
        if signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    // unix-only; the host and local dev are both Linux. Add a #[cfg(not(unix))]
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
