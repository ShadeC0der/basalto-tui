use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};
use std::io;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        terminal.draw(render)?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn render(frame: &mut ratatui::Frame) {
    let area = frame.area();

    // Root layout: title bar | body | status bar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Title bar
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" basalto ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("— ~/biblioteca ", Style::default().fg(Color::White)),
        Span::styled("[git: main ✓ sincronizado]", Style::default().fg(Color::Green)),
    ]));
    frame.render_widget(title, root[0]);

    // Body: sidebar | main
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(0),
        ])
        .split(root[1]);

    // Sidebar border
    let sidebar_block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(sidebar_block, body[0]);

    let sidebar_content = Paragraph::new(vec![
        Line::from(Span::styled("PLUGINS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" ▶ biblioteca", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("")),
        Line::from(Span::styled("TAGS", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("  # rust")),
        Line::from(Span::raw("  # python")),
        Line::from(Span::raw("")),
        Line::from(Span::styled("GIT", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  ● main", Style::default().fg(Color::Green))),
    ])
    .style(Style::default().fg(Color::White));
    frame.render_widget(sidebar_content, body[0]);

    // Main area: tabs | separator | content
    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(body[1]);

    let tabs = Tabs::new(vec!["biblioteca", "nueva entrada", "plugins", "git/github"])
        .select(0)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .divider(" ");
    frame.render_widget(tabs, main_area[0]);

    let separator = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(separator, main_area[1]);

    let placeholder = Paragraph::new(Span::styled(
        " biblioteca vacía — usa basalto add para agregar archivos",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(placeholder, main_area[2]);

    // Status bar: mode | info | keybindings
    let status_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(root[2]);

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" NORMAL ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled("  biblioteca · 0/0 ", Style::default().fg(Color::White)),
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
