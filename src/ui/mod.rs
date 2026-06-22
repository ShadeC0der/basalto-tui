use crate::app::{App, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

mod file_list;
mod modals;
mod sidebar;
mod status;
mod tabs;

// ─── Palette ─────────────────────────────────────────────────────────────────
pub(crate) const CYAN: Color    = Color::Cyan;
pub(crate) const MAGENTA: Color = Color::Magenta;
pub(crate) const YELLOW: Color  = Color::Yellow;
pub(crate) const GREEN: Color   = Color::Green;
pub(crate) const WHITE: Color   = Color::White;
pub(crate) const GRAY: Color    = Color::Gray;
pub(crate) const DIM: Color     = Color::DarkGray;
pub(crate) const BLACK: Color   = Color::Black;

pub(crate) fn accent()    -> Style { Style::default().fg(CYAN) }
pub(crate) fn dim()       -> Style { Style::default().fg(DIM) }
pub(crate) fn bold_cyan() -> Style { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if max == 0 { return String::new(); }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}

pub(crate) fn centered(area: Rect, max_w: u16, max_h: u16) -> Rect {
    let w = max_w.min(area.width.saturating_sub(4));
    let h = max_h.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w, height: h }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

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

    render_title(frame, app, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(root[1]);

    sidebar::render_sidebar(frame, app, body[0]);
    render_main(frame, app, body[1]);
    status::render_status(frame, app, root[2]);

    if let Mode::Command(input) = &app.mode {
        let input = input.clone();
        modals::render_command_popup(frame, &input, area);
    }

    if app.show_help {
        modals::render_help_popup(frame, app, area);
    }

    if app.modal.is_some() {
        modals::render_modal(frame, app, area);
    }
}

// ─── Title bar ───────────────────────────────────────────────────────────────

fn render_title(frame: &mut Frame, app: &App, area: Rect) {
    let path = if app.current_path.is_empty() {
        "~/biblioteca".to_string()
    } else {
        format!("~/biblioteca/{}", app.current_path)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" basalto", bold_cyan()),
            Span::styled(" ◆ ", Style::default().fg(MAGENTA).add_modifier(Modifier::DIM)),
            Span::styled(path, Style::default().fg(WHITE)),
        ])),
        area,
    );
}

// ─── Main area ───────────────────────────────────────────────────────────────

fn render_main(frame: &mut Frame, app: &mut App, area: Rect) {
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let tabs = Tabs::new(vec![
        " biblioteca ", " git/github ", " plugins ",
    ])
        .select(app.tab)
        .style(dim())
        .highlight_style(bold_cyan())
        .divider(Span::styled("│", dim()));
    frame.render_widget(tabs, main_area[0]);

    frame.render_widget(
        Block::default().borders(Borders::TOP).border_style(dim()),
        main_area[1],
    );

    match app.tab {
        0 => {
            let content = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(12)])
                .split(main_area[2]);
            file_list::render_file_list(frame, app, content[0]);
            render_preview(frame, app, content[1]);
        }
        1 => tabs::render_git(frame, app, main_area[2]),
        2 => tabs::render_plugins(frame, app, main_area[2]),
        _ => render_placeholder(frame, main_area[2]),
    }
}

// ─── Preview ─────────────────────────────────────────────────────────────────

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::DIM));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.entries.is_empty() {
        return;
    }

    let filename = &app.entries[app.selected].0;
    let visible  = inner.height.saturating_sub(1) as usize;
    let width    = inner.width.saturating_sub(1) as usize;

    let mut lines = vec![Line::from(vec![
        Span::styled(" VISTA PREVIA ", Style::default().fg(MAGENTA).add_modifier(Modifier::DIM | Modifier::BOLD)),
        Span::styled("─ ", dim()),
        Span::styled(filename.as_str(), bold_cyan()),
        Span::styled(
            if app.preview_git_info.is_empty() { String::new() }
            else { format!("  {}", app.preview_git_info) },
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

// ─── Placeholder ─────────────────────────────────────────────────────────────

fn render_placeholder(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Span::styled("  próximamente", dim())),
        area,
    );
}
