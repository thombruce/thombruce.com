//! Site content, discovered from the `content/` tree embedded at compile time.
//!
//! Document-driven: the directory structure *is* the site.
//! - `content/index.md` → `/`, `content/about.md` → `/about`.
//! - Every subdirectory is a listing: `content/blog/` → `/blog`, listing its
//!   docs and subdirectories; `content/blog/hello.md` → `/blog/hello`.
//! - A subdirectory's `index.md` is optional and supplies the listing's
//!   title, nav entry, and intro text. The root has no listing — its
//!   `index.md` is the home page.
//! - `_`/`.`-prefixed files and directories (drafts, partials, dotfiles) and
//!   non-Markdown files are ignored.
//!
//! Frontmatter is optional. Built-in keys: `title` (falls back to the first
//! `# heading`, then the filename), `nav` + `order` (nav entry), `date`
//! (shown, and sorts a listing newest-first when every entry has one). All
//! keys are kept in `Doc::meta`. Both frontends render the same `Content` —
//! HTTP to HTML, SSH to text. Malformed frontmatter, bad names, and route
//! clashes fail loudly at startup: the files are authored content, so an error
//! is a bug to surface, not swallow.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use include_dir::{Dir, include_dir};

static CONTENT: Dir = include_dir!("$CARGO_MANIFEST_DIR/content");

pub struct Doc {
    pub path: String,
    pub title: String,
    pub meta: BTreeMap<String, String>,
    pub body: String,
    // The listing this doc sits in; None for docs at the content root.
    pub parent: Option<usize>,
}

impl Doc {
    pub fn date(&self) -> Option<&str> {
        self.meta.get("date").map(String::as_str)
    }
}

// A directory's index page: its intro (the `index.md` body, if any) and entries.
pub struct Listing {
    pub path: String,
    pub title: String,
    pub intro: String,
    pub entries: Vec<Entry>,
    // The enclosing listing; None for directories at the content root.
    pub parent: Option<usize>,
}

// Something a link can open, as an index into `Content::docs`/`listings` — so
// the SSH frontend can open it directly; HTTP uses the path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Doc(usize),
    Listing(usize),
}

pub struct Entry {
    pub title: String,
    pub path: String,
    pub date: Option<String>,
    pub target: Target,
}

pub struct NavLink {
    pub label: String,
    pub path: String,
    pub target: Target,
}

// All content, loaded once at startup and shared by both frontends.
pub struct Content {
    pub docs: Vec<Doc>,
    pub listings: Vec<Listing>,
    // Docs and listings with a `nav` key, sorted by `order`.
    pub nav: Vec<NavLink>,
    // The doc at `/`, if any.
    pub home: Option<usize>,
}

pub fn load() -> Result<Content, String> {
    load_from(&CONTENT)
}

fn load_from(root: &Dir) -> Result<Content, String> {
    let mut loader = Loader::default();
    loader.walk(root, "", None)?;
    let Loader {
        docs,
        listings,
        mut nav,
        ..
    } = loader;
    nav.sort_by(|(a, x), (b, y)| a.cmp(b).then_with(|| x.path.cmp(&y.path)));
    let home = docs.iter().position(|d| d.path == "/");
    Ok(Content {
        docs,
        listings,
        nav: nav.into_iter().map(|(_, link)| link).collect(),
        home,
    })
}

#[derive(Default)]
struct Loader {
    docs: Vec<Doc>,
    listings: Vec<Listing>,
    nav: Vec<(i32, NavLink)>,
    paths: HashSet<String>,
}

