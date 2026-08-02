use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::response::Html;
use axum::routing::{MethodRouter, get};

use crate::content::Page;
use crate::view;

// /count — a visit counter, rendered per request. Shared mutable runtime state
// (an atomic) that the static pages can't hold; the number climbs on refresh.
// Ephemeral: resets when the process restarts (redeploy / scale-to-zero). The
// counter lives in this closure, so each `route()` call gets its own.
pub fn route(pages: Arc<Vec<Page>>) -> MethodRouter {
    let counter = Arc::new(AtomicU64::new(0));
    get(move || {
        // fetch_add returns the previous value; +1 (saturating, to satisfy the
        // arithmetic-side-effects lint) is this visit's count.
        let count = counter.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        let html = view::count_page(count, pages.as_slice());
        async move { Html(html) }
    })
}
