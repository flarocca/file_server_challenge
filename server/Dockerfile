FROM rust:1.89 AS builder

ARG SERVER_FEATURES

WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY client/Cargo.toml client/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY lib/Cargo.toml lib/Cargo.toml

COPY server ./server
COPY lib ./lib
COPY client ./client

RUN cargo build --manifest-path server/Cargo.toml --release --no-default-features --features "${SERVER_FEATURES}"

FROM debian:testing-slim
WORKDIR /
COPY --from=builder /app/target/release/file_server_server /server

ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/server"]

