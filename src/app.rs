use crate::index::{self, EntryMeta};
use std::collections::HashMap;

pub struct App {
    pub entries: Vec<(String, EntryMeta)>,
    pub selected: usize,
    pub tab: usize,
    pub tags: Vec<(String, usize)>, // (tag, count) sorted by count desc
}

impl App {
    pub fn new() -> Self {
        let entries = index::load();
        let tags = collect_tags(&entries);
        App { entries, selected: 0, tab: 0, tags }
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
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
