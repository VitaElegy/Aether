# Build Stage
FROM rust:1.76-slim-bookworm as builder
WORKDIR /app
COPY . .
# Ensure we build release
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
# Install OpenSSL/CA certificates
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy Binary
COPY --from=builder /app/target/release/aether_backend /app/aether_backend

# Copy Dictionary Data (Required by DictionaryLoader)
# Assumes 'data' exists in backend root
COPY --from=builder /app/data /app/data

# Environment Defaults
ENV RUST_LOG=info
EXPOSE 3000

CMD ["./aether_backend"]
