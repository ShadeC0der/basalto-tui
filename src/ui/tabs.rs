use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use super::{CYAN, YELLOW, GREEN, WHITE, DIM, dim, bold_cyan, accent, truncate};

// ─── Tab 1: Git ──────────────────────────────────────────────────────────────

pub fn render_git(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    // Header con rama actual
    let branch = get_branch(&app.git_lines);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ", dim()),
            Span::styled("● ", Style::default().fg(GREEN)),
            Span::styled(branch, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled("  —  biblioteca: ", dim()),
            Span::styled(app.active_library.clone(), Style::default().fg(CYAN)),
        ])),
        layout[0],
    );

    // Contenido scrollable
    if app.git_lines.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  Presiona r para cargar información de git",
                dim(),
            )),
            layout[1],
        );
    } else {
        let content: Vec<Line> = app.git_lines.iter().map(|l| {
            let style = if l.starts_with("──") {
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
            } else if l.contains("modified") || l.trim_start().starts_with('M') {
                Style::default().fg(YELLOW)
            } else if l.trim_start().starts_with('?') {
                Style::default().fg(Color::Red)
            } else if l.trim_start().starts_with('A') {
                Style::default().fg(GREEN)
            } else {
                Style::default().fg(WHITE)
            };
            Line::from(Span::styled(l.clone(), style))
        }).collect();

        frame.render_widget(
            Paragraph::new(content).scroll((app.git_scroll, 0)),
            layout[1],
        );
    }

    // Key hints
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  j/k/↑↓", accent()),
            Span::styled(" scroll    ", dim()),
            Span::styled("r", accent()),
            Span::styled(" refrescar", dim()),
        ])),
        layout[2],
    );
}

fn get_branch(git_lines: &[String]) -> String {
    // Intenta leer el branch desde HEAD directamente; no requiere parsear git_lines
    dirs::home_dir()
        .map(|_| {
            std::process::Command::new("git")
                .args(["branch", "--show-current"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            // Fallback: buscar en las líneas del log un indicador de HEAD
            git_lines.iter()
                .find(|l| l.contains("HEAD ->"))
                .and_then(|l| {
                    l.split("HEAD ->").nth(1)
                        .map(|s| s.split(&[',', ')'][..]).next().unwrap_or("").trim().to_string())
                })
                .unwrap_or_else(|| "main".to_string())
        })
}

// ─── Tab 4: Plugins ──────────────────────────────────────────────────────────

pub fn render_plugins(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Plugins del ecosistema Basalto", Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
        ])),
        layout[0],
    );

    if app.plugins.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  sin plugins instalados", dim())),
            layout[1],
        );
    } else {
        let width = layout[1].width as usize;
        let name_w = 24usize;
        let status_w = 10usize;
        let path_w = width.saturating_sub(4 + name_w + status_w + 4);

        let mut lines: Vec<Line> = Vec::new();
        for (i, plugin) in app.plugins.iter().enumerate() {
            let selected  = i == app.plugins_cursor;
            let indicator = if selected { "▶ " } else { "  " };
            let (dot, dot_color) = if plugin.enabled { ("● ", GREEN) } else { ("○ ", DIM) };

            let display   = plugin.name.strip_prefix("basalto-").unwrap_or(&plugin.name);
            let name_str  = truncate(display, name_w);
            let status_str = if plugin.enabled { "activo" } else { "inactivo" };

            let path_hint = plugin_path(&plugin.name);
            let path_str  = truncate(&path_hint, path_w);

            let base_style = if selected { bold_cyan() } else { Style::default().fg(WHITE) };

            lines.push(Line::from(vec![
                Span::styled(indicator, if selected { bold_cyan() } else { dim() }),
                Span::styled(dot, Style::default().fg(if selected { CYAN } else { dot_color })),
                Span::styled(format!("{:<w$}", name_str, w = name_w), base_style),
                Span::styled(
                    format!("  {:<w$}", status_str, w = status_w),
                    if plugin.enabled {
                        Style::default().fg(if selected { GREEN } else { GREEN }).add_modifier(Modifier::DIM)
                    } else {
                        dim()
                    },
                ),
                Span::styled(format!("  {}", path_str), dim()),
            ]));

            // Fila de detalle cuando está seleccionado
            if selected {
                lines.push(Line::from(vec![
                    Span::styled("    ", dim()),
                    Span::styled("nombre completo: ", dim()),
                    Span::styled(plugin.name.clone(), Style::default().fg(YELLOW)),
                ]));
            }
        }

        frame.render_widget(Paragraph::new(lines), layout[1]);
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  j/k/↑↓", accent()),
            Span::styled(" navegar", dim()),
        ])),
        layout[2],
    );
}

fn plugin_path(name: &str) -> String {
    // Busca el binario en $PATH
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            // Fallback a ruta esperada de cargo
            dirs::home_dir()
                .map(|h| format!("{}/.cargo/bin/{}", h.display(), name))
                .unwrap_or_default()
        })
}
