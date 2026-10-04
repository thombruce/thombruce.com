//! Layouts: named templates that render a page's body, selected by the
//! `layout` / `default_layout` / `default_index_layout` frontmatter keys.
//!
//! One file per layout, holding **both** versions side by side: HTML (maud),
//! which `view.rs` wraps in the page shell, and terminal text (ratatui), which
//! `ssh.rs` puts on screen. A layout without a terminal version falls back to
//! the built-in one over SSH, so it can never break the SSH frontend.
//!
//! To add a layout: write `src/layouts/<name>.rs` declaring a `LAYOUT`, then
//! add its `mod` line and its entry in `DOC` or `INDEX` below. Content may
//! only select names listed here (checked at startup).
//! ponytail: registered by hand; a build.rs or `inventory` could discover
//! layout files, and registration needs rethinking once the engine is split
//! from this site (layouts would come from the site, not the engine).

mod default;
mod post;
mod project;
mod projects;

use maud::{Markup, PreEscaped, html};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};

use crate::content::{Content, Doc, Entry, Listing};
use crate::markdown;

// A layout for docs.
pub struct DocLayout {
    pub name: &'static str,
    pub html: fn(&Doc) -> Markup,
    // Terminal version; None uses the built-in one over SSH.
    pub text: Option<fn(&Doc) -> Text<'static>>,
}

// A layout for listings. Over SSH a listing keeps its numbered entries (the
// digit keys select them), so its terminal version can't replace the screen:
// it adds detail lines under each entry instead.
pub struct IndexLayout {
    pub name: &'static str,
    pub html: fn(&Listing, &Content) -> Markup,
    pub detail: Option<fn(&Entry, &Content) -> Vec<String>>,
}

static DOC: [DocLayout; 2] = [post::LAYOUT, project::LAYOUT];
static INDEX: [IndexLayout; 1] = [projects::LAYOUT];

// Names content may select, for validating it at load time.
pub fn doc_names() -> Vec<&'static str> {
    DOC.iter().map(|l| l.name).collect()
}

pub fn index_names() -> Vec<&'static str> {
    INDEX.iter().map(|l| l.name).collect()
}

fn doc_layout(doc: &Doc) -> Option<&'static DocLayout> {
    DOC.iter().find(|l| Some(l.name) == doc.layout.as_deref())
}

fn index_layout(listing: &Listing) -> Option<&'static IndexLayout> {
    INDEX
        .iter()
        .find(|l| Some(l.name) == listing.layout.as_deref())
}

// A doc's body as HTML, by its layout or the built-in.
pub fn doc_html(doc: &Doc) -> Markup {
    doc_layout(doc).map_or_else(|| default::doc_html(doc), |l| (l.html)(doc))
}

// A doc's body as terminal text, by its layout's terminal version or the
// built-in.
pub fn doc_text(doc: &Doc) -> Text<'static> {
    doc_layout(doc)
        .and_then(|l| l.text)
        .map_or_else(|| default::doc_text(doc), |text| text(doc))
}

// A listing's body as HTML, by its layout or the built-in.
pub fn listing_html(listing: &Listing, content: &Content) -> Markup {
    index_layout(listing).map_or_else(
        || default::listing_html(listing, content),
        |l| (l.html)(listing, content),
    )
}

// Extra lines under a listing entry over SSH, from the listing's layout.
pub fn entry_detail(listing: &Listing, entry: &Entry, content: &Content) -> Vec<String> {
    index_layout(listing)
        .and_then(|l| l.detail)
        .map(|detail| detail(entry, content))
        .unwrap_or_default()
}

// Helpers shared by the layouts.

// A listing's intro, or its title as a heading if it has none.
fn listing_intro(listing: &Listing) -> Markup {
    html! {
        @if listing.intro.trim().is_empty() {
            h1 { (listing.title) }
        } @else {
            (PreEscaped(markdown::html(&listing.intro)))
        }
    }
}

// A terminal title line: bold.
fn title_line(title: &str) -> Line<'static> {
    Line::styled(
        title.to_owned(),
        Style::default().add_modifier(Modifier::BOLD),
    )
}

// A secondary terminal line (date, details): dimmed.
fn dim_line(text: String) -> Line<'static> {
    Line::styled(text, Style::default().add_modifier(Modifier::DIM))
}

// Markdown as terminal text lines, to follow a layout's header lines.
fn body_lines(markdown: &str) -> Vec<Line<'static>> {
    Text::from(markdown::text(markdown)).lines
}
