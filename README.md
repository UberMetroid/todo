# RustDo

A blazing fast, single-purpose todo list application written in **100% Rust** using **Axum** on the backend and **Yew (WebAssembly)** on the frontend. RustDo features compile-time type safety across client-server boundaries, timing-safe authentication, atomic file-based persistence, multi-list support, 8 preloaded languages (i18n), and 5 custom-designed themes.

---

## Overview

RustDo is a modern, lightweight, self-hosted task management application that operates completely without bloated JavaScript frameworks, external databases, or runtime tracking. The codebase is architected with a monorepo workspace structure, separating backend Axum API server routes, frontend Yew-based WASM rendering, and a shared data validation layer. The application is packaged into a highly optimized, multi-stage Alpine Docker image containing a built-in healthcheck wrapper.

---

## Features

- 🌓 **Dynamic Theme System**: Zero-flicker client theme persistence with support for 5 designer themes: **Light (Indigo)**, **Dark (Slate)**, **Nord**, **Dracula**, and **Sepia**.
- 🌍 **Type-Safe i18n**: Clean client-side localization system supporting 8 languages: English (en), Simplified Chinese (zh), Spanish (es), German (de), Japanese (ja), French (fr), Portuguese (pt), and Russian (ru).
- 🔒 **Timing-Safe Protection**: Secure client-IP rate limiting and timing-safe constant-time string comparisons preventing password/PIN enumeration.
- ⏳ **Jitter Delay Enforcement**: Dynamic login delay (50ms–150ms) to frustrate automated dictionary attacks.
- 💾 **Atomic File-Based Storage**: High-integrity persistence utilizing atomic file renames to guarantee data safety and prevent disk corruptions.
- 📂 **Multi-List Management**: Switch, add, rename, or delete todo lists on the fly (configurable to a single-list mode via environment variables).
- 🔗 **Smart Linkification**: Dynamic conversion of URL patterns inside task items to secure (`target="_blank" rel="noopener noreferrer"`) anchor tags.
- 🚀 **PWA Support**: Full progressive web app configuration with offline service worker asset caching.

---

## Prerequisites & Environment Variables

### System Prerequisites

To build and run RustDo locally from source, the following dependencies are required:
- **Rust Toolchain**: `stable` (v1.75+ recommended)
- **WASM compilation target**: `wasm32-unknown-unknown`
- **Trunk**: WebAssembly web application bundler for Rust

### Environment Variables

| Variable | Description | Default | Required |
|:---|:---|:---|:---|
| `PORT` | The port number the Axum backend HTTP server will bind to | `3000` | No |
| `RUSTDO_PIN` | Lock todo access behind a secure digital PIN (4–10 digits) | *None (open)* | No |
| `RUSTDO_SITE_TITLE` | Override the browser title, metadata headers, and PWA name | `RustDo` | No |
| `SINGLE_LIST` | Force UI to hide list switcher and display only "List 1" | `false` | No |
| `ALLOWED_ORIGINS` | Restrict CORS allowed origins (comma-separated list) | `*` | No |
| `NODE_ENV` | Environment context. Disables CORS check if set to `development` | `production` | No |

---

## Quick Start

Get RustDo up and running locally from source in under 2 minutes:

```bash
# 1. Clone the repository
git clone https://github.com/UberMetroid/RustDo.git
cd RustDo

# 2. Add the WebAssembly compilation target
rustup target add wasm32-unknown-unknown

# 3. Install Trunk for compiling frontend WASM assets
cargo install --locked trunk

# 4. Compile frontend distribution static assets
cd frontend && trunk build --release && cd ..

# 5. Start the Axum backend server
PORT=3000 RUSTDO_PIN=1234 cargo run --bin backend --release
```

Open your browser and navigate to `http://localhost:3000` to access the application.

---

## Docker & Docker Compose Configurations

### Docker Compose (Recommended)

Run the entire application in detached mode with persistent volume storage out-of-the-box.

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  rustdo:
    build: .
    image: ubermetroid/rustdo:latest
    container_name: rustdo
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data
    environment:
      - PORT=3000
      - RUSTDO_PIN=1234
      - RUSTDO_SITE_TITLE=My Todo App
      - SINGLE_LIST=false
      - ALLOWED_ORIGINS=*
```

Start the container:
```bash
docker-compose up --build -d
```

### Docker CLI

Build and execute the multi-stage optimized runtime container directly using Docker:

```bash
# Build the optimized Alpine-based release image
docker build -t rustdo:latest .

# Run the container with persistent folder mapping
docker run -d \
  --name rustdo \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -e RUSTDO_PIN=1234 \
  -e RUSTDO_SITE_TITLE="Team Board" \
  rustdo:latest
