use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Config ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct LibrariesConfig {
    #[serde(default)]
    active: String,
}

#[derive(Deserialize, Default)]
struct Config {
    #[serde(default)]
    libraries: LibrariesConfig,
}

fn read_config() -> Config {
    let home = dirs::home_dir().unwrap();
    let path = format!("{}/.basalto/config.toml", home.to_str().unwrap());
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn active_library() -> String {
    let name = read_config().libraries.active;
    if name.is_empty() { "main".to_string() } else { name }
}

// ─── Plugin list ─────────────────────────────────────────────────────────────

pub struct PluginInfo {
    pub name: String,
    pub enabled: bool,
}

#[derive(Deserialize)]
struct PluginToml {
    #[serde(default = "default_true")]
    enabled: bool,
}

fn default_true() -> bool { true }

pub fn load_plugins() -> Vec<PluginInfo> {
    let home = dirs::home_dir().unwrap();
    let plugins_dir = format!("{}/.basalto/plugins", home.to_str().unwrap());

    let mut list: Vec<PluginInfo> = std::fs::read_dir(&plugins_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "toml" { return None; }
            let stem = path.file_stem()?.to_str()?.to_string();
            let text = std::fs::read_to_string(&path).ok()?;
            let conf: PluginToml = toml::from_str(&text).ok()?;
            Some(PluginInfo { name: stem, enabled: conf.enabled })
        })
        .collect();

    list.sort_by(|a, b| a.name.cmp(&b.name));

    let tui_configured = !read_config().libraries.active.is_empty()
        || std::fs::read_to_string(
            format!("{}/.basalto/config.toml", dirs::home_dir().unwrap().to_str().unwrap())
        ).ok().map(|c| c.contains("[tui]")).unwrap_or(false);

    list.push(PluginInfo {
        name: "basalto-tui".to_string(),
        enabled: tui_configured && which_basalto_tui(),
    });

    list
}

fn which_basalto_tui() -> bool {
    std::process::Command::new("basalto-tui")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ─── Library index ───────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct EntryMeta {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip)]
    pub is_dir: bool,
    #[serde(default)]
    pub usos: u32,
    #[serde(default)]
    pub agregado: String,
}

#[derive(Deserialize, Serialize, Default)]
struct LibraryIndex {
    #[serde(default)]
    files: HashMap<String, EntryMeta>,
}

pub fn load_dir(rel_path: &str) -> Vec<(String, EntryMeta)> {
    let lib = lib_path();
    let target = if rel_path.is_empty() {
        lib.clone()
    } else {
        format!("{}/{}", lib, rel_path)
    };

    let indexed = read_index();
    let mut entries = Vec::new();

    let Ok(dir_read) = std::fs::read_dir(&target) else {
        return entries;
    };

    let mut items: Vec<_> = dir_read.flatten().collect();
    items.sort_by_key(|e| e.file_name());

    for item in items {
        let name = item.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }

        let is_dir = item.path().is_dir();
        let index_key = if rel_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel_path, name)
        };

        let mut meta = indexed.get(&index_key).cloned().unwrap_or_default();
        meta.is_dir = is_dir;
        entries.push((name, meta));
    }

    entries.sort_by(|a, b| match (a.1.is_dir, b.1.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });

    if !rel_path.is_empty() {
        entries.insert(0, ("..".to_string(), EntryMeta { is_dir: true, ..Default::default() }));
    }

    entries
}

pub fn load_all_tags() -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for meta in read_index().values() {
        for tag in &meta.tags {
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut tags: Vec<(String, usize)> = counts.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    tags
}

fn read_index() -> HashMap<String, EntryMeta> {
    std::fs::read_to_string(index_path())
        .ok()
        .and_then(|c| toml::from_str::<LibraryIndex>(&c).ok())
        .unwrap_or_default()
        .files
}

pub fn lib_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/cache/libraries/{}", home.to_str().unwrap(), active_library())
}

fn index_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/{}.index.toml", home.to_str().unwrap(), active_library())
}

// ─── Write operations ────────────────────────────────────────────────────────

pub fn add_entry(key: &str, desc: &str, tags: Vec<String>) {
    let mut files = read_index();
    files.insert(key.to_string(), EntryMeta {
        description: desc.to_string(),
        tags,
        is_dir: false,
        usos: 0,
        agregado: today(),
    });
    write_index(files);
}

pub fn increment_usos(key: &str) {
    let mut files = read_index();
    if let Some(meta) = files.get_mut(key) {
        meta.usos += 1;
        write_index(files);
    }
}

pub fn remove_entry(key: &str) {
    let mut files = read_index();
    files.remove(key);
    write_index(files);
}

pub fn load_all_entries() -> Vec<(String, EntryMeta)> {
    let mut entries: Vec<(String, EntryMeta)> = read_index().into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

pub fn run_push() -> Vec<String> {
    let dir = lib_path();
    match std::process::Command::new("git").args(["push"]).current_dir(&dir).output() {
        Ok(o) => {
            let mut lines: Vec<String> = String::from_utf8_lossy(&o.stdout)
                .lines().map(|l| l.to_string()).collect();
            for l in String::from_utf8_lossy(&o.stderr).lines() {
                lines.push(l.to_string());
            }
            if lines.iter().all(|l| l.trim().is_empty()) {
                lines.push(if o.status.success() {
                    "✓ push exitoso".to_string()
                } else {
                    "✗ push falló".to_string()
                });
            }
            lines.retain(|l| !l.trim().is_empty());
            lines
        }
        Err(e) => vec![format!("error: {}", e)],
    }
}

fn write_index(files: HashMap<String, EntryMeta>) {
    if let Ok(text) = toml::to_string_pretty(&LibraryIndex { files }) {
        std::fs::write(index_path(), text).ok();
    }
}

fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut days = (secs / 86400) as u32;
    let mut year = 1970u32;
    loop {
        let diy = if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) { 366 } else { 365 };
        if days < diy { break; }
        days -= diy;
        year += 1;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let months = [31u32, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let (mut month, mut day) = (1u32, days);
    for m in &months {
        if day < *m { break; }
        day -= m;
        month += 1;
    }
    format!("{:04}-{:02}-{:02}", year, month, day + 1)
}
