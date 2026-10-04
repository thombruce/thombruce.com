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
//! keys are kept in `Doc::meta`.
//!
//! A directory's `index.md` may also declare a schema for the docs beneath it
//! — `required` (comma-separated keys), `sort` (`key`/`-key`), `strict`
//! (reject other keys) — each inherited per key from the nearest ancestor that
//! sets it, so the root `index.md` sets site-wide rules. See `Schema`. Both frontends render the same `Content` —
//! HTTP to HTML, SSH to text. Malformed frontmatter, bad names, and route
//! clashes fail loudly at startup: the files are authored content, so an error
//! is a bug to surface, not swallow.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use include_dir::{Dir, include_dir};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

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
    loader.walk(root, "", None, &Schema::default())?;
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
    // Walk one directory. `url` is its route prefix ("" at the root),
    // `listing` its listing index (None at the root, which has no listing),
    // and `inherited` the schema in force from its ancestors. Returns the
    // entries for that listing.
    fn walk(
        &mut self,
        dir: &Dir,
        url: &str,
        listing: Option<usize>,
        inherited: &Schema,
    ) -> Result<Vec<Entry>, String> {
        // Parse every file first: the directory's index.md carries the schema
        // its siblings are checked against, so it must be read before them.
        let mut index = None;
        let mut files = Vec::new();
        for file in dir.files() {
            let Some(name) = segment(file.path(), true)? else {
                continue;
            };
            let source = file.path().display().to_string();
            let raw = file
                .contents_utf8()
                .ok_or_else(|| format!("{source}: not valid UTF-8"))?;
            let (meta, body) = parse(raw).map_err(|e| format!("{source}: {e}"))?;
            if name == "index" {
                index = Some((source, meta, body));
            } else {
                files.push((name, source, meta, body));
            }
        }

        let mut schema = inherited.clone();
        if let Some((source, mut meta, body)) = index {
            let at = |e| format!("{source}: {e}");
            schema = Schema::take(&mut meta).map_err(at)?.over(inherited);
            match listing {
                // A subdirectory's index.md describes its listing rather
                // than being a doc of its own.
                Some(idx) => {
                    self.add_nav(&meta, url, Target::Listing(idx)).map_err(at)?;
                    let l = self
                        .listings
                        .get_mut(idx)
                        .ok_or("listing index out of range")?;
                    // Untitled index.md keeps the directory name, not "index".
                    l.title = title(&meta, &body, &l.title);
                    l.intro = body;
                }
                // The root index.md is the home page (and is in no listing).
                None => {
                    self.add_doc("/".to_owned(), "index", meta, body, None, &source)?;
                }
            }
        }

        let mut entries = Vec::new();
        for (name, source, meta, body) in files {
            schema.check(&meta).map_err(|e| format!("{source}: {e}"))?;
            let path = format!("{url}/{name}");
            let entry = self.add_doc(path, name, meta, body, listing, &source)?;
            entries.push(entry);
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
            let sub_entries = self.walk(sub, &path, Some(idx), &schema)?;
            let l = self
                .listings
                .get_mut(idx)
                .ok_or("listing index out of range")?;
            l.entries = sub_entries;
            entries.push((
                Entry {
                    title: l.title.clone(),
                    path,
                    date: None,
                    target: Target::Listing(idx),
                },
                BTreeMap::new(),
            ));
        }

        schema.sort(&mut entries);
        Ok(entries.into_iter().map(|(entry, _)| entry).collect())
    }

    // Register a doc (route, nav, the doc itself) and return its listing
    // entry, paired with its frontmatter for sorting.
    fn add_doc(
        &mut self,
        path: String,
        name: &str,
        meta: BTreeMap<String, String>,
        body: String,
        listing: Option<usize>,
        source: &str,
    ) -> Result<(Entry, BTreeMap<String, String>), String> {
        self.claim(&path, source)?;
        let target = Target::Doc(self.docs.len());
        self.add_nav(&meta, &path, target)
            .map_err(|e| format!("{source}: {e}"))?;
        let title = title(&meta, &body, name);
        let entry = Entry {
            title: title.clone(),
            path: path.clone(),
            date: meta.get("date").cloned(),
            target,
        };
        self.docs.push(Doc {
            path,
            title,
            meta: meta.clone(),
            body,
            parent: listing,
        });
        Ok((entry, meta))
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

// Frontmatter keys every doc may use, strict schema or not.
const BUILT_IN_KEYS: [&str; 4] = ["title", "nav", "order", "date"];
// Keys that configure a directory's schema; only valid in its index.md.
const SCHEMA_KEYS: [&str; 3] = ["required", "sort", "strict"];

// A directory's schema, set in its index.md. Each key is None when unset, so
// it inherits from the nearest ancestor that sets it (see `over`); an explicit
// empty `required:`/`sort:` or `strict: false` clears an inherited rule. Set in
// the root index.md, a key applies site-wide.
#[derive(Clone, Default)]
struct Schema {
    // Frontmatter keys every doc must have.
    required: Option<Vec<String>>,
    // Key to sort the listing by, `-`-prefixed for descending; empty = filename.
    sort: Option<String>,
    // Reject keys outside `required` and the built-ins (catches typos).
    strict: Option<bool>,
}

impl Schema {
    // Remove the schema keys from an index.md's frontmatter, so they aren't
    // exposed in `meta`.
    fn take(meta: &mut BTreeMap<String, String>) -> Result<Self, String> {
        let strict = meta
            .remove("strict")
            .map(|v| match v.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(format!("strict must be true or false: {v}")),
            })
            .transpose()?;
        Ok(Self {
            required: meta.remove("required").map(|v| {
                v.split(',')
                    .map(str::trim)
                    .filter(|k| !k.is_empty())
                    .map(str::to_owned)
                    .collect()
            }),
            sort: meta.remove("sort"),
            strict,
        })
    }

    // This schema, with any unset key taken from `parent`.
    fn over(self, parent: &Self) -> Self {
        Self {
            required: self.required.or_else(|| parent.required.clone()),
            sort: self.sort.or_else(|| parent.sort.clone()),
            strict: self.strict.or(parent.strict),
        }
    }

    // Check a doc's frontmatter against the schema.
    fn check(&self, meta: &BTreeMap<String, String>) -> Result<(), String> {
        if let Some(key) = meta.keys().find(|k| SCHEMA_KEYS.contains(&k.as_str())) {
            return Err(format!("`{key}` is only valid in a directory's index.md"));
        }
        let required = self.required.as_deref().unwrap_or_default();
        if let Some(key) = required.iter().find(|k| !meta.contains_key(*k)) {
            return Err(format!("missing required frontmatter key: {key}"));
        }
        if self.strict == Some(true)
            && let Some(key) = meta
                .keys()
                .find(|k| !BUILT_IN_KEYS.contains(&k.as_str()) && !required.contains(k))
        {
            return Err(format!("unknown frontmatter key (strict schema): {key}"));
        }
        Ok(())
    }

    // Sort listing entries by the schema's `sort` key: entries missing the key
    // last, ties by path. Unset anywhere → newest-first when every entry is
    // dated, else by filename (paths share the directory prefix).
    fn sort(&self, entries: &mut [(Entry, BTreeMap<String, String>)]) {
        let spec = self.sort.as_deref().unwrap_or_else(|| {
            if entries.iter().all(|(e, _)| e.date.is_some()) {
                "-date"
            } else {
                ""
            }
        });
        let (key, descending) = spec.strip_prefix('-').map_or((spec, false), |k| (k, true));
        entries.sort_by(|(a, am), (b, bm)| {
            // `title` sorts on the resolved title (heading/filename fallback,
            // and subdirectory titles), not just frontmatter.
            let value = |e: &'_ Entry, m: &'_ BTreeMap<String, String>| {
                if key == "title" {
                    Some(e.title.clone())
                } else {
                    m.get(key).cloned()
                }
            };
            let by_key = match (value(a, am), value(b, bm)) {
                (Some(x), Some(y)) => {
                    let o = compare_values(&x, &y);
                    if descending { o.reverse() } else { o }
                }
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };
            by_key.then_with(|| a.path.cmp(&b.path))
        });
    }
}

