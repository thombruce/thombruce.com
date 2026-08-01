use axum::http::StatusCode;
use maud::{Markup, html};

use crate::view::shell;

pub async fn not_found() -> (StatusCode, Markup) {
    (
        StatusCode::NOT_FOUND,
        shell(
            "Not Found",
            &html! {
                h1 { "404" }
                p { "That page doesn’t exist." }
                p { a href="/" { "Go home" } }
            },
        ),
    )
}
