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

- `src/main.rs` — builds the `axum::Router`, registers routes, binds the TCP listener. Add new routes here.
- `src/handlers/` — request handlers, one module per concern. `pages.rs` holds page handlers; each is an `async fn` returning the response. Register the module in `handlers/mod.rs` and wire routes in `main.rs`.
- `main.rs` reads `PORT` from env and shuts down gracefully on SIGTERM/SIGINT — required by Render.

## Deployment

Deployed to Render via `render.yaml` (Blueprint), using Render's **native Rust runtime** — no Dockerfile. Push to the default branch auto-deploys: Render runs `cargo build --release` and starts `./target/release/thombruce`. Render provides TLS, reverse proxy, and process supervision; it injects `PORT` and sends SIGTERM on deploy/stop. `RUST_VERSION` is pinned in `render.yaml`.

## Linting

Strict clippy config in `Cargo.toml` `[lints.clippy]`: panic-prone patterns (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, etc.) are **denied** — a long-running server must not panic. `pedantic`/`nursery` are `warn`. Propagate errors with `Result`/`?` instead of unwrapping; `cargo clippy` must be clean.
