use axum::http::StatusCode;

pub async fn not_found() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_FOUND,
        r"404 Not Found

This page does not exist.",
    )
}
