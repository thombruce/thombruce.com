# thombruce.com

Personal website of Thom Bruce, served two ways from one binary:

- **HTTP**: [thombruce.com](https://thombruce.com), HTML rendered by [Axum](https://github.com/tokio-rs/axum) and styled with the classless [drizzle-css](https://crates.io/crates/drizzle-css).
- **SSH**: `ssh thombruce.com`, the same content as a terminal UI built with [russh](https://github.com/Eugeny/russh) and [ratatui](https://ratatui.rs).

Content is plain Markdown in `content/`. The directory structure *is* the site: there are no routes to wire up and no nav to maintain.

## Quick start

```sh
cargo run
```

Then open <http://localhost:3000>, or connect over SSH:

```sh
ssh -p 2222 localhost
```

| Variable       | Default   | Purpose                                                                                     |
| -------------- | --------- | ------------------------------------------------------------------------------------------- |
| `PORT`         | `3000`    | HTTP port                                                                                   |
| `SSH_PORT`     | `2222`    | SSH port                                                                                    |
| `SSH_HOST_KEY` | ephemeral | OpenSSH-format private key; set it in production so the host fingerprint stays stable |

Content is embedded at compile time, so restart `cargo run` after editing it.

## Writing content

### Layout

```
content/index.md        → /              (the home page)
content/about.md        → /about
content/blog/           → /blog          (a listing of everything inside)
content/blog/index.md   → optional: the listing's title, nav entry, and intro
content/blog/hello.md   → /blog/hello
```

- Every Markdown file is a page, and every directory is a listing page with links to its pages and subdirectories, nested as deep as you like.
- Names must be lowercase letters, digits, and hyphens (`on-simplicity.md`).
- Files and directories starting with `_` (drafts, partials) or `.` are ignored, as are non-Markdown files.
- Two files can't claim the same route (`blog.md` beside `blog/`); that's a startup error.

### Frontmatter

Frontmatter is optional. When present, it's a block of flat `key: value` lines:

```markdown
---
title: Hello, World!
date: 2026-10-04
nav: Hello
order: 3
---

# Hello, World!
```

| Key     | Meaning                                                                                                                                      |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `title` | Page title. Falls back to the first `# heading`, then the filename.                                                                          |
| `date`  | Shown on the page and in listings. A listing sorts newest-first when every entry has one. Use ISO dates (`2026-10-04`).                      |
| `nav`   | Adds the page (or, in a directory's `index.md`, the listing) to the site nav with this label.                                               |
| `order` | Position in the nav, lowest first (default `0`).                                                                                             |
| `layout` | Template for this page only; see [Templates](#templates).                                                                                  |

Any other key is accepted and kept with the page, unless a schema says otherwise.

### Schemas

A directory's `index.md` can declare rules for the pages beneath it:

```markdown
---
title: Blog
nav: Blog
order: 4
required: title, date
sort: -date
strict: true
---
```

| Key        | Meaning                                                                                                                                                        |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `required` | Comma-separated frontmatter keys every page must have. A missing key is a startup error naming the file.                                                       |
| `sort`     | Key to sort the listing by; prefix `-` for descending. Numbers sort numerically; entries without the key go last. `title` uses the displayed title.            |
| `strict`   | `true` rejects any key other than `required` ones and the built-ins (`title`, `date`, `nav`, `order`, `layout`), catching typos.                                          |

**Rules are inherited, key by key.** A subdirectory inherits each rule from the nearest directory above it that sets it, and overrides only what it sets itself. To switch a rule off, set it explicitly: `required:` (empty), `strict: false`, or `sort:` (empty, meaning sort by filename).

**Rules in `content/index.md` apply site-wide.** The home page's frontmatter doubles as site config, so `strict: true` there makes the whole site strict.

Schema keys only work in an `index.md`; anywhere else they're an error. `index.md` files aren't checked against the schema themselves.

### Templates

Pages render through a built-in template unless they pick another by name:

| Key                    | Where             | Applies to                                    |
| ---------------------- | ----------------- | --------------------------------------------- |
| `layout`               | any file          | this page only                                |
| `default_layout`       | a folder's `index.md` | pages in that folder and below, inherited  |
| `default_index_layout` | a folder's `index.md` | listings in that folder and below, inherited |

A page uses its own `layout` if it has one, else the nearest default, else the built-in. An empty value means the built-in template. The home page (`content/index.md`) is a page, not a listing, so a `layout` there affects only the home page. Defaults set in the root apply site-wide. For example (these template names are illustrative; the table below lists the ones that exist):

```markdown
---
layout: home                 # the home page only
default_layout: page         # every page on the site
default_index_layout: list   # every listing on the site
---
```

Available templates:

| Name   | Kind | Renders                                                                                   |
| ------ | ---- | ----------------------------------------------------------------------------------------- |
| `post` | page | Title, a date byline under it, then the body (a leading `# heading` isn't repeated). |

The blog uses `default_layout: post`. An unknown template name, or a page template used for a listing, stops the server at startup with an error naming the file.

Templates are Rust functions (`DOC_TEMPLATES`/`INDEX_TEMPLATES` in `src/view.rs`). A template can also have a terminal version under the same name (`TEXT_TEMPLATES` in `src/ssh.rs`); without one, SSH shows the page generically.

### Dynamic routes

Some routes are registered in code rather than written as content: `/style.css`, plus the per-request `/count` and `/echo` pages. They take precedence. A content file or directory at one of those paths is skipped, with a warning when the server starts. To add one, see `REGISTERED` in `src/routes.rs`.

## Over SSH

| Where              | Keys                                                                         |
| ------------------ | ---------------------------------------------------------------------------- |
| Everywhere         | `↑`/`↓` scroll a line · `PgUp`/`PgDn` scroll a page · `q` quit                |
| Top-level pages    | the bracketed letter in the footer opens that nav entry · `space` scrolls     |
| Listings           | `1`–`9`, `0` open an entry · `>`/`<` next/previous page · `b` up · `h` home   |
| Pages in a listing | `b` back to the listing · `h` home · `space` scrolls                          |

## Development

```sh
cargo fmt
cargo clippy --all-targets
cargo test
```

Clippy is strict. Panic-prone patterns (`unwrap`, indexing, unchecked arithmetic, and so on) are denied, because a long-running server shouldn't panic. CI runs all three on every push and pull request.

Architecture notes for contributors (and coding agents) are in [CLAUDE.md](CLAUDE.md).

## Deployment

The site runs on [Fly.io](https://fly.io). One machine serves HTTP and raw TCP port 22 for SSH. Pushes to `main` deploy automatically once CI passes; `fly deploy` deploys manually. The build is the multi-stage `Dockerfile`, and the config is in `fly.toml`.

SSH needs a dedicated IPv4 address and a stable `SSH_HOST_KEY` secret. These and other deployment gotchas are listed in [CLAUDE.md](CLAUDE.md#deployment).
