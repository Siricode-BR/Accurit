# syntax=docker/dockerfile:1

FROM rust:1.98-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release \
    && rm -rf src target/release/backend target/release/deps/backend-*

COPY src ./src
COPY migrations ./migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch src/main.rs \
    && cargo build --release

FROM alpine:3.22 AS runtime

RUN apk add --no-cache ca-certificates tzdata libgcc \
    && addgroup -S accurit \
    && adduser -S -G accurit accurit

WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

ENV PORT=8080 \
    RUST_LOG=info

USER accurit

EXPOSE 8080

ENTRYPOINT ["backend"]
