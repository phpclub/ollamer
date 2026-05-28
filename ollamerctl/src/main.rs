use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Types (mirror web/src/main.rs Model + Index)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    update_status: String,
    remote_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
// HTTP helpers (via curl subprocess)
// ---------------------------------------------------------------------------

fn curl_get(url: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-s", "--max-time", "30", url])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn curl_post(url: &str, body: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-s", "--max-time", "30", "-X", "POST",
               "-H", "Content-Type: application/json",
               "-d", body, url])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn curl_get_with_header(url: &str, header: &str) -> Result<String, String> {
    let out = Command::new("curl")
        .args(["-s", "--max-time", "30", "-H", header, url])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// ---------------------------------------------------------------------------
// Parse helpers
// ---------------------------------------------------------------------------

fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, mo, d, h, mi, s) = epoch_to_ymd(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}

fn epoch_to_ymd(mut t: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s  = t % 60; t /= 60;
    let mi = t % 60; t /= 60;
    let h  = t % 24; t /= 24;
    let mut y = 1970u64;
    loop {
        let dy = if is_leap(y) { 366 } else { 365 };
        if t < dy { break; }
        t -= dy; y += 1;
    }
    let months = [31u64, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 1u64;
    for dm in &months {
        if t < *dm { break; }
        t -= dm; mo += 1;
    }
    (y, mo, t + 1, h, mi, s)
}

fn is_leap(y: u64) -> bool { y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) }

fn parse_params(raw: &str) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();
    for line in raw.lines() {
        let mut parts = line.splitn(2, char::is_whitespace);
        let key = match parts.next() { Some(k) if !k.is_empty() => k.to_string(), _ => continue };
        let val = parts.next().unwrap_or("").trim();
        let jval = if let Ok(n) = val.parse::<i64>() {
            serde_json::Value::Number(n.into())
        } else if let Ok(f) = val.parse::<f64>() {
            serde_json::json!(f)
        } else if val == "true" {
            serde_json::Value::Bool(true)
        } else if val == "false" {
            serde_json::Value::Bool(false)
        } else {
            serde_json::Value::String(val.to_string())
        };
        map.insert(key, jval);
    }
    map
}

fn extract_system(modelfile: &str) -> Option<String> {
    let mut in_system = false;
    let mut lines: Vec<&str> = Vec::new();
    for line in modelfile.lines() {
        if line.to_uppercase().starts_with("SYSTEM") {
            in_system = true;
            let rest = line["SYSTEM".len()..].trim().trim_matches('"');
            if !rest.is_empty() { lines.push(rest); }
            continue;
        }
        if in_system {
            let first = line.split_whitespace().next().unwrap_or("");
            if !first.is_empty() && first == first.to_uppercase() && first.len() > 2 { break; }
            lines.push(line);
        }
    }
    let joined = lines.join("\n").trim().to_string();
    if joined.is_empty() { None } else { Some(joined) }
}

// ---------------------------------------------------------------------------
// cmd: init — build index.json from Ollama API
// ---------------------------------------------------------------------------