```

---

## Technical Details

### Architecture & Workspaces

The repository is structured as a Cargo workspace with three member crates:
1. **`shared`**: Houses the core Data Transfer Objects (DTOs) and request/response models. Used to ensure identical serialization shapes on both sides of the network boundary.
2. **`backend`**: Axum HTTP server. Serves API endpoints under `/api/*` and acts as a static file server to deliver WASM files, stylesheet designs, manifest properties, and service workers from `frontend/dist/` fallback.
3. **`frontend`**: Single Page Application built using Yew. Compiles to a WASM binary target orchestrated via Trunk.

### Security Configurations

- **Timing-Safe PIN Verification**: Verification of security codes uses XOR comparison operations over 10 characters padding (`secure_compare` in `backend/src/auth.rs`), preventing timing side-channel attacks.
- **Client-IP Resolving & Proxy Safety**: Client IPs are resolved by inspecting headers (`cf-connecting-ip`, `x-forwarded-for`, `x-real-ip`) first before defaulting to the TCP connection socket address, preventing proxy-based rate limit bypasses.
- **Lockout and Rate-Limiting**: IP-based locking tracking restricts client connections to **5 attempts** per 15 minutes. Lockout statuses are checked in memory, with active sweeps removing stale entries.
- **Timing Jitter**: Verification responses are delayed by a randomized block of `50ms` to `150ms` (`tokio::time::sleep`) to prevent automated dictionary attacks.

### Data Persistence

Data saving actions enforce structural integrity:
- **Renames**: Payloads are serialized in memory using `serde_json`, written to a temporary scratch file (`todos.json.tmp`), flushed to disk, and moved to `todos.json` via a syscall rename. This guarantees that file write interruptions or crashes never corrupt the primary database file.
- **Migrations**: On startup, a migration runner inspects all stored lists, dynamically generates cryptographically secure 9-character alphanumeric IDs using `/dev/urandom` for any items missing keys, and saves the updated manifest atomically.

---

## File Tree

```
RustDo/
├── Cargo.toml                  # Workspace definitions (release optimizations)
├── Dockerfile                  # 3-Stage Alpine builder (precompiled trunk binary)
├── docker-compose.yml          # Container composition definition
├── data/                       # Persistent JSON storage directory (auto-created)
│   └── todos.json
├── shared/                     # Data shapes compiled into frontend & backend
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs              # Models (TodoItem, SiteConfig, VerifyPinRequest)
├── backend/                    # Server backend service API
│   ├── Cargo.toml
│   └── src/
│       ├── auth.rs             # Safe comparators, migrations, and ID generator
│       ├── handlers.rs         # Serialization handlers, PIN verify logic
│       ├── main.rs             # Axum routes, CORS configuration, listener bindings
│       ├── middleware.rs       # Client auth validations, origin checking
│       ├── state.rs            # App state properties, IP-resolver helper
│       └── static_files.rs     # Manifest generator, directory static endpoints
└── frontend/                   # WebAssembly Client UI
    ├── Cargo.toml
    ├── index.html              # HTML shell entry layout
    ├── styles.css              # Custom Vanilla CSS visual engine (all themes)
    ├── service-worker.js       # Offline service worker caching assets
    └── src/
        ├── api.rs              # Fetch API handler (login, fetch, save)
        ├── i18n/               # Dictionary translation dictionaries
        ├── i18n.rs             # Custom translation dispatcher hook
        ├── list_handlers.rs    # List state event handlers
        ├── list_selector.rs    # List picker select component
        ├── login.rs            # Login layout
        ├── main.rs             # App mounting entry and context binding
        ├── toast.rs            # Toast notifications component
        ├── todo_form.rs        # Text form block for adding todo tasks
        ├── todo_header.rs      # Header title, selectors, theme buttons
        ├── todo_item.rs        # Task rows supporting dragging & editing
        ├── todo_items_list.rs  # Component separating active vs done tasks
        ├── todo_list_handlers.rs # Task item toggle and delete handlers
        ├── todo_list.rs        # Todo container component
        └── types.rs            # Toast models and properties
```

---

## Testing & Linting

Enforce quality standards, type checks, and formatting rules matching the CI integration:

```bash
# Formatter check
cargo fmt --all --check

# Clippy Lints (with strict warnings denied)
cargo clippy --workspace --all-targets -- -D warnings

# Execute test suite
cargo test --workspace
```

---

## Contributing

RustDo maintains strict architectural rules to enforce maintainability:
1. **Idiomatic Design**: Code should be written in idiomatic, readable Rust.
2. **File Size Limits**: To keep components focused and modular, **every source file must remain strictly between 25 and 250 lines of code**. If a file exceeds this range, it must be broken down into sub-modules.
3. **No Heavy External Dependencies**: Rely on zero-allocation and native language features. Do not add heavy external crates (like heavy crypto packages, datetime libraries, or CSS preprocessors).
4. **Style Guidelines**: Custom styles should be written using Vanilla CSS classes inside `frontend/styles.css`. Avoid inline styles or utility CSS utilities (like Tailwind) unless explicitly requested.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](file:///home/jeryd/Projects/ubermetroid/RustDo/LICENSE) file for details.
