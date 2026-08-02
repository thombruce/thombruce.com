// ponytail: all HTML views share this file. Fine at this size; split into a
// view/ submodule (or co-locate per-page views with their handlers) if it grows
// unwieldy — trigger and options tracked in issue #8.
use maud::{DOCTYPE, Markup, PreEscaped, html};
use pulldown_cmark::{Parser, html::push_html};

use crate::content::{Page, Post};

// Nav links, in page order — generated from the discovered pages, with the Blog
// section appended, so adding a page updates the nav everywhere with no edit here.
fn nav_links(pages: &[Page]) -> Vec<(&str, &str)> {
    pages
        .iter()
        .map(|p| (p.path.as_str(), p.nav.as_str()))
        .chain(std::iter::once(("/blog", "Blog")))
        .collect()
}

// Markdown -> HTML string.
fn markdown(body: &str) -> String {
    let mut out = String::new();
    push_html(&mut out, Parser::new(body));
    out
}

// Shared HTML shell: doctype, head (stylesheet), generated nav, main content.
fn shell(title: &str, body: &Markup, nav: &[(&str, &str)]) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                link rel="stylesheet" href="/style.css";
                title { (title) " · Thom Bruce" }
            }
            body {
                nav {
                    @for (i, (path, label)) in nav.iter().enumerate() {
                        @if i > 0 { " · " }
                        a href=(path) { (label) }
                    }
                }
                main {
                    (body)
                }
            }
        }
    }
}

// A page rendered to a full HTML document (nav reflects all pages).
pub fn render_page(page: &Page, pages: &[Page]) -> String {
    shell(
        &page.title,
        &PreEscaped(markdown(&page.body)),
        &nav_links(pages),
    )
    .into_string()
}

// /count — the visit counter, rendered fresh each request. Demonstrates
// server-side state: the number changes on refresh, which no static page can.
pub fn count_page(count: u64, pages: &[Page]) -> String {
    shell(
        "Count",
        &html! {
            h1 { "Visits" }
            p {
                "This page has been served "
                strong { (count) }
                @if count == 1 { " time" } @else { " times" }
                " since the server last started."
            }
            p { "Refresh — the number goes up. The static pages can’t do that; they’re baked once at startup." }
        },
        &nav_links(pages),
    )
    .into_string()
}

// /echo — the request reflected back, rendered server-side. Demonstrates
// request-awareness: static pages ignore the request entirely.
pub fn echo_page(method: &str, path: &str, headers: &[(String, String)], pages: &[Page]) -> String {
    shell(
        "Echo",
        &html! {
            h1 { "Echo" }
            p { "The server rendered this table from your request:" }
            table {
                tr { th { "Method" } td { (method) } }
                tr { th { "Path" } td { (path) } }
                @for (name, value) in headers {
                    tr { th { (name) } td { (value) } }
                }
            }
        },
        &nav_links(pages),
    )
    .into_string()
}

// /blog — the post index, newest first (posts arrive pre-sorted). Each entry
// links to its /blog/<slug> page.
pub fn blog_index(posts: &[Post], pages: &[Page]) -> String {
    shell(
        "Blog",
        &html! {
            h1 { "Blog" }
            ul {
                @for post in posts {
                    li {
                        a href=(format!("/blog/{}", post.slug)) { (post.title) }
                        " — "
                        time datetime=(post.date) { (post.date) }
                    }
                }
            }
        },
        &nav_links(pages),
    )
    .into_string()
}

// /blog/<slug> — a single post: title, date, then the rendered Markdown body.
pub fn post_page(post: &Post, pages: &[Page]) -> String {
    shell(
        &post.title,
        &html! {
            article {
                h1 { (post.title) }
                p { time datetime=(post.date) { (post.date) } }
                (PreEscaped(markdown(&post.body)))
            }
        },
        &nav_links(pages),
    )
    .into_string()
}

// The 404 document, sharing the same shell and nav.
pub fn not_found(pages: &[Page]) -> String {
    shell(
        "Not Found",
        &html! {
            h1 { "404" }
            p { "That page doesn’t exist." }
            p { a href="/" { "Go home" } }
        },
        &nav_links(pages),
    )
    .into_string()
}
