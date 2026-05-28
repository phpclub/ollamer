use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::process::Command as TokioCommand;
use std::process::Stdio;
use tokio::sync::Mutex;
use tokio::net::TcpListener;

// ---------------------------------------------------------------------------
// Localisation
// ---------------------------------------------------------------------------

const LOGO: &str = "data:image/png;base64,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";

const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("ru", "Русский"),
];

struct T {
    // header
    models: &'static str,
    occupied: &'static str,
    free: &'static str,
    checked: &'static str,
    // search
    search_placeholder: &'static str,
    // filters
    f_all: &'static str,
    f_updates: &'static str,
    type_models: &'static str,
    type_embedding: &'static str,
    // sort
    sort_label: &'static str,
    sort_name: &'static str,
    sort_size: &'static str,
    sort_date: &'static str,
    of: &'static str,
    // detail
    back: &'static str,
    sect_arch: &'static str,
    sect_ctx: &'static str,
    sect_params: &'static str,
    sect_system: &'static str,
    no_prompt: &'static str,
    no_params: &'static str,
    kv_family: &'static str,
    kv_parameters: &'static str,
    kv_quant: &'static str,
    kv_emb_len: &'static str,
    kv_max_ctx: &'static str,
    kv_num_ctx: &'static str,
    kv_size: &'static str,
    kv_modified: &'static str,
    kv_local_id: &'static str,
    kv_remote: &'static str,
    kv_requires: &'static str,
    kv_parent: &'static str,
    // update badges
    upd_new: &'static str,
    upd_new_title: &'static str,
    upd_local: &'static str,
    upd_local_title: &'static str,
    upd_nr: &'static str,
    upd_nr_title: &'static str,
    upd_unk: &'static str,
    upd_unk_title: &'static str,
    // pull
    pull_btn: &'static str,
    pull_cancel_btn: &'static str,
    pull_busy: &'static str,
    pull_done: &'static str,
    // misc
    not_found: &'static str,
}

fn t(lang: &str) -> T {
    match lang {
        "ru" => T {
            models:            "моделей",
            occupied:          "занято",
            free:              "свободно",
            checked:           "проверено",
            search_placeholder:"Поиск по названию...",
            f_all:             "Все",
            f_updates:         "⬆ обновления",
            type_models:       "Модели",
            type_embedding:    "Embedding",
            sort_label:        "сортировка:",
            sort_name:         "по имени",
            sort_size:         "размер",
            sort_date:         "дата",
            of:                "из",
            back:              "← Назад к списку",
            sect_arch:         "Архитектура",
            sect_ctx:          "Контекст и размер",
            sect_params:       "Inference параметры",
            sect_system:       "System Prompt",
            no_prompt:         "— системный промпт не задан —",
            no_params:         "— не настроены —",
            kv_family:         "Family",
            kv_parameters:     "Parameters",
            kv_quant:          "Quantization",
            kv_emb_len:        "Embedding length",
            kv_max_ctx:        "Max context",
            kv_num_ctx:        "num_ctx (загружен)",
            kv_size:           "Размер на диске",
            kv_modified:       "Изменён",
            kv_local_id:       "Local ID",
            kv_remote:         "Remote digest",
            kv_requires:       "Требует Ollama",
            kv_parent:         "На основе",
            upd_new:           "⬆ обновление",
            upd_new_title:     "Доступна новая версия в реестре",
            upd_local:         "🔨 custom",
            upd_local_title:   "Кастомный Modelfile, без записи в реестре",
            upd_nr:            "? реестр",
            upd_nr_title:      "Не найдена в реестре",
            upd_unk:           "– неизвестно",
            upd_unk_title:     "Статус не определён",
            pull_btn:          "⬆ Обновить",
            pull_cancel_btn:   "× Стоп",
            pull_busy:         "Уже идёт скачивание другой модели",
            pull_done:         "Готово!",
            not_found:         "Модель не найдена",
        },
        _ => T {
            models:            "models",
            occupied:          "occupied",
            free:              "free",
            checked:           "checked",
            search_placeholder:"Search by name...",
            f_all:             "All",
            f_updates:         "⬆ updates",
            type_models:       "Models",
            type_embedding:    "Embedding",
            sort_label:        "sort:",
            sort_name:         "by name",
            sort_size:         "size",
            sort_date:         "date",
            of:                "of",
            back:              "← Back to list",
            sect_arch:         "Architecture",
            sect_ctx:          "Context & Size",
            sect_params:       "Inference params",
            sect_system:       "System Prompt",
            no_prompt:         "— system prompt not set —",
            no_params:         "— not configured —",
            kv_family:         "Family",
            kv_parameters:     "Parameters",
            kv_quant:          "Quantization",
            kv_emb_len:        "Embedding length",
            kv_max_ctx:        "Max context",
            kv_num_ctx:        "num_ctx (loaded)",
            kv_size:           "Size on disk",
            kv_modified:       "Modified",
            kv_local_id:       "Local ID",
            kv_remote:         "Remote digest",
            kv_requires:       "Requires Ollama",
            kv_parent:         "Based on",
            upd_new:           "⬆ update",
            upd_new_title:     "Newer version available in registry",
            upd_local:         "🔨 custom",
            upd_local_title:   "Custom Modelfile, no registry entry",
            upd_nr:            "? registry",
            upd_nr_title:      "Not found in registry",
            upd_unk:           "– unknown",
            upd_unk_title:     "Status could not be determined",
            pull_btn:          "⬆ Pull update",
            pull_cancel_btn:   "× Stop",
            pull_busy:         "Another model is already being pulled",
            pull_done:         "Done!",
            not_found:         "Model not found",
        },
    }
}

