use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use super::{CYAN, YELLOW, GREEN, DIM, dim, accent};

pub fn render_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.sidebar_focused {
        Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(CYAN).add_modifier(Modifier::DIM)
    };
    frame.render_widget(
        Block::default().borders(Borders::RIGHT).border_style(border_style),
        area,
    );

    let content_area = Rect { width: area.width.saturating_sub(1), ..area };
    app.sidebar_height = content_area.height;

    let focused = app.sidebar_focused;
    let cursor  = app.sidebar_cursor;
    let col     = app.sidebar_collapsed;
    let mut idx = 0usize;
    let mut lines: Vec<Line> = Vec::new();

    let sel      = |i: usize| focused && i == cursor;
    let hdr_style = |i: usize| -> Style {
        if sel(i) { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }
        else      { Style::default().fg(DIM).add_modifier(Modifier::BOLD)  }
    };

    // ── PLUGINS ──────────────────────────────────────────────────────────────
    let arrow0 = if col[0] { "▶ " } else { "▼ " };
    lines.push(Line::from(vec![
        Span::styled(arrow0, hdr_style(idx)),
        Span::styled("PLUGINS", hdr_style(idx)),
    ]));
    idx += 1;

    if !col[0] {
        if app.plugins.is_empty() {
            lines.push(Line::from(Span::styled("  sin plugins", dim())));
        } else {
            for plugin in &app.plugins {
                let (dot, dot_color) = if plugin.enabled { ("● ", GREEN) } else { ("○ ", DIM) };
                let display = plugin.name.strip_prefix("basalto-").unwrap_or(&plugin.name);
                let s = if sel(idx) { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }
                        else if plugin.enabled { accent() }
                        else { dim() };
                lines.push(Line::from(vec![
                    Span::styled(if sel(idx) { "▸ " } else { "  " }, s),
                    Span::styled(dot, Style::default().fg(if sel(idx) { CYAN } else { dot_color })),
                    Span::styled(display.to_string(), s),
                ]));
                idx += 1;
            }
        }
    }

    lines.push(Line::from(Span::raw("")));

    // ── TAGS ─────────────────────────────────────────────────────────────────
    let arrow1 = if col[1] { "▶ " } else { "▼ " };
    lines.push(Line::from(vec![
        Span::styled(arrow1, hdr_style(idx)),
        Span::styled("TAGS", hdr_style(idx)),
    ]));
    idx += 1;

    if !col[1] {
        if app.tags.is_empty() {
            lines.push(Line::from(Span::styled("  sin tags", dim())));
        } else {
            for (tag, count) in &app.tags {
                let s = if sel(idx) { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }
                        else { accent() };
                lines.push(Line::from(vec![
                    Span::styled(if sel(idx) { "▸ " } else { "  " },
                        if sel(idx) { Style::default().fg(CYAN) } else { dim() }),
                    Span::styled("#", if sel(idx) { Style::default().fg(CYAN) } else { dim() }),
                    Span::styled(tag.clone(), s),
                    Span::styled(format!(" {}", count),
                        Style::default().fg(YELLOW).add_modifier(Modifier::DIM)),
                ]));
                idx += 1;
            }
        }
    }

    lines.push(Line::from(Span::raw("")));

    // ── GIT ──────────────────────────────────────────────────────────────────
    let arrow2 = if col[2] { "▶ " } else { "▼ " };
    lines.push(Line::from(vec![
        Span::styled(arrow2, hdr_style(idx)),
        Span::styled("GIT", hdr_style(idx)),
    ]));
    idx += 1;

    if !col[2] {
        let s = if sel(idx) { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) }
                else { Style::default().fg(Color::White) };
        lines.push(Line::from(vec![
            Span::styled(if sel(idx) { "▸ " } else { "  " },
                if sel(idx) { Style::default().fg(CYAN) } else { dim() }),
            Span::styled("● ", Style::default().fg(if sel(idx) { CYAN } else { GREEN })),
            Span::styled("main", s),
        ]));
    }

    let total      = lines.len();
    let visible    = content_area.height as usize;
    let max_scroll = total.saturating_sub(visible);
    if app.sidebar_scroll > max_scroll { app.sidebar_scroll = max_scroll; }

    frame.render_widget(
        Paragraph::new(lines).scroll((app.sidebar_scroll as u16, 0)),
        content_area,
    );
}
