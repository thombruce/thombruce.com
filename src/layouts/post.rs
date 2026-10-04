//! `post`: the title, a date byline under it, then the body. The body's
//! leading H1 is dropped so the title isn't repeated.

use maud::{Markup, PreEscaped, html};
use ratatui::text::{Line, Text};

use super::{DocLayout, body_lines, dim_line, title_line};
use crate::content::{Doc, strip_leading_h1};
use crate::markdown;

pub const LAYOUT: DocLayout = DocLayout {
    name: "post",
    html,
    text: Some(text),
};

fn html(doc: &Doc) -> Markup {
    html! {
        article {
            h1 { (doc.title) }
            @if let Some(date) = doc.date() {
                p { time datetime=(date) { (date) } }
            }
            (PreEscaped(markdown::html(&strip_leading_h1(&doc.body))))
        }
    }
}

// Bold title, dimmed date, then the body.
fn text(doc: &Doc) -> Text<'static> {
    let mut lines = vec![title_line(&doc.title)];
    if let Some(date) = doc.date() {
        lines.push(dim_line(date.to_owned()));
    }
    lines.push(Line::default());
    lines.extend(body_lines(&strip_leading_h1(&doc.body)));
    Text::from(lines)
}
