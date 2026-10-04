//! Markdown rendering, shared by the layouts, the HTML views, and the SSH
//! frontend: the same source rendered to HTML or to plain terminal text.

use pulldown_cmark::{Event, Parser, Tag, TagEnd, html::push_html};

// Markdown -> HTML string.
pub fn html(markdown: &str) -> String {
    let mut out = String::new();
    push_html(&mut out, Parser::new(markdown));
    out
}

// Markdown -> plain text. Drops syntax markers; blocks separated by blank
// lines, list items prefixed with a bullet.
// ponytail: ordered lists render as bullets too; number them if it matters.
pub fn text(markdown: &str) -> String {
    let mut out = String::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Text(t) | Event::Code(t) => out.push_str(&t),
            Event::Start(Tag::Item) => out.push_str("- "),
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Item | TagEnd::List(_)) => {
                out.push('\n');
            }
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => out.push_str("\n\n"),
            _ => {}
        }
    }
    out.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::text;

    #[test]
    fn text_strips_syntax_and_bullets_lists() {
        let out = text("# Title\n\nHello\n\n1. one\n2. two");
        assert!(out.contains("Title"), "heading text kept");
        assert!(!out.contains('#'), "heading marker dropped");
        assert!(out.contains("Hello"));
        assert!(
            out.contains("- one") && out.contains("- two"),
            "items bulleted"
        );
    }
}
