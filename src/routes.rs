use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};

use crate::content::Content;
use crate::handlers::{assets, count, echo};
use crate::view;

// Build the router: one route per discovered page and per blog post (all
// pre-rendered at startup), the /blog index, the stylesheet, two dynamic
// (per-request) pages, and a 404 fallback. Adding a page or post needs no edit
// here — it appears once its file is in content/pages/ or content/blog/. The
// dynamic pages carry their own logic in handlers/; here we just register them.
pub fn app(content: &Arc<Content>) -> Router {
    let mut router = Router::new();
    for page in &content.pages {
        let html = view::render_page(page, &content.pages);
        router = router.route(&page.path, get(serve_html(html)));
    }
    for post in &content.posts {
        let html = view::post_page(post, &content.pages);
        router = router.route(&format!("/blog/{}", post.slug), get(serve_html(html)));
    }

    let blog_index = view::blog_index(&content.posts, &content.pages);
    let not_found = view::not_found(&content.pages);
    router
        .route("/blog", get(serve_html(blog_index)))
        .route("/style.css", get(assets::stylesheet))
        .route("/count", count::route(Arc::clone(content)))
        .route("/echo", echo::route(Arc::clone(content)))
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
    async fn blog_index_and_posts_are_served() -> TestResult {
        // A post's slug drives its route; take a real one from the loaded content.
        let content = crate::content::load()?;
        let slug = content
            .posts
            .first()
            .map(|p| p.slug.clone())
            .ok_or("no demo posts embedded")?;
        let app = app(&Arc::new(content));

        let index = app
            .clone()
            .oneshot(Request::builder().uri("/blog").body(Body::empty())?)
            .await?;
        assert_eq!(index.status(), StatusCode::OK);
        assert!(
            body_string(index).await?.contains("/blog/"),
            "index links posts"
        );

        let post = app
            .oneshot(
                Request::builder()
                    .uri(format!("/blog/{slug}"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(post.status(), StatusCode::OK);
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
