use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{MethodRouter, get},
};

use crate::content::{Content, Registry};
use crate::handlers::{assets, count, echo};
use crate::view;

// Routes registered in code: the stylesheet and the dynamic (per-request)
// pages, each built from the loaded content. Their logic lives in handlers/.
// This table is the single source for both the router and `reserved()`, so
// content can never claim a path the router also registers.
type Factory = fn(Arc<Content>) -> MethodRouter;
const REGISTERED: [(&str, Factory); 3] = [
    ("/style.css", |_| get(assets::stylesheet)),
    ("/count", count::route),
    ("/echo", echo::route),
];

// Everything content is loaded against: paths owned by registered routes
// (which win — content at one of them is skipped with a warning) and the
// template names content may select.
pub fn registry() -> Registry {
    Registry {
        routes: REGISTERED.iter().map(|(path, _)| *path).collect(),
        doc_layouts: view::doc_layouts(),
        index_layouts: view::index_layouts(),
    }
}

// Build the router: one route per discovered doc and per directory listing
// (all pre-rendered at startup), the registered routes, and a 404 fallback.
// Adding content needs no edit here — routes derive from the content/ tree.
pub fn app(content: &Arc<Content>) -> Router {
    let nav = &content.nav;
    let mut router = Router::new();
    for doc in &content.docs {
        router = router.route(&doc.path, get(serve_html(view::doc_page(doc, nav))));
    }
    for listing in &content.listings {
        let html = view::listing_page(listing, nav);
        router = router.route(&listing.path, get(serve_html(html)));
    }

    for (path, route) in REGISTERED {
        router = router.route(path, route(Arc::clone(content)));
    }

    let not_found = view::not_found(nav);
    router
        .fallback(move || {
            let html = not_found.clone();
            async move { (StatusCode::NOT_FOUND, Html(html)) }
        })
        .layer(from_fn(www_redirect))
}

// A handler that serves one pre-rendered HTML string, cloned per request (the
// router closure is reusable, so it can't move the captured string out).
fn serve_html(html: String) -> impl Fn() -> std::future::Ready<Html<String>> + Clone {
    move || std::future::ready(Html(html.clone()))
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
        Ok(app(&Arc::new(crate::content::load(registry())?)))
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
    async fn every_doc_and_listing_is_served() -> TestResult {
        let content = Arc::new(crate::content::load(registry())?);
        let app = app(&content);
        let paths = content
            .docs
            .iter()
            .map(|d| &d.path)
            .chain(content.listings.iter().map(|l| &l.path));
        for path in paths {
            let res = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty())?)
                .await?;
            assert_eq!(res.status(), StatusCode::OK, "{path}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn listing_links_its_entries() -> TestResult {
        // content/code/inkpot.md has no frontmatter; it's titled by its heading.
        let req = Request::builder().uri("/code").body(Body::empty())?;
        let body = body_string(test_app()?.oneshot(req).await?).await?;
        assert!(
            body.contains(r#"<a href="/code/inkpot">Inkpot</a>"#),
            "{body}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn post_layout_renders_title_once_with_byline() -> TestResult {
        // content/blog/index.md sets `default_layout: post`.
        let req = Request::builder()
            .uri("/blog/hello-world")
            .body(Body::empty())?;
        let body = body_string(test_app()?.oneshot(req).await?).await?;
        assert!(
            body.contains(r#"<article><h1>Hello, World!</h1><p><time datetime="2026-10-04">"#),
            "{body}"
        );
        assert_eq!(body.matches("<h1>").count(), 1, "title not repeated");
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
