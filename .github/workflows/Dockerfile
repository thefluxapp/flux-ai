FROM rust:alpine AS builder

WORKDIR /app

COPY . .
RUN apk --no-cache add musl-dev protoc protobuf-dev
RUN cargo build --release -p flux-ai

FROM alpine

WORKDIR /app

RUN apk --no-cache add libssl3

COPY --from=builder /app/target/release/flux-ai /app/
COPY --from=builder /app/settings/default.toml /app/
