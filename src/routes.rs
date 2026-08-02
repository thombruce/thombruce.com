use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};

use crate::content::Page;
use crate::handlers::{assets, count, echo};
use crate::view;

// Build the router: one route per discovered content page (pre-rendered at
// startup), the stylesheet, two dynamic (per-request) pages, and a 404 fallback.
// Adding a *content* page needs no edit here — it appears once its file is in
// content/pages/. The dynamic pages carry their own logic in handlers/; here we
// just register them, like the stylesheet.
pub fn app(pages: &Arc<Vec<Page>>) -> Router {
    let mut router = Router::new();
    for page in pages.iter() {
        let html = view::render_page(page, pages.as_slice());
        router = router.route(
            &page.path,
            get(move || {
                let html = html.clone();
                async move { Html(html) }
            }),
        );
    }

    let not_found = view::not_found(pages.as_slice());
    router
        .route("/style.css", get(assets::stylesheet))
        .route("/count", count::route(Arc::clone(pages)))
        .route("/echo", echo::route(Arc::clone(pages)))
        .fallback(move || {
            let html = not_found.clone();
            async move { (StatusCode::NOT_FOUND, Html(html)) }
        })
        .layer(from_fn(www_redirect))
}

// Redirect www.* to the apex host, preserving path, so www doesn't dead-end.
// ponytail: assumes no port in the Host header (true behind Fly's :443 proxy).
async fn www_redirect(req: Request, next: Next) -> Response {
    if let Some(apex) = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
        .and_then(|host| host.strip_prefix("www."))
    {
        let path = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
        return Redirect::permanent(&format!("https://{apex}{path}")).into_response();
    }
    next.run(req).await
}

// Tests are allowed to panic (asserts); the panic-restriction lints target the
// long-running server, not the test harness.
#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header::HOST;
    use tower::ServiceExt; // for `oneshot`

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn test_app() -> Result<Router, String> {
        Ok(app(&Arc::new(crate::content::load()?)))
    }

    async fn body_string(res: Response) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    #[tokio::test]
    async fn count_increments_across_requests() -> TestResult {
        // Router clones share the same atomic counter, so two hits count 1 then 2.
        let app = test_app()?;
        let first = app
            .clone()
            .oneshot(Request::builder().uri("/count").body(Body::empty())?)
            .await?;
        let second = app
            .oneshot(Request::builder().uri("/count").body(Body::empty())?)
            .await?;

        assert!(
            body_string(first)
                .await?
                .contains("served <strong>1</strong> time")
        );
        assert!(
            body_string(second)
                .await?
                .contains("served <strong>2</strong> times")
        );
        Ok(())
    }

    #[tokio::test]
    async fn echo_reflects_the_request() -> TestResult {
        let req = Request::builder()
            .uri("/echo")
            .header("user-agent", "test-agent")
            .body(Body::empty())?;
        let body = body_string(test_app()?.oneshot(req).await?).await?;

        assert!(body.contains("/echo"));
        assert!(body.contains("test-agent"));
        Ok(())
    }

    #[tokio::test]
    async fn www_redirects_to_apex_preserving_path() -> TestResult {
        let req = Request::builder()
            .uri("/about")
            .header(HOST, "www.thombruce.com")
            .body(Body::empty())?;
        let res = test_app()?.oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
        let location = res.headers().get("location").and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("https://thombruce.com/about"));
        Ok(())
    }

    #[tokio::test]
    async fn home_route_is_served() -> TestResult {
        let req = Request::builder()
            .uri("/")
            .header(HOST, "thombruce.com")
            .body(Body::empty())?;
        let res = test_app()?.oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn stylesheet_served_as_css() -> TestResult {
        let req = Request::builder().uri("/style.css").body(Body::empty())?;
        let res = test_app()?.oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok());
        assert_eq!(ct, Some("text/css; charset=utf-8"));
        Ok(())
    }

    #[tokio::test]
    async fn unknown_route_is_html_404() -> TestResult {
        let req = Request::builder().uri("/nope").body(Body::empty())?;
        let res = test_app()?.oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok());
        assert_eq!(ct, Some("text/html; charset=utf-8"));
        Ok(())
    }
}
