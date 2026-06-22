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

// Merges index metadata with filesystem scan.
// Entries not in the index appear with empty metadata.
// Folders appear before files, both groups sorted alphabetically.
pub fn load() -> Vec<(String, EntryMeta)> {
    let mut indexed = read_index();

    let lib = lib_path();
    for (path, is_dir) in scan_entries(&lib, &lib) {
        let meta = indexed.entry(path).or_insert_with(EntryMeta::default);
        meta.is_dir = is_dir;
    }

    let mut entries: Vec<(String, EntryMeta)> = indexed.into_iter().collect();
    entries.sort_by(|a, b| {
        match (a.1.is_dir, b.1.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        }
    });
    entries
}

fn read_index() -> HashMap<String, EntryMeta> {
    std::fs::read_to_string(index_path())
        .ok()
        .and_then(|c| toml::from_str::<LibraryIndex>(&c).ok())
        .unwrap_or_default()
        .files
}

// Recursively collect files and top-level directories relative to root.
fn scan_entries(dir: &str, root: &str) -> Vec<(String, bool)> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return result;
    };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        let relative = path
            .to_string_lossy()
            .trim_start_matches(&format!("{}/", root))
            .to_string();

        if path.is_dir() {
            result.push((relative.clone(), true));
            result.extend(scan_entries(&path.to_string_lossy(), root));
        } else {
            result.push((relative, false));
        }
    }
    result
}

pub fn lib_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/cache/library", home.to_str().unwrap())
}

fn index_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/library.index.toml", home.to_str().unwrap())
}
