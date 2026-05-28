# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Ollamer is a local Ollama model management system with two independent Rust crates and a shared JSON catalog:

```
/kvm/ollama/index.json   # shared catalog (source of truth)
web/                     # Axum web UI — port 7777
ollamerctl/              # CLI tool — ollamerctl
```

There is no workspace `Cargo.toml`. Each crate is built independently.

## Build & run

```bash
# Web UI
cd web && cargo build --release
./target/release/ollamer /kvm/ollama/index.json
# → http://0.0.0.0:7777

# CLI
cd ollamerctl && cargo build --release
./target/release/ollamerctl init        # build index.json from Ollama API
./target/release/ollamerctl update      # re-check freshness against registry
./target/release/ollamerctl --help
```

Debug builds work fine for development — just drop `--release`.

Neither crate has tests. Lint with `cargo clippy` inside each subdirectory.

## Architecture

### web/ — single-file Axum server

`web/src/main.rs` is the entire web application. Key design decisions:

- **No template engine.** All HTML, CSS, and JS are `&'static str` / `String` assembled in `list_page()` and `detail_page()`. The CSS block is the `CSS` constant; JS is inlined via `pull_js()`.
- **No static file serving.** Everything renders server-side from in-memory state.
- **State:** `Arc<AppData>` holds an `RwLock<Index>` (read on every request) and a `Mutex<Option<PullProgress>>` + `Mutex<Option<JoinHandle>>` for the pull workflow.
- **Pull:** `POST /api/pull` spawns a `tokio::task` that drives `curl` as a subprocess (streaming NDJSON from Ollama's `/api/pull`). Cancel aborts the task (drops the TCP connection).
- **Language:** Detected from a `lang=` cookie, resolved by `lang_from_headers()`. The `T` struct holds all UI strings; `t("ru")` / `t("en")` are the only two variants.
- **Model URL slugs:** `/` → `__`, `:` → `--`, `.` → `_DOT_` (see `encode_name` / `decode_name`).
- **Disk stats:** `libc::statvfs` on `/kvm/ollama` per request.

Routes: `GET /`, `GET /?filter=<cap|domain|lang|update_available>`, `GET /?type=embedding`, `GET /model/<slug>`, `GET /api/models`, `POST /api/pull`, `POST /api/pull/cancel`, `GET /api/pull/status`.

### app/ — single-file CLI

`app/src/main.rs` has no async runtime. All HTTP is done via `curl` subprocess (`curl_get`, `curl_post`, `curl_get_with_header`).

- **`init`**: Calls `GET /api/tags` then `POST /api/show` for each model, parses modelfile for system prompt and inference params, writes `index.json`.
- **`update`**: Reads local OCI manifest files from `/kvm/ollama/models/manifests/`, computes `sha256sum` via subprocess, fetches remote manifest from `registry.ollama.ai`, compares digests to set `update_status`.

### Shared data model

`Model` and `Index` structs are **duplicated** in both crates (not a shared library). Keep them in sync when adding fields.

`update_status` values: `up_to_date`, `update_available`, `local_only`, `not_in_registry`, `unknown`.

## index.json

Default path: `/kvm/ollama/index.json`. Both binaries accept an alternate path as the first argument. The web server writes back to this file after a successful pull (sets `update_status = "up_to_date"`).

## Typical workflow after adding a model

```bash
ollama pull <model>
/kvm/srv/rust/ollamer/ollamerctl/target/release/ollamerctl init
/kvm/srv/rust/ollamer/ollamerctl/target/release/ollamerctl update
fuser -k 7777/tcp
/kvm/srv/rust/ollamer/web/target/release/ollamer /kvm/ollama/index.json
```