fn lang_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                p.trim().strip_prefix("lang=").map(|v| v.trim().to_string())
            })
        })
        .filter(|l| LANGUAGES.iter().any(|(c, _)| c == l))
        .unwrap_or_else(|| "en".to_string())
}

fn lang_select(current: &str) -> String {
    let mut s = String::from(
        r#"<select onchange="setLang(this.value)" style="background:#1e2130;border:1px solid #2d3148;color:#e2e8f0;padding:0.3rem 0.6rem;border-radius:6px;font-size:0.8rem;cursor:pointer;margin-left:1rem">"#,
    );
    for (code, name) in LANGUAGES {
        let sel = if *code == current { " selected" } else { "" };
        s.push_str(&format!(r#"<option value="{code}"{sel}>{name}</option>"#));
    }
    s.push_str("</select>");
    s
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Model {
    name: String,
    id: String,
    size_bytes: u64,
    modified_at: String,
    architecture: String,
    parameters: String,
    context_length: u64,
    num_ctx: Option<u64>,
    embedding_length: u64,
    quantization: String,
    requires_ollama: Option<String>,
    capabilities: Vec<String>,
    role: String,
    domain: String,
    language: String,
    tags: Vec<String>,
    system_prompt: Option<String>,
    inference_params: HashMap<String, serde_json::Value>,
    #[serde(default)]
    update_status: String,
    #[serde(default)]
    remote_digest: Option<String>,
    #[serde(default)]
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Index {
    generated_at: String,
    ollama_host: String,
    total_models: usize,
    total_size_bytes: u64,
    total_size_gb: f64,
    #[serde(default)]
    freshness_checked_at: String,
    models: Vec<Model>,
}

// ---------------------------------------------------------------------------
// Pull state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
struct PullProgress {
    model: String,
    status: String,
    total: u64,
    completed: u64,
    running: bool,
    success: bool,
    error: Option<String>,
}

struct AppData {
    index: tokio::sync::RwLock<Index>,
    index_path: String,
    pull_progress: Mutex<Option<PullProgress>>,
    pull_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

type AppState = Arc<AppData>;

// ---------------------------------------------------------------------------
// Pull logic
// ---------------------------------------------------------------------------

async fn run_pull(model: String, state: AppState) {
    let body = format!(r#"{{"model":"{}","stream":true}}"#, model);

    let mut child = match TokioCommand::new("curl")
        .args([
            "-s", "-X", "POST",
            "http://localhost:11434/api/pull",
            "-H", "Content-Type: application/json",
            "-d", &body,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let mut p = state.pull_progress.lock().await;
            if let Some(ref mut prog) = *p {
                prog.running = false;
                prog.error = Some(format!("Cannot spawn curl: {e}"));
            }
            return;
        }
    };

    let stdout = child.stdout.take().unwrap();
    let mut reader = TokioBufReader::new(stdout);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    let status    = v["status"].as_str().unwrap_or("").to_string();
                    let total     = v["total"].as_u64().unwrap_or(0);
                    let completed = v["completed"].as_u64().unwrap_or(0);

                    let mut p = state.pull_progress.lock().await;
                    if let Some(ref mut prog) = *p {
                        if !status.is_empty() { prog.status = status.clone(); }
                        if total > 0 { prog.total = total; }
                        if completed > 0 { prog.completed = completed; }
                        if status == "success" {
                            prog.running = false;
                            prog.success = true;
                            drop(p);
                            // update in-memory index and persist to disk
                            {
                                let mut idx = state.index.write().await;
                                if let Some(m) = idx.models.iter_mut().find(|m| m.name == model) {
                                    m.update_status = "up_to_date".to_string();
                                }
                                let _ = std::fs::write(
                                    &state.index_path,
                                    serde_json::to_string_pretty(&*idx).unwrap_or_default(),
                                );
                            }
                            let _ = child.kill().await;
                            return;
                        }
                    } else {
                        let _ = child.kill().await;
                        return;
                    }
                }
            }
        }
    }

    let _ = child.wait().await;
    let mut p = state.pull_progress.lock().await;
    if let Some(ref mut prog) = *p {
        if prog.running {
            prog.running = false;
            if !prog.success {
                prog.error = Some("Pull ended unexpectedly".to_string());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn disk_free_gb(path: &str) -> Option<f64> {
    use std::ffi::CString;
    let cpath = CString::new(path).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(cpath.as_ptr(), &mut st) } != 0 {
        return None;
    }
    Some((st.f_bavail as u64 * st.f_frsize as u64) as f64 / 1e9)
}

fn fmt_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1e9)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1e6)
    } else {
        format!("{:.0} KB", bytes as f64 / 1e3)
    }
}

fn update_badge(status: &str, tr: &T) -> String {
    let (class, text, title) = match status {
        "up_to_date"       => return String::new(),
        "update_available" => ("upd-new",   tr.upd_new,   tr.upd_new_title),
        "local_only"       => ("upd-local", tr.upd_local, tr.upd_local_title),
        "not_in_registry"  => ("upd-nr",    tr.upd_nr,    tr.upd_nr_title),
        _                  => ("upd-unk",   tr.upd_unk,   tr.upd_unk_title),
    };
    format!(r#"<span class="badge {class}" title="{title}">{text}</span>"#)
}

fn capability_badge(cap: &str) -> String {
    let (class, label) = match cap {
        "completion" => ("cap-completion", "completion"),
        "tools"      => ("cap-tools",      "tools"),
        "thinking"   => ("cap-thinking",   "thinking"),
        "vision"     => ("cap-vision",     "vision"),
        "embedding"  => ("cap-embedding",  "embedding"),
        "insert"     => ("cap-insert",     "insert"),
        _            => ("",               cap),
    };
    format!(r#"<span class="badge {class}">{label}</span>"#)
}

fn domain_badge(domain: &str) -> &'static str {
    match domain {
        "pharma"  => r#"<span class="badge dom-pharma">pharma</span>"#,
        "medical" => r#"<span class="badge dom-medical">medical</span>"#,
        "coding"  => r#"<span class="badge dom-coding">coding</span>"#,
        _         => "",
    }
}

fn is_embedding_only(m: &Model) -> bool {
    m.capabilities.len() == 1 && m.capabilities[0] == "embedding"
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn encode_name(name: &str) -> String {
    name.replace('/', "__").replace(':', "--").replace('.', "_DOT_")
}

fn decode_name(slug: &str) -> String {
    slug.replace("__", "/").replace("--", ":").replace("_DOT_", ".")
}

// Returns (display_label, href) for the "Based on" row.
// Uses explicit parent if set; otherwise derives from the model name itself.
fn model_source(name: &str, parent: Option<&str>, local_names: &[String]) -> (String, String) {
    let target = parent.unwrap_or(name);

    if let Some(rest) = target.strip_prefix("hf.co/") {
        let model_part = rest.split(':').next().unwrap_or(rest);
        return (target.to_string(), format!("https://huggingface.co/{}", model_part));
    }

    if parent.is_some() && local_names.iter().any(|n| n == target) {
        return (target.to_string(), format!("/model/{}", encode_name(target)));
    }

    // ollama.com link: strip tag, add library/ prefix for unnamespaced models
    let without_tag = target.split(':').next().unwrap_or(target);
    let path = if without_tag.contains('/') {
        without_tag.to_string()
    } else {
        format!("library/{}", without_tag)
    };
    (without_tag.to_string(), format!("https://ollama.com/{}", path))
}

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

const CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: 'Segoe UI', system-ui, sans-serif; background: #0f1117; color: #e2e8f0; min-height: 100vh; }
a { color: #7dd3fc; text-decoration: none; }
a:hover { text-decoration: underline; }

header { background: #1e2130; border-bottom: 1px solid #2d3148; padding: 0.75rem 2rem; display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; }
header h1 { font-size: 1.4rem; font-weight: 700; color: #f1f5f9; }
header .meta { font-size: 0.8rem; color: #94a3b8; }
header .spacer { margin-left: auto; display: flex; align-items: center; }

.container { max-width: 1200px; margin: 0 auto; padding: 1.5rem 2rem; }
.filters { display: flex; gap: 0.6rem; margin-bottom: 1.5rem; flex-wrap: wrap; align-items: center; }
.filters input[type=text] { background: #1e2130; border: 1px solid #2d3148; color: #e2e8f0; padding: 0.4rem 0.75rem; border-radius: 6px; font-size: 0.85rem; width: 200px; }
.filters input[type=text]:focus { outline: 2px solid #3b82f6; }
.filter-btn { background: #1e2130; border: 1px solid #2d3148; color: #94a3b8; padding: 0.35rem 0.7rem; border-radius: 6px; font-size: 0.8rem; cursor: pointer; }
.filter-btn:hover, .filter-btn.active { background: #3b4170; color: #e2e8f0; border-color: #4f5fa8; }
.count { font-size: 0.8rem; color: #64748b; }

.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 1rem; }
.card { background: #1a1d2e; border: 1px solid #2a2d45; border-radius: 10px; padding: 1rem 1.2rem; transition: border-color 0.15s, transform 0.1s; position: relative; }
.card:hover { border-color: #4f5fa8; transform: translateY(-2px); }
.card-name { font-size: 0.92rem; font-weight: 600; color: #f1f5f9; margin-bottom: 0.4rem; word-break: break-all; }
.card-meta { font-size: 0.75rem; color: #64748b; margin-bottom: 0.6rem; }
.card-badges { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: 0.5rem; align-items: center; }

.badge { display: inline-block; padding: 0.15rem 0.45rem; border-radius: 4px; font-size: 0.7rem; font-weight: 600; }
.cap-completion { background: #1e3a5f; color: #7dd3fc; }
.cap-tools      { background: #3b2a1a; color: #fb923c; }
.cap-thinking   { background: #2a1e3b; color: #c084fc; }
.cap-vision     { background: #1e3b2a; color: #4ade80; }
.cap-embedding  { background: #1e2e3b; color: #38bdf8; }
.cap-insert     { background: #2a2a1e; color: #facc15; }
.dom-pharma     { background: #2d1a2e; color: #e879f9; }
.dom-medical    { background: #1a2d20; color: #34d399; }
.dom-coding     { background: #1a1e2d; color: #818cf8; }
.tag  { background: #1e2130; color: #64748b; border: 1px solid #2d3148; }
.lang { background: #1e2d2d; color: #5eead4; }
.upd-ok    { background: #1a2e1a; color: #4ade80; border: 1px solid #166534; }
.upd-new   { background: #2e1a1a; color: #f87171; border: 1px solid #991b1b; font-weight: 700; }
.upd-local { background: #1e2130; color: #94a3b8; border: 1px solid #334155; }
.upd-nr    { background: #2a2a1e; color: #a3a320; border: 1px solid #4b4b00; }
.upd-unk   { background: #1e1e2e; color: #52525b; border: 1px solid #27272a; }

.pull-btn { background: #1e3a1e; border: 1px solid #166534; color: #4ade80; padding: 0.2rem 0.55rem; border-radius: 4px; font-size: 0.72rem; font-weight: 600; cursor: pointer; margin-left: auto; }
.pull-btn:hover { background: #166534; }
.pull-btn:disabled { opacity: 0.4; cursor: not-allowed; }

.back { font-size: 0.85rem; color: #7dd3fc; margin-bottom: 1rem; display: inline-block; }
.detail-header { margin-bottom: 1.5rem; }
.detail-header h2 { font-size: 1.2rem; font-weight: 700; word-break: break-all; margin-bottom: 0.5rem; }
.info-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem; }
@media (max-width: 640px) { .info-grid { grid-template-columns: 1fr; } }
.info-box { background: #1a1d2e; border: 1px solid #2a2d45; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }
.info-box h3 { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b; margin-bottom: 0.75rem; }
.kv { display: flex; justify-content: space-between; align-items: baseline; padding: 0.25rem 0; border-bottom: 1px solid #1e2130; font-size: 0.82rem; gap: 1rem; }
.kv:last-child { border-bottom: none; }
.kv-key { color: #94a3b8; flex-shrink: 0; }
.kv-val { color: #e2e8f0; text-align: right; word-break: break-all; }
.system-box { background: #1a1d2e; border: 1px solid #2a2d45; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }
.system-box h3 { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b; margin-bottom: 0.75rem; }
.system-text { font-family: 'Courier New', monospace; font-size: 0.82rem; color: #cbd5e1; line-height: 1.6; white-space: pre-wrap; word-break: break-word; }
.no-prompt { color: #4b5563; font-style: italic; font-size: 0.85rem; }

/* pull progress banner */
#pull-banner { display:none; position:fixed; bottom:0; left:0; right:0; background:#1a1d2e; border-top:2px solid #2a2d45; padding:0.6rem 1.5rem; z-index:200; }
#pull-banner .pb-row { display:flex; align-items:center; gap:0.9rem; }
#pull-model { color:#7dd3fc; font-size:0.82rem; font-weight:600; min-width:160px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; max-width:280px; }
#pull-track { flex:1; background:#2d3148; border-radius:3px; height:5px; overflow:hidden; min-width:80px; }
#pull-bar   { height:100%; background:#3b82f6; width:0%; transition:width 0.4s; }
#pull-status { font-size:0.78rem; color:#94a3b8; min-width:120px; }
#pull-pct    { font-size:0.82rem; color:#e2e8f0; font-variant-numeric:tabular-nums; min-width:3.5rem; text-align:right; }
.pb-cancel { background:#2e1a1a; border:1px solid #991b1b; color:#f87171; padding:0.22rem 0.6rem; border-radius:4px; cursor:pointer; font-size:0.8rem; font-weight:600; flex-shrink:0; }
.pb-cancel:hover { background:#3d2020; }

.detail-pull { margin-bottom: 1.2rem; }
.detail-pull-btn { background:#1e3a1e; border:1px solid #166534; color:#4ade80; padding:0.45rem 1.1rem; border-radius:6px; font-size:0.88rem; font-weight:600; cursor:pointer; }
.detail-pull-btn:hover { background:#166534; }
.detail-pull-btn:disabled { opacity:0.4; cursor:not-allowed; }
"#;

// ---------------------------------------------------------------------------
// Pull banner + JS (shared between pages)
// ---------------------------------------------------------------------------

fn pull_js(tr: &T) -> String {
    format!(r#"
<div id="pull-banner">
  <div class="pb-row">
    <span id="pull-model"></span>
    <div id="pull-track"><div id="pull-bar"></div></div>
    <span id="pull-pct"></span>
    <span id="pull-status"></span>
    <button class="pb-cancel" onclick="cancelPull()">{cancel}</button>
  </div>
</div>
<script>
let _pullTimer = null;
function setLang(l) {{
  document.cookie = 'lang=' + l + ';path=/;max-age=31536000';
  location.reload();
}}
function startPull(model) {{
  fetch('/api/pull', {{
    method: 'POST',
    headers: {{'Content-Type': 'application/json'}},
    body: JSON.stringify({{model}})
  }}).then(r => r.json()).then(d => {{
    if (d.error) {{ alert(d.error); return; }}
    _startPolling();
  }});
}}
function cancelPull() {{
  fetch('/api/pull/cancel', {{method:'POST'}}).then(() => _checkStatus());
}}
function _startPolling() {{
  if (_pullTimer) return;
  _pullTimer = setInterval(_checkStatus, 600);
  _checkStatus();
}}
function _stopPolling() {{
  if (_pullTimer) {{ clearInterval(_pullTimer); _pullTimer = null; }}
}}
async function _checkStatus() {{
  let d;
  try {{ d = await fetch('/api/pull/status').then(r => r.json()); }} catch(e) {{ return; }}
  const banner = document.getElementById('pull-banner');
  if (!d || !d.model) {{ banner.style.display = 'none'; _stopPolling(); return; }}
  banner.style.display = 'block';
  document.getElementById('pull-model').textContent = d.model;
  const pct = d.total > 0 ? Math.round(d.completed * 100 / d.total) : 0;
  document.getElementById('pull-bar').style.width = pct + '%';
  document.getElementById('pull-pct').textContent = d.total > 0 ? pct + '%' : '';
  let st = d.status || '';
  if (d.total > 0) st += ' ' + (d.completed/1e9).toFixed(1) + '/' + (d.total/1e9).toFixed(1) + ' GB';
  if (d.success) {{ st = '✓ {done}'; document.getElementById('pull-bar').style.background='#22c55e'; }}
  if (d.error)   {{ st = '✗ ' + d.error; document.getElementById('pull-bar').style.background='#ef4444'; }}
  document.getElementById('pull-status').textContent = st;
  document.querySelector('.pb-cancel').style.display = d.running ? '' : 'none';
  if (!d.running) {{ setTimeout(() => {{ banner.style.display='none'; _stopPolling(); }}, 4000); _stopPolling(); }}
  else _startPolling();
  // disable all pull buttons while running
  document.querySelectorAll('.pull-btn,.detail-pull-btn').forEach(b => b.disabled = d.running);
}}
document.addEventListener('DOMContentLoaded', _checkStatus);
</script>"#,
        cancel = tr.pull_cancel_btn,
        done   = tr.pull_done,
    )
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

fn list_page(index: &Index, filter: Option<&str>, show_embedding: bool, disk_free: Option<f64>, lang: &str, tr: &T) -> String {
    let models: Vec<&Model> = if let Some(f) = filter {
        index.models.iter().filter(|m| {
            m.capabilities.iter().any(|c| c == f)
                || m.domain == f
                || m.language == f
                || m.role.contains(f)
                || m.tags.iter().any(|t| t == f)
                || m.update_status == f
        }).collect()
    } else if show_embedding {
        index.models.iter().filter(|m| is_embedding_only(m)).collect()
    } else {
        index.models.iter().filter(|m| !is_embedding_only(m)).collect()
    };

    let cards: String = models.iter().map(|m| {
        let caps: String = m.capabilities.iter().map(|c| capability_badge(c)).collect::<Vec<_>>().join(" ");
        let dom = domain_badge(&m.domain);
        let lang_badge = format!(r#"<span class="badge lang">{}</span>"#, m.language);
        let upd = update_badge(&m.update_status, tr);
        let date_raw = &m.modified_at[..10];
        let date = if m.update_status == "up_to_date" {
            format!(r#"{date_raw} <span style="color:#64748b;font-size:0.8em">latest</span>"#)
        } else {
            date_raw.to_string()
        };
        let size = fmt_size(m.size_bytes);
        let arch_params = format!("{} · {} · {}", m.architecture, m.parameters, m.quantization);
        let slug = encode_name(&m.name);
        let pull_btn = if m.update_status == "update_available" {
            format!(
                r#"<button class="pull-btn" onclick="event.preventDefault();event.stopPropagation();startPull('{}')">{}</button>"#,
                html_escape(&m.name), tr.pull_btn
            )
        } else {
            String::new()
        };
        format!(
            r#"<a href="/model/{slug}" style="display:block" data-size="{size_bytes}" data-name="{name_raw}" data-date="{date_raw}"><div class="card">
  <div class="card-name">{name}</div>
  <div class="card-meta">{arch_params} &nbsp;·&nbsp; {size} &nbsp;·&nbsp; {date_display}</div>
  <div class="card-badges">{caps} {dom} {lang_badge} {upd} {pull_btn}</div>
</div></a>"#,
            slug = slug,
            name = html_escape(&m.name),
            name_raw = m.name.to_lowercase(),
            size_bytes = m.size_bytes,
            date_raw = date_raw,
            date_display = date,
            arch_params = html_escape(&arch_params),
            size = size,
        )
    }).collect();

    let active = |f: Option<&str>| if filter == f { " active" } else { "" };
    let checked_date = if index.freshness_checked_at.len() >= 10 { &index.freshness_checked_at[..10] } else { "—" };
    let disk_str = disk_free.map(|gb| format!("{gb:.0} GB {}", tr.free)).unwrap_or_else(|| "—".to_string());
    let lang_sel = lang_select(lang);
    let pull_banner = pull_js(tr);

    format!(r#"<!DOCTYPE html>
<html lang="{lang}">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ollamer</title>
<style>{css}</style>
</head>
<body>
<header>
  <h1><a href="/" style="text-decoration:none;color:inherit"><img src="{logo}" alt="" style="height:32px;width:32px;object-fit:contain;vertical-align:middle;margin-right:0.4rem"> Ollamer</a></h1>
  <div class="meta">{total} {tr_models} &nbsp;·&nbsp; {size_gb} GB {tr_occupied} &nbsp;·&nbsp; {disk_str} &nbsp;·&nbsp; {tr_checked}: {checked_date}</div>
  <div class="spacer">{lang_sel}</div>
</header>
<div class="container">
  <div class="filters">
    <input type="text" id="search" placeholder="{tr_search}" oninput="applyFilters()">
    <a href="/" class="filter-btn{a_all}">{tr_all}</a>
    <a href="/?filter=completion" class="filter-btn{a_comp}">completion</a>
    <a href="/?filter=tools" class="filter-btn{a_tools}">tools</a>
    <a href="/?filter=vision" class="filter-btn{a_vis}">vision</a>
    <a href="/?filter=thinking" class="filter-btn{a_think}">thinking</a>
    <a href="/?filter=pharma" class="filter-btn{a_pharma}">pharma</a>
    <a href="/?filter=medical" class="filter-btn{a_med}">medical</a>
    <a href="/?filter=coding" class="filter-btn{a_cod}">coding</a>
    <a href="/?filter=ru" class="filter-btn{a_ru}">🇷🇺 ru</a>
    <a href="/?filter=update_available" class="filter-btn{a_upd}">{tr_updates}</a>
    <select onchange="location.href=this.value" style="background:#1e2130;border:1px solid #2d3148;color:#e2e8f0;padding:0.35rem 0.6rem;border-radius:6px;font-size:0.8rem;cursor:pointer">
      <option value="/" {sel_models}>{tr_type_models}</option>
      <option value="/?type=embedding" {sel_emb}>{tr_type_emb}</option>
    </select>
    <div style="display:flex;align-items:center;gap:0.5rem;margin-left:auto">
      <span style="font-size:0.75rem;color:#64748b">{tr_sort_label}</span>
      <button class="filter-btn active" id="sort-name" onclick="sortCards('name')">{tr_sort_name}</button>
      <button class="filter-btn" id="sort-size" onclick="sortCards('size')">{tr_sort_size} ↑</button>
      <button class="filter-btn" id="sort-date" onclick="sortCards('date')">{tr_sort_date} ↓</button>
      <span class="count" id="count">{shown} {tr_of} {total}</span>
    </div>
  </div>
  <div class="grid" id="grid">{cards}</div>
</div>
{pull_banner}
<script>
const TR_OF = '{tr_of}';
function applyFilters() {{
  const q = document.getElementById('search').value.toLowerCase();
  const cards = document.querySelectorAll('#grid > a');
  let shown = 0;
  cards.forEach(c => {{
    const vis = !q || c.textContent.toLowerCase().includes(q);
    c.style.display = vis ? '' : 'none';
    if (vis) shown++;
  }});
  document.getElementById('count').textContent = shown + ' ' + TR_OF + ' {total}';
}}
const SORT_SIZE_LABEL = '{tr_sort_size}';
const SORT_DATE_LABEL = '{tr_sort_date}';
let _sortMode = 'name';
function sortCards(mode) {{
  if (mode === 'size') {{
    _sortMode = (_sortMode === 'size-asc') ? 'size-desc' : 'size-asc';
    document.getElementById('sort-size').textContent = SORT_SIZE_LABEL + (_sortMode === 'size-asc' ? ' ↑' : ' ↓');
  }} else if (mode === 'date') {{
    _sortMode = (_sortMode === 'date-desc') ? 'date-asc' : 'date-desc';
    document.getElementById('sort-date').textContent = SORT_DATE_LABEL + (_sortMode === 'date-desc' ? ' ↓' : ' ↑');
  }} else {{
    _sortMode = 'name';
  }}
  const activeId = _sortMode === 'name' ? 'name' : _sortMode.startsWith('size') ? 'size' : 'date';
  ['name','size','date'].forEach(m => document.getElementById('sort-'+m).classList.remove('active'));
  document.getElementById('sort-'+activeId).classList.add('active');
  const grid = document.getElementById('grid');
  const cards = Array.from(grid.querySelectorAll(':scope > a'));
  cards.sort((a, b) => {{
    if (_sortMode === 'name') return a.dataset.name.localeCompare(b.dataset.name);
    if (_sortMode === 'date-desc') return b.dataset.date.localeCompare(a.dataset.date);
    if (_sortMode === 'date-asc')  return a.dataset.date.localeCompare(b.dataset.date);
    const sa = parseInt(a.dataset.size), sb = parseInt(b.dataset.size);
    return _sortMode === 'size-asc' ? sa - sb : sb - sa;
  }});
  cards.forEach(c => grid.appendChild(c));
}}
</script>
</body></html>"#,
        lang = lang, css = CSS, logo = LOGO,
        total = index.total_models, size_gb = index.total_size_gb,
        shown = models.len(), cards = cards, lang_sel = lang_sel,
        disk_str = disk_str, checked_date = checked_date, pull_banner = pull_banner,
        tr_models = tr.models, tr_occupied = tr.occupied, tr_checked = tr.checked,
        tr_search = tr.search_placeholder, tr_all = tr.f_all, tr_updates = tr.f_updates,
        tr_type_models = tr.type_models, tr_type_emb = tr.type_embedding,
        tr_sort_label = tr.sort_label, tr_sort_name = tr.sort_name,
        tr_sort_size = tr.sort_size, tr_sort_date = tr.sort_date, tr_of = tr.of,
        a_all = active(None), a_comp = active(Some("completion")),
        a_tools = active(Some("tools")), a_vis = active(Some("vision")), a_think = active(Some("thinking")),
        a_pharma = active(Some("pharma")), a_med = active(Some("medical")),
        a_cod = active(Some("coding")), a_ru = active(Some("ru")),
        a_upd = active(Some("update_available")),
        sel_models = if !show_embedding { "selected" } else { "" },
        sel_emb    = if  show_embedding { "selected" } else { "" },
    )
}

fn detail_page(model: &Model, lang: &str, tr: &T, index: &Index) -> String {
    let caps: String = model.capabilities.iter().map(|c| capability_badge(c)).collect::<Vec<_>>().join(" ");
    let dom = domain_badge(&model.domain);
    let tags: String = model.tags.iter()
        .map(|tag| format!(r#"<span class="badge tag">{}</span>"#, html_escape(tag)))
        .collect::<Vec<_>>().join(" ");
    let upd_badge = update_badge(&model.update_status, tr);

    let pull_block = if model.update_status == "update_available" {
        format!(
            r#"<div class="detail-pull">
  <button class="detail-pull-btn" onclick="startPull('{}')">{}</button>
</div>"#,
            html_escape(&model.name), tr.pull_btn
        )
    } else {
        String::new()
    };

    let system_block = match &model.system_prompt {
        Some(p) => format!(r#"<pre class="system-text">{}</pre>"#, html_escape(p)),
        None    => format!(r#"<div class="no-prompt">{}</div>"#, tr.no_prompt),
    };
    let params_block: String = if model.inference_params.is_empty() {
        format!(r#"<div class="no-prompt">{}</div>"#, tr.no_params)
    } else {
        model.inference_params.iter().map(|(k, v)| format!(
            r#"<div class="kv"><span class="kv-key">{}</span><span class="kv-val">{}</span></div>"#,
            html_escape(k), html_escape(&v.to_string())
        )).collect()
    };

    let requires    = model.requires_ollama.as_deref().unwrap_or("—");
    let num_ctx_str = model.num_ctx.map(|v| v.to_string()).unwrap_or_else(|| "—".to_string());
    let remote_dig  = model.remote_digest.as_deref().unwrap_or("—");
    let modified_str = if model.update_status == "up_to_date" {
        format!(r#"{} <span style="color:#64748b;font-size:0.85em">latest</span>"#, &model.modified_at[..10])
    } else {
        model.modified_at[..10].to_string()
    };
    let local_names: Vec<String> = index.models.iter().map(|m| m.name.clone()).collect();
    let (src_label, src_href) = model_source(&model.name, model.parent.as_deref(), &local_names);
    let src_target = if src_href.starts_with("https://") { r#" target="_blank" rel="noopener""# } else { "" };
    let parent_row = format!(
        r#"<div class="kv"><span class="kv-key">{}</span><span class="kv-val"><a href="{}"{} style="color:#60a5fa">{}</a></span></div>"#,
        tr.kv_parent, html_escape(&src_href), src_target, html_escape(&src_label)
    );
    let lang_sel    = lang_select(lang);
    let pull_banner = pull_js(tr);

    format!(r#"<!DOCTYPE html>
<html lang="{lang}">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{name} — Ollamer</title>
<style>{css}</style>
</head>
<body>
<header>
  <h1><a href="/" style="text-decoration:none;color:inherit"><img src="{logo}" alt="" style="height:32px;width:32px;object-fit:contain;vertical-align:middle;margin-right:0.4rem"> Ollamer</a></h1>
  <div class="meta">{size}</div>
  <div class="spacer">{lang_sel}</div>
</header>
<div class="container">
  <a class="back" href="/">{tr_back}</a>
  <div class="detail-header">
    <h2>{name}</h2>
    <div class="card-badges">{caps} {dom} <span class="badge lang">{model_lang}</span> {tags} {upd_badge}</div>
  </div>
  {pull_block}

  <div class="info-grid">
    <div class="info-box">
      <h3>{tr_sect_arch}</h3>
      <div class="kv"><span class="kv-key">{tr_family}</span><span class="kv-val">{arch}</span></div>
      <div class="kv"><span class="kv-key">{tr_parameters}</span><span class="kv-val">{params}</span></div>
      <div class="kv"><span class="kv-key">{tr_quant}</span><span class="kv-val">{quant}</span></div>
      <div class="kv"><span class="kv-key">{tr_emb_len}</span><span class="kv-val">{emb}</span></div>
    </div>
    <div class="info-box">
      <h3>{tr_sect_ctx}</h3>
      <div class="kv"><span class="kv-key">{tr_max_ctx}</span><span class="kv-val">{ctx_max}</span></div>
      <div class="kv"><span class="kv-key">{tr_num_ctx}</span><span class="kv-val">{num_ctx}</span></div>
      <div class="kv"><span class="kv-key">{tr_size}</span><span class="kv-val">{size}</span></div>
      <div class="kv"><span class="kv-key">{tr_modified}</span><span class="kv-val">{modified_str}</span></div>
      <div class="kv"><span class="kv-key">{tr_local_id}</span><span class="kv-val">{model_id}</span></div>
      <div class="kv"><span class="kv-key">{tr_remote}</span><span class="kv-val">{remote_dig}</span></div>
      <div class="kv"><span class="kv-key">{tr_requires}</span><span class="kv-val">{requires}</span></div>
      {parent_row}
    </div>
  </div>

  <div class="info-box">
    <h3>{tr_sect_params}</h3>
    {params_block}
  </div>

  <div class="system-box">
    <h3>{tr_sect_system}</h3>
    {system_block}
  </div>
</div>
{pull_banner}
</body></html>"#,
        lang = lang, css = CSS, logo = LOGO,
        name = html_escape(&model.name), size = fmt_size(model.size_bytes),
        arch = html_escape(&model.architecture), params = html_escape(&model.parameters),
        quant = html_escape(&model.quantization), emb = model.embedding_length,
        ctx_max = model.context_length, num_ctx = num_ctx_str,
        modified_str = modified_str, model_id = html_escape(&model.id),
        remote_dig = html_escape(remote_dig), requires = html_escape(requires),
        model_lang = html_escape(&model.language),
        caps = caps, dom = dom, tags = tags, upd_badge = upd_badge,
        lang_sel = lang_sel, pull_block = pull_block, pull_banner = pull_banner,
        params_block = params_block, system_block = system_block,
        tr_back = tr.back, tr_sect_arch = tr.sect_arch, tr_sect_ctx = tr.sect_ctx,
        tr_sect_params = tr.sect_params, tr_sect_system = tr.sect_system,
        tr_family = tr.kv_family, tr_parameters = tr.kv_parameters,
        tr_quant = tr.kv_quant, tr_emb_len = tr.kv_emb_len,
        tr_max_ctx = tr.kv_max_ctx, tr_num_ctx = tr.kv_num_ctx,
        tr_size = tr.kv_size, tr_modified = tr.kv_modified,
        tr_local_id = tr.kv_local_id, tr_remote = tr.kv_remote,
        tr_requires = tr.kv_requires,
        parent_row = parent_row,
    )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Html<String> {
    let lang = lang_from_headers(&headers);
    let tr = t(&lang);
    let filter = params.get("filter").map(|s| s.as_str());
    let show_embedding = params.get("type").map(|s| s == "embedding").unwrap_or(false);
    let disk_free = disk_free_gb("/kvm/ollama");
    Html(list_page(&*state.index.read().await, filter, show_embedding, disk_free, &lang, &tr))
}

async fn detail_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let lang = lang_from_headers(&headers);
    let tr = t(&lang);
    let name = decode_name(&slug);
    let index = state.index.read().await;
    match index.models.iter().find(|m| m.name == name) {
        Some(model) => Html(detail_page(model, &lang, &tr, &index)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Html(format!("<h1>{}: {}</h1>", tr.not_found, html_escape(&name))),
        ).into_response(),
    }
}

async fn api_models(State(state): State<AppState>) -> Json<Vec<Model>> {
    Json(state.index.read().await.models.clone())
}

#[derive(Deserialize)]
struct PullRequest {
    model: String,
}

async fn pull_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PullRequest>,
) -> Json<serde_json::Value> {
    let lang = lang_from_headers(&headers);
    let tr = t(&lang);

    let mut progress = state.pull_progress.lock().await;
    if progress.as_ref().map(|p| p.running).unwrap_or(false) {
        return Json(serde_json::json!({"error": tr.pull_busy}));
    }

    *progress = Some(PullProgress {
        model: body.model.clone(),
        status: "starting…".to_string(),
        total: 0,
        completed: 0,
        running: true,
        success: false,
        error: None,
    });
    drop(progress);

    let model = body.model.clone();
    let state2 = state.clone();
    let handle = tokio::spawn(async move {
        run_pull(model, state2).await;
    });

    *state.pull_task.lock().await = Some(handle);
    Json(serde_json::json!({"ok": true}))
}

async fn pull_cancel(State(state): State<AppState>) -> Json<serde_json::Value> {
    // abort the task (drops TCP connection → Ollama cancels pull)
    if let Some(handle) = state.pull_task.lock().await.take() {
        handle.abort();
    }
    let mut p = state.pull_progress.lock().await;
    if let Some(ref mut prog) = *p {
        prog.running = false;
        prog.error = Some("Cancelled".to_string());
    }
    Json(serde_json::json!({"ok": true}))
}

async fn pull_status(State(state): State<AppState>) -> Json<Option<PullProgress>> {
    let p = state.pull_progress.lock().await;
    Json(p.clone())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let index_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{home}/.ollama/index.json")
        });

    let data = std::fs::read_to_string(&index_path)
        .unwrap_or_else(|e| panic!("Cannot read {index_path}: {e}"));
    let index: Index =
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("Cannot parse {index_path}: {e}"));

    println!("Loaded {} models from {}", index.total_models, index_path);
    println!("Languages: {}", LANGUAGES.iter().map(|(c, _)| *c).collect::<Vec<_>>().join(", "));

    let state = Arc::new(AppData {
        index: tokio::sync::RwLock::new(index),
        index_path: index_path.to_string(),
        pull_progress: Mutex::new(None),
        pull_task: Mutex::new(None),
    });

    let app = Router::new()
        .route("/", get(list_handler))
        .route("/model/:slug", get(detail_handler))
        .route("/api/models", get(api_models))
        .route("/api/pull", post(pull_start))
        .route("/api/pull/cancel", post(pull_cancel))
        .route("/api/pull/status", get(pull_status))
        .with_state(state);

    let addr = "0.0.0.0:7777";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
