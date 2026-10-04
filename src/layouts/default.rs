//! The built-in rendering, used when a page has no layout.

use maud::{Markup, PreEscaped, html};
use ratatui::text::Text;

use super::listing_intro;
use crate::content::{Content, Doc, Listing};
use crate::markdown;

// The date (if any), then the body.
pub fn doc_html(doc: &Doc) -> Markup {
    html! {
        @if let Some(date) = doc.date() {
            p { time datetime=(date) { (date) } }
        }
        (PreEscaped(markdown::html(&doc.body)))
    }
}

pub fn doc_text(doc: &Doc) -> Text<'static> {
    let body = markdown::text(&doc.body);
    Text::from(match doc.date() {
        Some(date) => format!("{date}\n\n{body}"),
        None => body,
    })
}

// The intro (or title), then a list of entries with their dates.
pub fn listing_html(listing: &Listing, _content: &Content) -> Markup {
    html! {
        (listing_intro(listing))
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
