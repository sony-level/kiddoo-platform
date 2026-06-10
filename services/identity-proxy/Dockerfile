FROM rust:1-trixie AS builder

LABEL maintainer="sony-level"

WORKDIR /app
COPY . .
RUN cargo build --release -p identity-proxy

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/identity-proxy /usr/local/bin/identity-proxy

EXPOSE 8001

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8001

CMD ["identity-proxy"]
