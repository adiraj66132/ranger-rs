use ratatui::style::Color;

/// All theme colors are resolved once at startup. Rendering only reads `Style`s
/// built from these constants, so themes cost nothing per frame.
#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub dir: Color,
    pub symlink: Color,
    pub executable: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub status_fg: Color,
    pub status_bg: Color,
    pub border: Color,
    pub accent: Color,
    pub muted: Color,
    pub warning: Color,
}

pub const DEFAULT_NAME: &str = "catppuccin-mocha";

const fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

static DEFAULT: Theme = Theme {
    name: "default",
    bg: Color::Reset,
    fg: Color::Reset,
    dir: Color::Blue,
    symlink: Color::Cyan,
    executable: Color::Green,
    selected_fg: Color::White,
    selected_bg: Color::DarkGray,
    status_fg: Color::White,
    status_bg: Color::Blue,
    border: Color::DarkGray,
    accent: Color::Blue,
    muted: Color::DarkGray,
    warning: Color::Yellow,
};

static CATPPUCCIN_MOCHA: Theme = Theme {
    name: "catppuccin-mocha",
    bg: rgb(0x1e1e2e),
    fg: rgb(0xcdd6f4),
    dir: rgb(0x89b4fa),
    symlink: rgb(0x74c7ec),
    executable: rgb(0xa6e3a1),
    selected_fg: rgb(0x1e1e2e),
    selected_bg: rgb(0x89b4fa),
    status_fg: rgb(0x1e1e2e),
    status_bg: rgb(0x89b4fa),
    border: rgb(0x45475a),
    accent: rgb(0xb4befe),
    muted: rgb(0x6c7086),
    warning: rgb(0xf9e2af),
};

static CATPPUCCIN_MACCHIATO: Theme = Theme {
    name: "catppuccin-macchiato",
    bg: rgb(0x24273a),
    fg: rgb(0xcad3f5),
    dir: rgb(0x8aadf4),
    symlink: rgb(0x7dc4e4),
    executable: rgb(0xa6da95),
    selected_fg: rgb(0x24273a),
    selected_bg: rgb(0x8aadf4),
    status_fg: rgb(0x24273a),
    status_bg: rgb(0x8aadf4),
    border: rgb(0x494d64),
    accent: rgb(0xb7bdf8),
    muted: rgb(0x6e738d),
    warning: rgb(0xf5a97f),
};

static CATPPUCCIN_FRAPPE: Theme = Theme {
    name: "catppuccin-frappe",
    bg: rgb(0x303446),
    fg: rgb(0xc6d0f5),
    dir: rgb(0x8caaee),
    symlink: rgb(0x85c1dc),
    executable: rgb(0xa6d189),
    selected_fg: rgb(0x303446),
    selected_bg: rgb(0x8caaee),
    status_fg: rgb(0x303446),
    status_bg: rgb(0x8caaee),
    border: rgb(0x51576d),
    accent: rgb(0xbabbf1),
    muted: rgb(0x737994),
    warning: rgb(0xef9f76),
};

static CATPPUCCIN_LATTE: Theme = Theme {
    name: "catppuccin-latte",
    bg: rgb(0xeff1f5),
    fg: rgb(0x4c4f69),
    dir: rgb(0x1e66f5),
    symlink: rgb(0x209fb5),
    executable: rgb(0x40a02b),
    selected_fg: rgb(0xeff1f5),
    selected_bg: rgb(0x1e66f5),
    status_fg: rgb(0xeff1f5),
    status_bg: rgb(0x1e66f5),
    border: rgb(0xccd0da),
    accent: rgb(0x7287fd),
    muted: rgb(0x8c8fa1),
    warning: rgb(0xdf8e1d),
};

static TOKYO_NIGHT: Theme = Theme {
    name: "tokyo-night",
    bg: rgb(0x1a1b26),
    fg: rgb(0xc0caf5),
    dir: rgb(0x7aa2f7),
    symlink: rgb(0x7dcfff),
    executable: rgb(0x9ece6a),
    selected_fg: rgb(0x1a1b26),
    selected_bg: rgb(0x7aa2f7),
    status_fg: rgb(0x1a1b26),
    status_bg: rgb(0x7aa2f7),
    border: rgb(0x3b4261),
    accent: rgb(0xbb9af7),
    muted: rgb(0x565f89),
    warning: rgb(0xe0af68),
};

