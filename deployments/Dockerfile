FROM rust:1.75 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --bin dashboard_api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/dashboard_api /usr/local/bin/dashboard_api

EXPOSE 8080 9090

CMD ["dashboard_api"]
