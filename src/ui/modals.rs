use crate::app::{App, Modal};
use crate::index::EntryMeta;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use super::{CYAN, MAGENTA, YELLOW, GREEN, WHITE, GRAY, DIM, BLACK, dim, bold_cyan, truncate, centered};

// ─── Command popup ────────────────────────────────────────────────────────────

pub fn render_command_popup(frame: &mut Frame, input: &str, area: Rect) {
    let popup = centered(area, 62, 3);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " basalto command ",
            Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(": ", Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD)),
            Span::styled(input.to_string(), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default().fg(BLACK).bg(MAGENTA)),
        ])),
        inner,
    );
}

// ─── Help popup ───────────────────────────────────────────────────────────────

pub fn render_help_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered(area, 62, 36);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " ? ayuda · basalto-tui ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let hdr = |s: &str| -> Line<'static> {
        Line::from(Span::styled(
            format!(" {} ", s),
            Style::default().fg(BLACK).bg(CYAN).add_modifier(Modifier::BOLD),
        ))
    };
    let key = |k: &str, d: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("  {:16}", k), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(d.to_string(), Style::default().fg(WHITE)),
        ])
    };
    let blank = Line::from(Span::raw(""));

    let lines: Vec<Line> = vec![
        hdr("MODO NORMAL"),
        key("q",           "salir"),
        key("? / :help",   "esta ayuda"),
        key("i",           "editar archivos inline (insert mode)"),
        key("dd",          "marcar archivo/carpeta para eliminar"),
        key("Ctrl+S",      "confirmar eliminaciones pendientes"),
        key("Esc",         "cancelar eliminaciones pendientes"),
        key("j / ↓",       "mover abajo"),
        key("k / ↑",       "mover arriba"),
        key("l / →",       "tab siguiente"),
        key("h / ←",       "tab anterior"),
        key("Enter",       "entrar a carpeta / cambiar biblioteca"),
        key("- / Bksp",    "volver a carpeta anterior"),
        key("Ctrl+dir",    "enfocar / salir del sidebar"),
        key(":",           "abrir prompt de comandos basalto"),
        blank.clone(),
        hdr("MODO INSERT  (i)"),
        key("j/k  ↑↓",    "navegar entre archivos"),
        key("← →",         "mover cursor dentro del nombre"),
        key("Enter",       "nueva línea (crear archivo/carpeta)"),
        key("nombre/",     "terminar con / para crear carpeta"),
        key("Ctrl+S",      "guardar cambios (renombrar + crear)"),
        key("Esc",         "cancelar sin guardar"),
        blank.clone(),
        hdr("SIDEBAR"),
        key("Ctrl+dir",    "enfocar sidebar"),
        key("j / k",       "navegar ítems"),
        key("Enter",       "colapsar / expandir sección"),
        blank.clone(),
        hdr("COMANDOS  (:)"),
        key(":help",       "esta ayuda"),
        key(":add [arch]", "agregar/editar metadata de un archivo"),
        key(":list",       "listar entradas de la biblioteca"),
        key(":show [n]",   "ver metadata de un archivo"),
        key(":remove [n]", "eliminar del índice"),
        key(":push",       "push de la biblioteca a git"),
        blank.clone(),
        Line::from(Span::styled(
            "  cualquier tecla para cerrar  ·  j/k para scroll",
            Style::default().fg(DIM),
        )),
    ];

    let total     = lines.len() as u16;
    let visible   = inner.height;
    let max_scroll = total.saturating_sub(visible);
    let scroll    = app.help_scroll.min(max_scroll);

    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), inner);
}

// ─── Modal dispatcher ─────────────────────────────────────────────────────────

pub fn render_modal(frame: &mut Frame, app: &App, area: Rect) {
    match &app.modal {
        Some(Modal::Add { fields, focused })         => render_modal_add(frame, fields, *focused, area),
        Some(Modal::Show { lines, scroll })           => render_modal_show(frame, lines, *scroll, area),
        Some(Modal::Remove { key })                   => render_modal_remove(frame, key, area),
        Some(Modal::List { entries, cursor })         => render_modal_list(frame, entries, *cursor, area),
        Some(Modal::Output { title, lines, scroll }) => render_modal_output(frame, title, lines, *scroll, area),
        Some(Modal::Use { key, dest, is_dir })        => render_modal_use(frame, key, dest, *is_dir, area),
        None => {}
    }
}

// ─── :add ─────────────────────────────────────────────────────────────────────

fn render_modal_add(frame: &mut Frame, fields: &[String; 3], focused: usize, area: Rect) {
    let popup = centered(area, 60, 9);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " + agregar al índice ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let labels = ["Archivo:", "Descripción:", "Tags (separados por ,):"];
    let mut lines: Vec<Line> = vec![Line::from(Span::raw(""))];

    for (i, (label, value)) in labels.iter().zip(fields.iter()).enumerate() {
        let active = i == focused;
        let label_span = Span::styled(
            format!("  {:<24}", label),
            if active { Style::default().fg(CYAN).add_modifier(Modifier::BOLD) } else { dim() },
        );
        let value_span = Span::styled(
            value.clone(),
            if active { Style::default().fg(WHITE).add_modifier(Modifier::BOLD) } else { Style::default().fg(GRAY) },
        );
        let cursor = Span::styled(
            " ",
            if active { Style::default().fg(BLACK).bg(CYAN) } else { Style::default() },
        );
        lines.push(Line::from(vec![label_span, value_span, cursor]));
        lines.push(Line::from(Span::raw("")));
    }

    lines.push(Line::from(vec![
        Span::styled("  tab/↵ ", Style::default().fg(CYAN)),
        Span::styled("siguiente campo    ", dim()),
        Span::styled("enter en último ", Style::default().fg(GREEN)),
        Span::styled("guardar    ", dim()),
        Span::styled("esc ", Style::default().fg(Color::Red)),
        Span::styled("cancelar", dim()),
    ]));

    frame.render_widget(Paragraph::new(lines), inner);
}

