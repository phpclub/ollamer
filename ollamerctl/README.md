# ollamerctl

CLI tool to generate and maintain the [Ollamer](https://crates.io/crates/ollamer) model catalog (`index.json`) from the local Ollama API.

## Install

```bash
cargo install ollamerctl
```

Requires `curl` and `sha256sum` in PATH (standard on Linux/macOS).

## Commands

### `init` — generate index.json from scratch

Queries the local Ollama API and builds a fresh `index.json`.

```bash
ollamerctl init
ollamerctl init ~/my-index.json
ollamerctl init ~/my-index.json --host http://localhost:11434
```

For each model, fetches:
- Size, digest, modification date — from `GET /api/tags`
- Architecture, context length, embedding length, quantization — from `POST /api/show`
- Capabilities, system prompt, inference parameters — from `POST /api/show`

Fields that cannot be derived automatically (`domain`, `language`, `tags`, `parent`) are set to defaults and can be edited manually in the resulting JSON.

### `update` — check freshness against registry

Reads an existing `index.json` and updates `update_status` and `remote_digest` for each model.

```bash
ollamerctl update
ollamerctl update ~/my-index.json
```

| Status | Meaning |
|---|---|
| `up_to_date` | Local manifest matches registry |
| `update_available` | Newer version exists in registry |
| `local_only` | Custom Modelfile, no registry entry |
| `not_in_registry` | Model not found in registry |
| `unknown` | Registry unreachable or check failed |

## Typical workflow

```bash
# After pulling new models with `ollama pull`:
ollamerctl init

# Check which models have updates available:
ollamerctl update

# Start the web UI:
ollamer index.json
```

Default index path: `~/.ollama/index.json`.  
HuggingFace models (`hf.co/...`) are always marked `local_only`.
