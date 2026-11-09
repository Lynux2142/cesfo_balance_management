FROM rust:1.99-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release --bin server --bin scraper

FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/target/release/scraper /app/scraper
COPY templates /app/templates
COPY static /app/static

EXPOSE 3000

ENV MONGODB_URL="mongo"
ENV MONGODB_PORT="27017"
ENV MONGODB_USER="user"
ENV MONGODB_PASSWORD="password"
ENV CRON_SCHEDULE="0 0 2,8 * * *"

CMD ["/app/server"]
