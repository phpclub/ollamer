# ollamerctl

CLI tool for managing the Ollamer model catalog (`index.json`). Replaces the Python generation scripts.

## Commands

### `init` — generate index.json from scratch

Queries the local Ollama API and builds a fresh `index.json`.

```bash
ollamerctl init
ollamerctl init /kvm/ollama/index.json
ollamerctl init /kvm/ollama/index.json --host http://localhost:11434
```

For each model, fetches:
- Size, digest, modification date — from `GET /api/tags`
- Architecture, context length, embedding length, quantization — from `POST /api/show`
- Capabilities, system prompt, inference parameters — from `POST /api/show`

Fields that cannot be derived automatically (`domain`, `language`, `tags`, `parent`) are set to defaults and should be edited manually in the resulting JSON.

### `update` — check freshness against registry

Reads an existing `index.json` and checks each model against the Ollama registry, updating `update_status` and `remote_digest`.

```bash
ollamerctl update
ollamerctl update /kvm/ollama/index.json
```

Status values written:

| Status | Meaning |
|---|---|
| `up_to_date` | Local manifest SHA256 matches registry |
| `update_available` | Newer version exists in registry |
| `local_only` | No manifest found in local registry path (custom Modelfile) |
| `not_in_registry` | Manifest exists locally but model not found in registry |
| `unknown` | Registry unreachable or check failed |

Freshness is determined by computing `SHA256(manifest_body_without_trailing_newline)` and comparing with the remote manifest from `registry.ollama.ai`.

## Build

```bash
cd /kvm/srv/rust/ollamer/app
cargo build --release
# binary: target/release/ollamerctl
```

## Typical workflow

```bash
# First time or after pulling new models:
ollamerctl init

# Check which models have updates available:
ollamerctl update

# Restart the web UI to pick up the new index.json:
fuser -k 7777/tcp
/kvm/srv/rust/ollamer/web/target/release/ollamer /kvm/ollama/index.json
```

## Notes

- Requires `curl` and `sha256sum` in PATH (standard on Linux).
- Default index path is `/kvm/ollama/index.json` for all commands.
- HuggingFace models (`hf.co/...`) are always marked `local_only` — the registry check only covers `registry.ollama.ai`.
