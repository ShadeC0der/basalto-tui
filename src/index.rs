use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default, Clone)]
pub struct EntryMeta {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
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

// Loads index entries merged with filesystem files.
// Files not in the index appear with empty metadata.
pub fn load() -> Vec<(String, EntryMeta)> {
    let mut indexed = read_index();

    let lib = lib_path();
    let fs_files = scan_files(&lib, &lib);

    for path in fs_files {
        indexed.entry(path).or_insert_with(EntryMeta::default);
    }

    let mut entries: Vec<(String, EntryMeta)> = indexed.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn read_index() -> HashMap<String, EntryMeta> {
    std::fs::read_to_string(index_path())
        .ok()
        .and_then(|c| toml::from_str::<LibraryIndex>(&c).ok())
        .unwrap_or_default()
        .files
}

// Recursively collect all file paths relative to `root`.
fn scan_files(dir: &str, root: &str) -> Vec<String> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        // Skip hidden files and git internals
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            files.extend(scan_files(&path.to_string_lossy(), root));
        } else {
            let relative = path
                .to_string_lossy()
                .trim_start_matches(&format!("{}/", root))
                .to_string();
            files.push(relative);
        }
    }
    files
}

pub fn lib_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/cache/library", home.to_str().unwrap())
}

fn index_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/library.index.toml", home.to_str().unwrap())
}
