FROM rust:1.91-alpine AS builder

WORKDIR /app
COPY . /app

RUN apk add --no-cache musl-dev
RUN cargo build --release --locked

FROM alpine:3

WORKDIR /app
COPY --from=builder /app/target/release/zmq-tool /app

ENTRYPOINT [ "/app/zmq-tool" ]