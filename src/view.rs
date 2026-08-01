use maud::{DOCTYPE, Markup, PreEscaped, html};
use pulldown_cmark::{Parser, html::push_html};

use crate::content::Page;

// Nav links, in page order — generated from the discovered pages so adding a
// page updates the nav everywhere with no edit here.
fn nav_links(pages: &[Page]) -> Vec<(&str, &str)> {
    pages
        .iter()
        .map(|p| (p.path.as_str(), p.nav.as_str()))
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
