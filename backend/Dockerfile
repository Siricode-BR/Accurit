# syntax=docker/dockerfile:1

FROM rust:1.98-alpine AS builder

RUN apk add --no-cache musl-dev pkgconf postgresql-dev

ENV RUSTFLAGS="-C target-feature=-crt-static"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src \
    && printf '' > src/lib.rs \
    && printf 'fn main() {}\n' > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --locked \
    && rm -rf src \
        target/release/backend \
        target/release/backend.d \
        target/release/deps/backend-* \
        target/release/deps/libaccurit_backend-* \
        target/release/deps/accurit_backend-* \
        target/release/deps/libbackend-* \
        target/release/deps/backend*.d

COPY src ./src
COPY migrations ./migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    touch src/main.rs src/lib.rs \
    && cargo build --release --locked

FROM builder AS tester

COPY tests ./tests

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo test --no-run --locked

CMD ["cargo", "test", "--locked"]

FROM rust:1.98-alpine AS diesel-tool

RUN apk add --no-cache musl-dev pkgconf postgresql-dev postgresql-client

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN cargo install diesel_cli --no-default-features --features postgres \
    && rustup component add rustfmt

WORKDIR /work

CMD ["diesel", "--version"]

FROM alpine:3.24 AS runtime

RUN apk add --no-cache ca-certificates tzdata libgcc libpq \
    && addgroup -S accurit \
    && adduser -S -G accurit accurit

WORKDIR /app

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

ENV PORT=8080 \
    RUST_LOG=info

USER accurit

EXPOSE 8080

ENTRYPOINT ["backend"]
