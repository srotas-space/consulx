CONSUL_HTTP_ADDR=http://127.0.0.1:8500 cargo run --bin consulx

# consulx — Consul KV CLI + Rust Client Library

[![Crates.io](https://img.shields.io/crates/v/consulx)](https://crates.io/crates/consulx)
[![Documentation](https://docs.rs/consulx/badge.svg)](https://docs.rs/consulx)
[![License](https://img.shields.io/crates/l/consulx)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/consulx)](https://crates.io/crates/consulx)
[![Recent Downloads](https://img.shields.io/crates/dr/consulx)](https://crates.io/crates/consulx)

**consulx** is a modern Rust toolkit for working with **Consul KV**, providing:

- 🔧 **Interactive REPL** (like redis-cli, but for Consul)  
- 🗂️ **Tree view**, **watch**, **prefix watch**, **edit**, **get-json**, **put-json**  
- ⚙️ Lightweight **HTTP-only Consul client** (no SDKs)
- 🌐 Integrations for **Actix** and **Axum**
- 🔁 First-class support for dynamic config & feature flags

Everything is built **from scratch** on top of `reqwest`.

---

## ✨ Features

### 🧵 REPL Interface
- Auto-completion
- Commands: `get`, `put`, `del`, `list`, `tree`, `watch`, `watch-prefix`, `edit`,  
  `get-json`, `put-json`

### 🗄️ JSON-aware KV
- Pretty-print JSON  
- Validate JSON on write  
- Typed JSON config loader for Rust apps

### 🔁 Watches
- Watch single keys  
- Watch entire prefixes  
- Uses Consul **blocking queries** (`x-consul-index`)

### 🌍 Actix & Axum Ready
Re-usable client that plugs into web frameworks easily.

### ⚡ Pure HTTP client
No consulrs, no SDK — just reqwest.

---

## 🚀 Installation

```toml
[dependencies]
consulx = "0.1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## 🔧 CLI Usage

```bash
cargo install consulx
consulx
```

Specify endpoint:

```bash
CONSUL_HTTP_ADDR=http://127.0.0.1:8501 consulx
```

---

## 🖥️ REPL Commands

| Command | Description |
|--------|-------------|
| `get <key>` | Fetch raw value |
| `put <key> <value>` | Store value |
| `del <key>` | Delete key |
| `list <prefix>` | List keys |
| `tree <prefix>` | ASCII tree |
| `get-json <key>` | Pretty JSON |
| `put-json <key> <json>` | Validate+store |
| `edit <key>` | Open in $EDITOR |
| `watch <key>` | Watch key |
| `watch-prefix <prefix>` | Watch tree |
| `help` | Help |
| `exit` | Quit |

---

## 🌳 Example — Tree View

```
app
├── config
│   ├── max_workers
│   ├── log_level
│   └── feature_beta
└── secrets
    └── jwt_key
```

---

## 🧩 Using consulx in Actix Web

```rust
use actix_web::{App, HttpServer, web, Responder};
use consulx::ConsulXClient;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    max_workers: u32,
    feature_beta: bool,
}

#[derive(Clone)]
struct AppState {
    consul: ConsulXClient,
}

async fn config_handler(data: web::Data<AppState>) -> impl Responder {
    let cfg = data.consul
        .kv_get_json::<AppConfig>("app/config")
        .await
        .unwrap()
        .unwrap();

    web::Json(cfg)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let consul = ConsulXClient::from_env().unwrap();
    let state = AppState { consul };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/config", web::get().to(config_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

---

## 🧩 Using consulx in Axum

```rust
use axum::{Router, routing::get, extract::State, Json};
use consulx::ConsulXClient;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
struct AppState {
    consul: ConsulXClient,
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    max_workers: u32,
    feature_beta: bool,
}

async fn handler(State(state): State<AppState>) -> Json<AppConfig> {
    Json(
        state.consul
            .kv_get_json::<AppConfig>("app/config")
            .await
            .unwrap()
            .unwrap(),
    )
}

#[tokio::main]
async fn main() {
    let consul = ConsulXClient::from_env().unwrap();
    let state = AppState { consul };

    let app = Router::new()
        .route("/config", get(handler))
        .with_state(state);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

---

## 📦 Library API Overview

```rust
ConsulXClient::new(url)
ConsulXClient::from_env()

kv_get_raw / kv_put / kv_delete / kv_list

kv_get_json<T> / kv_put_json<T>
kv_watch / kv_watch_prefix
```

---

## 📄 License

MIT License

---

Made with ❤️ by Srotas Space.
