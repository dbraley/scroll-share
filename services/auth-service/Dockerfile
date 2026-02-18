FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Copy manifests and create a dummy main to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p src/routes
RUN echo "pub mod app; pub mod config; pub mod db; pub mod errors; pub mod routes;" > src/lib.rs
RUN echo "pub mod health;" > src/routes/mod.rs
RUN touch src/app.rs src/config.rs src/db.rs src/errors.rs src/routes/health.rs
RUN cargo build --release 2>/dev/null || true

# Copy actual source and rebuild
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service

EXPOSE 8081

CMD ["auth-service"]