// Numbers compare numerically (so 10 sorts after 9) and before any text;
// text compares as a string, which is also correct for ISO-8601 dates. Must be
// a total order: `sort_by` may panic on an inconsistent comparison, and mixing
// numeric and string comparison without ranking one first isn't transitive.
fn compare_values(a: &str, b: &str) -> Ordering {
    match (a.parse::<i64>(), b.parse::<i64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
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

// `title` frontmatter, else the body's first H1 heading, else `fallback`.
// The heading is found by the Markdown parser, so a `# comment` in a code
// fence isn't mistaken for it.
fn title(meta: &BTreeMap<String, String>, body: &str, fallback: &str) -> String {
    meta.get("title")
        .cloned()
        .or_else(|| first_h1(body))
        .unwrap_or_else(|| fallback.to_owned())
}

fn first_h1(body: &str) -> Option<String> {
    let mut events = Parser::new(body);
    events.find(|e| {
        matches!(
            e,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            })
        )
    })?;
    let text: String = events
        .take_while(|e| !matches!(e, Event::End(TagEnd::Heading(_))))
        .filter_map(|e| match e {
            Event::Text(t) | Event::Code(t) => Some(t.into_string()),
            _ => None,
        })
        .collect();
    Some(text.trim().to_owned()).filter(|t| !t.is_empty())
}

