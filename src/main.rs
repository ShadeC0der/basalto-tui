use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod icons;
mod index;
mod ui;

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--version") {
        println!("basalto-tui v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = app::App::new();

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press { continue; }
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Backspace | KeyCode::Char('-') => app.navigate_up(),

                    KeyCode::Enter => {
                        if app.sidebar_focused { app.sidebar_toggle_section() }
                        else { app.nav_enter() }
                    }

                    // Ctrl + any direction → toggle sidebar focus
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                    | KeyCode::Char('j') | KeyCode::Char('k')
                    | KeyCode::Char('h') | KeyCode::Char('l')
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                        => app.toggle_sidebar_focus(),

                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.sidebar_focused { app.sidebar_nav_down() } else { app.nav_down() }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.sidebar_focused { app.sidebar_nav_up() } else { app.nav_up() }
                    }

                    KeyCode::Char('l') | KeyCode::Right
                        if !app.sidebar_focused => app.next_tab(),
                    KeyCode::Char('h') | KeyCode::Left
                        if !app.sidebar_focused => app.prev_tab(),

                    _ => {}
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    app.handle_click(mouse.column, mouse.row);
                }
                MouseEventKind::ScrollDown => app.nav_down(),
                MouseEventKind::ScrollUp   => app.nav_up(),
                _ => {}
            },
            _ => {}
        }
    }
}
