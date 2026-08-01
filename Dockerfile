# ---- build ----
FROM rust:1-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ---- runtime ----
FROM debian:bookworm-slim
# ponytail: no cargo-chef dep-caching layer; add when compile time bites.
COPY --from=builder /app/target/release/thombruce /usr/local/bin/thombruce
# non-root; SSH binds 2222 (unprivileged), so no root needed.
USER 1000:1000
CMD ["thombruce"]
