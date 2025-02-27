# Build Stage
FROM rust:1.85.0-alpine3.21 AS builder

RUN apk add --no-cache musl-dev perl make

WORKDIR /usr/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

# Bundle Stage
FROM alpine:3.21

COPY --from=builder /usr/app/target/release/api /usr/app

CMD ["/usr/app"]