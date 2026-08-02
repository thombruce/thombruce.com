//! Site content, discovered from Markdown files embedded at compile time.
//!
//! Two sections, both embedded from disk:
//! - `content/pages/` — static pages (`title`, `nav`, `order`, `path` frontmatter).
//! - `content/blog/` — blog posts (`title`, `date` frontmatter; slug from filename).
//!
//! `load()` parses both into a `Content` bundle that both frontends render —
//! HTTP to HTML, SSH to text. Adding a page or post is just dropping a file in
//! the matching directory. Malformed frontmatter fails loudly at startup — the
//! files are authored content, so a parse error is a bug to surface, not swallow.

use include_dir::{Dir, include_dir};

static PAGES: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/pages");
static POSTS: Dir = include_dir!("$CARGO_MANIFEST_DIR/content/blog");

pub struct Page {
    pub title: String,
    pub nav: String,
    pub order: i32,
    pub path: String,
    pub body: String,
}

pub struct Post {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub body: String,
}

// Both content sections, loaded once at startup and shared by both frontends.
pub struct Content {
    pub pages: Vec<Page>,
    pub posts: Vec<Post>,
}

// Discover and parse every page and post.
pub fn load() -> Result<Content, String> {
    Ok(Content {
        pages: load_pages()?,
        posts: load_posts()?,
    })
}

// Parse every embedded page, sorted by `order`.
fn load_pages() -> Result<Vec<Page>, String> {
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

// Parse every embedded post, newest first. The slug comes from the filename;
// ISO-8601 dates sort correctly as plain strings, so no date type is needed.
fn load_posts() -> Result<Vec<Post>, String> {
    let mut posts = Vec::new();
    for file in POSTS.files() {
        let name = file.path().display();
        let slug = file
            .path()
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("{name}: could not derive slug from filename"))?
            .to_owned();
        let raw = file
            .contents_utf8()
            .ok_or_else(|| format!("{name}: not valid UTF-8"))?;
        let post = parse_post(raw, slug).map_err(|e| format!("{name}: {e}"))?;
        posts.push(post);
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(posts)
}

// Split a `---\n…\n---\n` frontmatter block from the Markdown body that follows.
fn split_frontmatter(raw: &str) -> Result<(&str, &str), String> {
    let after = raw
        .strip_prefix("---\n")
        .ok_or("missing opening frontmatter delimiter (---)")?;
    let split = after
        .split_once("\n---\n")
        .ok_or("missing closing frontmatter delimiter (---)")?;
    Ok(split)
}

fn parse(raw: &str) -> Result<Page, String> {
    let (frontmatter, body) = split_frontmatter(raw)?;

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

fn parse_post(raw: &str, slug: String) -> Result<Post, String> {
    let (frontmatter, body) = split_frontmatter(raw)?;

    let mut title = None;
    let mut date = None;

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
            "date" => date = Some(value),
            other => return Err(format!("unknown frontmatter key: {other}")),
        }
    }

    Ok(Post {
        title: title.ok_or("missing frontmatter key: title")?,
        date: date.ok_or("missing frontmatter key: date")?,
        slug,
        body: body.to_owned(),
    })
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{load_posts, parse, parse_post};

    #[test]
    fn parses_post_frontmatter_and_slug() -> Result<(), String> {
        let post = parse_post(
            "---\ntitle: Hello\ndate: 2025-01-02\n---\n# Hello\n\nBody.\n",
            "hello".to_owned(),
        )?;
        assert_eq!(post.title, "Hello");
        assert_eq!(post.date, "2025-01-02");
        assert_eq!(post.slug, "hello");
        assert_eq!(post.body, "# Hello\n\nBody.\n");
        Ok(())
    }

    #[test]
    fn post_rejects_page_only_keys() {
        // `nav`/`order`/`path` are page keys, not post keys.
        assert!(parse_post("---\ntitle: X\norder: 1\n---\nb\n", "x".to_owned()).is_err());
    }

    #[test]
    fn posts_load_newest_first() -> Result<(), String> {
        let posts = load_posts()?;
        assert!(posts.len() >= 2, "demo posts are embedded");
        assert!(
            posts.is_sorted_by(|a, b| a.date >= b.date),
            "posts sorted by date descending"
        );
        Ok(())
    }

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
