use crate::index::{self, EntryMeta, PluginInfo};

fn common_prefix<'a>(mut iter: impl Iterator<Item = &'a str>) -> String {
    let Some(first) = iter.next() else { return String::new(); };
    let mut prefix = first.to_string();
    for s in iter {
        while !s.starts_with(prefix.as_str()) {
            prefix.pop();
            if prefix.is_empty() { return prefix; }
        }
    }
    prefix
}

fn copy_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<usize> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        let mut count = 0;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            count += copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
        Ok(count)
    } else {
        if let Some(parent) = dst.parent() { std::fs::create_dir_all(parent)?; }
        std::fs::copy(src, dst)?;
        Ok(1)
    }
}
use ratatui::layout::Rect;

// ─── Modales ─────────────────────────────────────────────────────────────────

pub enum Modal {
    Add    { fields: [String; 3], focused: usize },   // [nombre, desc, tags]
    Show   { lines: Vec<(String, String)>, scroll: u16 },
    Remove { key: String },
    List   { entries: Vec<(String, index::EntryMeta)>, cursor: usize },
    Output { title: String, lines: Vec<String>, scroll: u16 },
    Use    { key: String, dest: String, is_dir: bool },
}

pub struct EditItem {
    pub original: Option<String>,  // None = item nuevo
    pub current: String,
    pub is_dir: bool,
}

pub struct EditBuffer {
    pub items: Vec<EditItem>,
    pub cursor: usize,
    pub col: usize,
}

pub enum Mode {
    Normal,
    Insert(EditBuffer),
    Command(String),
}

pub enum SidebarRow {
    Header(usize),   // section index: 0=plugins, 1=tags, 2=git
    Plugin,
    Tag,
    GitBranch,
}

pub struct App {
    // ── Biblioteca tab ───────────────────────────────────────────────────────
    pub entries: Vec<(String, EntryMeta)>,
    pub selected: usize,
    pub preview_lines: Vec<String>,
    pub preview_git_info: String,
    pub list_area: Rect,
    pub current_path: String,
    path_stack: Vec<(String, usize)>,
    lib_path: String,

    pub active_library: String,

    // ── Mode ─────────────────────────────────────────────────────────────────
    pub mode: Mode,

    // ── Pending deletions (dd en modo normal) ────────────────────────────────
    pub d_pressed: bool,
    pub pending_deletions: Vec<String>,

    pub launch_dir: String,
    pub should_quit: bool,
    pub show_help: bool,
    pub help_scroll: u16,
    pub modal: Option<Modal>,
    pub pending_open: Option<String>,

    // ── Global ───────────────────────────────────────────────────────────────
    pub tab: usize,
    pub tags: Vec<(String, usize)>,
    pub plugins: Vec<PluginInfo>,

    // ── Sidebar ──────────────────────────────────────────────────────────────
    pub sidebar_focused: bool,
    pub sidebar_cursor: usize,
    pub sidebar_collapsed: [bool; 3],
    pub sidebar_scroll: usize,
    pub sidebar_height: u16,

    // ── Tab 1: Git ───────────────────────────────────────────────────────────
    pub git_lines: Vec<String>,
    pub git_scroll: u16,

    // ── Tab 2: Plugins ───────────────────────────────────────────────────────
    pub plugins_cursor: usize,
}

impl App {
    pub fn new() -> Self {
        let active_library = index::active_library();
        let lib_path = index::lib_path();
        let tags     = index::load_all_tags();
        let plugins  = index::load_plugins();
        let entries  = index::load_dir("");

        let mut app = App {
            entries,
            selected: 0,
            preview_lines: Vec::new(),
            preview_git_info: String::new(),
            list_area: Rect::default(),
            current_path: String::new(),
            path_stack: Vec::new(),
            lib_path,
            active_library,
            tab: 0,
            tags,
            plugins,
            mode: Mode::Normal,
            d_pressed: false,
            pending_deletions: Vec::new(),
            launch_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            should_quit: false,
            show_help: false,
            help_scroll: 0,
            modal: None,
            pending_open: None,
            sidebar_focused: false,
            sidebar_cursor: 0,
            sidebar_collapsed: [false; 3],
            sidebar_scroll: 0,
            sidebar_height: 0,
            git_lines: Vec::new(),
            git_scroll: 0,
            plugins_cursor: 0,
        };
        app.load_preview();
        app
    }

