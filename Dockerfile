# Stage 1: Build the frontend WASM application
FROM rust:1.96-alpine AS frontend-builder
RUN apk add --no-cache musl-dev wget tar
WORKDIR /app

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install Trunk (precompiled binary to save build time)
RUN wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-musl.tar.gz" | tar -xzf- -C /usr/local/bin

# Copy cargo files & all crates
COPY Cargo.toml /app/Cargo.toml
COPY shared /app/shared
COPY backend /app/backend
COPY frontend /app/frontend

# Build frontend
WORKDIR /app/frontend
RUN trunk build --release

# Stage 2: Build the backend server
FROM rust:1.96-alpine AS backend-builder
RUN apk add --no-cache musl-dev
WORKDIR /app

# Copy cargo files & all crates
COPY Cargo.toml /app/Cargo.toml
COPY shared /app/shared
COPY backend /app/backend
COPY frontend /app/frontend

# Build backend
WORKDIR /app/backend
RUN cargo build --release --bin backend

# Stage 3: Runtime image
FROM alpine:3.18
LABEL org.opencontainers.image.source="https://github.com/UberMetroid/RustDo"
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache wget libc6-compat

# Copy binaries and assets
COPY --from=backend-builder /app/target/release/backend /app/backend
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist

# Setup data directory with correct ownership
RUN mkdir -p /app/data && chown -R 99:100 /app

# Expose server port (internal port)
EXPOSE 4403

# Healthcheck
HEALTHCHECK --interval=20s --timeout=5s --start-period=20s --retries=3 \
    CMD wget --spider -q http://127.0.0.1:4403/api/config || exit 1

# Run as Unraid nobody:users
USER 99:100

# Run the server
CMD ["/app/backend"]
