# Colophon

This site is built in Rust with [Axum](https://github.com/tokio-rs/axum), and served two ways from a single binary: over HTTP as HTML, and over SSH as a terminal UI — try `ssh thombruce.com`.

Pages are written in Markdown. HTML is rendered with [maud](https://maud.land) and styled with [drizzle-css](https://github.com/thombruce/drizzle), a classless framework of my own.

Deployed on [Fly.io](https://fly.io).

The source is on [GitHub](https://github.com/thombruce/thombruce.com).
