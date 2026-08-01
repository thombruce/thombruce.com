use axum::http::header;
use axum::response::IntoResponse;

// Serve the drizzle-css stylesheet from an embedded constant (no static files).
pub async fn stylesheet() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        drizzle_css::CSS_MIN,
    )
}
