// HTML pages: the shared shell, content pages (their bodies come from
// layouts/), the dynamic pages, and the 404.
// ponytail: these views share one file. Page bodies moved to layouts/ (#26);
// split the rest further if it grows unwieldy — options tracked in issue #8.
use maud::{DOCTYPE, Markup, html};

use crate::content::{Content, Doc, Listing, NavLink};
use crate::layouts;

// Shared HTML shell: doctype, head (stylesheet), generated nav, main content.
// The nav is derived from content (`nav` frontmatter), so it stays in sync.
fn shell(title: &str, body: &Markup, nav: &[NavLink]) -> Markup {
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
                    @for (i, link) in nav.iter().enumerate() {
                        @if i > 0 { " · " }
                        a href=(link.path) { (link.label) }
                    }
                }
                main {
                    (body)
                }
            }
        }
    }
}

// A doc as a full HTML document: its layout's body in the shell.
pub fn doc_page(doc: &Doc, nav: &[NavLink]) -> String {
    shell(&doc.title, &layouts::doc_html(doc), nav).into_string()
}

// /count — the visit counter, rendered fresh each request. Demonstrates
// server-side state: the number changes on refresh, which no static page can.
pub fn count_page(count: u64, nav: &[NavLink]) -> String {
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
        nav,
    )
    .into_string()
}

// /echo — the request reflected back, rendered server-side. Demonstrates
// request-awareness: static pages ignore the request entirely.
pub fn echo_page(
    method: &str,
    path: &str,
    headers: &[(String, String)],
    nav: &[NavLink],
) -> String {
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
        nav,
    )
    .into_string()
}

// A directory listing as a full HTML document: its layout's body in the shell.
pub fn listing_page(listing: &Listing, content: &Content) -> String {
    shell(
        &listing.title,
        &layouts::listing_html(listing, content),
        &content.nav,
    )
    .into_string()
}

// The 404 document, sharing the same shell and nav.
pub fn not_found(nav: &[NavLink]) -> String {
    shell(
        "Not Found",
        &html! {
            h1 { "404" }
            p { "That page doesn’t exist." }
            p { a href="/" { "Go home" } }
        },
        nav,
    )
    .into_string()
}
