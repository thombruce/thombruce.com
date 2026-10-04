//! `project`: the title, a line with its `language` and a link to its GitHub
//! `repo` (each if present), then the body (leading H1 dropped).

use std::collections::BTreeMap;

use maud::{Markup, PreEscaped, html};
use ratatui::text::{Line, Text};

use super::{DocLayout, body_lines, dim_line, title_line};
use crate::content::{Doc, strip_leading_h1};
use crate::markdown;

pub const LAYOUT: DocLayout = DocLayout {
    name: "project",
    html,
    text: Some(text),
};

fn html(doc: &Doc) -> Markup {
    html! {
        article {
            h1 { (doc.title) }
            (details_html(&doc.meta))
            (PreEscaped(markdown::html(&strip_leading_h1(&doc.body))))
        }
    }
}

// Bold title, dimmed "language · repo", then the body.
fn text(doc: &Doc) -> Text<'static> {
    let mut lines = vec![title_line(&doc.title)];
    if let Some(details) = details_text(&doc.meta) {
        lines.push(dim_line(details));
    }
    lines.push(Line::default());
    lines.extend(body_lines(&strip_leading_h1(&doc.body)));
    Text::from(lines)
}

// "Rust · thombruce/inkpot", the repo linked to GitHub; empty if neither key
// is set. Shared with the `projects` listing.
pub(super) fn details_html(meta: &BTreeMap<String, String>) -> Markup {
    let language = meta.get("language");
    let repo = meta.get("repo");
    html! {
        @if language.is_some() || repo.is_some() {
            p {
                @if let Some(language) = language { (language) }
                @if language.is_some() && repo.is_some() { " · " }
                @if let Some(repo) = repo {
                    a href=(github_url(repo)) { (repo) }
                }
            }
        }
    }
}

// The same details as plain text; None if neither key is set.
pub(super) fn details_text(meta: &BTreeMap<String, String>) -> Option<String> {
    let parts: Vec<&str> = [meta.get("language"), meta.get("repo")]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

// `owner/name` → its GitHub URL. Always prefixed, so frontmatter can't
// inject another scheme into the href.
fn github_url(repo: &str) -> String {
    format!("https://github.com/{repo}")
}
