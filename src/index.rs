use serde::Deserialize;
use std::collections::HashMap;

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

// Immediate children of rel_path (relative to lib root).
// Dirs come before files. Adds ".." at the top if not at root.
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
        if name.starts_with('.') {
            continue;
        }

        let is_dir = item.path().is_dir();

        // Index key is always the full relative path from lib root
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

    // Add ".." at top when inside a subdirectory
    if !rel_path.is_empty() {
        let mut parent = EntryMeta::default();
        parent.is_dir = true;
        entries.insert(0, ("..".to_string(), parent));
    }

    entries
}

// Used only to build the full tag list for the sidebar
pub fn load_all_tags() -> Vec<(String, usize)> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
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
    format!("{}/.basalto/cache/library", home.to_str().unwrap())
}

fn index_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/library.index.toml", home.to_str().unwrap())
}
