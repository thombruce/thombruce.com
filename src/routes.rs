use axum::{
    Router,
    extract::Request,
    http::header::HOST,
    middleware::{Next, from_fn},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};

use crate::handlers::{assets, errors, pages};

pub fn app() -> Router {
    Router::new()
        .route("/", get(pages::home))
        .route("/about", get(pages::about))
        .route("/style.css", get(assets::stylesheet))
        .fallback(errors::not_found)
        .layer(from_fn(www_redirect))
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

// Tests are allowed to panic (asserts); the panic-restriction lints target the
// long-running server, not the test harness.
#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use tower::ServiceExt; // for `oneshot`

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn www_redirects_to_apex_preserving_path() -> TestResult {
        let req = Request::builder()
            .uri("/about")
            .header(HOST, "www.thombruce.com")
            .body(Body::empty())?;
        let res = app().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
        let location = res.headers().get("location").and_then(|v| v.to_str().ok());
        assert_eq!(location, Some("https://thombruce.com/about"));
        Ok(())
    }

    #[tokio::test]
    async fn apex_host_passes_through() -> TestResult {
        let req = Request::builder()
            .uri("/")
            .header(HOST, "thombruce.com")
            .body(Body::empty())?;
        let res = app().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn stylesheet_served_as_css() -> TestResult {
        let req = Request::builder().uri("/style.css").body(Body::empty())?;
        let res = app().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);
        let ct = res
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok());
        assert_eq!(ct, Some("text/css; charset=utf-8"));
        Ok(())
    }

    #[tokio::test]
    async fn unknown_route_is_404() -> TestResult {
        let req = Request::builder().uri("/nope").body(Body::empty())?;
        let res = app().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        Ok(())
    }
}
