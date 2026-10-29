use crate::fs::{self, Entry};
use crate::theme::Theme;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// The selection marker plus its trailing space. Every row reserves this width
/// so names line up whether or not the row is selected.
pub const PREFIX_WIDTH: usize = 2;

/// Below this width the size column is dropped before names start truncating.
const SIZE_MIN_WIDTH: usize = 14;

/// `ls -F`-style suffix so file kind never depends on color alone.
pub fn marker(entry: &Entry) -> &'static str {
    if entry.is_symlink {
        "@"
    } else if entry.is_dir {
        "/"
    } else if entry.mode & 0o111 != 0 {
        "*"
    } else {
        ""
    }
}

pub fn display_name(entry: &Entry) -> String {
    format!("{}{}", clean(&entry.name), marker(entry))
}

pub fn size_label(entry: &Entry) -> String {
    if entry.is_dir {
        "-".into()
    } else {
        fs::format_size(entry.size)
    }
}

/// Widest size label in the directory, capped so one huge file cannot squeeze
/// every name out of the column.
pub fn size_column(entries: &[Entry]) -> usize {
    entries
        .iter()
        .map(|entry| size_label(entry).width())
        .max()
        .unwrap_or(0)
        .min(6)
}

/// Truncate to `width` display cells, using an ellipsis when text is cut.
pub fn truncate(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if text.width() <= width {
        return text.to_string();
    }
    let limit = width - 1;
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let cell = ch.width().unwrap_or(0);
        if used + cell > limit {
            break;
        }
        out.push(ch);
        used += cell;
    }
    out.push('…');
    out
}

/// Wrap on character boundaries. Used by the info panel, where paths have no
/// spaces to break on.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let cell = ch.width().unwrap_or(0);
        if used + cell > width && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            used = 0;
        }
        current.push(ch);
        used += cell;
    }
    lines.push(current);
    lines
}

pub fn selection_style(theme: &Theme) -> Style {
    Style::default()
        .bg(theme.selection_bg)
        .fg(theme.selection_fg)
}

fn name_style(entry: &Entry, theme: &Theme) -> Style {
    let color = if entry.is_symlink {
        theme.muted
    } else if entry.is_dir {
        theme.accent
    } else {
        theme.fg
    };
    Style::default().fg(color).bg(theme.bg)
}

/// Build one row: selection marker, name with type suffix, right-aligned size.
pub fn row(
    entry: &Entry,
    theme: &Theme,
    width: usize,
    size_width: usize,
    selected: bool,
) -> Line<'static> {
    if width == 0 {
        return Line::default();
    }
    if width < PREFIX_WIDTH {
        let symbol = if selected { ">" } else { " " };
        return Line::from(Span::styled(
            symbol,
            Style::default().fg(theme.accent).bg(theme.bg),
        ));
    }

    let prefix = if selected { "> " } else { "  " };
    let show_size = width >= SIZE_MIN_WIDTH && size_width > 0;
    let label = if show_size {
        size_label(entry)
    } else {
        String::new()
    };
    let label_width = label.width();
    let reserved = if show_size {
        size_width.max(label_width)
    } else {
        0
    };
    let gap = usize::from(show_size);
    let name_width = width.saturating_sub(PREFIX_WIDTH + reserved + gap);
    let name = truncate(&display_name(entry), name_width);
    let used = PREFIX_WIDTH + name.width();

    let mut spans = Vec::with_capacity(3);
    spans.push(Span::styled(
        prefix,
        Style::default()
            .fg(if selected { theme.accent } else { theme.muted })
            .bg(theme.bg),
    ));
    spans.push(Span::styled(name, name_style(entry, theme)));
    if show_size {
        let padding = width.saturating_sub(used + label_width);
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.push(Span::styled(
            label,
            Style::default().fg(theme.muted).bg(theme.bg),
        ));
    }
    Line::from(spans)
}

/// Filenames are attacker-controlled input as far as the terminal is concerned.
pub fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(name: &str, is_dir: bool, is_symlink: bool, mode: u32, size: u64) -> Entry {
        Entry {
            name: name.into(),
            path: PathBuf::from(name),
            is_dir,
            is_symlink,
            size,
            mode,
        }
    }

    #[test]
    fn markers_match_ls_f() {
        assert_eq!(marker(&entry("src", true, false, 0o755, 0)), "/");
        assert_eq!(marker(&entry("link", false, true, 0o777, 0)), "@");
        assert_eq!(marker(&entry("run", false, false, 0o755, 0)), "*");
        assert_eq!(marker(&entry("note.txt", false, false, 0o644, 0)), "");
        assert_eq!(display_name(&entry("src", true, false, 0o755, 0)), "src/");
    }

    #[test]
    fn directories_do_not_show_a_size() {
        assert_eq!(size_label(&entry("src", true, false, 0o755, 4096)), "-");
        assert_eq!(
            size_label(&entry("note.txt", false, false, 0o644, 1536)),
            "1.5K"
        );
    }

    #[test]
    fn truncation_respects_display_width() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("日本語", 4), "日…");
        assert_eq!(truncate("日本語", 6), "日本語");
        assert_eq!(truncate("abc", 0), "");
    }

    #[test]
    fn wrapped_text_never_exceeds_the_width() {
        for width in 1..12 {
            for line in wrap("some/very/long/path/to/a/file.txt", width) {
                assert!(line.width() <= width, "{line:?} wider than {width}");
            }
        }
    }

    #[test]
    fn rows_fill_exactly_the_requested_width() {
        let theme = crate::theme::default_theme();
        let file = entry("app.rs", false, false, 0o644, 2048);
        for width in 14..80 {
            for size_width in 4..=6 {
                let line = row(&file, theme, width, size_width, false);
                assert_eq!(line.width(), width, "width {width} size {size_width}");
            }
        }
    }

    #[test]
    fn rows_never_exceed_the_requested_width() {
        let theme = crate::theme::default_theme();
        let file = entry(
            "a-very-long-file-name.txt",
            false,
            false,
            0o644,
            123_456_789,
        );
        for width in 1..80 {
            for size_width in [0, 4, 6] {
                let line = row(&file, theme, width, size_width, false);
                assert!(
                    line.width() <= width,
                    "width {} exceeded by {} (reserve {size_width})",
                    width,
                    line.width()
                );
            }
        }
    }

    #[test]
    fn sizes_are_right_aligned_in_the_reserved_column() {
        let theme = crate::theme::default_theme();
        let file = entry("a.txt", false, false, 0o644, 12);
        let line = row(&file, theme, 20, 6, false);
        let text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(line.width(), 20);
        assert!(text.ends_with("12B"), "{text:?}");
    }

    #[test]
    fn narrow_rows_drop_the_size_column() {
        let theme = crate::theme::default_theme();
        let file = entry("app.rs", false, false, 0o644, 2048);
        let line = row(&file, theme, 12, 4, false);
        let text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert_eq!(text, "  app.rs");
        assert_eq!(line.width(), 8);
    }

    #[test]
    fn selection_marker_is_visible_without_color() {
        let theme = crate::theme::default_theme();
        let file = entry("app.rs", false, false, 0o644, 2048);
        let line = row(&file, theme, 20, 4, true);
        let text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.starts_with("> "));
    }
}
