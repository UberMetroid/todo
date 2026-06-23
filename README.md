# RustDo - Blazing Fast Todo List

RustDo is a blazing fast, single-purpose todo list application written in 100% Rust using Axum on the backend and Yew (WebAssembly) on the frontend.

---

## 🐳 Container Installation

### Option 1: Docker Compose (Recommended)

1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  rustdo:
    image: ubermetroid/rustdo:latest
    container_name: rustdo
    restart: unless-stopped
    ports:
      - 4403:4403
    volumes:
      - ./data:/app/data
    environment:
      - PORT=4403
      - RUSTDO_PIN=1234
      - RUSTDO_SITE_TITLE=RustDo
      - SINGLE_LIST=false
      - ALLOWED_ORIGINS=*
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4403`.

### Option 2: Docker CLI

Run the following command to start the container:

```bash
docker run -d \
  --name rustdo \
  --restart unless-stopped \
  -p 4403:4403 \
  -v $(pwd)/data:/app/data \
  -e RUSTDO_PIN=1234 \
  ubermetroid/rustdo:latest
```

---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to. | `4403` |
| `RUSTDO_PIN` | Lock todo access behind a secure digital PIN (4–10 digits). | None |
| `RUSTDO_SITE_TITLE` | Override the browser title, metadata headers, and PWA name. | `RustDo` |
| `SINGLE_LIST` | Force UI to hide list switcher and display only a single list. | `false` |
| `ALLOWED_ORIGINS` | Restrict CORS allowed origins (comma-separated list). | `*` |
