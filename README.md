# RustDo

A high-performance, single-purpose todo list application written in **100% Rust**. Powered by **Axum** on the backend and **Yew (WebAssembly)** on the frontend.

No heavy databases, no bloated JavaScript runtime, no tracking—just todos, compiled to native code.

---

## Features

- ✨ **Clean, Minimal Interface**: A premium responsive layout optimized for mobile and desktop.
- 🌓 **Automatic Dark/Light Mode**: Synced to system preference with localStorage override.
- 💾 **Atomic File-Based Storage**: Todos are persisted safely using atomic file renames to prevent corruption.
- 🚀 **Blazing Fast WASM**: Client UI is powered by Rust compiled to WebAssembly (via Yew & Trunk).
- 🔒 **PIN Lockout Protection**: Secure client-IP rate limiting and timing-safe comparisons.
- 🌐 **PWA & Offline Support**: Fully installable as a web app with service-worker caching.

---

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PORT` | The port number the Axum server will listen on | `3000` | No |
| `RUSTDO_PIN` | Secure PIN code for accessing todos (4-10 digits) | - | No |
| `RUSTDO_SITE_TITLE` | Override the UI and HTML title | `RustDo` | No |
| `SINGLE_LIST` | Show a single list of todos (without selector controls) | `false` | No |
| `ALLOWED_ORIGINS` | Restrict CORS origins (e.g. `https://sub.domain.com`) | `*` | No |

---

## Quick Start

### 1. Build and Run Locally

Ensure you have Rust and **Trunk** installed:
```bash
# Verify wasm target is installed
rustup target add wasm32-unknown-unknown

# Install Trunk for WASM compiling
cargo install trunk
```

1. **Clone the repository** and navigate to it:
   ```bash
   git clone https://github.com/UberMetroid/RustDo.git
   cd RustDo
   ```

2. **Compile the Yew frontend**:
   ```bash
   cd frontend
   trunk build --release
   cd ..
   ```

3. **Start the Axum server**:
   ```bash
   cargo run --bin backend --release
   ```

4. Open `http://localhost:3000` in your browser.

---

### 2. Using Docker Compose

Build and spin up the entire application inside a lightweight Alpine container:

```bash
docker-compose up --build -d
```

Your `docker-compose.yml` service configuration:

```yaml
services:
  rustdo:
    build: .
    image: ubermetroid/rustdo:latest
    container_name: rustdo
    restart: unless-stopped
    ports:
      - ${RUSTDO_PORT:-3000}:3000
    volumes:
      - ${RUSTDO_DATA_PATH:-./data}:/app/data
    environment:
      - RUSTDO_PIN=${RUSTDO_PIN-}
      - RUSTDO_SITE_TITLE=RustDo
      - SINGLE_LIST=${SINGLE_LIST:-false}
```

---

## Project Structure

```
RustDo/
├── Cargo.toml          # Workspace manifest
├── Dockerfile          # Multi-stage optimized Rust builder
├── docker-compose.yml  # Docker Compose config
├── data/               # Persistent todo storage (JSON format)
│   └── todos.json
├── shared/             # Shared Rust libraries (serde structures)
│   ├── Cargo.toml
│   └── src/lib.rs
├── backend/            # Axum HTTP server & APIs
│   ├── Cargo.toml
│   └── src/main.rs
└── frontend/           # Yew frontend (SPA compiled to WASM)
    ├── Cargo.toml
    ├── index.html      # Trunk HTML entry point
    ├── styles.css      # App stylesheet
    ├── service-worker.js
    └── src/main.rs     # Yew components
```
