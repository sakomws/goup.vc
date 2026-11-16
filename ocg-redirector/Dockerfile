# Build redirector
FROM rust:1.96.0-alpine3.23 AS builder
RUN apk --no-cache add musl-dev
WORKDIR /ocg
COPY Cargo.* ./
COPY crates crates
COPY ocg-redirector ocg-redirector
COPY ocg-server/Cargo.* ocg-server/
RUN mkdir -p ocg-server/src \
    && touch ocg-server/src/lib.rs ocg-server/src/main.rs \
    && cargo build --locked --release -p ocg-redirector

# Final stage
FROM alpine:3.23.5
RUN apk --no-cache add ca-certificates && addgroup -S ocg && adduser -S ocg -G ocg
USER ocg
WORKDIR /home/ocg
COPY --from=builder /ocg/target/release/ocg-redirector /usr/local/bin
