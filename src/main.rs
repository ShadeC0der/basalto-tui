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

use app::Mode;

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

        if app.should_quit { return Ok(()); }

        if let Some(path) = app.pending_open.take() {
            open_in_editor(terminal, &path)?;
            app.reload_all();
            continue;
        }

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press { continue; }

                // Modal tiene prioridad sobre el modo normal
                if app.modal.is_some() {
                    if let Some(cmd) = handle_modal(&mut app, key.code) {
                        run_command(terminal, &cmd)?;
                        app.reload_all();
                    }
                    continue;
                }

                match &app.mode {
                    Mode::Insert(_) => handle_insert(&mut app, key.code, key.modifiers),
                    Mode::Command(_) => {
                        if let Some(cmd) = handle_command(&mut app, key.code) {
                            run_command(terminal, &cmd)?;
                            app.reload_all();
                        }
                    }
                    Mode::Normal => handle_normal(&mut app, key.code, key.modifiers),
                }
            }
            Event::Mouse(mouse) => {
                if matches!(app.mode, Mode::Normal) && app.modal.is_none() {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => app.handle_click(mouse.column, mouse.row),
                        MouseEventKind::ScrollDown => app.nav_down(),
                        MouseEventKind::ScrollUp   => app.nav_up(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_normal(app: &mut app::App, code: KeyCode, _mods: KeyModifiers) {
    // Cuando la ayuda está abierta j/k scrollean, cualquier otra tecla la cierra
    if app.show_help {
        match code {
            KeyCode::Char('j') | KeyCode::Down  => app.help_scroll = app.help_scroll.saturating_add(1),
            KeyCode::Char('k') | KeyCode::Up    => app.help_scroll = app.help_scroll.saturating_sub(1),
            _ => { app.show_help = false; app.help_scroll = 0; }
        }
        return;
    }
    let mods = _mods;

    // dd → marcar/desmarcar para eliminar
    if code == KeyCode::Char('d') && !mods.contains(KeyModifiers::CONTROL) {
        if app.d_pressed {
            app.toggle_delete();
            app.d_pressed = false;
        } else {
            app.d_pressed = true;
        }
        return;
    }
    app.d_pressed = false;

    match code {
        KeyCode::Char('q') => { app.should_quit = true; }
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char('i') => app.enter_insert(),
        KeyCode::Char(':') => app.enter_command(),
        KeyCode::Char('s') if mods.contains(KeyModifiers::CONTROL) => app.commit_deletions(),
        KeyCode::Esc => app.cancel_deletions(),
        KeyCode::Backspace | KeyCode::Char('-') => app.navigate_up(),

        KeyCode::Enter => {
            if app.sidebar_focused { app.sidebar_toggle_section() }
            else { app.nav_enter() }
        }

        // Ctrl + any direction → toggle sidebar focus
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        | KeyCode::Char('j') | KeyCode::Char('k')
        | KeyCode::Char('h') | KeyCode::Char('l')
            if mods.contains(KeyModifiers::CONTROL)
            => app.toggle_sidebar_focus(),

        KeyCode::Char('j') | KeyCode::Down => {
            if app.sidebar_focused { app.sidebar_nav_down() } else { app.nav_down() }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.sidebar_focused { app.sidebar_nav_up() } else { app.nav_up() }
        }

        KeyCode::Char('l') | KeyCode::Right if !app.sidebar_focused => app.next_tab(),
        KeyCode::Char('h') | KeyCode::Left  if !app.sidebar_focused => app.prev_tab(),
        KeyCode::Char('u') if app.tab == 0 => {
            let key = app.selected_key();
            if !key.is_empty() { app.open_use_modal(&key); }
        }
        KeyCode::Char('r') if app.tab == 1 => app.load_git_info(),

        _ => {}
    }
}

fn handle_insert(app: &mut app::App, code: KeyCode, mods: KeyModifiers) {
    match code {
        KeyCode::Esc                                                  => app.mode_cancel(),
        KeyCode::Backspace                                            => app.insert_pop(),
        KeyCode::Enter                                                => app.insert_newline(),
        KeyCode::Char('s') if mods.contains(KeyModifiers::CONTROL)   => { app.confirm_insert(); }
        KeyCode::Up   | KeyCode::Char('k')                           => app.insert_nav_up(),
        KeyCode::Down | KeyCode::Char('j')                           => app.insert_nav_down(),
        KeyCode::Left                                                 => app.insert_col_left(),
        KeyCode::Right                                                => app.insert_col_right(),
        KeyCode::Char(c)                                              => app.insert_push(c),
        _                                                             => {}
    }
}

// Maneja teclas cuando hay un modal abierto. Devuelve Some(cmd) solo si el
// modal necesita ejecutar un comando en terminal.
fn handle_modal(app: &mut app::App, code: KeyCode) -> Option<String> {
    use app::Modal;
    match &app.modal {
        Some(Modal::Add { .. }) => handle_modal_add(app, code),
        Some(Modal::Remove { .. }) => handle_modal_remove(app, code),
        Some(Modal::Use { .. }) => handle_modal_use(app, code),
        Some(Modal::Show { .. })
        | Some(Modal::List { .. })
        | Some(Modal::Output { .. }) => {
            match code {
                KeyCode::Esc | KeyCode::Char('q') => app.close_modal(),
                KeyCode::Char('j') | KeyCode::Down  => app.modal_scroll_down(),
                KeyCode::Char('k') | KeyCode::Up    => app.modal_scroll_up(),
                _ => {}
            }
            None
        }
        None => None,
    }
}

fn handle_modal_add(app: &mut app::App, code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc            => { app.close_modal(); None }
        KeyCode::Backspace      => { app.modal_add_pop(); None }
        KeyCode::Tab            => { app.modal_add_tab(true); None }
        KeyCode::BackTab        => { app.modal_add_tab(false); None }
        KeyCode::Enter          => { app.modal_add_enter(); None }
        KeyCode::Char(c)        => { app.modal_add_push(c); None }
        _                       => None,
    }
}

fn handle_modal_use(app: &mut app::App, code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc       => { app.close_modal(); None }
        KeyCode::Backspace => { app.modal_use_pop(); None }
        KeyCode::Tab       => { app.modal_use_tab_complete(); None }
        KeyCode::Enter     => { app.modal_confirm_use(); None }
        KeyCode::Char(c)   => { app.modal_use_push(c); None }
        _                  => None,
    }
}

fn handle_modal_remove(app: &mut app::App, code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc | KeyCode::Char('n') => { app.close_modal(); None }
        KeyCode::Char('y') | KeyCode::Enter => { app.modal_confirm_remove(); None }
        _ => None,
    }
}

