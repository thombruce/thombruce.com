use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{Next, from_fn},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};

use crate::content::Page;
use crate::handlers::assets;
use crate::view;

// Build the router from the discovered pages: one route per page (pre-rendered
// at startup), the stylesheet, and a 404 fallback. Adding a page needs no edit
// here — it appears once its file is in content/pages/.
pub fn app(pages: &[Page]) -> Router {
    let mut router = Router::new();
    for page in pages {
        let html = view::render_page(page, pages);
        router = router.route(
            &page.path,
            get(move || {
                let html = html.clone();
                async move { Html(html) }
            }),
        );
    }

    let not_found = view::not_found(pages);
    router
        .route("/style.css", get(assets::stylesheet))
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
        Ok(app(&crate::content::load()?))
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
