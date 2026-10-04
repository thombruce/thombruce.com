// ponytail: all HTML views share this file. Fine at this size; split into a
// view/ submodule (or co-locate per-page views with their handlers) if it grows
// unwieldy — trigger and options tracked in issue #8.
use maud::{DOCTYPE, Markup, PreEscaped, html};
use pulldown_cmark::{Parser, html::push_html};

use crate::content::{Doc, Listing, NavLink, strip_leading_h1};

// Markdown -> HTML string.
fn markdown(body: &str) -> String {
    let mut out = String::new();
    push_html(&mut out, Parser::new(body));
    out
}

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

// A doc rendered to a full HTML document: its date (if any), then the body.
pub fn doc_page(doc: &Doc, nav: &[NavLink]) -> String {
    let template = lookup(&DOC_TEMPLATES, doc.layout.as_deref()).unwrap_or(default_doc);
    shell(&doc.title, &template(doc), nav).into_string()
}

// Templates render a page's body; `shell` wraps every page. Content selects
// one by name (`layout`, `default_layout`, `default_index_layout`); the names
// are derived from these tables, so content can't name a template that
// doesn't exist (checked at startup). A template is any function returning
// Markup — a Sailfish template can sit here too, wrapping its rendered String.
type DocTemplate = fn(&Doc) -> Markup;
type IndexTemplate = fn(&Listing) -> Markup;
const DOC_TEMPLATES: [(&str, DocTemplate); 1] = [("post", post)];
const INDEX_TEMPLATES: [(&str, IndexTemplate); 0] = [];

pub fn doc_layouts() -> Vec<&'static str> {
    DOC_TEMPLATES.iter().map(|(name, _)| *name).collect()
}

pub fn index_layouts() -> Vec<&'static str> {
    INDEX_TEMPLATES.iter().map(|(name, _)| *name).collect()
}

// The template registered under `name`, if any.
fn lookup<T: Copy>(table: &[(&str, T)], name: Option<&str>) -> Option<T> {
    let name = name?;
    table.iter().find(|(n, _)| *n == name).map(|(_, t)| *t)
}

// Built-in doc template: the date (if any), then the body.
fn default_doc(doc: &Doc) -> Markup {
    html! {
        @if let Some(date) = doc.date() {
            p { time datetime=(date) { (date) } }
        }
        (PreEscaped(markdown(&doc.body)))
    }
}

// `post`: the title, a date byline under it, then the body (its leading H1
// dropped so the title isn't repeated).
fn post(doc: &Doc) -> Markup {
    html! {
        article {
            h1 { (doc.title) }
            @if let Some(date) = doc.date() {
                p { time datetime=(date) { (date) } }
            }
            (PreEscaped(markdown(strip_leading_h1(&doc.body))))
        }
    }
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

// A directory listing: its intro (or title, if it has none), then its entries
// — title, plus date when present. Entries arrive pre-sorted.
pub fn listing_page(listing: &Listing, nav: &[NavLink]) -> String {
    let template = lookup(&INDEX_TEMPLATES, listing.layout.as_deref()).unwrap_or(default_listing);
    shell(&listing.title, &template(listing), nav).into_string()
}

// Built-in listing template: the intro (or title), then the entries.
fn default_listing(listing: &Listing) -> Markup {
    html! {
            @if listing.intro.trim().is_empty() {
                h1 { (listing.title) }
            } @else {
                (PreEscaped(markdown(&listing.intro)))
            }
            @if listing.entries.is_empty() {
                p { "Nothing here yet." }
            } @else {
                ul {
                    @for entry in &listing.entries {
                        li {
                            a href=(entry.path) { (entry.title) }
                            @if let Some(date) = &entry.date {
                                " — "
                                time datetime=(date) { (date) }
                            }
                        }
                    }
                }
            }
    }
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
