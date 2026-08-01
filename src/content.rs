//! Site content, discovered from Markdown files embedded at compile time.
//!
//! Each file in `content/pages/` has a `---` frontmatter block (title, nav,
//! order, path) followed by a Markdown body. `load()` parses them into `Page`s
//! that both frontends render — HTTP to HTML, SSH to text. Adding a page is
//! just dropping a file in that directory.

use include_dir::{Dir, include_dir};

static PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/pages");

pub struct Page {
    pub title: String,
    pub nav: String,
    pub order: i32,
    pub path: String,
    pub body: String,
}

// Parse every embedded page and return them sorted by `order`. Fails loudly on
// malformed frontmatter — a page is authored content, so a parse error is a bug
// to surface at startup, not to swallow.
pub fn load() -> Result<Vec<Page>, String> {
    let mut pages = Vec::new();
    for file in PAGES.files() {
        let name = file.path().display();
        let raw = file
            .contents_utf8()
            .ok_or_else(|| format!("{name}: not valid UTF-8"))?;
        let page = parse(raw).map_err(|e| format!("{name}: {e}"))?;
        pages.push(page);
    }
    pages.sort_by_key(|p| p.order);
    Ok(pages)
}

fn parse(raw: &str) -> Result<Page, String> {
    let after = raw
        .strip_prefix("---\n")
        .ok_or("missing opening frontmatter delimiter (---)")?;
    let (frontmatter, body) = after
        .split_once("\n---\n")
        .ok_or("missing closing frontmatter delimiter (---)")?;

    let mut title = None;
    let mut nav = None;
    let mut order = None;
    let mut path = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line is not `key: value`: {line}"))?;
        let value = value.trim().to_owned();
        match key.trim() {
            "title" => title = Some(value),
            "nav" => nav = Some(value),
            "path" => path = Some(value),
            "order" => {
                order = Some(
                    value
                        .parse()
                        .map_err(|_| format!("order not an integer: {value}"))?,
                );
            }
            other => return Err(format!("unknown frontmatter key: {other}")),
        }
    }

    Ok(Page {
        title: title.ok_or("missing frontmatter key: title")?,
        nav: nav.ok_or("missing frontmatter key: nav")?,
        order: order.ok_or("missing frontmatter key: order")?,
        path: path.ok_or("missing frontmatter key: path")?,
        body: body.to_owned(),
    })
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::parse;

    #[test]
    fn parses_frontmatter_and_body() -> Result<(), String> {
        let page =
            parse("---\ntitle: About\nnav: About\norder: 3\npath: /about\n---\n# About\n\nHi.\n")?;
        assert_eq!(page.title, "About");
        assert_eq!(page.nav, "About");
        assert_eq!(page.order, 3);
        assert_eq!(page.path, "/about");
        assert_eq!(page.body, "# About\n\nHi.\n");
        Ok(())
    }

    #[test]
    fn rejects_missing_frontmatter() {
        assert!(parse("# No frontmatter\n").is_err());
    }

    #[test]
    fn rejects_missing_key() {
        assert!(parse("---\ntitle: X\nnav: X\norder: 0\n---\nbody\n").is_err());
    }
}
