use crate::app::{App, Mode, HINTS};
use crate::fs::{self, Entry};
use crate::theme::{self, Theme};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

#[derive(Default)]
pub struct UiState {
    current: ListState,
    parent: ListState,
    theme: ListState,
}

pub fn draw(frame: &mut Frame, app: &App, state: &mut UiState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
        ])
        .split(rows[0]);

    draw_parent(frame, app, state, cols[0]);
    draw_current(frame, app, state, cols[1]);
    draw_preview(frame, app, cols[2]);
    draw_status(frame, app, rows[1]);
    draw_command(frame, app, rows[2]);
    if app.mode == Mode::ThemePicker {
        draw_theme_picker(frame, app, state);
    }
    if app.show_info {
        draw_info_popup(frame, app);
    }
}

fn draw_parent(frame: &mut Frame, app: &App, state: &mut UiState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .parent_entries
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(Span::styled(
                entry_name(entry),
                entry_style(entry, app.theme),
            )))
        })
        .collect();
    let title = app
        .cwd
        .parent()
        .map(|path| clean(&path.display().to_string()))
        .unwrap_or_else(|| "/".into());
    state.parent.select(app.parent_selected);
    let list = List::new(items)
        .block(block(&title, app.theme))
        .style(Style::default().bg(app.theme.bg))
        .highlight_style(selection_style(app.theme));
    frame.render_stateful_widget(list, area, &mut state.parent);
}

fn draw_current(frame: &mut Frame, app: &App, state: &mut UiState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .map(|entry| {
            ListItem::new(Line::from(Span::styled(
                entry_name(entry),
                entry_style(entry, app.theme),
            )))
        })
        .collect();
    let title = clean(&app.cwd.display().to_string());
    state
        .current
        .select((!app.entries.is_empty()).then_some(app.selected));
    let list = List::new(items)
        .block(block(&title, app.theme))
        .style(Style::default().bg(app.theme.bg))
        .highlight_style(selection_style(app.theme))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, &mut state.current);
}

fn info_popup_area(area: Rect, lines: usize) -> Rect {
    let width = area.width.saturating_sub(4).clamp(20, 60).min(area.width);
    // Never let the popup extend past the terminal: widgets index the buffer
    // directly and panic when asked to draw outside it.
    let max_height = area.height.saturating_sub(4).max(3).min(area.height);
    let height = lines.max(1).saturating_add(2).min(max_height as usize) as u16;
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn draw_info_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup = info_popup_area(area, app.info_lines.len());
    frame.render_widget(Clear, popup);

    let lines: Vec<Line> = app
        .info_lines
        .iter()
        .map(|(label, value)| {
            Line::from(vec![
                Span::styled(
                    format!("{label}: "),
                    Style::default().fg(app.theme.accent).bg(app.theme.bg),
                ),
                Span::styled(
                    clean(value),
                    Style::default().fg(app.theme.fg).bg(app.theme.bg),
                ),
            ])
        })
        .collect();
    let title = app
        .entries
        .get(app.selected)
        .map(|entry| clean(&entry.name))
        .unwrap_or_else(|| "info".into());
    let paragraph = Paragraph::new(lines).block(block(&format!("{title}  Tab close"), app.theme));
    frame.render_widget(paragraph, popup);
}

fn draw_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let lines: Vec<Line> = app
        .preview_lines
        .iter()
        .map(|line| Line::from(clean(line)))
        .collect();
    let title = app
        .entries
        .get(app.selected)
        .map(|entry| clean(&entry.name))
        .unwrap_or_else(|| "preview".into());
    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(app.theme.bg).fg(app.theme.fg))
        .block(block(&title, app.theme));
    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let info = match app.entries.get(app.selected) {
        Some(entry) => format!(
            "{}  {}",
            fs::mode_string(entry.mode, entry.is_dir, entry.is_symlink),
            if entry.is_dir {
                "-".to_string()
            } else {
                fs::format_size(entry.size)
            }
        ),
        None => String::new(),
    };
    let text = format!(
        " {}  |  {}  |  {} entries ",
        clean(&app.cwd.display().to_string()),
        info,
        app.entries.len()
    );
    let paragraph = Paragraph::new(text).style(
        Style::default()
            .bg(app.theme.status_bg)
            .fg(app.theme.status_fg),
    );
    frame.render_widget(paragraph, area);
}

fn draw_command(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.mode == Mode::Command {
        let text = format!("{}{}", app.prompt, clean(&app.input));
        frame.render_widget(
            Paragraph::new(text).style(Style::default().bg(app.theme.bg).fg(app.theme.fg)),
            area,
        );
        let offset = 1 + app.input.chars().count() as u16;
        if area.width > 1 && offset < area.width {
            frame.set_cursor_position((area.x + offset, area.y));
        }
    } else if !app.status.is_empty() {
        let paragraph = Paragraph::new(clean(&app.status))
            .style(Style::default().bg(app.theme.bg).fg(app.theme.warning));
        frame.render_widget(paragraph, area);
    } else {
        let paragraph =
            Paragraph::new(HINTS).style(Style::default().bg(app.theme.bg).fg(app.theme.muted));
        frame.render_widget(paragraph, area);
    }
}

fn draw_theme_picker(frame: &mut Frame, app: &App, state: &mut UiState) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).clamp(10, 34).min(area.width);
    let height = (theme::all().len() as u16 + 2)
        .min(area.height.saturating_sub(2).max(3))
        .min(area.height);
    let popup = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = theme::all()
        .iter()
        .map(|theme| {
            ListItem::new(Line::from(Span::styled(
                format!(" {}", theme.name),
                Style::default().fg(theme.accent).bg(theme.bg),
            )))
        })
        .collect();
    state.theme.select(Some(app.theme_choice));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(app.theme.bg).fg(app.theme.fg))
                .border_style(Style::default().fg(app.theme.accent))
                .title(Line::from(Span::styled(
                    " Themes  Enter save  Esc cancel ",
                    Style::default().fg(app.theme.accent),
                ))),
        )
        .style(Style::default().bg(app.theme.bg))
        .highlight_style(selection_style(app.theme))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, popup, &mut state.theme);
}

fn block(title: &str, theme: &Theme) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .border_style(Style::default().fg(theme.border))
        .title(Line::from(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme.accent),
        )))
}

fn selection_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.selected_bg)
        .fg(theme.selected_fg)
        .add_modifier(Modifier::BOLD)
}

fn entry_name(entry: &Entry) -> String {
    if entry.is_dir {
        format!("{}/", entry.name)
    } else {
        entry.name.clone()
    }
}

fn entry_style(entry: &Entry, theme: &Theme) -> Style {
    let style = if entry.is_symlink {
        Style::default().fg(theme.symlink)
    } else if entry.is_dir {
        Style::default().fg(theme.dir).add_modifier(Modifier::BOLD)
    } else if entry.mode & 0o111 != 0 {
        Style::default()
            .fg(theme.executable)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };
    style.bg(theme.bg)
}

/// Filenames are attacker-controlled input as far as the terminal is concerned.
fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_popup_never_exceeds_the_terminal() {
        for height in 0..=10 {
            let area = Rect::new(0, 0, 40, height);
            let popup = info_popup_area(area, 50);
            assert!(
                popup.bottom() <= area.bottom(),
                "popup {popup:?} exceeds area {area:?}"
            );
            assert!(popup.right() <= area.right());
        }
    }
}
