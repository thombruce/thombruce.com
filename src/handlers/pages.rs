use maud::{Markup, PreEscaped};
use pulldown_cmark::{Parser, html::push_html};

use crate::content::{self, Page};
use crate::view::shell;

// Markdown -> HTML string. ponytail: parsed per request; these docs are tiny
// and traffic is low. Cache in a LazyLock if render cost ever shows up.
fn render_html(markdown: &str) -> String {
    let mut out = String::new();
    push_html(&mut out, Parser::new(markdown));
    out
}

fn layout(page: &Page) -> Markup {
    shell(page.title, &PreEscaped(render_html(page.markdown)))
}

pub async fn home() -> Markup {
    layout(&content::HOME)
}

pub async fn about() -> Markup {
    layout(&content::ABOUT)
}

pub async fn colophon() -> Markup {
    layout(&content::COLOPHON)
}