impl Loader {
    // Walk one directory. `url` is its route prefix ("" at the root) and
    // `listing` its listing index (None at the root, which has no listing).
    // Returns the entries for that listing.
    fn walk(&mut self, dir: &Dir, url: &str, listing: Option<usize>) -> Result<Vec<Entry>, String> {
        let mut entries = Vec::new();

        for file in dir.files() {
            let Some(name) = segment(file.path(), true)? else {
                continue;
            };
            let source = file.path().display();
            let raw = file
                .contents_utf8()
                .ok_or_else(|| format!("{source}: not valid UTF-8"))?;
            let (meta, body) = parse(raw).map_err(|e| format!("{source}: {e}"))?;
            let title = title(&meta, body, name);

            // A subdirectory's index.md describes its listing rather than
            // being a doc of its own.
            if let (Some(idx), "index") = (listing, name) {
                let target = Target::Listing(idx);
                self.add_nav(&meta, url, target)
                    .map_err(|e| format!("{source}: {e}"))?;
                let l = self
                    .listings
                    .get_mut(idx)
                    .ok_or("listing index out of range")?;
                l.title = title;
                body.clone_into(&mut l.intro);
                continue;
            }

            let path = if name == "index" {
                "/".to_owned()
            } else {
                format!("{url}/{name}")
            };
            self.claim(&path, &source.to_string())?;
            let target = Target::Doc(self.docs.len());
            self.add_nav(&meta, &path, target)
                .map_err(|e| format!("{source}: {e}"))?;
            entries.push(Entry {
                title: title.clone(),
                path: path.clone(),
                date: meta.get("date").cloned(),
                target,
            });
            self.docs.push(Doc {
                path,
                title,
                meta,
                body: body.to_owned(),
                parent: listing,
            });
        }

        for sub in dir.dirs() {
            let Some(name) = segment(sub.path(), false)? else {
                continue;
            };
            let path = format!("{url}/{name}");
            self.claim(&path, &sub.path().display().to_string())?;
            let idx = self.listings.len();
            self.listings.push(Listing {
                path: path.clone(),
                title: name.to_owned(),
                intro: String::new(),
                entries: Vec::new(),
                parent: listing,
            });
            let sub_entries = self.walk(sub, &path, Some(idx))?;
            let l = self
                .listings
                .get_mut(idx)
                .ok_or("listing index out of range")?;
            l.entries = sub_entries;
            entries.push(Entry {
                title: l.title.clone(),
                path,
                date: None,
                target: Target::Listing(idx),
            });
        }

        // Dated listings (e.g. a blog) read newest-first; anything else by
        // filename. ISO-8601 dates sort correctly as plain strings.
        if entries.iter().all(|e| e.date.is_some()) {
            entries.sort_by(|a, b| b.date.cmp(&a.date));
        } else {
            entries.sort_by(|a, b| a.path.cmp(&b.path));
        }
        Ok(entries)
    }

    // Reserve a route, failing if two files claim it (e.g. `blog.md` beside `blog/`).
    fn claim(&mut self, path: &str, source: &str) -> Result<(), String> {
        if self.paths.insert(path.to_owned()) {
            Ok(())
        } else {
            Err(format!("{source}: route {path} is already taken"))
        }
    }

    fn add_nav(
        &mut self,
        meta: &BTreeMap<String, String>,
        path: &str,
        target: Target,
    ) -> Result<(), String> {
        let Some(label) = meta.get("nav") else {
            return Ok(());
        };
        let order = meta
            .get("order")
            .map(|o| o.parse().map_err(|_| format!("order not an integer: {o}")))
            .transpose()?
            .unwrap_or(0);
        self.nav.push((
            order,
            NavLink {
                label: label.clone(),
                path: path.to_owned(),
                target,
            },
        ));
        Ok(())
    }
}

// The route segment for a file (its stem) or directory (its name), or None if
// it isn't content: non-Markdown files, and `_`/`.`-prefixed entries.
fn segment(path: &Path, is_file: bool) -> Result<Option<&str>, String> {
    if is_file && path.extension().and_then(|e| e.to_str()) != Some("md") {
        return Ok(None);
    }
    let name = if is_file {
        path.file_stem()
    } else {
        path.file_name()
    }
    .and_then(|s| s.to_str())
    .ok_or_else(|| format!("{}: name is not valid UTF-8", path.display()))?;
    if name.starts_with(['_', '.']) {
        return Ok(None);
    }
    if !is_valid_slug(name) {
        return Err(format!(
            "{}: '{name}' must be lowercase [a-z0-9-]",
            path.display()
        ));
    }
    Ok(Some(name))
}

// A slug must be URL- and route-safe: non-empty, lowercase ascii letters,
// digits, and hyphens only. Guards against axum route-pattern panics and
// percent-encoding mismatches from odd filenames.
fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

// `title` frontmatter, else the body's first `# heading`, else the filename.
fn title(meta: &BTreeMap<String, String>, body: &str, name: &str) -> String {
    meta.get("title")
        .cloned()
        .or_else(|| {
            body.lines()
                .find_map(|l| l.strip_prefix("# "))
                .map(|t| t.trim().to_owned())
        })
        .unwrap_or_else(|| name.to_owned())
}