    // Unified navigation — behavior depends on active tab
    pub fn nav_down(&mut self) {
        match self.tab {
            1 => { let max = self.git_lines.len().saturating_sub(1) as u16; if self.git_scroll < max { self.git_scroll += 1; } }
            2 => { if self.plugins_cursor < self.plugins.len().saturating_sub(1) { self.plugins_cursor += 1; } }
            _ => self.move_down(),
        }
    }

    pub fn nav_up(&mut self) {
        match self.tab {
            1 => { self.git_scroll = self.git_scroll.saturating_sub(1); }
            2 => { if self.plugins_cursor > 0 { self.plugins_cursor -= 1; } }
            _ => self.move_up(),
        }
    }

    pub fn nav_enter(&mut self) {
        self.enter_selected();
    }

    fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
            self.load_preview();
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.load_preview();
        }
    }

    // ── Biblioteca tab ───────────────────────────────────────────────────────

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
        } else {
            self.pending_open = Some(self.full_path(name));
        }
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
        self.pending_deletions.clear();
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

    // ── Mode management ──────────────────────────────────────────────────────

    // Entra en modo Insert oil-like: la lista de archivos se vuelve editable
    pub fn enter_insert(&mut self) {
        if self.tab != 0 { return; }

        let has_parent = self.entries.first().map(|(n, _)| n == "..").unwrap_or(false);

        let items: Vec<EditItem> = self.entries.iter()
            .filter(|(name, _)| name != "..")
            .map(|(name, meta)| EditItem {
                original: Some(name.clone()),
                current: name.clone(),
                is_dir: meta.is_dir,
            })
            .collect();

        let cursor = if has_parent {
            self.selected.saturating_sub(1)
        } else {
            self.selected
        }.min(items.len().saturating_sub(1));

        let col = items.get(cursor)
            .map(|it| it.current.chars().count())
            .unwrap_or(0);

        self.mode = Mode::Insert(EditBuffer { items, cursor, col });
    }

    pub fn enter_command(&mut self) {
        self.mode = Mode::Command(String::new());
    }

    pub fn mode_cancel(&mut self) {
        self.mode = Mode::Normal;
    }

    // ── Insert mode (oil-like) ────────────────────────────────────────────────

    pub fn insert_push(&mut self, c: char) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        let Some(item) = buf.items.get_mut(buf.cursor) else { return; };
        let mut chars: Vec<char> = item.current.chars().collect();
        let col = buf.col.min(chars.len());
        chars.insert(col, c);
        item.current = chars.iter().collect();
        buf.col = col + 1;
    }

    pub fn insert_pop(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        if let Some(item) = buf.items.get_mut(buf.cursor) {
            if buf.col > 0 {
                let mut chars: Vec<char> = item.current.chars().collect();
                chars.remove(buf.col - 1);
                item.current = chars.iter().collect();
                buf.col -= 1;
            } else if item.original.is_none() {
                // Eliminar item nuevo vacío
                let cur = buf.cursor;
                buf.items.remove(cur);
                if buf.cursor > 0 { buf.cursor -= 1; }
                buf.col = buf.items.get(buf.cursor)
                    .map(|it| it.current.chars().count())
                    .unwrap_or(0);
            }
        }
    }

    pub fn insert_nav_down(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        if buf.cursor + 1 < buf.items.len() {
            buf.cursor += 1;
            buf.col = buf.items[buf.cursor].current.chars().count();
        }
    }

    pub fn insert_nav_up(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        if buf.cursor > 0 {
            buf.cursor -= 1;
            buf.col = buf.items[buf.cursor].current.chars().count();
        }
    }

    // Enter añade una línea nueva debajo del cursor
    pub fn insert_newline(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        let new_idx = buf.cursor + 1;
        buf.items.insert(new_idx, EditItem {
            original: None,
            current: String::new(),
            is_dir: false,
        });
        buf.cursor = new_idx;
        buf.col = 0;
    }

    pub fn insert_col_left(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        if buf.col > 0 { buf.col -= 1; }
    }

    pub fn insert_col_right(&mut self) {
        let Mode::Insert(buf) = &mut self.mode else { return; };
        let max = buf.items.get(buf.cursor)
            .map(|it| it.current.chars().count())
            .unwrap_or(0);
        if buf.col < max { buf.col += 1; }
    }

    // Ctrl+S: aplica renombres y creaciones. Si se crearon archivos nuevos,
    // abre el modal de metadata para el primero automáticamente.
    pub fn confirm_insert(&mut self) -> bool {
        if !matches!(self.mode, Mode::Insert(_)) { return false; }
        let dir = self.current_dir_path();
        let Mode::Insert(buf) = &self.mode else { return false; };
        let items: Vec<(Option<String>, String, bool)> = buf.items.iter()
            .map(|it| (it.original.clone(), it.current.clone(), it.current.ends_with('/')))
            .collect();

        let mut first_new_key: Option<String> = None;

        for (original, current, is_dir_new) in &items {
            match original {
                Some(orig) if orig != current && !current.is_empty() => {
                    std::fs::rename(
                        format!("{}/{}", dir, orig),
                        format!("{}/{}", dir, current),
                    ).ok();
                }
                None if !current.is_empty() => {
                    let name = current.trim_end_matches('/');
                    let target = format!("{}/{}", dir, current);
                    if *is_dir_new {
                        std::fs::create_dir_all(&target).ok();
                    } else {
                        if let Some(p) = std::path::Path::new(&target).parent() {
                            std::fs::create_dir_all(p).ok();
                        }
                        std::fs::File::create(&target).ok();
                    }
                    if first_new_key.is_none() {
                        first_new_key = Some(if self.current_path.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", self.current_path, name)
                        });
                    }
                }
                _ => {}
            }
        }

        self.mode = Mode::Normal;
        self.reload();

        if let Some(key) = first_new_key {
            // Pre-llenar nombre, cursor en descripción para que el usuario
            // solo tenga que escribir la descripción y los tags.
            self.modal = Some(Modal::Add {
                fields: [key, String::new(), String::new()],
                focused: 1,
            });
        }

        true
    }

    // ── Command mode ──────────────────────────────────────────────────────────

    pub fn mode_push(&mut self, c: char) {
        if let Mode::Command(s) = &mut self.mode { s.push(c); }
    }

    pub fn mode_pop(&mut self) {
        if let Mode::Command(s) = &mut self.mode { s.pop(); }
    }

    pub fn take_command(&mut self) -> Option<String> {
        let Mode::Command(s) = &self.mode else { return None; };
        let cmd = s.trim().to_string();
        self.mode = Mode::Normal;
        if cmd.is_empty() { return None; }

        let mut parts = cmd.splitn(2, ' ');
        let verb = parts.next().unwrap_or("").to_lowercase();
        let arg  = parts.next().unwrap_or("").trim().to_string();

        match verb.as_str() {
            "help" | "?" => { self.show_help = true; None }
            "add"  => { self.open_add_modal(&arg); None }
            "show" => {
                let key = if arg.is_empty() { self.selected_key() } else { arg };
                if !key.is_empty() { self.open_show_modal(&key); }
                None
            }
            "remove" | "rm" => {
                let key = if arg.is_empty() { self.selected_key() } else { arg };
                if !key.is_empty() { self.modal = Some(Modal::Remove { key }); }
                None
            }
            "list" => { self.open_list_modal(); None }
            "push" => { self.open_push_modal(); None }
            "use"  => {
                let key = if arg.is_empty() { self.selected_key() } else { arg };
                if !key.is_empty() { self.open_use_modal(&key); }
                None
            }
            // Estos siguen usando terminal
            other => {
                let full = if arg.is_empty() {
                    format!("basalto {}", other)
                } else {
                    format!("basalto {} {}", other, arg)
                };
                Some(full)
            }
        }
    }

    pub fn selected_key(&self) -> String {
        if self.tab != 0 { return String::new(); }
        self.entries.get(self.selected)
            .filter(|(n, _)| n != "..")
            .map(|(n, _)| {
                if self.current_path.is_empty() { n.clone() }
                else { format!("{}/{}", self.current_path, n) }
            })
            .unwrap_or_default()
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if !self.show_help { self.help_scroll = 0; }
    }

    // ── Modal management ──────────────────────────────────────────────────────

    pub fn open_add_modal(&mut self, arg: &str) {
        let nombre = if !arg.is_empty() {
            arg.to_string()
        } else {
            self.selected_key()
        };
        self.modal = Some(Modal::Add {
            fields: [nombre, String::new(), String::new()],
            focused: 0,
        });
    }

    pub fn open_show_modal(&mut self, key: &str) {
        let all = index::load_all_entries();
        let meta = all.iter().find(|(k, _)| k == key)
            .map(|(_, m)| m.clone())
            .unwrap_or_default();
        let lines = vec![
            ("Archivo".to_string(),     key.to_string()),
            ("Descripción".to_string(), meta.description.clone()),
            ("Tags".to_string(),        meta.tags.join(", ")),
            ("Usos".to_string(),        meta.usos.to_string()),
            ("Agregado".to_string(),    meta.agregado.clone()),
        ];
        self.modal = Some(Modal::Show { lines, scroll: 0 });
    }

    pub fn open_list_modal(&mut self) {
        let entries = index::load_all_entries();
        self.modal = Some(Modal::List { entries, cursor: 0 });
    }

    pub fn open_push_modal(&mut self) {
        let lines = index::run_push();
        self.modal = Some(Modal::Output { title: "git push".to_string(), lines, scroll: 0 });
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn modal_add_push(&mut self, c: char) {
        if let Some(Modal::Add { fields, focused }) = &mut self.modal {
            fields[*focused].push(c);
        }
    }

    pub fn modal_add_pop(&mut self) {
        if let Some(Modal::Add { fields, focused }) = &mut self.modal {
            fields[*focused].pop();
        }
    }

    pub fn modal_add_tab(&mut self, forward: bool) {
        if let Some(Modal::Add { focused, .. }) = &mut self.modal {
            if forward && *focused < 2 { *focused += 1; }
            else if !forward && *focused > 0 { *focused -= 1; }
        }
    }

    pub fn modal_add_enter(&mut self) {
        if let Some(Modal::Add { focused, .. }) = &mut self.modal
            && *focused < 2 { *focused += 1; return; }
        self.modal_confirm_add();
    }

    pub fn modal_confirm_add(&mut self) {
        let Some(Modal::Add { fields, .. }) = &self.modal else { return; };
        let key  = fields[0].trim().to_string();
        let desc = fields[1].trim().to_string();
        let tags: Vec<String> = fields[2].split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        if key.is_empty() { return; }
        index::add_entry(&key, &desc, tags);
        self.modal = None;
        self.reload_all();
    }

    pub fn modal_confirm_remove(&mut self) {
        let Some(Modal::Remove { key }) = &self.modal else { return; };
        let key = key.clone();
        index::remove_entry(&key);
        self.modal = None;
        self.reload_all();
    }

    pub fn open_use_modal(&mut self, key: &str) {
        if key.is_empty() { return; }
        let src    = format!("{}/{}", self.lib_path, key);
        let is_dir = std::path::Path::new(&src).is_dir();
        let basename = std::path::Path::new(key)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(key)
            .to_string();
        let home = dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|| "~".to_string());
        let dest = format!("{}/{}", home, basename);
        self.modal = Some(Modal::Use { key: key.to_string(), dest, is_dir });
    }

    pub fn modal_use_push(&mut self, c: char) {
        if let Some(Modal::Use { dest, .. }) = &mut self.modal { dest.push(c); }
    }

    pub fn modal_use_pop(&mut self) {
        if let Some(Modal::Use { dest, .. }) = &mut self.modal { dest.pop(); }
    }

    pub fn modal_use_tab_complete(&mut self) {
        let Some(Modal::Use { dest, .. }) = &mut self.modal else { return; };

        // Expandir ~ para la búsqueda
        let expanded = if dest.starts_with("~/") {
            let home = dirs::home_dir()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("{}/{}", home, dest.strip_prefix("~/").unwrap_or(dest))
        } else {
            dest.clone()
        };

        // Separar directorio base y prefijo a completar
        let (base_dir, prefix) = if expanded.ends_with('/') {
            (expanded.clone(), String::new())
        } else {
            let p = std::path::Path::new(&expanded);
            let dir = p.parent()
                .map(|d| {
                    let s = d.to_string_lossy().to_string();
                    if s.is_empty() { ".".to_string() } else { s }
                })
                .unwrap_or_else(|| ".".to_string());
            let pfx = p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            (dir, pfx)
        };

        // Listar entradas del directorio que coincidan con el prefijo
        let Ok(rd) = std::fs::read_dir(&base_dir) else { return; };
        let mut matches: Vec<(String, bool)> = rd
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { return None; }
                if name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    let is_d = e.path().is_dir();
                    Some((name, is_d))
                } else {
                    None
                }
            })
            .collect();
        matches.sort_by(|a, b| a.0.cmp(&b.0));

        if matches.is_empty() { return; }

        let completed = if matches.len() == 1 {
            // Completado único: agrega el nombre y / si es directorio
            let (name, is_d) = &matches[0];
            let suffix = if *is_d { "/" } else { "" };
            format!("{}/{}{}", base_dir.trim_end_matches('/'), name, suffix)
        } else {
            // Múltiples coincidencias: completa hasta el prefijo común
            let common = common_prefix(matches.iter().map(|(n, _)| n.as_str()));
            format!("{}/{}", base_dir.trim_end_matches('/'), common)
        };

        // Si el dest original usaba ~, intentamos mantenerlo legible con ~
        let home = dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_default();
        *dest = if !home.is_empty() && completed.starts_with(&home) {
            format!("~{}", &completed[home.len()..])
        } else {
            completed
        };
    }

    pub fn modal_confirm_use(&mut self) {
        let Some(Modal::Use { key, dest, .. }) = &self.modal else { return; };
        let key  = key.clone();
        let dest = dest.trim().to_string();

        let src = format!("{}/{}", self.lib_path, key);
        let expanded = if dest.starts_with("~/") {
            let home = dirs::home_dir()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("{}/{}", home, dest.strip_prefix("~/").unwrap_or(dest.as_str()))
        } else {
            dest.clone()
        };
        let dst = if expanded.starts_with('/') {
            expanded
        } else {
            format!("{}/{}", self.launch_dir, expanded.trim_start_matches("./"))
        };

        match copy_recursive(std::path::Path::new(&src), std::path::Path::new(&dst)) {
            Ok(n) => {
                index::increment_usos(&key);
                let summary = if n == 1 {
                    "1 archivo copiado".to_string()
                } else {
                    format!("{} archivos copiados", n)
                };
                self.modal = Some(Modal::Output {
                    title: "✓ copiado".to_string(),
                    lines: vec![
                        format!("desde:  {}", key),
                        format!("hacia:  {}", dst),
                        summary,
                    ],
                    scroll: 0,
                });
            }
            Err(e) => {
                self.modal = Some(Modal::Output {
                    title: "✗ error al copiar".to_string(),
                    lines: vec![e.to_string()],
                    scroll: 0,
                });
            }
        }
    }

    pub fn modal_scroll_down(&mut self) {
        match &mut self.modal {
            Some(Modal::Show   { scroll, .. }) => *scroll = scroll.saturating_add(1),
            Some(Modal::Output { scroll, .. }) => *scroll = scroll.saturating_add(1),
            Some(Modal::List   { cursor, entries }) => {
                if *cursor + 1 < entries.len() { *cursor += 1; }
            }
            _ => {}
        }
    }

    pub fn modal_scroll_up(&mut self) {
        match &mut self.modal {
            Some(Modal::Show   { scroll, .. }) => *scroll = scroll.saturating_sub(1),
            Some(Modal::Output { scroll, .. }) => *scroll = scroll.saturating_sub(1),
            Some(Modal::List   { cursor, .. }) => { if *cursor > 0 { *cursor -= 1; } }
            _ => {}
        }
    }

    pub fn current_dir_path(&self) -> String {
        if self.current_path.is_empty() {
            self.lib_path.clone()
        } else {
            format!("{}/{}", self.lib_path, self.current_path)
        }
    }

    pub fn reload_all(&mut self) {
        self.entries = index::load_dir(&self.current_path);
        self.tags    = index::load_all_tags();
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.load_preview();
    }

    // ── Pending deletions ────────────────────────────────────────────────────

    pub fn toggle_delete(&mut self) {
        let Some((name, _)) = self.entries.get(self.selected) else { return; };
        if name == ".." { return; }
        let name = name.clone();
        if let Some(pos) = self.pending_deletions.iter().position(|s| s == &name) {
            self.pending_deletions.remove(pos);
        } else {
            self.pending_deletions.push(name);
        }
    }

    pub fn commit_deletions(&mut self) {
        if self.pending_deletions.is_empty() { return; }
        let dir = self.current_dir_path();
        for name in &self.pending_deletions {
            let path = std::path::Path::new(&dir).join(name);
            if path.is_dir() {
                std::fs::remove_dir_all(&path).ok();
            } else {
                std::fs::remove_file(&path).ok();
            }
        }
        self.pending_deletions.clear();
        self.reload();
    }

    pub fn cancel_deletions(&mut self) {
        self.pending_deletions.clear();
    }

    pub fn toggle_sidebar_focus(&mut self) {
        self.sidebar_focused = !self.sidebar_focused;
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % 3;
        if self.tab == 1 && self.git_lines.is_empty() { self.load_git_info(); }
    }

    pub fn prev_tab(&mut self) {
        self.tab = (self.tab + 2) % 3;
        if self.tab == 1 && self.git_lines.is_empty() { self.load_git_info(); }
    }

    // All navigable items in order, respecting collapse state (no blanks)
    pub fn sidebar_items(&self) -> Vec<SidebarRow> {
        let mut items = Vec::new();
        items.push(SidebarRow::Header(0));
        if !self.sidebar_collapsed[0] {
            for _ in 0..self.plugins.len() { items.push(SidebarRow::Plugin); }
        }
        items.push(SidebarRow::Header(1));
        if !self.sidebar_collapsed[1] {
            for _ in 0..self.tags.len() { items.push(SidebarRow::Tag); }
        }
        items.push(SidebarRow::Header(2));
        if !self.sidebar_collapsed[2] {
            items.push(SidebarRow::GitBranch);
        }
        items
    }

    // Maps each item index to its line number in the rendered paragraph.
    // Blank separator lines (between sections) are accounted for but not navigable.
    pub fn sidebar_para_lines(&self) -> Vec<usize> {
        let items = self.sidebar_items();
        let mut para = 0usize;
        let mut result = Vec::new();
        for item in &items {
            if let SidebarRow::Header(s) = item
                && *s > 0 { para += 1; } // blank separator before sections 1 and 2
            result.push(para);
            para += 1;
            // Placeholder line ("sin plugins" / "sin tags") when section is empty + expanded
            if let SidebarRow::Header(s) = item {
                let empty = match s { 0 => self.plugins.is_empty(), 1 => self.tags.is_empty(), _ => false };
                if !self.sidebar_collapsed[*s] && empty { para += 1; }
            }
        }
        result
    }

    pub fn sidebar_nav_up(&mut self) {
        if self.sidebar_cursor > 0 {
            self.sidebar_cursor -= 1;
            self.scroll_to_cursor();
        }
    }

    pub fn sidebar_nav_down(&mut self) {
        let max = self.sidebar_items().len().saturating_sub(1);
        if self.sidebar_cursor < max {
            self.sidebar_cursor += 1;
            self.scroll_to_cursor();
        }
    }

    pub fn sidebar_toggle_section(&mut self) {
        let items = self.sidebar_items();
        let section = match items.get(self.sidebar_cursor) {
            Some(SidebarRow::Header(s)) => *s,
            _ => return,
        };
        self.sidebar_collapsed[section] = !self.sidebar_collapsed[section];
        // Clamp cursor if items shrank after collapse
        let total = self.sidebar_items().len();
        if self.sidebar_cursor >= total { self.sidebar_cursor = total.saturating_sub(1); }
        self.scroll_to_cursor();
    }

    fn scroll_to_cursor(&mut self) {
        let para_lines = self.sidebar_para_lines();
        let Some(&line) = para_lines.get(self.sidebar_cursor) else { return; };
        let visible = self.sidebar_height as usize;
        if visible == 0 { return; }
        if line < self.sidebar_scroll {
            self.sidebar_scroll = line;
        } else if line >= self.sidebar_scroll + visible {
            self.sidebar_scroll = line.saturating_sub(visible - 1);
        }
    }

    pub fn full_path(&self, name: &str) -> String {
        if self.current_path.is_empty() {
            format!("{}/{}", self.lib_path, name)
        } else {
            format!("{}/{}/{}", self.lib_path, self.current_path, name)
        }
    }

    // ── Tab 1: Git ───────────────────────────────────────────────────────────

    pub fn load_git_info(&mut self) {
        let dir = self.lib_path.clone();
        let mut lines: Vec<String> = Vec::new();

        lines.push("── git status ──────────────────────────────────".to_string());
        match std::process::Command::new("git").args(["status", "--short"]).current_dir(&dir).output() {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout);
                let sl: Vec<&str> = out.lines().collect();
                if sl.is_empty() {
                    lines.push("  sin cambios pendientes".to_string());
                } else {
                    for l in sl { lines.push(format!("  {}", l)); }
                }
            }
            Err(e) => lines.push(format!("  error: {}", e)),
        }

        lines.push(String::new());
        lines.push("── git log ─────────────────────────────────────".to_string());
        match std::process::Command::new("git")
            .args(["log", "--oneline", "--graph", "--decorate", "-25"])
            .current_dir(&dir)
            .output()
        {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout);
                if out.trim().is_empty() {
                    lines.push("  sin commits todavía".to_string());
                } else {
                    for l in out.lines() { lines.push(format!("  {}", l)); }
                }
            }
            Err(e) => lines.push(format!("  error: {}", e)),
        }

        self.git_lines = lines;
        self.git_scroll = 0;
    }
}