// Returns the command string when the user presses Enter, None otherwise.
fn handle_command(app: &mut app::App, code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc       => { app.mode_cancel(); None }
        KeyCode::Backspace => { app.mode_pop(); None }
        KeyCode::Enter     => app.take_command(),
        KeyCode::Char(c)   => { app.mode_push(c); None }
        _                  => None,
    }
}

fn open_in_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    path: &str,
) -> io::Result<()> {
    use std::io::Write;

    let editor = find_terminal_editor();

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    io::stdout().flush()?;

    let _ = std::process::Command::new(&editor).arg(path).status();

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    Ok(())
}

fn find_terminal_editor() -> String {
    // Editores GUI que no funcionan en terminal (abren ventana separada y salen)
    const GUI_EDITORS: &[&str] = &["code", "code-insiders", "subl", "subl3", "atom",
                                    "gedit", "kate", "mousepad", "xed", "notepad"];

    let from_env = std::env::var("BASALTO_EDITOR")  // override explícito
        .or_else(|_| std::env::var("EDITOR"))
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_default();

    let bin_name = from_env.split('/').next_back().unwrap_or("").trim().to_string();
    if !bin_name.is_empty() && !GUI_EDITORS.contains(&bin_name.as_str()) {
        return from_env;
    }

    // Buscar el primer editor de terminal disponible en el sistema
    for candidate in &["nvim", "vim", "nano", "vi"] {
        if std::process::Command::new("which")
            .arg(candidate)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return candidate.to_string();
        }
    }

    from_env // último recurso: usar lo que haya aunque sea GUI
}

fn run_command(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cmd: &str,
) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    println!();
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if let Some((bin, args)) = parts.split_first() && let Err(e) = std::process::Command::new(bin).args(args).status() {
        eprintln!("error: {}", e);
    }

    // Resetear atributos del terminal y esperar Enter para consumir
    // cualquier input bufferizado antes de reanudar el TUI
    print!("\x1b[0m\n\x1b[2m── Presiona Enter para volver ──\x1b[0m ");
    {
        use std::io::{BufRead, Write};
        io::stdout().flush().ok();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut String::new()).ok();
    }

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    Ok(())
}
