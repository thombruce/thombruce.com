use std::net::SocketAddr;

use axum::{
    Router,
    extract::Request,
    http::header::HOST,
    middleware::{Next, from_fn},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use tokio::signal;

mod content;
mod handlers;
mod ssh;
use handlers::{errors, pages};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(pages::home))
        .route("/about", get(pages::about))
        .fallback(errors::not_found)
        .layer(from_fn(www_redirect));

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

// Redirect www.* to the apex host, preserving path, so www doesn't dead-end.
// ponytail: assumes no port in the Host header (true behind Fly's :443 proxy).
async fn www_redirect(req: Request, next: Next) -> Response {
    if let Some(apex) = req
        .headers()
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .and_then(|host| host.strip_prefix("www."))
    {
        let path = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
        return Redirect::permanent(&format!("https://{apex}{path}")).into_response();
    }
    next.run(req).await
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
