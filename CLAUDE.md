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

Tests live in `#[cfg(test)]` modules beside the code (`routes.rs`, `ssh.rs`). The router is exercised by building `routes::app()` and driving it with `tower`'s `oneshot` (no live port). Only non-trivial logic is tested (www redirect, 404, stylesheet content-type, SSH markdown-to-text); const-returning handlers are not. Test modules carry `#[allow(clippy::panic_in_result_fn)]` because assertions panic by design — the panic-restriction lints target the server, not tests.

## Architecture

Two frontends serve the same content from one binary.

- `src/content.rs` — page content as data, decoupled from any frontend. `Page { title, markdown }`; each page's body is a Markdown file in `content/` embedded at compile time via `include_str!`. Single source for both frontends.
- `src/main.rs` — bootstrap only: spawns both listeners (axum HTTP in the foreground with graceful shutdown, the SSH server as a background task), reads `PORT`/`SSH_PORT` via `env_port`.
- `src/routes.rs` — the route table (`app()`), the `www_redirect` middleware, and the router tests. This is the one place to see every route.
- `src/handlers/` — HTTP handlers, registered in `handlers/mod.rs`. `pages.rs` renders a `Page`'s Markdown to HTML (pulldown-cmark) wrapped in the shared shell; `errors.rs` returns the HTML 404 fallback; `assets.rs` serves `/style.css` from drizzle-css's embedded `CSS_MIN` constant.
- `src/view.rs` — `shell(title, body)`, the shared maud layout (doctype, head, stylesheet link, nav) wrapping each route's body. Route-specific bodies live in the handlers.
- `src/ssh.rs` — SSH/TUI frontend (russh). Renders the same `Page` Markdown to plain terminal text over a PTY with single-key nav (h/a/q); loads its host key via `host_key()`. v1 is raw ANSI; ratatui is deferred (issue #2).

Rendering: HTML uses maud (`html!` macro, compile-checked); Markdown is parsed with pulldown-cmark (per request — a `ponytail:` note marks the `LazyLock` cache upgrade if it ever matters). The HTTP path renders Markdown to HTML; the SSH path walks the same Markdown to text. Styling is drizzle-css (classless — no classes in the markup), so the semantic HTML and Markdown output are styled for free.

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
- **`content/` must be in the build context** — page Markdown is pulled in with `include_str!` at compile time, so the `Dockerfile` copies `content/` alongside `src/`. A new page file just works as long as it's committed.
- **Clean builds are slow** — drizzle-css compiles its CSS via lightningcss at build time, dragging a large dependency tree. It's build-time only (no runtime cost) and cached after the first build.

## Linting

Strict clippy config in `Cargo.toml` `[lints.clippy]`: panic-prone patterns (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, etc.) are **denied** — a long-running server must not panic. `pedantic`/`nursery` are `warn`. Propagate errors with `Result`/`?` instead of unwrapping; `cargo clippy` must be clean.
