# Build server
FROM rust:1.96.0-alpine3.23 AS chef
RUN apk --no-cache add musl-dev
RUN cargo install --locked cargo-chef
WORKDIR /ocg

FROM chef AS planner
COPY Cargo.* ./
COPY crates crates
COPY ocg-redirector/Cargo.* ocg-redirector/
COPY ocg-server/Cargo.* ocg-server/
RUN mkdir -p ocg-redirector/src ocg-server/src \
    && touch ocg-redirector/src/main.rs ocg-server/src/lib.rs ocg-server/src/main.rs \
    && cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG OCG_COMMIT_SHA
ENV OCG_COMMIT_SHA=$OCG_COMMIT_SHA
ENV CARGO_HTTP_MULTIPLEXING=false
ENV CARGO_HTTP_TIMEOUT=120
ENV CARGO_NET_RETRY=10
RUN wget -O /usr/local/bin/tailwindcss https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.10/tailwindcss-linux-x64-musl \
    && chmod +x /usr/local/bin/tailwindcss
COPY --from=planner /ocg/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    OCG_SKIP_ASSETS=1 cargo chef cook --release --locked -p ocg-server --recipe-path recipe.json
COPY Cargo.* ./
COPY crates crates
COPY ocg-redirector/Cargo.* ocg-redirector/
COPY docs docs
COPY ocg-server ocg-server
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --locked --release -p ocg-server

# Final stage
FROM alpine:3.23.5
RUN apk --no-cache add ca-certificates postgresql17-client && addgroup -S ocg && adduser -S ocg -G ocg
USER ocg
WORKDIR /home/ocg
COPY --from=builder /ocg/target/release/ocg-server /usr/local/bin
