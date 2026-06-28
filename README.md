# Todo - Blazing Fast Todo List

<p align="center">
  <img src="https://raw.githubusercontent.com/UberMetroid/todo/main/frontend/Assets/favicon.png?v=3.0.1" alt="Todo Logo" width="128" height="128">
</p>

Todo is a blazing fast, single-purpose todo list application written in 100% Rust. Built with a high-performance Rust (Axum/Tokio) backend and a WebAssembly (Yew) frontend.

---

## ⚡ Key Features

*   **Dynamic Themes**: Dynamic theme options.
*   **Access PIN Security**: Lock down the interface with an optional numerical PIN for absolute privacy.
*   **Internationalization**: Built-in multilingual translation selector support.
*   **Print Optimization**: Customized print stylesheet layout and print header action button.
*   **Performance First**: Tiny resource footprint, zero external JS engine dependencies, and rapid page load speeds.
*   **Multi-List Tasking**: Create and switch between multiple list tasks, or lock to a single list view.
*   **Keyboard Shortcuts**: Quick keyboard management for lists and tasks.

---

## 📦 Container Registry

The Docker image is built with **Nix** (no Alpine, fully reproducible) and published to Docker Hub:

*   **Docker Hub**: [ubermetroid/todo](https://hub.docker.com/r/ubermetroid/todo)

---

## 🐳 Container Installation

1. Create a `docker-compose.yml` file:

```yaml
version: '3'
services:
  todo:
    image: ubermetroid/todo:latest
    container_name: todo
    restart: unless-stopped
    ports:
      - 4403:4403
    volumes:
      - ./data:/app/data
    environment:
      - PORT=4403
      - SITE_TITLE=Todo
      - BASE_URL=http://localhost:4403
      - ALLOWED_ORIGINS=*
      - TODO_PIN=1234
      - TZ=UTC
      - ENABLE_TRANSLATION=false
      - ENABLE_THEMES=true
      - ENABLE_PRINT=false
```

2. Run the container:

```bash
docker compose up -d
```

3. Open your browser and navigate to `http://localhost:4403`.

### Building the Image Locally

To build the Docker container locally from the source files using Nix:

```bash
nix build .#dockerImage
docker load < result
docker tag todo-nix:latest ubermetroid/todo:latest
```

The image is Nix-built (no Alpine, no Docker daemon dependency for the build).
For development iteration, use the devShell:

```bash
nix develop
```

---

## 📋 Configuration Options

Configure these settings inside your Docker Compose environment or container environment variables:

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | The port number the backend HTTP server will bind to inside the container. | `4403` |
| `SITE_TITLE` | Custom website title rendered in navigation headers, browser tabs, and PWA manifest. | `Todo` |
| `BASE_URL` | Application base URL. Essential when deploying behind reverse proxies to ensure redirect and websocket links are resolved correctly. | `http://localhost:4403` |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed HTTP request origins (CORS filter). Use `*` to allow all origins. | `*` |
| `TODO_PIN` | Optional 4–10 digit PIN (numerical only) to lock access to the interface. Leave empty for public mode. | None |
| `TZ` | Timezone for the container processes and logs. | `UTC` |
| `SINGLE_LIST` | Force UI to hide list switcher and display only a single list. | `false` |
| `ENABLE_TRANSLATION` | Enable the multi-language / translation selector in the navigation header (true/false). | `false` |
| `ENABLE_THEMES` | Enable the theme selector in the navigation header (true/false). | `true` |
| `ENABLE_PRINT` | Enable the print button in the navigation header (true/false). | `false` |
| `MAX_ATTEMPTS` | Number of failed PIN attempts permitted before locking out the user client IP address. | `5` |

---

*Note: This repository was forked from [DumbDo](https://github.com/DumbWareio/DumbDo).*
