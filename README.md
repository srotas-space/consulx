# consulx — Consul KV CLI + Rust Client Library

[![Crates.io](https://img.shields.io/crates/v/consulx)](https://crates.io/crates/consulx)
[![Documentation](https://docs.rs/consulx/badge.svg)](https://docs.rs/consulx)
[![License](https://img.shields.io/crates/l/consulx)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/consulx)](https://crates.io/crates/consulx)
[![Recent Downloads](https://img.shields.io/crates/dr/consulx)](https://crates.io/crates/consulx)

![SnmFDBCLI](https://srotas-suite-space.s3.ap-south-1.amazonaws.com/srotas-space.png)


**consulx** is a modern Rust toolkit for working with **Consul KV**, providing:

- 🔧 Interactive REPL (like redis-cli, but for Consul)
- 🗂️ Tree view, watch, prefix watch, edit, get-json, put-json
- ⚙️ Lightweight HTTP-only Consul client (no SDKs)
- 🌐 Integrations for Actix and Axum
- 🔁 First-class support for dynamic config & feature flags
- 🧩 Typed JSON prefix loading (`kv_list_json`)

Everything is implemented from scratch using `reqwest`.

---

## ✨ Features

### 🧵 REPL Interface
- Auto-completion (reedline)
- Commands:
  get, put, del, list, tree,
  get-json, put-json,
  edit,
  watch, watch-prefix

### 🗄️ JSON-aware KV APIs
- kv_get_json<T> — load typed JSON config
- kv_put_json<T> — write typed JSON
- kv_list_json<T> — load multiple JSON configs under a prefix

### 🔁 Watches
- Watch individual keys
- Watch entire prefixes
- Efficient blocking queries (index + wait)

### 🌍 Actix & Axum Integrations
Drop ConsulXClient into your application state.

### ⚡ Pure HTTP client
No consulrs. No SDK. No hidden magic.

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
CONSUL_HTTP_ADDR=http://127.0.0.1:8500 consulx
```

---

## 🖥️ REPL Commands

| Command        | Usage                   | Description                       |
| -------------- | ----------------------- | --------------------------------- |
| `get`          | `get <key>`             | Fetch raw value stored at a key   |
| `put`          | `put <key> <value>`     | Store a raw string value          |
| `del`          | `del <key>`             | Delete a key                      |
| `list`         | `list <prefix>`         | List all keys under a prefix      |
| `tree`         | `tree <prefix>`         | Show keys as an ASCII tree        |
| `get-json`     | `get-json <key>`        | Fetch and pretty-print JSON value |
| `put-json`     | `put-json <key> <json>` | Validate & store JSON value       |
| `edit`         | `edit <key>`            | Edit value in `$EDITOR`           |
| `watch`        | `watch <key>`           | Watch a single key for changes    |
| `watch-prefix` | `watch-prefix <prefix>` | Watch all keys under a prefix     |
| `help`         | `help`                  | Show available commands           |
| `exit`         | `exit` / `quit`         | Exit the REPL                     |


---

## 🧩 JSON Prefix Loading (kv_list_json)

```rust
#[derive(Deserialize)]
struct FeatureFlag {
    enabled: bool,
}

let flags = consul.kv_list_json::<FeatureFlag>("app/features/").await?;

for (key, flag) in flags {
    println!("{key} => {:?}", flag);
}
```

---

## 📦 Library API Summary

```rust
// Client creation
ConsulXClient::new(url)          // Create client with explicit Consul URL
ConsulXClient::from_env()        // Uses CONSUL_HTTP_ADDR (default: http://127.0.0.1:8500)

// Raw KV operations
kv_get_raw(key)                  // -> Option<String>
kv_put(key, value)               // Store raw value
kv_delete(key)                   // Delete a key
kv_list(prefix)                  // List keys under a prefix

// JSON helpers (typed)
kv_get_json<T>(key)              // -> Option<T>
kv_put_json<T>(key, &T)          // Serialize & store JSON
kv_list_json<T>(prefix)          // Load JSON objects under prefix

// Watches (blocking queries)
kv_watch(key)                    // Watch a single key
kv_watch_prefix(prefix)          // Watch all keys under prefix
```

---

Made with ❤️ by the [Srotas Space] (https://srotas.space/open-source)


---

## 👥 Contributors

- **[Snm Maurya](https://github.com/srotas-space)** - Creator & Lead Developer
  <img src="https://srotas-suite-space.s3.ap-south-1.amazonaws.com/snm.jpeg" alt="Snm Maurya" width="80" height="80" style="border-radius: 50%;">
  [LinkedIn](https://www.linkedin.com/in/srotas-space/)


[![GitHub stars](https://img.shields.io/github/stars/srotas-space/consulx?style=social)](https://github.com/srotas-space/consulx)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
