# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Personal website for thombruce.com — an Axum (Rust) web server. Early stage; routes currently return plain-text placeholder pages ("Site Under Construction").

## Commands

- `cargo run` — start the server, listens on `0.0.0.0:$PORT` (default 3000)
- `cargo build` / `cargo build --release` — build
- `cargo test` — run tests
- `cargo fmt` — format
- `cargo clippy` / `cargo clippy --all-targets` — lint (the latter also lints test code)

Rust edition 2024.

Tests live in `#[cfg(test)]` modules beside the code (`main.rs`, `ssh.rs`). HTTP handlers are exercised by building `app()` and driving it with `tower`'s `oneshot` (no live port). Only non-trivial logic is tested (www redirect, 404, SSH `screen`); const-returning page handlers are not. Test modules carry `#[allow(clippy::panic_in_result_fn)]` because assertions panic by design — the panic-restriction lints target the server, not tests.

## Architecture

Two frontends serve the same content from one binary:

- `src/content.rs` — page content as data, decoupled from any frontend.
- `src/main.rs` — spawns both listeners: axum HTTP (foreground, graceful shutdown) and the SSH server (background task). Reads `PORT`/`SSH_PORT` from env via `env_port`.
- `src/handlers/` — HTTP handlers. `pages.rs` renders `content::*` as text; `errors.rs` is the 404 fallback. Register modules in `handlers/mod.rs`, wire routes in `main.rs`.
- `src/ssh.rs` — SSH/TUI frontend (russh). Renders `content::*` over a PTY with single-key nav (h/a/q). v1 is raw ANSI; ratatui is deferred (issue #2).

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
- **Ephemeral SSH host key**: regenerated each boot, so the fingerprint changes on redeploy. Persist via a Fly secret if stable fingerprints matter.

## Linting

Strict clippy config in `Cargo.toml` `[lints.clippy]`: panic-prone patterns (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, etc.) are **denied** — a long-running server must not panic. `pedantic`/`nursery` are `warn`. Propagate errors with `Result`/`?` instead of unwrapping; `cargo clippy` must be clean.
