//! `projects` (a listing): the intro, then each entry as its own section with
//! its title, language and repo (for docs), and a one-paragraph summary.

use maud::{Markup, html};

use super::project::{details_html, details_text};
use super::{IndexLayout, listing_intro};
use crate::content::{Content, Entry, Listing};

pub const LAYOUT: IndexLayout = IndexLayout {
    name: "projects",
    html,
    detail: Some(detail),
};

fn html(listing: &Listing, content: &Content) -> Markup {
    html! {
        (listing_intro(listing))
        @if listing.entries.is_empty() {
            p { "Nothing here yet." }
        }
        @for entry in &listing.entries {
            section {
                h2 { a href=(entry.path) { (entry.title) } }
                @if let Some(meta) = content.doc_meta(entry.target) {
                    (details_html(meta))
                }
                @if let Some(summary) = content.summary(entry.target) {
                    p { (summary) }
                }
            }
        }
    }
}

// Over SSH, under each numbered entry: "language · repo" (for docs), then
// the summary.
fn detail(entry: &Entry, content: &Content) -> Vec<String> {
    content
        .doc_meta(entry.target)
        .and_then(details_text)
        .into_iter()
        .chain(content.summary(entry.target))
        .collect()
}
