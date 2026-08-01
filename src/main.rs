use axum::{
    Router,
    routing::get,
};

mod handlers;
use handlers::pages;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(pages::home))
        .route("/about", get(pages::about));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
