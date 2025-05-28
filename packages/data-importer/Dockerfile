FROM rust:1.87-slim AS builder

WORKDIR /app
COPY . /app

RUN cargo build --release --locked

###

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/data-importer /app

ENTRYPOINT [ "/app/data-importer" ]