// ─── :show ────────────────────────────────────────────────────────────────────

fn render_modal_show(frame: &mut Frame, lines: &[(String, String)], scroll: u16, area: Rect) {
    let popup = centered(area, 60, 10);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " ≡ metadata del archivo ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut content: Vec<Line> = vec![Line::from(Span::raw(""))];
    for (label, value) in lines {
        content.push(Line::from(vec![
            Span::styled(format!("  {:<16}", label), Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(value.clone(), Style::default().fg(WHITE)),
        ]));
    }
    content.push(Line::from(Span::raw("")));
    content.push(Line::from(Span::styled("  q / esc para cerrar", dim())));

    frame.render_widget(Paragraph::new(content).scroll((scroll, 0)), inner);
}

// ─── :remove ──────────────────────────────────────────────────────────────────

fn render_modal_remove(frame: &mut Frame, key: &str, area: Rect) {
    let popup = centered(area, 52, 7);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " ✗ confirmar eliminación ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  Eliminar del índice: ", Style::default().fg(WHITE)),
                Span::styled(key.to_string(), Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  (no elimina el archivo en disco)", dim())),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  y / Enter ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("confirmar    ", dim()),
                Span::styled("n / Esc ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("cancelar", dim()),
            ]),
        ]),
        inner,
    );
}

// ─── :list ────────────────────────────────────────────────────────────────────

fn render_modal_list(frame: &mut Frame, entries: &[(String, EntryMeta)], cursor: usize, area: Rect) {
    let popup = centered(area, 70, 20);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            " ≡ entradas en el índice ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let height  = inner.height as usize;
    let scroll  = if cursor >= height.saturating_sub(2) {
        cursor.saturating_sub(height.saturating_sub(3))
    } else {
        0
    };

    let mut lines: Vec<Line> = if entries.is_empty() {
        vec![Line::from(Span::styled("  índice vacío", dim()))]
    } else {
        entries.iter().enumerate().map(|(i, (key, meta))| {
            let selected  = i == cursor;
            let indicator = if selected { "▶ " } else { "  " };
            let tags_str  = meta.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            Line::from(vec![
                Span::styled(indicator, if selected { bold_cyan() } else { dim() }),
                Span::styled(
                    truncate(key, 30),
                    if selected { bold_cyan() } else { Style::default().fg(WHITE) },
                ),
                Span::styled("  ", dim()),
                Span::styled(
                    truncate(&meta.description, 22),
                    if selected { Style::default().fg(GRAY) } else { dim() },
                ),
                Span::styled("  ", dim()),
                Span::styled(
                    truncate(&tags_str, 14),
                    if selected { Style::default().fg(YELLOW) }
                    else { Style::default().fg(CYAN).add_modifier(Modifier::DIM) },
                ),
            ])
        }).collect()
    };

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled("  j/k navegar    q / esc cerrar", dim())));

    frame.render_widget(Paragraph::new(lines).scroll((scroll as u16, 0)), inner);
}

// ─── :use / Usar en proyecto ──────────────────────────────────────────────────

fn render_modal_use(frame: &mut Frame, key: &str, dest: &str, is_dir: bool, area: Rect) {
    let popup = centered(area, 62, 9);
    frame.render_widget(Clear, popup);

    let (title, icon) = if is_dir {
        (" ↓ copiar plantilla/carpeta ", " ")
    } else {
        (" ↓ usar archivo en proyecto ", " ")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        .title(Span::styled(title, Style::default().fg(GREEN).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let type_hint = if is_dir {
        Span::styled("  (se copian todos los archivos internos)", dim())
    } else {
        Span::styled("", Style::default())
    };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  origen:   ", dim()),
                Span::styled(icon, Style::default().fg(if is_dir { YELLOW } else { CYAN })),
                Span::styled(key.to_string(), Style::default().fg(if is_dir { YELLOW } else { CYAN }).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(type_hint),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  copiar a: ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(dest.to_string(), Style::default().fg(WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default().fg(BLACK).bg(GREEN)),
            ]),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("  enter ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("copiar    ", dim()),
                Span::styled("esc ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("cancelar", dim()),
            ]),
        ]),
        inner,
    );
}

// ─── :push / Output ───────────────────────────────────────────────────────────

fn render_modal_output(frame: &mut Frame, title: &str, lines: &[String], scroll: u16, area: Rect) {
    let popup = centered(area, 66, 16);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut content: Vec<Line> = lines.iter().map(|l| {
        let style = if l.starts_with('✓')      { Style::default().fg(GREEN) }
                    else if l.starts_with('✗') { Style::default().fg(Color::Red) }
                    else if l.starts_with("error") { Style::default().fg(Color::Red) }
                    else { Style::default().fg(WHITE) };
        Line::from(Span::styled(format!(" {}", l), style))
    }).collect();

    content.push(Line::from(Span::raw("")));
    content.push(Line::from(Span::styled(" j/k scroll    q / esc cerrar", dim())));

    frame.render_widget(Paragraph::new(content).scroll((scroll, 0)), inner);
}
