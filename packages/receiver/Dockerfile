FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . /app

RUN cargo build --release

###

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/receiver /app

ENTRYPOINT [ "/app/receiver" ]