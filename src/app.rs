use crate::index::{self, EntryMeta};
use std::collections::HashMap;

pub struct App {
    pub entries: Vec<(String, EntryMeta)>,
    pub selected: usize,
    pub tab: usize,
    pub tags: Vec<(String, usize)>,
    pub preview_lines: Vec<String>,
    pub preview_git_info: String,
    lib_path: String,
}

impl App {
    pub fn new() -> Self {
        let entries = index::load();
        let tags = collect_tags(&entries);
        let lib_path = {
            let home = dirs::home_dir().unwrap();
            format!("{}/.basalto/cache/library", home.to_str().unwrap())
        };
        let mut app = App {
            entries,
            selected: 0,
            tab: 0,
            tags,
            preview_lines: Vec::new(),
            preview_git_info: String::new(),
            lib_path,
        };
        app.load_preview();
        app
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
            self.load_preview();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.load_preview();
        }
    }

    fn load_preview(&mut self) {
        let Some((name, _)) = self.entries.get(self.selected) else {
            self.preview_lines = Vec::new();
            self.preview_git_info = String::new();
            return;
        };

        let file_path = format!("{}/{}", self.lib_path, name);

        self.preview_lines = std::fs::read_to_string(&file_path)
            .unwrap_or_default()
            .lines()
            .map(|l| l.to_string())
            .collect();

        // git log -1 on the file — empty string if not a git repo yet
        self.preview_git_info = std::process::Command::new("git")
            .args(["log", "-1", "--format=%h · %cr", "--", name])
            .current_dir(&self.lib_path)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_string();
    }
}

fn collect_tags(entries: &[(String, EntryMeta)]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, meta) in entries {
        for tag in &meta.tags {
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut tags: Vec<(String, usize)> = counts.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    tags
}
