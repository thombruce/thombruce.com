# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Personal website for thombruce.com. One binary serves two frontends from a shared content source: an Axum (Rust) HTTP site rendering HTML, and an SSH/TUI frontend rendering text. Page content is authored as Markdown; HTML pages are styled with the classless drizzle-css framework.

## Commands

- `cargo run` — start the server, listens on `0.0.0.0:$PORT` (default 3000)
- `cargo build` / `cargo build --release` — build
- `cargo test` — run tests
- `cargo fmt` — format
- `cargo clippy` / `cargo clippy --all-targets` — lint (the latter also lints test code)

Rust edition 2024.

Tests live in `#[cfg(test)]` modules beside the code (`routes.rs`, `ssh.rs`). The router is exercised by building `routes::app()` and driving it with `tower`'s `oneshot` (no live port). Only non-trivial logic is tested (www redirect, 404, stylesheet content-type, SSH markdown-to-text, nav-key assignment/footer, scroll clamping, PTY-size clamping, and a CrosstermBackend render smoke test); const-returning handlers are not. Test modules carry `#[allow(clippy::panic_in_result_fn)]` because assertions panic by design — the panic-restriction lints target the server, not tests.

## Architecture

Two frontends serve the same content from one binary. Content is **discovered**, not hand-wired: **adding a page is dropping a Markdown file in `content/pages/`, and a blog post is dropping one in `content/blog/`** (then rebuilding — the embed is compile-time). Routes, nav (both frontends), blog index/pagination, and SSH keys all derive from the files.

