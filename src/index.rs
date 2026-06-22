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

pub fn load() -> Vec<(String, EntryMeta)> {
    let path = index_path();

    let index: LibraryIndex = std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default();

    let mut entries: Vec<(String, EntryMeta)> = index.files.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn index_path() -> String {
    let home = dirs::home_dir().unwrap();
    format!("{}/.basalto/library.index.toml", home.to_str().unwrap())
}
