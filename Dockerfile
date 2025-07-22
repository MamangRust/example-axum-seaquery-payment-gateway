FROM rust:1.87 AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin example_sea_query_payment_gateway

FROM alpine:3.20

RUN apk --no-cache add ca-certificates

RUN addgroup -g 1000 appuser && \
    adduser -D -s /bin/sh -u 1000 -G appuser appuser

WORKDIR /app


COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/example_sea_query_payment_gateway .

COPY .env .env

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8000

ENV RUST_LOG=info

CMD ["./example_sea_query_payment_gateway"]