- `src/content.rs` — content as data, decoupled from any frontend. Two embedded sections (`include_dir!`, compile-time): `content/pages/` → `Page { title, nav, order, path, body }` (frontmatter `title`/`nav`/`order`/`path`), and `content/blog/` → `Post { title, date, slug, body }` (frontmatter `title`/`date`; slug from filename; sorted newest-first, ISO dates sort as strings). `load()` returns a `Content { pages, posts }` bundle shared by both frontends; `split_frontmatter` is shared by both parsers. Fails loudly on malformed frontmatter — runs once at startup.
- `src/main.rs` — bootstrap only: `content::load()`s the `Content` into an `Arc` shared by both frontends, spawns both listeners (axum HTTP foreground with graceful shutdown, SSH background), reads `PORT`/`SSH_PORT` via `env_port`. (Env/config is read ad hoc here and in `ssh.rs` — centralize into a `Config` struct if it outgrows a few vars: issue #10.)
- `src/routes.rs` — `app(content)` builds the router: one pre-rendered route per discovered page and per blog post (`/blog/<slug>`), the `/blog` index, `/style.css`, the dynamic `/count`/`/echo` pages, and the 404 fallback; also the `www_redirect` middleware (extract into a `middleware/` module if a second one lands: issue #9) and the router tests. Pages/posts are auto-wired; dynamic pages are hand-registered from `handlers/`. At the current N, routes are pre-registered per file rather than a param-route + lookup map (issue #4's alternative) — revisit if post count grows large.
- `src/view.rs` — HTML rendering: `shell(title, body, nav)` (the shared maud layout — doctype, head, stylesheet, generated nav), `render_page`, `blog_index`, `post_page`, `not_found`, and the dynamic-page views `count_page`/`echo_page`. Nav links come from the page list with a `Blog` entry appended, so it stays in sync automatically. All HTML views share this one file; split into a `view/` submodule if it grows unwieldy: issue #8.
- `src/handlers/` — logic-bearing routes: `assets.rs` serves `/style.css` from drizzle-css's embedded `CSS_MIN` constant; `count.rs`/`echo.rs` render the per-request dynamic pages as `route(content)` factories (the visit counter's `AtomicU64` lives in `count`'s closure). (Registered in `handlers/mod.rs`.)
- `src/ssh.rs` — SSH/TUI frontend (russh + ratatui). Three screens on an `App { screen, blog_page, scroll, … }` state machine: **Page** (a static page, reached by its nav key — first free letter of its label, collisions fall through via `assign_keys`/`nav_keys`; `Blog` is a nav target), **BlogIndex** (paginated post list, 10/page — digits `1`-`9`/`0` open an entry, `>`/`<` page, `h` home), and **Post** (`b` back to the index). All screens scroll (`space`/arrows/PageUp-Down); `q`/Ctrl-C/Ctrl-D quit. Footers and lists derive from `Content`, so adding a page/post needs no change here. ratatui is used for scroll + resize, not chrome: renders through a `CrosstermBackend` over a `Vec` (a `Viewport::Fixed` sized from the PTY via `dim()`, which caps each dimension so a hostile pty-req can't trigger a giant buffer alloc; never queries the server tty), ANSI drained down the channel after each `Terminal::draw`, inside a centered `CONTENT_WIDTH` column. `pty_request`/`window_change_request` keep the size current (resize re-wraps). Client cleared with a raw `\x1b[2J` escape, not `Terminal::clear()` — its Fixed-viewport path opens the controlling tty and fails (ENXIO) on a headless server. Loads its host key via `host_key()`.

Rendering: HTML uses maud (`html!`, compile-checked). Content pages are rendered **once at startup** (the router captures the strings); the dynamic pages (`/count`, `/echo`) render **per request**. Markdown is parsed with pulldown-cmark. The SSH path walks the same Markdown to text on demand. Styling is drizzle-css (classless — no classes in the markup), so the HTML is styled for free.

## Deployment

Deployed to **Fly.io** — one machine publishes HTTP (via `[http_service]`) and raw TCP `:22 → :2222` for SSH (via `[[services]]`). Build is the `Dockerfile` (multi-stage, non-root); config in `fly.toml`. `main.rs` shuts down gracefully on SIGTERM/SIGINT. CI is `.github/workflows/ci.yml`: a `check` job (fmt, clippy `--all-targets`, test) runs on every push to `main` and on PRs; a `deploy` job `needs: check` and runs only on push to `main`, so a red check blocks the deploy. Deploy needs the `FLY_API_TOKEN` repo secret. `fly deploy` deploys manually.

Common flyctl commands:

- `fly logs` / `fly status` — tail logs / machine state (first stop when a deploy misbehaves)
- `fly deploy` — build + release from the local `Dockerfile`
- `fly ssh console` — shell into the machine (Fly's own admin SSH, separate from this app's SSH frontend)
- `fly certs add <host>` — provision a TLS cert (e.g. `www.thombruce.com`)
- `fly ips list` / `fly ips allocate-v4`

Deployment gotchas (all hit in practice):

- **SSH needs a dedicated IP** (`fly ips allocate-v4`, ~$2/mo) — shared IPv4 only routes HTTP/TLS, so raw TCP `:22` is unreachable without it.
- **glibc must match** between builder and runtime images, or the binary crash-loops with `GLIBC_x.y not found` — both are pinned to bookworm in the `Dockerfile`.
- **`www` needs its own cert** (`fly certs add www.thombruce.com`) even though `main.rs` redirects it to the apex — the TLS handshake happens before the redirect can be sent. Also add a `www` CNAME.
- **Scale-to-zero** (`min_machines_running = 0`): a first SSH connection to a cold machine may time out; reconnect wakes it. Set to `1` for always-on.
- **SSH host key** is loaded from the `SSH_HOST_KEY` secret (an OpenSSH-format private key); set it (`fly secrets set SSH_HOST_KEY="$(cat keyfile)"`) so the fingerprint stays stable across deploys. If unset, `ssh.rs` falls back to an ephemeral key (fine for local dev) — but then a redeploy changes the fingerprint and clients get `REMOTE HOST IDENTIFICATION HAS CHANGED`. Set once; a Fly secret persists across deploys.
- **`content/` must be in the build context** — page and post Markdown is embedded from `content/pages/` and `content/blog/` with `include_dir!` at compile time, so the `Dockerfile` copies `content/` alongside `src/`. A new page or post appears once its file is committed and the image rebuilt.

## Linting

Strict clippy config in `Cargo.toml` `[lints.clippy]`: panic-prone patterns (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, etc.) are **denied** — a long-running server must not panic. `pedantic`/`nursery` are `warn`. Propagate errors with `Result`/`?` instead of unwrapping; `cargo clippy` must be clean.