static TOKYO_NIGHT_STORM: Theme = Theme {
    name: "tokyo-night-storm",
    bg: rgb(0x24283b),
    fg: rgb(0xc0caf5),
    dir: rgb(0x7aa2f7),
    symlink: rgb(0x7dcfff),
    executable: rgb(0x9ece6a),
    selected_fg: rgb(0x24283b),
    selected_bg: rgb(0x7aa2f7),
    status_fg: rgb(0x24283b),
    status_bg: rgb(0x7aa2f7),
    border: rgb(0x414868),
    accent: rgb(0xbb9af7),
    muted: rgb(0x565f89),
    warning: rgb(0xe0af68),
};

static TOKYO_NIGHT_DAY: Theme = Theme {
    name: "tokyo-night-day",
    bg: rgb(0xe1e2e7),
    fg: rgb(0x3760bf),
    dir: rgb(0x2e7de9),
    symlink: rgb(0x007197),
    executable: rgb(0x587539),
    selected_fg: rgb(0xe1e2e7),
    selected_bg: rgb(0x2e7de9),
    status_fg: rgb(0xe1e2e7),
    status_bg: rgb(0x2e7de9),
    border: rgb(0xc4c8da),
    accent: rgb(0x9854f1),
    muted: rgb(0x8990b3),
    warning: rgb(0xb15c00),
};

static NORD: Theme = Theme {
    name: "nord",
    bg: rgb(0x2e3440),
    fg: rgb(0xd8dee9),
    dir: rgb(0x88c0d0),
    symlink: rgb(0x8fbcbb),
    executable: rgb(0xa3be8c),
    selected_fg: rgb(0x2e3440),
    selected_bg: rgb(0x88c0d0),
    status_fg: rgb(0x2e3440),
    status_bg: rgb(0x88c0d0),
    border: rgb(0x434c5e),
    accent: rgb(0xb48ead),
    muted: rgb(0x4c566a),
    warning: rgb(0xebcb8b),
};

static DRACULA: Theme = Theme {
    name: "dracula",
    bg: rgb(0x282a36),
    fg: rgb(0xf8f8f2),
    dir: rgb(0x8be9fd),
    symlink: rgb(0xff79c6),
    executable: rgb(0x50fa7b),
    selected_fg: rgb(0x282a36),
    selected_bg: rgb(0xbd93f9),
    status_fg: rgb(0x282a36),
    status_bg: rgb(0xbd93f9),
    border: rgb(0x44475a),
    accent: rgb(0xff79c6),
    muted: rgb(0x6272a4),
    warning: rgb(0xffb86c),
};

static GRUVBOX: Theme = Theme {
    name: "gruvbox",
    bg: rgb(0x282828),
    fg: rgb(0xebdbb2),
    dir: rgb(0x83a598),
    symlink: rgb(0x8ec07c),
    executable: rgb(0xb8bb26),
    selected_fg: rgb(0x282828),
    selected_bg: rgb(0x83a598),
    status_fg: rgb(0x282828),
    status_bg: rgb(0x83a598),
    border: rgb(0x3c3836),
    accent: rgb(0xfabd2f),
    muted: rgb(0x928374),
    warning: rgb(0xfe8019),
};

static THEMES: &[&Theme] = &[
    &CATPPUCCIN_MOCHA,
    &CATPPUCCIN_MACCHIATO,
    &CATPPUCCIN_FRAPPE,
    &CATPPUCCIN_LATTE,
    &TOKYO_NIGHT,
    &TOKYO_NIGHT_STORM,
    &TOKYO_NIGHT_DAY,
    &NORD,
    &DRACULA,
    &GRUVBOX,
    &DEFAULT,
];

pub fn get(name: &str) -> Option<&'static Theme> {
    let name = name.trim();
    if let Some(theme) = THEMES
        .iter()
        .copied()
        .find(|theme| theme.name.eq_ignore_ascii_case(name))
    {
        return Some(theme);
    }
    if name.eq_ignore_ascii_case("catppuccin") {
        return Some(&CATPPUCCIN_MOCHA);
    }
    if name.eq_ignore_ascii_case("tokyo") || name.eq_ignore_ascii_case("tokyonight") {
        return Some(&TOKYO_NIGHT);
    }
    None
}

pub fn default_theme() -> &'static Theme {
    &CATPPUCCIN_MOCHA
}

pub fn all() -> &'static [&'static Theme] {
    THEMES
}

pub fn names() -> Vec<&'static str> {
    THEMES.iter().map(|theme| theme.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_name_resolves() {
        for name in names() {
            assert!(get(name).is_some(), "{name}");
        }
        assert_eq!(
            get("catppuccin").map(|theme| theme.name),
            Some("catppuccin-mocha")
        );
        assert_eq!(get("Tokyo").map(|theme| theme.name), Some("tokyo-night"));
        assert!(get("nope").is_none());
    }
}
