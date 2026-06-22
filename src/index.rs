use serde::Deserialize;
use std::collections::HashMap;

// ─── Config ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone)]
pub struct LibraryEntry {
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Deserialize, Default)]
struct LibrariesConfig {
    #[serde(default)]
    active: String,
    #[serde(default)]
    list: Vec<LibraryEntry>,
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

pub fn load_libraries() -> Vec<LibraryEntry> {
    read_config().libraries.list
}

// Write the new active library name to config.toml, preserving other fields.
pub fn write_active_library(name: &str) {
    let home = dirs::home_dir().unwrap();
    let path = format!("{}/.basalto/config.toml", home.to_str().unwrap());
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    // Replace the active = "..." line inside [libraries]; add section if missing.
    let new_line = format!("active = \"{}\"", name);
    if content.contains("active = ") {
        let updated = content
            .lines()
            .map(|l| if l.trim_start().starts_with("active = ") { new_line.as_str() } else { l })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, updated + "\n").ok();
    } else if content.contains("[libraries]") {
        let updated = content.replace("[libraries]", &format!("[libraries]\n{}", new_line));
        std::fs::write(&path, updated).ok();
    }
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

    let tui_configured = read_config().libraries.active.len() > 0
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

#[derive(Deserialize, Default, Clone)]
pub struct EntryMeta {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip)]
    pub is_dir: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub usos: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub agregado: String,
}

#[derive(Deserialize, Default)]
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
        let mut parent = EntryMeta::default();
        parent.is_dir = true;
        entries.insert(0, ("..".to_string(), parent));
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
