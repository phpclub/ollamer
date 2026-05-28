# Ollamer

A lightweight web UI for browsing local [Ollama](https://ollama.com) models. Built with Rust + Axum, reads a pre-built `index.json` catalog and serves a dark-theme dashboard on port 7777.

## Features

- **Model catalog** — cards with architecture, parameter count, quantization, size, and modification date
- **Capability filters** — completion, tools, thinking, vision, pharma, medical, coding, Russian-language
- **Embedding toggle** — embedding-only models are hidden by default, revealed via checkbox
- **Update status** — checks each model against the Ollama registry (`registry.ollama.ai`) and HuggingFace, shows ✓ актуальна / ⬆ обновление / ⚙ локальная badges
- **Disk usage** — header shows total model storage used and free space on the Ollama volume (live `statvfs` per request)
- **Sort** — by name, size ↑/↓, or modification date
- **Detail page** — full system prompt, inference parameters, local ID vs. remote digest comparison
- **JSON API** — `GET /api/models` returns the full model list

## Requirements

- Rust 1.75+ (edition 2024)
- Ollama running on `localhost:11434`
- Python 3 (only for generating/refreshing `index.json`)

## Setup

### 1. Generate index.json

Run once (or after pulling new models) from the Ollama data directory:

```bash
cd /kvm/ollama
ollama list           # verify models are present
python3 generate_index.py   # produces index.json
```

The generation script calls `ollama show --system` for each model and queries the Ollama REST API (`/api/tags`) for precise sizes and digests.

### 2. Check freshness against registry

```bash
python3 check_freshness.py   # updates update_status / remote_digest fields in index.json
```

This computes `SHA256(manifest_body)` for each model and compares it to the local digest. Custom Modelfiles (no registry entry) are marked `local_only`.

### 3. Build and run

```bash
cd /kvm/srv/rust/ollamer/web
cargo build --release
./target/release/ollamer /kvm/ollama/index.json
# Listening on http://0.0.0.0:7777
```

Pass the path to `index.json` as the first argument (default: `/kvm/ollama/index.json`).

## Routes

| Route | Description |
|---|---|
| `GET /` | Model list with filters and sort |
| `GET /?filter=<value>` | Filtered list — capability, domain, language, `update_available`, `ru` |
| `GET /model/<slug>` | Detail page with system prompt and inference params |
| `GET /api/models` | Full model array as JSON |

Model name slug encoding: `/` → `__`, `:` → `--`, `.` → `_DOT_`.

## index.json schema

```jsonc
{
  "generated_at": "2026-05-28T00:00:00+03:00",
  "ollama_host": "http://localhost:11434",
  "total_models": 25,
  "total_size_bytes": 173938529816,
  "total_size_gb": 173.94,
  "freshness_checked_at": "2026-05-28T06:00:00+03:00",
  "models": [
    {
      "name": "qwen3:8b-q8_0",
      "id": "e56358ca25dd",          // short digest (first 12 hex chars)
      "size_bytes": 8851089538,
      "modified_at": "2026-02-19T03:01:48+03:00",
      "architecture": "qwen3",
      "parameters": "8.2B",
      "context_length": 40960,
      "num_ctx": null,               // configured num_ctx override, if set
      "embedding_length": 4096,
      "quantization": "Q8_0",
      "requires_ollama": null,       // minimum Ollama version, if any
      "capabilities": ["completion", "tools", "thinking"],
      "role": "general",
      "domain": "general",          // general | coding | pharma | medical
      "language": "multilingual",   // ru | en | multilingual
      "tags": ["tools", "thinking", "medium"],
      "system_prompt": null,
      "inference_params": { "temperature": 0.6, "top_k": 20 },
      "update_status": "up_to_date", // up_to_date | update_available | local_only | not_in_registry | unknown
      "remote_digest": "e56358ca25dd..."
    }
  ]
}
```

## Model collection overview

25 models, 173.94 GB total (as of 2026-05-28):

| Domain | Count |
|---|---|
| General | 14 |
| Medical | 4 |
| Coding | 4 |
| Pharma (RU) | 3 |

| Capability | Count |
|---|---|
| completion | 19 |
| tools | 8 |
| vision | 4 |
| thinking | 3 |
| embedding | 6 |
| insert | 2 |

## Tech stack

| Component | Library |
|---|---|
| HTTP server | [axum](https://github.com/tokio-rs/axum) 0.7 |
| Async runtime | [tokio](https://tokio.rs) 1 |
| JSON | serde_json 1 |
| Disk stats | libc `statvfs` |
| Frontend | Vanilla HTML/CSS/JS (no build step) |
