use crate::app::{App, Mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use super::{CYAN, MAGENTA, GREEN, dim, accent};

pub fn render_status(frame: &mut Frame, app: &mut App, area: Rect) {
    let zones = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let total   = app.entries.len();
    let current = if total == 0 { 0 } else { app.selected + 1 };

    match &app.mode {
        Mode::Normal => render_status_normal(frame, app, zones[0], zones[1], current, total),
        Mode::Insert(_) => render_status_insert(frame, zones[0], zones[1], current, total),
        Mode::Command(_) => render_status_command(frame, zones[0], zones[1]),
    }
}

fn render_status_normal(
    frame: &mut Frame,
    app: &App,
    top: Rect,
    bottom: Rect,
    current: usize,
    total: usize,
) {
    let del_count = app.pending_deletions.len();
    let mut spans = vec![
        Span::styled(" NORMAL ", Style::default().bg(CYAN).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled("  ", dim()),
        Span::styled(format!("biblioteca · {}/{}", current, total), Style::default().fg(Color::White)),
    ];

    if del_count > 0 {
        spans.push(Span::styled(
            format!("   ✗ {} pendiente{}  ^s confirmar  esc cancelar",
                del_count, if del_count == 1 { "" } else { "s" }),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    } else if app.d_pressed {
        spans.push(Span::styled("   d█  dd para eliminar", Style::default().fg(Color::Red)));
    } else {
        spans.push(Span::styled("   git:", dim()));
        spans.push(Span::styled("main", Style::default().fg(GREEN)));
        spans.push(Span::styled("   basalto-tui v", dim()));
        spans.push(Span::styled(env!("CARGO_PKG_VERSION"), dim()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), top);

    let keys = if app.sidebar_focused {
        Paragraph::new(Line::from(vec![
            Span::styled(" jk/↑↓", accent()),
            Span::styled(" sección    ", dim()),
            Span::styled("enter", accent()),
            Span::styled(" colapsar    ", dim()),
            Span::styled("^dir", accent()),
            Span::styled(" salir sidebar    ", dim()),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::styled(" salir", dim()),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" jk/↑↓", accent()),
            Span::styled(" nav    ", dim()),
            Span::styled("hl/←→", accent()),
            Span::styled(" tabs    ", dim()),
            Span::styled("i", accent()),
            Span::styled(" crear    ", dim()),
            Span::styled(":", accent()),
            Span::styled(" comando    ", dim()),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::styled(" salir", dim()),
        ]))
    };
    frame.render_widget(keys, bottom);
}

fn render_status_insert(frame: &mut Frame, top: Rect, bottom: Rect, current: usize, total: usize) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" INSERT ", Style::default().bg(GREEN).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("  ", dim()),
            Span::styled(format!("biblioteca · {}/{}", current, total), Style::default().fg(Color::White)),
        ])),
        top,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓/jk", accent()),
            Span::styled(" nav    ", dim()),
            Span::styled("←→", accent()),
            Span::styled(" cursor    ", dim()),
            Span::styled("enter", accent()),
            Span::styled(" nueva línea    ", dim()),
            Span::styled("^s", Style::default().fg(GREEN)),
            Span::styled(" guardar    ", dim()),
            Span::styled("/ al final para carpeta    ", dim()),
            Span::styled("esc", Style::default().fg(Color::Red)),
            Span::styled(" cancelar", dim()),
        ])),
        bottom,
    );
}

fn render_status_command(frame: &mut Frame, top: Rect, bottom: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" COMMAND ", Style::default().bg(MAGENTA).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("  ", dim()),
            Span::styled("ejecutar comando basalto", Style::default().fg(Color::White)),
        ])),
        top,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" enter", Style::default().fg(MAGENTA)),
            Span::styled(" ejecutar    ", dim()),
            Span::styled("esc", Style::default().fg(Color::Red)),
            Span::styled(" cancelar", dim()),
        ])),
        bottom,
    );
}

