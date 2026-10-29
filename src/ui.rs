use crate::app::{App, Mode, HINTS};
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

mod overlay;
mod row;

#[derive(Default)]
pub struct UiState {
    current: ListState,
    parent: ListState,
    theme: ListState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRects {
    pub header: Option<Rect>,
    pub header_rule: Option<Rect>,
    pub body: Rect,
    pub footer_rule: Option<Rect>,
    pub footer: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnRects {
    pub parent: Option<Rect>,
    pub current: Rect,
    pub preview: Option<Rect>,
    pub dividers: Vec<Rect>,
}

pub fn draw(frame: &mut Frame, app: &App, state: &mut UiState) {
    frame.render_widget(
        Block::default().style(Style::default().bg(app.theme.bg).fg(app.theme.fg)),
        frame.area(),
    );

    let layout = layout_for(frame.area());
    if let Some(header) = layout.header {
        draw_headers(frame, app, header);
    }
    if let Some(rule) = layout.header_rule {
        draw_rule(frame, rule, app.theme);
    }
    draw_body(frame, app, state, layout.body);
    if let Some(rule) = layout.footer_rule {
        draw_rule(frame, rule, app.theme);
    }
    if let Some(footer) = layout.footer {
        draw_footer(frame, app, footer);
    }

    match app.mode {
        Mode::ThemePicker => overlay::draw_theme_picker(frame, app, &mut state.theme),
        Mode::Help => overlay::draw_help(frame, app),
        Mode::Normal | Mode::Command => {
            if app.show_info {
                overlay::draw_info(frame, app);
            }
        }
    }
}

/// Chrome is dropped before the body on short terminals. At eight rows or more
/// the frame is header, rule, body, rule, footer; below that it is body, footer.
pub fn layout_for(area: Rect) -> LayoutRects {
    let full = area.height >= 8;
    let mut constraints = Vec::with_capacity(5);
    if full {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(1));
    if full {
        constraints.push(Constraint::Length(1));
    }
    if area.height >= 2 {
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut index = 0;
    let header = if full {
        let rect = chunks[index];
        index += 1;
        Some(rect)
    } else {
        None
    };
    let header_rule = if full {
        let rect = chunks[index];
        index += 1;
        Some(rect)
    } else {
        None
    };
    let body = chunks[index];
    index += 1;
    let footer_rule = if full {
        let rect = chunks[index];
        index += 1;
        Some(rect)
    } else {
        None
    };
    let footer = if area.height >= 2 {
        Some(chunks[index])
    } else {
        None
    };
    LayoutRects {
        header,
        header_rule,
        body,
        footer_rule,
        footer,
    }
}

/// The parent column drops first, then the preview; the current column always
/// keeps the remaining space.
pub fn columns_for(area: Rect) -> ColumnRects {
    if area.width >= 72 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(28),
                Constraint::Length(1),
                Constraint::Percentage(36),
                Constraint::Length(1),
                Constraint::Min(10),
            ])
            .split(area);
        ColumnRects {
            parent: Some(chunks[0]),
            current: chunks[2],
            preview: Some(chunks[4]),
            dividers: vec![chunks[1], chunks[3]],
        }
    } else if area.width >= 40 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(52),
                Constraint::Length(1),
                Constraint::Min(10),
            ])
            .split(area);
        ColumnRects {
            parent: None,
            current: chunks[0],
            preview: Some(chunks[2]),
            dividers: vec![chunks[1]],
        }
    } else {
        ColumnRects {
            parent: None,
            current: area,
            preview: None,
            dividers: Vec::new(),
        }
    }
}

fn draw_headers(frame: &mut Frame, app: &App, area: Rect) {
    let columns = columns_for(area);
    if let Some(parent) = columns.parent {
        let title = app
            .cwd
            .parent()
            .map(display_path)
            .unwrap_or_else(|| "/".into());
        draw_header(frame, &title, parent, app.theme, false);
    }
    draw_header(
        frame,
        &display_path(&app.cwd),
        columns.current,
        app.theme,
        true,
    );
    if let Some(preview) = columns.preview {
        let title = app
            .entries
            .get(app.selected)
            .map(row::display_name)
            .unwrap_or_else(|| "preview".into());
        draw_header(frame, &title, preview, app.theme, false);
    }
}

fn draw_header(frame: &mut Frame, text: &str, area: Rect, theme: &Theme, primary: bool) {
    let style = if primary {
        Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };
    let text = row::truncate(&row::clean(text), area.width as usize);
    frame.render_widget(Paragraph::new(Line::from(Span::styled(text, style))), area);
}

fn draw_body(frame: &mut Frame, app: &App, state: &mut UiState, area: Rect) {
    let columns = columns_for(area);
    if let Some(parent) = columns.parent {
        draw_parent(frame, app, state, parent);
    }
    draw_current(frame, app, state, columns.current);
    if let Some(preview) = columns.preview {
        draw_preview(frame, app, preview);
    }
    for divider in columns.dividers {
        draw_vertical(frame, divider, app.theme);
    }
}

fn draw_parent(frame: &mut Frame, app: &App, state: &mut UiState, area: Rect) {
    let width = area.width as usize;
    let items: Vec<ListItem> = app
        .parent_entries
        .iter()
        .map(|entry| ListItem::new(row::row(entry, app.theme, width, 0, false)))
        .collect();
    state.parent.select(app.parent_selected);
    let list = List::new(items)
        .style(Style::default().bg(app.theme.bg))
        .highlight_style(Style::default().bg(app.theme.surface_alt).fg(app.theme.fg));
    frame.render_stateful_widget(list, area, &mut state.parent);
}