// Split optional `---\n…\n---\n` frontmatter (flat `key: value` lines) from
// the Markdown body. No opening delimiter → no frontmatter, all body.
// ponytail: flat key/value only; switch to a YAML parser when lists/nesting
// (e.g. tags) are needed.
fn parse(raw: &str) -> Result<(BTreeMap<String, String>, &str), String> {
    let mut meta = BTreeMap::new();
    let Some(after) = raw.strip_prefix("---\n") else {
        return Ok((meta, raw));
    };
    let (frontmatter, body) = after
        .split_once("\n---\n")
        .ok_or("missing closing frontmatter delimiter (---)")?;
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line is not `key: value`: {line}"))?;
        meta.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    Ok((meta, body))
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{Target, is_valid_slug, load, load_from, parse};
    use include_dir::{Dir, DirEntry, File};

    #[test]
    fn slug_validation_accepts_safe_rejects_unsafe() {
        assert!(is_valid_slug("on-simplicity"));
        assert!(is_valid_slug("post-42"));
        assert!(!is_valid_slug(""), "empty rejected");
        assert!(!is_valid_slug("Caps"), "uppercase rejected");
        assert!(!is_valid_slug("has space"), "space rejected");
        assert!(!is_valid_slug("route{param}"), "axum metachars rejected");
    }

    #[test]
    fn parses_frontmatter_and_body() -> Result<(), String> {
        let (meta, body) = parse("---\ntitle: Hello\ndate: 2025-01-02\nx: y\n---\n# Hi\n")?;
        assert_eq!(meta.get("title").map(String::as_str), Some("Hello"));
        assert_eq!(
            meta.get("x").map(String::as_str),
            Some("y"),
            "unknown keys kept"
        );
        assert_eq!(body, "# Hi\n");
        Ok(())
    }

    #[test]
    fn frontmatter_is_optional() -> Result<(), String> {
        let (meta, body) = parse("# Inkpot\n\nHello.\n")?;
        assert!(meta.is_empty());
        assert_eq!(body, "# Inkpot\n\nHello.\n");
        assert!(parse("---\ntitle: X\n").is_err(), "unclosed frontmatter");
        Ok(())
    }

    static TREE: Dir = Dir::new(
        "",
        &[
            DirEntry::File(File::new(
                "index.md",
                b"---\ntitle: Home\nnav: Home\n---\nHi\n",
            )),
            DirEntry::File(File::new("about.md", b"# About me\n")),
            DirEntry::File(File::new("_draft.md", b"ignored")),
            DirEntry::File(File::new("notes.txt", b"ignored")),
            DirEntry::Dir(Dir::new(
                "blog",
                &[
                    DirEntry::File(File::new(
                        "blog/index.md",
                        b"---\ntitle: Blog\nnav: Blog\norder: 2\n---\n",
                    )),
                    DirEntry::File(File::new("blog/old.md", b"---\ndate: 2024-01-01\n---\n")),
                    DirEntry::File(File::new("blog/new.md", b"---\ndate: 2025-01-01\n---\n")),
                ],
            )),
            DirEntry::Dir(Dir::new(
                "code",
                &[
                    DirEntry::File(File::new("code/zeta.md", b"z")),
                    DirEntry::File(File::new("code/inkpot.md", b"# Inkpot\n")),
                ],
            )),
        ],
    );

    #[test]
    fn routes_and_listings_derive_from_the_tree() -> Result<(), String> {
        let c = load_from(&TREE)?;
        let mut paths: Vec<&str> = c.docs.iter().map(|d| d.path.as_str()).collect();
        paths.sort_unstable();
        assert_eq!(
            paths,
            [
                "/",
                "/about",
                "/blog/new",
                "/blog/old",
                "/code/inkpot",
                "/code/zeta"
            ],
            "drafts, non-Markdown, and listing index.md files are not docs"
        );
        assert_eq!(
            c.home.and_then(|i| c.docs.get(i)).map(|d| d.title.as_str()),
            Some("Home")
        );

        let blog = c
            .listings
            .iter()
            .find(|l| l.path == "/blog")
            .ok_or("no /blog")?;
        assert_eq!(blog.title, "Blog", "listing titled by its index.md");
        let order: Vec<&str> = blog.entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            order,
            ["/blog/new", "/blog/old"],
            "dated entries newest-first"
        );

        let code = c
            .listings
            .iter()
            .find(|l| l.path == "/code")
            .ok_or("no /code")?;
        assert_eq!(code.title, "code", "no index.md: titled by directory name");
        let titles: Vec<&str> = code.entries.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(
            titles,
            ["Inkpot", "zeta"],
            "undated by filename; heading/filename titles"
        );

        let nav: Vec<(&str, &str)> = c
            .nav
            .iter()
            .map(|n| (n.label.as_str(), n.path.as_str()))
            .collect();
        assert_eq!(
            nav,
            [("Home", "/"), ("Blog", "/blog")],
            "nav sorted by order"
        );
        assert!(matches!(
            c.nav.get(1).map(|n| n.target),
            Some(Target::Listing(_))
        ));
        Ok(())
    }

    #[test]
    fn route_clash_fails() {
        static CLASH: Dir = Dir::new(
            "",
            &[
                DirEntry::File(File::new("blog.md", b"x")),
                DirEntry::Dir(Dir::new("blog", &[])),
            ],
        );
        assert!(load_from(&CLASH).is_err());
    }

    #[test]
    fn bad_name_fails() {
        static BAD: Dir = Dir::new("", &[DirEntry::File(File::new("Bad Name.md", b"x"))]);
        assert!(load_from(&BAD).is_err());
    }

    #[test]
    fn real_content_loads() -> Result<(), String> {
        let c = load()?;
        assert!(c.home.is_some(), "content/index.md is the home page");
        assert!(c.docs.iter().any(|d| d.path == "/code/inkpot"));
        Ok(())
    }
}
