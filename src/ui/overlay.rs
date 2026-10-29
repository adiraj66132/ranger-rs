use crate::app::{App, HELP_COMMANDS, HELP_KEYS};
use crate::theme::{self, Theme};
use crate::ui::row;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

pub fn draw_info(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = info_popup_width(area);
    let theme = app.theme;
    let inner_width = width.saturating_sub(4) as usize;
    let label_width = app
        .info_lines
        .iter()
        .map(|(label, _)| label.width())
        .max()
        .unwrap_or(4)
        .min(inner_width.saturating_sub(4).max(1));
    let value_width = inner_width.saturating_sub(label_width + 2).max(1);

    let mut lines = Vec::new();
    if app.info_lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no selection)",
            Style::default().fg(theme.muted),
        )));
    }
    for (label, value) in &app.info_lines {
        let chunks = row::wrap(&row::clean(value), value_width);
        for (index, chunk) in chunks.into_iter().enumerate() {
            let label_span = if index == 0 {
                Span::styled(
                    format!("{label:<label_width$}"),
                    Style::default().fg(theme.muted),
                )
            } else {
                Span::raw(" ".repeat(label_width))
            };
            lines.push(Line::from(vec![
                label_span,
                Span::raw("  "),
                Span::styled(chunk, Style::default().fg(theme.fg)),
            ]));
        }
    }

    let popup = info_popup_area(area, lines.len());
    if popup.width == 0 || popup.height == 0 {
        return;
    }
    frame.render_widget(Clear, popup);
    let paragraph = Paragraph::new(lines).block(panel("Info  Tab close", theme));
    frame.render_widget(paragraph, popup);
}

pub fn draw_theme_picker(frame: &mut Frame, app: &App, state: &mut ListState) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).clamp(24, 40).min(area.width);
    let height = (theme::all().len() as u16 + 2)
        .min(area.height.saturating_sub(2).max(3))
        .min(area.height);
    if width == 0 || height == 0 {
        return;
    }
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = theme::all()
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let marker = if index == app.theme_choice {
                "> "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(app.theme.accent)),
                Span::styled("  ", Style::default().bg(candidate.accent)),
                Span::styled("  ", Style::default().bg(candidate.surface_alt)),
                Span::raw(" "),
                Span::styled(candidate.name, Style::default().fg(app.theme.fg)),
            ]))
        })
        .collect();

    state.select(Some(app.theme_choice));
    let title = picker_title(
        theme::all().len(),
        height.saturating_sub(2) as usize,
        app.theme_choice,
    );
    let list = List::new(items)
        .block(panel(&title, app.theme))
        .style(Style::default().bg(app.theme.surface))
        .highlight_style(row::selection_style(app.theme));
    frame.render_stateful_widget(list, popup, state);
}

pub fn draw_help(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).clamp(28, 76).min(area.width);
    let height = area.height.saturating_sub(4).clamp(8, 32).min(area.height);
    if width == 0 || height == 0 {
        return;
    }
    let popup = centered(area, width, height);
    frame.render_widget(Clear, popup);

    let theme = app.theme;
    let key_width = HELP_KEYS
        .iter()
        .chain(HELP_COMMANDS.iter())
        .map(|(key, _)| key.width())
        .max()
        .unwrap_or(8)
        .min(18);

    let mut lines = Vec::new();
    lines.push(section("Keys", theme));
    for (key, action) in HELP_KEYS {
        lines.push(field(key, action, key_width, theme));
    }
    lines.push(Line::default());
    lines.push(section("Commands", theme));
    for (key, action) in HELP_COMMANDS {
        lines.push(field(key, action, key_width, theme));
    }

    let inner_height = popup.height.saturating_sub(2) as usize;
    let offset = app
        .help_scroll
        .min(lines.len().saturating_sub(inner_height));
    let paragraph = Paragraph::new(lines)
        .block(panel("Help  ? close", theme))
        .scroll((offset as u16, 0));
    frame.render_widget(paragraph, popup);
}

fn section(title: &str, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))
}

fn field(key: &str, action: &str, key_width: usize, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<key_width$}"), Style::default().fg(theme.fg)),
        Span::raw("  "),
        Span::styled(action.to_string(), Style::default().fg(theme.muted)),
    ])
}

/// Only advertise a position when the list cannot show every theme at once.
fn picker_title(count: usize, visible: usize, choice: usize) -> String {
    if count > visible {
        format!("Themes {}/{}  Enter save  Esc cancel", choice + 1, count)
    } else {
        "Themes  Enter save  Esc cancel".to_string()
    }
}

fn panel(title: &str, theme: &Theme) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.divider))
        .style(Style::default().bg(theme.surface).fg(theme.fg))
        .padding(Padding::new(1, 1, 0, 0))
        .title(Line::from(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme.fg).bg(theme.surface),
        )))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn info_popup_width(area: Rect) -> u16 {
    area.width.saturating_sub(4).clamp(20, 60).min(area.width)
}

fn info_popup_area(area: Rect, lines: usize) -> Rect {
    // Never let the popup extend past the terminal: widgets index the buffer
    // directly and panic when asked to draw outside it.
    let max_height = area.height.saturating_sub(4).max(3).min(area.height);
    let height = lines.max(1).saturating_add(2).min(max_height as usize) as u16;
    centered(area, info_popup_width(area), height)
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

    #[test]
    fn help_and_picker_popups_stay_inside_tiny_terminals() {
        for width in 0..=40 {
            for height in 0..=20 {
                let area = Rect::new(0, 0, width, height);
                let popup = centered(
                    area,
                    area.width.saturating_sub(4).clamp(24, 40).min(area.width),
                    area.height.saturating_sub(2).max(3).min(area.height),
                );
                assert!(popup.right() <= area.right(), "{popup:?} vs {area:?}");
                assert!(popup.bottom() <= area.bottom(), "{popup:?} vs {area:?}");
            }
        }
    }

    #[test]
    fn picker_title_only_shows_position_when_it_scrolls() {
        assert_eq!(picker_title(19, 30, 4), "Themes  Enter save  Esc cancel");
        assert_eq!(
            picker_title(19, 10, 4),
            "Themes 5/19  Enter save  Esc cancel"
        );
        assert_eq!(picker_title(19, 19, 0), "Themes  Enter save  Esc cancel");
    }
}
