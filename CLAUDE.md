# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Personal website for thombruce.com — an Axum (Rust) web server. Early stage; routes currently return plain-text placeholder pages ("Site Under Construction").

## Commands

- `cargo run` — start the server, listens on `0.0.0.0:$PORT` (default 3000)
- `cargo build` / `cargo build --release` — build
- `cargo test` — run tests (none yet)
- `cargo fmt` — format
- `cargo clippy` — lint

Rust edition 2024.

## Architecture

Two frontends serve the same content from one binary:

- `src/content.rs` — page content as data, decoupled from any frontend.
- `src/main.rs` — spawns both listeners: axum HTTP (foreground, graceful shutdown) and the SSH server (background task). Reads `PORT`/`SSH_PORT` from env via `env_port`.
- `src/handlers/` — HTTP handlers. `pages.rs` renders `content::*` as text; `errors.rs` is the 404 fallback. Register modules in `handlers/mod.rs`, wire routes in `main.rs`.
- `src/ssh.rs` — SSH/TUI frontend (russh). Renders `content::*` over a PTY with single-key nav (h/a/q). v1 is raw ANSI; ratatui is deferred (issue #2).

## Deployment

Deployed to **Fly.io** — one machine publishes HTTP (via `[http_service]`) and raw TCP `:22 → :2222` for SSH (via `[[services]]`). SSH needs a **dedicated IP** (`fly ips allocate-v4`); shared IPv4 only routes HTTP/TLS. Build is the `Dockerfile` (multi-stage, non-root). Deploy with `fly deploy`; config in `fly.toml`. `main.rs` shuts down gracefully on SIGTERM/SIGINT.

## Linting

Strict clippy config in `Cargo.toml` `[lints.clippy]`: panic-prone patterns (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, etc.) are **denied** — a long-running server must not panic. `pedantic`/`nursery` are `warn`. Propagate errors with `Result`/`?` instead of unwrapping; `cargo clippy` must be clean.
