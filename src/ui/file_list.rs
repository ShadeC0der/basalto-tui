use crate::app::{App, Mode};
use crate::icons;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use super::{CYAN, YELLOW, GREEN, WHITE, GRAY, BLACK, dim, bold_cyan, accent, truncate};

pub fn render_file_list(frame: &mut Frame, app: &mut App, area: Rect) {
    app.list_area = area;

    if let Mode::Insert(_) = &app.mode {
        render_file_list_insert(frame, app, area);
        return;
    }

    if app.entries.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  biblioteca vacía — usa basalto add para agregar archivos",
                dim(),
            )),
            area,
        );
        return;
    }

    let height = area.height as usize;
    let scroll = if app.selected >= height { app.selected - height + 1 } else { 0 };
    let width  = area.width as usize;
    let num_w  = 3usize;
    let name_w = 24usize;
    let tags_w = 18usize;
    let desc_w = width.saturating_sub(2 + num_w + 2 + name_w + 2 + tags_w + 1);

    let mut lines: Vec<Line> = Vec::new();

    for (i, (name, meta)) in app.entries.iter().enumerate().skip(scroll).take(height) {
        let selected = i == app.selected;
        let is_dir   = meta.is_dir;
        let del      = app.pending_deletions.contains(name);

        let indicator = if del {
            Span::styled("✗ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else if selected {
            Span::styled("▶ ", bold_cyan())
        } else {
            Span::styled("  ", dim())
        };

        let num = if name == ".." {
            Span::styled("    ", dim())
        } else {
            Span::styled(format!("{:>3} ", i + 1), if selected { accent() } else { dim() })
        };

        let (icon_char, icon_color) = icons::icon_for(name, is_dir);
        let icon = Span::styled(
            icon_char,
            Style::default().fg(icon_color)
                .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
        );

        let name_display = if name == ".." { "..".to_string() } else { name.clone() };
        let name_str  = truncate(&name_display, name_w.saturating_sub(2));
        let name_span = Span::styled(
            format!("{:<w$}  ", name_str, w = name_w.saturating_sub(2)),
            if del {
                Style::default().fg(Color::Red).add_modifier(Modifier::CROSSED_OUT)
            } else if name == ".." {
                dim()
            } else if is_dir {
                if selected { Style::default().fg(YELLOW).add_modifier(Modifier::BOLD) }
                else        { Style::default().fg(YELLOW) }
            } else if selected {
                bold_cyan()
            } else {
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
            },
        );

        let desc_str  = truncate(&meta.description, desc_w);
        let desc_span = Span::styled(
            format!("{:<w$}  ", desc_str, w = desc_w),
            if selected { Style::default().fg(GRAY) } else { dim() },
        );

        let tags_raw  = meta.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
        let tags_str  = truncate(&tags_raw, tags_w);
        let tags_span = Span::styled(
            tags_str,
            if selected { Style::default().fg(YELLOW) }
            else        { Style::default().fg(CYAN).add_modifier(Modifier::DIM) },
        );

        lines.push(Line::from(vec![indicator, num, icon, name_span, desc_span, tags_span]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_file_list_insert(frame: &mut Frame, app: &App, area: Rect) {
    let Mode::Insert(buf) = &app.mode else { return; };

    let height = area.height as usize;
    let scroll = if buf.cursor >= height { buf.cursor - height + 1 } else { 0 };
    let mut lines: Vec<Line> = Vec::new();

    for (i, item) in buf.items.iter().enumerate().skip(scroll).take(height) {
        let selected = i == buf.cursor;
        let is_new   = item.original.is_none();
        let is_dir   = item.current.ends_with('/') || item.is_dir;

        let indicator = if selected { Span::styled("▶ ", bold_cyan()) }
                        else        { Span::styled("  ", dim()) };

        let num = Span::styled(
            format!("{:>3} ", i + 1),
            if selected { accent() } else { dim() },
        );

        let (icon_char, icon_color) = if item.current.is_empty() {
            (" ", GREEN)
        } else {
            icons::icon_for(&item.current, is_dir)
        };

        let icon  = Span::styled(
            icon_char,
            Style::default().fg(if is_new { GREEN } else { icon_color })
                .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
        );
        let space = Span::styled(" ", dim());

        if selected {
            let chars: Vec<char> = item.current.chars().collect();
            let col = buf.col.min(chars.len());
            let before: String = chars[..col].iter().collect();
            let cursor_char: String = if col < chars.len() {
                chars[col..col + 1].iter().collect()
            } else {
                " ".to_string()
            };
            let after: String = if col < chars.len() {
                chars[col + 1..].iter().collect()
            } else {
                String::new()
            };

            let name_style = if is_new {
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
            } else {
                bold_cyan()
            };

            lines.push(Line::from(vec![
                indicator,
                num,
                icon,
                space,
                Span::styled(before, name_style),
                Span::styled(cursor_char, Style::default().fg(BLACK).bg(if is_new { GREEN } else { CYAN })),
                Span::styled(after, name_style),
            ]));
        } else {
            let name_style = if is_new    { Style::default().fg(GREEN) }
                             else if is_dir { Style::default().fg(YELLOW) }
                             else          { Style::default().fg(WHITE).add_modifier(Modifier::BOLD) };
            lines.push(Line::from(vec![
                indicator,
                num,
                icon,
                space,
                Span::styled(item.current.clone(), name_style),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines), area);
}
