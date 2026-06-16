FROM rust:1-bookworm AS builder

LABEL maintainer="sony-level"

WORKDIR /app
COPY . .
RUN cargo build --release -p api-gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway

EXPOSE 8000

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD ["api-gateway"]