fn cmd_init(index_path: &str, host: &str) {
    println!("Fetching model list from {}...", host);
    let tags_json = match curl_get(&format!("{}/api/tags", host)) {
        Ok(s) => s,
        Err(e) => { eprintln!("Error: {}", e); return; }
    };
    let tags: serde_json::Value = match serde_json::from_str(&tags_json) {
        Ok(v) => v,
        Err(e) => { eprintln!("Parse error: {}", e); return; }
    };
    let raw_models = match tags["models"].as_array() {
        Some(a) => a.clone(),
        None => { eprintln!("No models in response"); return; }
    };

    let total = raw_models.len();
    let mut models: Vec<Model> = Vec::with_capacity(total);
    let mut total_size: u64 = 0;

    for (i, rm) in raw_models.iter().enumerate() {
        let name = rm["name"].as_str().unwrap_or("").to_string();
        print!("[{}/{}] {}                    \r", i + 1, total, name);
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let size_bytes  = rm["size"].as_u64().unwrap_or(0);
        total_size     += size_bytes;
        let modified_at = rm["modified_at"].as_str().unwrap_or("").to_string();
        let digest      = rm["digest"].as_str().unwrap_or("").to_string();
        let id = digest.strip_prefix("sha256:").unwrap_or(&digest)
            .chars().take(12).collect::<String>();

        let details      = &rm["details"];
        let architecture = details["family"].as_str().unwrap_or("").to_string();
        let parameters   = details["parameter_size"].as_str().unwrap_or("").to_string();
        let quantization = details["quantization_level"].as_str().unwrap_or("").to_string();

        let show_body = format!(r#"{{"name":"{}"}}"#, name);
        let show: serde_json::Value = curl_post(&format!("{}/api/show", host), &show_body)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let mi = &show["model_info"];
        let context_length   = mi[&format!("{}.context_length", architecture)].as_u64().unwrap_or(0);
        let embedding_length = mi[&format!("{}.embedding_length", architecture)].as_u64().unwrap_or(0);
        let requires_ollama  = show["requires"].as_str()
            .filter(|s| !s.is_empty()).map(|s| s.to_string());

        let capabilities: Vec<String> = show["capabilities"]
            .as_array().unwrap_or(&vec![])
            .iter().filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let inference_params = parse_params(show["parameters"].as_str().unwrap_or(""));
        let num_ctx = inference_params.get("num_ctx").and_then(|v| v.as_u64());
        let system_prompt = extract_system(show["modelfile"].as_str().unwrap_or(""));

        let role = if capabilities.contains(&"embedding".to_string()) {
            "embedding"
        } else { "general" }.to_string();

        models.push(Model {
            name, id, size_bytes, modified_at,
            architecture, parameters, context_length, num_ctx,
            embedding_length, quantization, requires_ollama,
            capabilities, role,
            domain: "general".to_string(),
            language: "multilingual".to_string(),
            tags: vec![],
            system_prompt, inference_params,
            update_status: "unknown".to_string(),
            remote_digest: None,
            parent: None,
        });
    }
    println!("\nFetched {} models.", total);

    let total_gb = (total_size as f64 / 1e9 * 100.0).round() / 100.0;
    let index = Index {
        generated_at: now_rfc3339(),
        ollama_host: host.to_string(),
        total_models: total,
        total_size_bytes: total_size,
        total_size_gb: total_gb,
        freshness_checked_at: String::new(),
        models,
    };

    let json = serde_json::to_string_pretty(&index).expect("serialize");
    std::fs::write(index_path, json).expect("write");
    println!("Written to {}", index_path);
    println!("Run `ollamerctl update` to check freshness.");
}

// ---------------------------------------------------------------------------
// cmd: update — check freshness against Ollama registry
// ---------------------------------------------------------------------------

fn sha256_of(data: &[u8]) -> String {
    use std::io::Write;
    let mut child = Command::new("sha256sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn().expect("sha256sum not found");
    if let Some(ref mut stdin) = child.stdin { let _ = stdin.write_all(data); }
    let out = child.wait_with_output().expect("sha256sum");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace().next().unwrap_or("").to_string()
}

fn manifest_path(name: &str) -> Option<std::path::PathBuf> {
    let base = std::path::Path::new("/kvm/ollama/models/manifests");
    let (host, rest) = if name.starts_with("hf.co/") {
        ("hf.co", &name["hf.co/".len()..])
    } else {
        ("registry.ollama.ai", name)
    };
    let (ns, model_tag) = if let Some(p) = rest.find('/') {
        (&rest[..p], &rest[p+1..])
    } else {
        ("library", rest)
    };
    let (model, tag) = if let Some(p) = model_tag.rfind(':') {
        (&model_tag[..p], &model_tag[p+1..])
    } else {
        (model_tag, "latest")
    };
    let path = base.join(host).join(ns).join(model).join(tag);
    if path.exists() { Some(path) } else { None }
}

fn registry_url(name: &str) -> Option<String> {
    if name.starts_with("hf.co/") { return None; }
    let (ns, rest) = if let Some(p) = name.find('/') {
        (&name[..p], &name[p+1..])
    } else {
        ("library", name)
    };
    let (model, tag) = if let Some(p) = rest.rfind(':') {
        (&rest[..p], &rest[p+1..])
    } else {
        (rest, "latest")
    };
    Some(format!("https://registry.ollama.ai/v2/{}/{}/manifests/{}", ns, model, tag))
}

fn cmd_update(index_path: &str) {
    let data = match std::fs::read_to_string(index_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Cannot read {}: {}", index_path, e); return; }
    };
    let mut index: Index = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(e) => { eprintln!("Parse error: {}", e); return; }
    };

    let total = index.models.len();
    println!("Checking {} models...", total);

    let accept = "Accept: application/vnd.docker.distribution.manifest.v2+json";

    for (i, model) in index.models.iter_mut().enumerate() {
        print!("[{}/{}] {}                    \r", i + 1, total, model.name);
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let Some(mpath) = manifest_path(&model.name) else {
            model.update_status = "local_only".to_string();
            continue;
        };

        let local_bytes = match std::fs::read(&mpath) {
            Ok(b) => b,
            Err(_) => { model.update_status = "unknown".to_string(); continue; }
        };
        let local_content = local_bytes.strip_suffix(b"\n").unwrap_or(&local_bytes);
        let local_sha = sha256_of(local_content);

        let Some(url) = registry_url(&model.name) else {
            model.update_status = "local_only".to_string();
            continue;
        };

        match curl_get_with_header(&url, accept) {
            Err(_) => { model.update_status = "unknown".to_string(); }
            Ok(body) => {
                if body.contains("\"errors\"") || body.is_empty() {
                    model.update_status = "not_in_registry".to_string();
                    model.remote_digest = None;
                } else {
                    let remote = body.as_bytes();
                    let remote_content = remote.strip_suffix(b"\n").unwrap_or(remote);
                    let remote_sha = sha256_of(remote_content);
                    model.remote_digest = Some(format!("sha256:{}", remote_sha));
                    model.update_status = if local_sha == remote_sha {
                        "up_to_date".to_string()
                    } else {
                        "update_available".to_string()
                    };
                }
            }
        }
    }

    println!("\nDone.");
    index.freshness_checked_at = now_rfc3339();

    let json = serde_json::to_string_pretty(&index).expect("serialize");
    std::fs::write(index_path, json).expect("write");
    println!("Saved to {}", index_path);

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for m in &index.models { *counts.entry(m.update_status.as_str()).or_insert(0) += 1; }
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by_key(|(k, _)| *k);
    for (status, count) in sorted {
        println!("  {:<22} {}", status, count);
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "\
ollamerctl — Ollamer index management CLI

Usage:
  ollamerctl init   [index.json] [--host http://localhost:11434]
  ollamerctl update [index.json]
  ollamerctl --help | -h
  ollamerctl --version | -V

Commands:
  init     Generate index.json from Ollama API (replaces Python script)
  update   Re-check freshness of all models against registry

Default index path: ~/.ollama/index.json";

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let default_index_owned = format!("{home}/.ollama/index.json");
    let default_index = default_index_owned.as_str();

    match args.get(1).map(|s| s.as_str()) {
        Some("-h") | Some("--help") | Some("help") => {
            println!("{}", usage);
        }
        Some("-V") | Some("--version") | Some("version") => {
            println!("ollamerctl {}", VERSION);
        }
        Some("init") => {
            let index_path = args.get(2).map(|s| s.as_str()).unwrap_or(default_index);
            let host = args.windows(2)
                .find(|w| w[0] == "--host")
                .map(|w| w[1].as_str())
                .unwrap_or("http://localhost:11434");
            cmd_init(index_path, host);
        }
        Some("update") => {
            let index_path = args.get(2).map(|s| s.as_str()).unwrap_or(default_index);
            cmd_update(index_path);
        }
        _ => { eprintln!("{}", usage); std::process::exit(1); }
    }
}
