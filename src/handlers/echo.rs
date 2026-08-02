use std::sync::Arc;

use axum::extract::Request;
use axum::response::Html;
use axum::routing::{MethodRouter, get};

use crate::content::Page;
use crate::view;

// /echo — reflects the request back (method, path, headers), rendered
// server-side. The static pages ignore the request entirely; this reads it.
pub fn route(pages: Arc<Vec<Page>>) -> MethodRouter {
    get(move |req: Request| {
        let method = req.method().as_str().to_owned();
        let path = req.uri().path().to_owned();
        let headers: Vec<(String, String)> = req
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_owned(),
                    v.to_str().unwrap_or("<binary>").to_owned(),
                )
            })
            .collect();
        let html = view::echo_page(&method, &path, &headers, pages.as_slice());
        async move { Html(html) }
    })
}
