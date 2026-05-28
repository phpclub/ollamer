# Adding a new language to Ollamer

All UI strings live in a single file: `src/main.rs`. Adding a language requires editing two places in that file — no external crates, no JSON files, no build scripts.

## Step 1 — Register the language code

Find the `LANGUAGES` constant near the top of `src/main.rs`:

```rust
const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("ru", "Русский"),
];
```

Append your language using its [BCP 47 subtag](https://www.iana.org/assignments/language-subtag-registry) and its native name:

```rust
    ("de", "Deutsch"),
```

This code will appear in the browser cookie (`lang=de`, `max-age` 1 year) and in the `<html lang="">` attribute.

## Step 2 — Add a match arm in `fn t()`

Find the function `fn t(lang: &str) -> T` and add a new arm **before** the default `_ =>` arm:

```rust
"de" => T {
    models:            "Modelle",
    occupied:          "belegt",
    free:              "frei",
    checked:           "geprüft",
    search_placeholder:"Nach Name suchen...",
    f_all:             "Alle",
    f_updates:         "⬆ Updates",
    type_models:       "Modelle",
    type_embedding:    "Embedding",
    sort_label:        "Sortierung:",
    sort_name:         "nach Name",
    sort_size_asc:     "Größe ↑",
    sort_size_desc:    "Größe ↓",
    sort_date:         "Datum ↓",
    of:                "von",
    back:              "← Zurück zur Liste",
    sect_arch:         "Architektur",
    sect_ctx:          "Kontext & Größe",
    sect_params:       "Inferenz-Parameter",
    sect_system:       "System-Prompt",
    no_prompt:         "— kein System-Prompt gesetzt —",
    no_params:         "— nicht konfiguriert —",
    kv_family:         "Familie",
    kv_parameters:     "Parameter",
    kv_quant:          "Quantisierung",
    kv_emb_len:        "Embedding-Länge",
    kv_max_ctx:        "Max. Kontext",
    kv_num_ctx:        "num_ctx (geladen)",
    kv_size:           "Größe auf Disk",
    kv_modified:       "Geändert",
    kv_local_id:       "Lokale ID",
    kv_remote:         "Remote-Digest",
    kv_requires:       "Erfordert Ollama",
    upd_ok:            "✓ aktuell",
    upd_ok_title:      "Neueste Version",
    upd_new:           "⬆ Update",
    upd_new_title:     "Neuere Version in der Registry verfügbar",
    upd_local:         "⚙ nur lokal",
    upd_local_title:   "Eigene Modelfile, kein Registry-Eintrag",
    upd_nr:            "? Registry",
    upd_nr_title:      "Nicht in der Registry gefunden",
    upd_unk:           "– unbekannt",
    upd_unk_title:     "Status konnte nicht ermittelt werden",
    not_found:         "Modell nicht gefunden",
},
```

Copy the English `_` arm as a starting point — every field must be filled; the compiler will error if any is missing.

## Step 3 — Rebuild

```bash
cd /kvm/srv/rust/ollamer/web
cargo build --release
./target/release/ollamer /kvm/ollama/index.json
```

The new language will appear immediately in the dropdown selector on every page.

---

## Field reference

| Field | Where it appears |
|---|---|
| `models` | Header: "25 models" |
| `occupied` | Header: "173 GB occupied" |
| `free` | Header: "69 GB free" |
| `checked` | Header: "checked: 2026-05-28" |
| `search_placeholder` | Search input placeholder |
| `f_all` | "All" filter button |
| `f_updates` | "⬆ updates" filter button |
| `type_models` | Type selector first option |
| `type_embedding` | Type selector second option |
| `sort_label` | Label before sort buttons |
| `sort_name` / `sort_size_asc` / `sort_size_desc` / `sort_date` | Sort buttons |
| `of` | Count display: "19 **of** 25"; also injected into JS |
| `back` | "← Back to list" link on detail page |
| `sect_arch` / `sect_ctx` / `sect_params` / `sect_system` | Section headings on detail page |
| `no_prompt` | Shown when model has no system prompt |
| `no_params` | Shown when inference params are empty |
| `kv_*` | Key labels in the detail page info boxes |
| `upd_ok` / `upd_new` / `upd_local` / `upd_nr` / `upd_unk` | Update badge text |
| `upd_*_title` | Update badge tooltip (`title` attribute) |
| `not_found` | 404 page heading |

## Notes

- Capability badge labels (`completion`, `tools`, `thinking`, `vision`, `embedding`, `insert`) and domain badges (`pharma`, `medical`, `coding`) are **not** translated — they are technical identifiers used as filter values.
- The language cookie is `lang=<code>`, path `/`, `max-age` 31 536 000 (1 year). It is set client-side via `document.cookie` on select change.
- Unknown or unregistered cookie values fall back to English.
