FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Copy manifests and build a dependency-only skeleton to cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source and rebuild
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service

EXPOSE 8081

CMD ["auth-service"]