fn draw_current(frame: &mut Frame, app: &App, state: &mut UiState, area: Rect) {
    let width = area.width as usize;
    let size_width = row::size_column(&app.entries);
    let mut items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            ListItem::new(row::row(
                entry,
                app.theme,
                width,
                size_width,
                index == app.selected,
            ))
        })
        .collect();
    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "(empty)",
            Style::default().fg(app.theme.muted),
        ))));
    }
    state
        .current
        .select((!app.entries.is_empty()).then_some(app.selected));
    let list = List::new(items)
        .style(Style::default().bg(app.theme.bg))
        .highlight_style(row::selection_style(app.theme));
    frame.render_stateful_widget(list, area, &mut state.current);
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width as usize;
    let lines: Vec<Line> = app
        .preview_lines
        .iter()
        .map(|line| {
            let text = row::truncate(&row::clean(line), width.saturating_sub(1));
            Line::from(Span::styled(
                format!(" {text}"),
                Style::default().fg(app.theme.fg),
            ))
        })
        .collect();
    let paragraph = Paragraph::new(lines).style(Style::default().bg(app.theme.bg));
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    if app.mode == Mode::Command {
        draw_command(frame, app, area);
        return;
    }

    let theme = app.theme;
    let (text, style) = if app.status.text.is_empty() {
        (HINTS.to_string(), Style::default().fg(theme.muted))
    } else {
        let color = match app.status.kind {
            crate::app::StatusKind::Info => theme.fg,
            crate::app::StatusKind::Success => theme.success,
            crate::app::StatusKind::Warning => theme.warning,
            crate::app::StatusKind::Error => theme.danger,
        };
        (row::clean(&app.status.text), Style::default().fg(color))
    };

    let width = area.width as usize;
    let right = app
        .entries
        .get(app.selected)
        .map(|entry| format!("{}  {}", row::display_name(entry), row::size_label(entry)))
        .unwrap_or_default();
    let right = row::clean(&right);
    let show_right = !right.is_empty() && width >= 40 && right.width() + 12 <= width;
    let left_width = if show_right {
        width.saturating_sub(right.width() + 2)
    } else {
        width
    };
    let left = row::truncate(&text, left_width);

    let mut spans = vec![Span::styled(left.clone(), style)];
    if show_right {
        let padding = width.saturating_sub(left.width() + right.width());
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.push(Span::styled(right, Style::default().fg(theme.muted)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_command(frame: &mut Frame, app: &App, area: Rect) {
    let text = format!("{}{}", app.prompt, row::clean(&app.input));
    frame.render_widget(
        Paragraph::new(Span::styled(text, Style::default().fg(app.theme.fg))),
        area,
    );
    let offset = 1 + app.input.width();
    if area.width > 1 && offset < area.width as usize {
        frame.set_cursor_position((area.x + offset as u16, area.y));
    }
}

fn draw_rule(frame: &mut Frame, area: Rect, theme: &Theme) {
    let text = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(text, Style::default().fg(theme.divider))),
        area,
    );
}

fn draw_vertical(frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let lines: Vec<Line> = (0..area.height)
        .map(|_| {
            Line::from(Span::styled(
                "│",
                Style::default().fg(theme.divider).bg(theme.bg),
            ))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn display_path(path: &Path) -> String {
    if let Ok(home) = std::env::var("HOME") {
        let home = Path::new(&home);
        if let Ok(rest) = path.strip_prefix(home) {
            if rest.as_os_str().is_empty() {
                return "~".into();
            }
            return format!("~/{}", rest.display());
        }
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_stays_inside_the_terminal() {
        for width in [20, 40, 80, 120] {
            for height in [1, 4, 5, 10, 24, 40] {
                let area = Rect::new(0, 0, width, height);
                let layout = layout_for(area);
                assert!(
                    layout.body.right() <= area.right(),
                    "{layout:?} vs {area:?}"
                );
                assert!(
                    layout.body.bottom() <= area.bottom(),
                    "{layout:?} vs {area:?}"
                );
                let columns = columns_for(layout.body);
                assert!(columns.current.width > 0, "{columns:?}");
                for divider in &columns.dividers {
                    assert_eq!(divider.width, 1);
                    assert!(divider.bottom() <= area.bottom());
                }
            }
        }
    }

    #[test]
    fn narrow_terminals_drop_parent_then_preview() {
        let wide = columns_for(Rect::new(0, 0, 120, 10));
        assert!(wide.parent.is_some());
        assert!(wide.preview.is_some());

        let medium = columns_for(Rect::new(0, 0, 60, 10));
        assert!(medium.parent.is_none());
        assert!(medium.preview.is_some());

        let narrow = columns_for(Rect::new(0, 0, 30, 10));
        assert!(narrow.parent.is_none());
        assert!(narrow.preview.is_none());
        assert_eq!(narrow.current, Rect::new(0, 0, 30, 10));
    }

    #[test]
    fn short_terminals_drop_the_header_before_the_body() {
        let short = layout_for(Rect::new(0, 0, 80, 5));
        assert!(short.header.is_none());
        assert!(short.footer.is_some());
        assert_eq!(short.body.height, 4);

        let tall = layout_for(Rect::new(0, 0, 80, 24));
        assert!(tall.header.is_some());
        assert!(tall.header_rule.is_some());
        assert!(tall.footer_rule.is_some());
        assert_eq!(tall.body.height, 20);
    }
}
