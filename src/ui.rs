use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Root layout: title bar | body | status bar + keybindings
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

    render_sidebar(frame, app, body[0]);
    render_main(frame, app, body[1]);
    render_status(frame, app, root[2]);
}

fn render_title(frame: &mut Frame, _app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" basalto ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("— ~/biblioteca ", Style::default().fg(Color::White)),
        Span::styled("[git: main ✓ sincronizado]", Style::default().fg(Color::Green)),
    ]));
    frame.render_widget(title, area);
}

fn render_sidebar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let sidebar_block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(sidebar_block, area);

    let mut lines = vec![
        Line::from(Span::styled("PLUGINS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" ▶ biblioteca", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        Line::from(Span::styled("TAGS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
    ];

    if app.tags.is_empty() {
        lines.push(Line::from(Span::styled("  sin tags", Style::default().fg(Color::DarkGray))));
    } else {
        for (tag, count) in &app.tags {
            lines.push(Line::from(vec![
                Span::styled(format!("  #{}", tag), Style::default().fg(Color::Cyan)),
                Span::styled(format!(" {}", count), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled("GIT", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(Span::styled("  ● main", Style::default().fg(Color::Green))));

    let content = Paragraph::new(lines).style(Style::default().fg(Color::White));
    frame.render_widget(content, area);
}

fn render_main(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    // Tabs
    let tabs = Tabs::new(vec!["biblioteca", "nueva entrada", "plugins", "git/github"])
        .select(app.tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .divider(" ");
    frame.render_widget(tabs, main_area[0]);

    let separator = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(separator, main_area[1]);

    render_file_list(frame, app, main_area[2]);
}

fn render_file_list(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.entries.is_empty() {
        let empty = Paragraph::new(Span::styled(
            " biblioteca vacía — usa basalto add para agregar archivos",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(empty, area);
        return;
    }

    let height = area.height as usize;
    // Keep selected entry visible by computing scroll offset
    let scroll = if app.selected >= height {
        app.selected - height + 1
    } else {
        0
    };

    let width = area.width as usize;
    let num_width = 3usize;
    let tag_col = 18usize;
    let name_col = 24usize;
    // Description fills the remaining space
    let desc_width = width.saturating_sub(num_width + 2 + name_col + 2 + tag_col + 1);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (name, meta)) in app.entries.iter().enumerate().skip(scroll).take(height) {
        let is_selected = i == app.selected;

        let num_span = Span::styled(
            format!("{:>3} ", i + 1),
            Style::default().fg(Color::DarkGray),
        );

        let name_str = truncate(name, name_col);
        let name_span = Span::styled(
            format!("{:<width$}  ", name_str, width = name_col),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        );

        let desc_str = truncate(&meta.description, desc_width);
        let desc_span = Span::styled(
            format!("{:<width$}  ", desc_str, width = desc_width),
            Style::default().fg(Color::DarkGray),
        );

        let tags_str = if meta.tags.is_empty() {
            String::new()
        } else {
            meta.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")
        };
        let tags_str = truncate(&tags_str, tag_col);
        let tags_span = Span::styled(tags_str, Style::default().fg(Color::Cyan));

        let line = if is_selected {
            Line::from(vec![
                num_span.style(Style::default().fg(Color::DarkGray).bg(Color::Blue)),
                name_span.style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD).bg(Color::Blue)),
                desc_span.style(Style::default().fg(Color::Gray).bg(Color::Blue)),
                tags_span.style(Style::default().fg(Color::Cyan).bg(Color::Blue)),
            ])
        } else {
            Line::from(vec![num_span, name_span, desc_span, tags_span])
        };

        lines.push(line);
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

fn render_status(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let total = app.entries.len();
    let current = if total == 0 { 0 } else { app.selected + 1 };

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" NORMAL ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("  biblioteca · {}/{} ", current, total),
            Style::default().fg(Color::White),
        ),
        Span::styled("  git:main ", Style::default().fg(Color::Green)),
        Span::styled("  basalto-tui v0.1.0", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(status_bar, status_area[0]);

    let keybindings = Paragraph::new(Line::from(vec![
        Span::styled(" j/k ", Style::default().fg(Color::Cyan)),
        Span::styled("mover  ", Style::default().fg(Color::DarkGray)),
        Span::styled("enter ", Style::default().fg(Color::Cyan)),
        Span::styled("abrir  ", Style::default().fg(Color::DarkGray)),
        Span::styled("n ", Style::default().fg(Color::Cyan)),
        Span::styled("nuevo  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q ", Style::default().fg(Color::Cyan)),
        Span::styled("salir", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(keybindings, status_area[1]);
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}…", chars[..max.saturating_sub(1)].iter().collect::<String>())
    }
}
