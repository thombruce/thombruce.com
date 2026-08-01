# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Personal website for thombruce.com — an Axum (Rust) web server. Early stage; routes currently return plain-text placeholder pages ("Site Under Construction").

## Commands

- `cargo run` — start the server, listens on `0.0.0.0:3000`
- `cargo build` / `cargo build --release` — build
- `cargo test` — run tests (none yet)
- `cargo fmt` — format
- `cargo clippy` — lint

Rust edition 2024.

## Architecture

- `src/main.rs` — builds the `axum::Router`, registers routes, binds the TCP listener. Add new routes here.
- `src/handlers/` — request handlers, one module per concern. `pages.rs` holds page handlers; each is an `async fn` returning the response. Register the module in `handlers/mod.rs` and wire routes in `main.rs`.
