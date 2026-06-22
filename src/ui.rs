use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

// ─── Palette ────────────────────────────────────────────────────────────────
const CYAN: Color    = Color::Cyan;
const MAGENTA: Color = Color::Magenta;
const YELLOW: Color  = Color::Yellow;
const GREEN: Color   = Color::Green;
const WHITE: Color   = Color::White;
const GRAY: Color    = Color::Gray;
const DIM: Color     = Color::DarkGray;
const BLACK: Color   = Color::Black;

fn accent() -> Style { Style::default().fg(CYAN) }
fn dim()    -> Style { Style::default().fg(DIM) }
fn bold_cyan() -> Style { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }

// ─── Entry point ────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    render_title(frame, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(root[1]);

    render_sidebar(frame, app, body[0]);
    render_main(frame, app, body[1]);
    render_status(frame, app, root[2]);
}

// ─── Title bar ──────────────────────────────────────────────────────────────

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" basalto", bold_cyan()),
        Span::styled(" ◆ ", Style::default().fg(MAGENTA).add_modifier(Modifier::DIM)),
        Span::styled("~/biblioteca", Style::default().fg(WHITE)),
    ]));
    frame.render_widget(title, area);
}

// ─── Sidebar ────────────────────────────────────────────────────────────────

fn render_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let border = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::DIM));
    frame.render_widget(border, area);

    let mut lines = vec![
        Line::from(Span::styled("PLUGINS", dim().add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(CYAN).add_modifier(Modifier::DIM)),
            Span::styled("biblioteca", bold_cyan()),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled("TAGS", dim().add_modifier(Modifier::BOLD))),
    ];

    if app.tags.is_empty() {
        lines.push(Line::from(Span::styled("  sin tags", dim())));
    } else {
        for (tag, count) in &app.tags {
            lines.push(Line::from(vec![
                Span::styled("  #", dim()),
                Span::styled(tag.clone(), accent()),
                Span::styled(format!(" {}", count), Style::default().fg(YELLOW).add_modifier(Modifier::DIM)),
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled("GIT", dim().add_modifier(Modifier::BOLD))));
    lines.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(GREEN)),
        Span::styled("main", Style::default().fg(WHITE)),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}

// ─── Main area ──────────────────────────────────────────────────────────────

fn render_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let tabs = Tabs::new(vec![" biblioteca ", " nueva entrada ", " plugins ", " git/github "])
        .select(app.tab)
        .style(dim())
        .highlight_style(bold_cyan())
        .divider(Span::styled("│", dim()));
    frame.render_widget(tabs, main_area[0]);

    frame.render_widget(
        Block::default().borders(Borders::TOP).border_style(dim()),
        main_area[1],
    );

    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(12)])
        .split(main_area[2]);

    render_file_list(frame, app, content[0]);
    render_preview(frame, app, content[1]);
}

// ─── File list ──────────────────────────────────────────────────────────────

fn render_file_list(frame: &mut Frame, app: &mut App, area: Rect) {
    app.list_area = area;

    if app.entries.is_empty() {
        let msg = Paragraph::new(Span::styled(
            "  biblioteca vacía — usa basalto add para agregar archivos",
            dim(),
        ));
        frame.render_widget(msg, area);
        return;
    }

    let height = area.height as usize;
    let scroll = if app.selected >= height { app.selected - height + 1 } else { 0 };

    let width    = area.width as usize;
    let num_w    = 3usize;
    let name_w   = 24usize;
    let tags_w   = 18usize;
    let desc_w   = width.saturating_sub(2 + num_w + 2 + name_w + 2 + tags_w + 1);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (name, meta)) in app.entries.iter().enumerate().skip(scroll).take(height) {
        let selected = i == app.selected;
        let is_dir   = meta.is_dir;

        let indicator = if selected {
            Span::styled("▶ ", bold_cyan())
        } else {
            Span::styled("  ", dim())
        };

        let num = Span::styled(
            format!("{:>3} ", i + 1),
            if selected { accent() } else { dim() },
        );

        // Icon: ▸ for folders (yellow), · for files (dim)
        let icon = if is_dir {
            Span::styled("▸ ", Style::default().fg(YELLOW))
        } else {
            Span::styled("· ", dim())
        };

        let name_str = truncate(name, name_w.saturating_sub(2)); // -2 for icon
        let name_span = Span::styled(
            format!("{:<w$}  ", name_str, w = name_w.saturating_sub(2)),
            if is_dir {
                if selected {
                    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(YELLOW)
                }
            } else if selected {
                bold_cyan()
            } else {
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
            },
        );

        let desc_str = truncate(&meta.description, desc_w);
        let desc_span = Span::styled(
            format!("{:<w$}  ", desc_str, w = desc_w),
            if selected { Style::default().fg(GRAY) } else { dim() },
        );

        let tags_raw = meta.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
        let tags_str = truncate(&tags_raw, tags_w);
        let tags_span = Span::styled(
            tags_str,
            if selected {
                Style::default().fg(YELLOW)
            } else {
                Style::default().fg(CYAN).add_modifier(Modifier::DIM)
            },
        );

        lines.push(Line::from(vec![indicator, num, icon, name_span, desc_span, tags_span]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

// ─── Preview ────────────────────────────────────────────────────────────────

fn render_preview(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::DIM));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.entries.is_empty() {
        return;
    }

    let filename = &app.entries[app.selected].0;
    let visible = inner.height.saturating_sub(1) as usize;
    let width   = inner.width.saturating_sub(1) as usize;

    let mut lines = vec![Line::from(vec![
        Span::styled(" VISTA PREVIA ", Style::default().fg(MAGENTA).add_modifier(Modifier::DIM | Modifier::BOLD)),
        Span::styled("─ ", dim()),
        Span::styled(filename.as_str(), bold_cyan()),
        Span::styled(
            if app.preview_git_info.is_empty() { String::new() } else { format!("  {}", app.preview_git_info) },
            Style::default().fg(YELLOW).add_modifier(Modifier::DIM),
        ),
    ])];

    if app.preview_lines.is_empty() {
        lines.push(Line::from(Span::styled("  (archivo vacío)", dim())));
    } else {
        for line in app.preview_lines.iter().take(visible) {
            lines.push(Line::from(Span::styled(
                format!(" {}", truncate(line, width)),
                Style::default().fg(WHITE),
            )));
        }
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

// ─── Status bar ─────────────────────────────────────────────────────────────

fn render_status(frame: &mut Frame, app: &mut App, area: Rect) {
    let zones = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let total   = app.entries.len();
    let current = if total == 0 { 0 } else { app.selected + 1 };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" NORMAL ", Style::default().bg(CYAN).fg(BLACK).add_modifier(Modifier::BOLD)),
        Span::styled("  ", dim()),
        Span::styled(format!("biblioteca · {}/{}", current, total), Style::default().fg(WHITE)),
        Span::styled("   git:", dim()),
        Span::styled("main", Style::default().fg(GREEN)),
        Span::styled("   basalto-tui v", dim()),
        Span::styled(env!("CARGO_PKG_VERSION"), dim()),
    ]));
    frame.render_widget(status, zones[0]);

    let keys = Paragraph::new(Line::from(vec![
        Span::styled(" ↑↓/jk", accent()),
        Span::styled(" nav    ", dim()),
        Span::styled("enter", accent()),
        Span::styled(" abrir    ", dim()),
        Span::styled("n", accent()),
        Span::styled(" nuevo    ", dim()),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::styled(" salir", dim()),
    ])).alignment(Alignment::Left);
    frame.render_widget(keys, zones[1]);
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if max == 0 { return String::new(); }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}
