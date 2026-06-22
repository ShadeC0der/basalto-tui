use crate::index::{self, EntryMeta, PluginInfo};
use ratatui::layout::Rect;

pub struct App {
    pub entries: Vec<(String, EntryMeta)>,
    pub selected: usize,
    pub tab: usize,
    pub tags: Vec<(String, usize)>,
    pub plugins: Vec<PluginInfo>,
    pub preview_lines: Vec<String>,
    pub preview_git_info: String,
    pub list_area: Rect,
    pub sidebar_scroll: usize,
    pub current_path: String,              // relative to lib root, empty = root
    path_stack: Vec<(String, usize)>,      // (path, selected_idx) for back navigation
    lib_path: String,
}

impl App {
    pub fn new() -> Self {
        let lib_path = {
            let home = dirs::home_dir().unwrap();
            format!("{}/.basalto/cache/library", home.to_str().unwrap())
        };
        let tags = index::load_all_tags();
        let plugins = index::load_plugins();
        let entries = index::load_dir("");

        let mut app = App {
            entries,
            selected: 0,
            tab: 0,
            tags,
            plugins,
            preview_lines: Vec::new(),
            preview_git_info: String::new(),
            list_area: Rect::default(),
            sidebar_scroll: 0,
            current_path: String::new(),
            path_stack: Vec::new(),
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

    // Enter on a folder navigates into it; on ".." goes back; on a file opens it
    pub fn enter_selected(&mut self) {
        let Some((name, meta)) = self.entries.get(self.selected) else { return; };

        if name == ".." {
            self.navigate_up();
            return;
        }

        if meta.is_dir {
            let new_path = if self.current_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", self.current_path, name)
            };
            self.path_stack.push((self.current_path.clone(), self.selected));
            self.current_path = new_path;
            self.selected = 0;
            self.reload();
        }
        // File: open in editor (TODO)
    }

    pub fn navigate_up(&mut self) {
        if let Some((prev_path, prev_idx)) = self.path_stack.pop() {
            self.current_path = prev_path;
            self.selected = prev_idx;
            self.reload();
        }
    }

    pub fn handle_click(&mut self, x: u16, y: u16) {
        let a = self.list_area;
        if a == Rect::default() { return; }
        if x < a.x || x >= a.x + a.width || y < a.y || y >= a.y + a.height { return; }

        let height = a.height as usize;
        let scroll  = if self.selected >= height { self.selected - height + 1 } else { 0 };
        let clicked = scroll + (y - a.y) as usize;

        if clicked < self.entries.len() {
            self.selected = clicked;
            self.load_preview();
        }
    }

    fn reload(&mut self) {
        self.entries = index::load_dir(&self.current_path);
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.load_preview();
    }

    fn load_preview(&mut self) {
        let Some((name, meta)) = self.entries.get(self.selected) else {
            self.preview_lines = Vec::new();
            self.preview_git_info = String::new();
            return;
        };

        if name == ".." {
            self.preview_lines = vec![format!(
                "← volver a: /{}",
                self.path_stack.last().map(|(p, _)| p.as_str()).unwrap_or("biblioteca")
            )];
            self.preview_git_info = String::new();
            return;
        }

        let full = self.full_path(name);
        let path = std::path::Path::new(&full);

        if meta.is_dir || path.is_dir() {
            self.preview_git_info = String::new();
            self.preview_lines = std::fs::read_dir(&full)
                .ok()
                .map(|rd| {
                    let mut items: Vec<_> = rd.flatten().collect();
                    items.sort_by_key(|e| e.file_name());
                    items
                        .iter()
                        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                        .map(|e| {
                            let (icon, _) = crate::icons::icon_for(
                                &e.file_name().to_string_lossy(),
                                e.path().is_dir(),
                            );
                            format!("{}{}", icon, e.file_name().to_string_lossy())
                        })
                        .collect()
                })
                .unwrap_or_default();
        } else {
            self.preview_lines = std::fs::read_to_string(&full)
                .unwrap_or_default()
                .lines()
                .map(|l| l.to_string())
                .collect();

            let rel_path = if self.current_path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", self.current_path, name)
            };

            self.preview_git_info = std::process::Command::new("git")
                .args(["log", "-1", "--format=%h · %cr", "--", &rel_path])
                .current_dir(&self.lib_path)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string();
        }
    }

    pub fn sidebar_up(&mut self) {
        self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1);
    }

    pub fn sidebar_down(&mut self) {
        let max = self.sidebar_total_lines().saturating_sub(1);
        if self.sidebar_scroll < max {
            self.sidebar_scroll += 1;
        }
    }

    pub fn sidebar_total_lines(&self) -> usize {
        let plugin_lines = if self.plugins.is_empty() { 1 } else { self.plugins.len() };
        let tag_lines    = if self.tags.is_empty()    { 1 } else { self.tags.len() };
        // PLUGINS + entries + blank + TAGS + entries + blank + GIT + branch
        1 + plugin_lines + 1 + 1 + tag_lines + 1 + 1 + 1
    }

    pub fn full_path(&self, name: &str) -> String {
        if self.current_path.is_empty() {
            format!("{}/{}", self.lib_path, name)
        } else {
            format!("{}/{}/{}", self.lib_path, self.current_path, name)
        }
    }
}
