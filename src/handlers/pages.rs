use maud::{DOCTYPE, Markup, PreEscaped, html};
use pulldown_cmark::{Parser, html::push_html};

use crate::content::{self, Page};

// Markdown -> HTML string. ponytail: parsed per request; these docs are tiny
// and traffic is low. Cache in a LazyLock if render cost ever shows up.
fn render_html(markdown: &str) -> String {
    let mut out = String::new();
    push_html(&mut out, Parser::new(markdown));
    out
}

// Shared page shell (doctype, head, nav chrome) wrapping the rendered body.
fn layout(page: &Page) -> Markup {
    let body = render_html(page.markdown);
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (page.title) " · Thom Bruce" }
            }
            body {
                nav {
                    a href="/" { "Home" }
                    " · "
                    a href="/about" { "About" }
                }
                main {
                    (PreEscaped(body))
                }
            }
        }
    }
}

pub async fn home() -> Markup {
    layout(&content::HOME)
}

pub async fn about() -> Markup {
    layout(&content::ABOUT)
}