// Split optional `---` frontmatter (flat `key: value` lines) from the
// Markdown body. No opening delimiter → no frontmatter, all body. CRLF is
// normalized; the closing `---` may be the last line, with no newline after.
// ponytail: flat key/value only; switch to a YAML parser when lists/nesting
// (e.g. tags) are needed.
fn parse(raw: &str) -> Result<(BTreeMap<String, String>, String), String> {
    let raw = raw.replace("\r\n", "\n");
    let mut meta = BTreeMap::new();
    let Some(after) = raw.strip_prefix("---\n") else {
        return Ok((meta, raw));
    };
    let mut lines = after.split_inclusive('\n');
    for line in lines.by_ref() {
        let line = line.trim();
        if line == "---" {
            return Ok((meta, lines.collect()));
        }
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line is not `key: value`: {line}"))?;
        meta.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    Err("missing closing frontmatter delimiter (---)".to_owned())
}

#[cfg(test)]
#[allow(clippy::panic_in_result_fn)]
mod tests {
    use super::{Schema, Target, compare_values, is_valid_slug, load, load_from, parse};
    use include_dir::{Dir, DirEntry, File};
    use std::collections::BTreeMap;

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

    #[test]
    fn frontmatter_edge_cases() -> Result<(), String> {
        let (meta, body) = parse("---\ntitle: Blog\n---")?;
        assert_eq!(
            meta.get("title").map(String::as_str),
            Some("Blog"),
            "no final newline"
        );
        assert_eq!(body, "");
        let (meta, body) = parse("---\n---\nBody\n")?;
        assert!(meta.is_empty(), "empty frontmatter");
        assert_eq!(body, "Body\n");
        let (meta, body) = parse("---\r\ntitle: Win\r\n---\r\nBody\r\n")?;
        assert_eq!(meta.get("title").map(String::as_str), Some("Win"), "CRLF");
        assert_eq!(body, "Body\n");
        Ok(())
    }

    #[test]
    fn title_ignores_code_fences() {
        let meta = std::collections::BTreeMap::new();
        let body = "```sh\n# install deps\n```\n\n# Real *Title*\n";
        assert_eq!(super::title(&meta, body, "file"), "Real Title");
        assert_eq!(super::title(&meta, "No heading.", "file"), "file");
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
                    // Untitled, and no final newline after the closing ---.
                    DirEntry::File(File::new("code/index.md", b"---\norder: 9\n---")),
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
        assert_eq!(
            code.title, "code",
            "untitled index.md: titled by directory name"
        );
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

    fn meta(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn schema_checks_required_strict_and_reserved_keys() -> Result<(), String> {
        let schema = Schema::take(&mut meta(&[
            ("required", "title, date"),
            ("strict", "true"),
        ]))?;
        assert!(
            schema
                .check(&meta(&[("title", "T"), ("date", "D"), ("nav", "N")]))
                .is_ok()
        );
        assert!(
            schema.check(&meta(&[("title", "T")])).is_err(),
            "missing date"
        );
        assert!(
            schema
                .check(&meta(&[("title", "T"), ("date", "D"), ("dtae", "x")]))
                .is_err(),
            "strict rejects typos"
        );
        assert!(
            Schema::default().check(&meta(&[("sort", "x")])).is_err(),
            "schema keys only in index.md"
        );
        assert!(Schema::take(&mut meta(&[("strict", "yes")])).is_err());
        Ok(())
    }

    #[test]
    fn schema_inherits_per_key_and_clears_explicitly() -> Result<(), String> {
        let parent = Schema::take(&mut meta(&[("required", "date"), ("strict", "true")]))?;
        let child = Schema::take(&mut meta(&[("sort", "title")]))?.over(&parent);
        assert_eq!(child.required, Some(vec!["date".to_owned()]), "inherited");
        assert_eq!(child.sort.as_deref(), Some("title"), "overridden");
        let cleared =
            Schema::take(&mut meta(&[("required", ""), ("strict", "false")]))?.over(&parent);
        assert!(
            cleared.check(&meta(&[("anything", "x")])).is_ok(),
            "rules cleared"
        );
        Ok(())
    }

    #[test]
    fn numbers_compare_numerically() {
        assert!(compare_values("9", "10").is_lt());
        assert!(compare_values("2025-01-02", "2025-01-10").is_lt());
        // Mixed values stay transitive: 2 < 10 < 1a (numbers before text).
        assert!(compare_values("10", "1a").is_lt());
        assert!(compare_values("2", "1a").is_lt());
    }

    #[test]
    fn date_is_built_in_under_strict() -> Result<(), String> {
        let schema = Schema::take(&mut meta(&[("strict", "true")]))?;
        assert!(
            schema
                .check(&meta(&[("title", "T"), ("date", "D")]))
                .is_ok()
        );
        Ok(())
    }

    static SCHEMA_TREE: Dir = Dir::new(
        "",
        &[DirEntry::Dir(Dir::new(
            "blog",
            &[
                DirEntry::File(File::new(
                    "blog/index.md",
                    b"---\nrequired: title, date\nsort: -date\nstrict: true\n---\n",
                )),
                DirEntry::File(File::new(
                    "blog/a.md",
                    b"---\ntitle: A\ndate: 2024-01-01\n---\n",
                )),
                DirEntry::File(File::new(
                    "blog/b.md",
                    b"---\ntitle: B\ndate: 2025-01-01\n---\n",
                )),
                DirEntry::Dir(Dir::new(
                    "blog/2026",
                    &[
                        DirEntry::File(File::new("blog/2026/index.md", b"---\nsort: title\n---\n")),
                        DirEntry::File(File::new(
                            "blog/2026/y.md",
                            b"---\ntitle: Zed\ndate: 2026-02-01\n---\n",
                        )),
                        DirEntry::File(File::new(
                            "blog/2026/z.md",
                            b"---\ntitle: Alpha\ndate: 2026-01-01\n---\n",
                        )),
                    ],
                )),
                DirEntry::Dir(Dir::new(
                    "blog/drafts",
                    &[
                        DirEntry::File(File::new(
                            "blog/drafts/index.md",
                            b"---\nrequired:\nstrict: false\nsort: title\n---\n",
                        )),
                        DirEntry::File(File::new(
                            "blog/drafts/idea.md",
                            b"---\nmood: vague\n---\n",
                        )),
                        // Titled only by its heading; sorts by it, not as missing.
                        DirEntry::File(File::new("blog/drafts/zz.md", b"# Aardvark\n")),
                    ],
                )),
            ],
        ))],
    );

    #[test]
    fn schema_applies_through_the_tree() -> Result<(), String> {
        let c = load_from(&SCHEMA_TREE)?;
        let entries = |path: &str| -> Result<Vec<String>, String> {
            let l = c
                .listings
                .iter()
                .find(|l| l.path == path)
                .ok_or_else(|| path.to_owned())?;
            Ok(l.entries.iter().map(|e| e.path.clone()).collect())
        };
        assert_eq!(
            entries("/blog")?,
            ["/blog/b", "/blog/a", "/blog/2026", "/blog/drafts"],
            "-date; undated subdirectories last"
        );
        assert_eq!(
            entries("/blog/2026")?,
            ["/blog/2026/z", "/blog/2026/y"],
            "sort overridden to title (Alpha, Zed)"
        );
        // drafts/idea.md has no title/date and an unknown key: loads only
        // because drafts/index.md clears the inherited rules.
        assert_eq!(
            entries("/blog/drafts")?,
            ["/blog/drafts/zz", "/blog/drafts/idea"],
            "sort: title uses resolved titles (Aardvark, idea)"
        );
        Ok(())
    }

    #[test]
    fn root_index_schema_applies_site_wide() {
        static MISSING: Dir = Dir::new(
            "",
            &[
                DirEntry::File(File::new("index.md", b"---\nrequired: date\n---\n")),
                DirEntry::Dir(Dir::new(
                    "notes",
                    &[DirEntry::File(File::new("notes/x.md", b"undated"))],
                )),
            ],
        );
        let err = load_from(&MISSING).err().unwrap_or_default();
        assert!(
            err.contains("notes/x.md") && err.contains("date"),
            "site-wide rule: {err}"
        );
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
