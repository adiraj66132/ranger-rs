use ratatui::style::Color;

/// All theme colors are resolved once at startup. Rendering only reads `Style`s
/// built from these constants, so themes cost nothing per frame.
#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub fg: Color,
    pub muted: Color,
    pub divider: Color,
    pub accent: Color,
    pub selection_fg: Color,
    pub selection_bg: Color,
    pub danger: Color,
    pub warning: Color,
    pub success: Color,
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
    surface: Color::Reset,
    surface_alt: Color::DarkGray,
    fg: Color::Reset,
    muted: Color::DarkGray,
    divider: Color::DarkGray,
    accent: Color::Blue,
    selection_fg: Color::White,
    selection_bg: Color::DarkGray,
    danger: Color::Red,
    warning: Color::Yellow,
    success: Color::Green,
};

static CATPPUCCIN_MOCHA: Theme = Theme {
    name: "catppuccin-mocha",
    bg: rgb(0x1e1e2e),
    surface: rgb(0x181825),
    surface_alt: rgb(0x313244),
    fg: rgb(0xcdd6f4),
    muted: rgb(0x7f849c),
    divider: rgb(0x45475a),
    accent: rgb(0x89b4fa),
    selection_fg: rgb(0xcdd6f4),
    selection_bg: rgb(0x313244),
    danger: rgb(0xf38ba8),
    warning: rgb(0xf9e2af),
    success: rgb(0xa6e3a1),
};

static CATPPUCCIN_MACCHIATO: Theme = Theme {
    name: "catppuccin-macchiato",
    bg: rgb(0x24273a),
    surface: rgb(0x1e2030),
    surface_alt: rgb(0x363a4f),
    fg: rgb(0xcad3f5),
    muted: rgb(0x8087a2),
    divider: rgb(0x494d64),
    accent: rgb(0x8aadf4),
    selection_fg: rgb(0xcad3f5),
    selection_bg: rgb(0x363a4f),
    danger: rgb(0xed8796),
    warning: rgb(0xeed49f),
    success: rgb(0xa6da95),
};

static CATPPUCCIN_FRAPPE: Theme = Theme {
    name: "catppuccin-frappe",
    bg: rgb(0x303446),
    surface: rgb(0x292c3c),
    surface_alt: rgb(0x414559),
    fg: rgb(0xc6d0f5),
    muted: rgb(0x838ba7),
    divider: rgb(0x51576d),
    accent: rgb(0x8caaee),
    selection_fg: rgb(0xc6d0f5),
    selection_bg: rgb(0x414559),
    danger: rgb(0xe78284),
    warning: rgb(0xe5c890),
    success: rgb(0xa6d189),
};

static CATPPUCCIN_LATTE: Theme = Theme {
    name: "catppuccin-latte",
    bg: rgb(0xeff1f5),
    surface: rgb(0xe6e9ef),
    surface_alt: rgb(0xccd0da),
    fg: rgb(0x4c4f69),
    muted: rgb(0x8c8fa1),
    divider: rgb(0xbcc0cc),
    accent: rgb(0x1e66f5),
    selection_fg: rgb(0x4c4f69),
    selection_bg: rgb(0xccd0da),
    danger: rgb(0xd20f39),
    warning: rgb(0xdf8e1d),
    success: rgb(0x40a02b),
};

static TOKYO_NIGHT: Theme = Theme {
    name: "tokyo-night",
    bg: rgb(0x1a1b26),
    surface: rgb(0x16161e),
    surface_alt: rgb(0x292e42),
    fg: rgb(0xc0caf5),
    muted: rgb(0x565f89),
    divider: rgb(0x3b4261),
    accent: rgb(0x7aa2f7),
    selection_fg: rgb(0xc0caf5),
    selection_bg: rgb(0x292e42),
    danger: rgb(0xf7768e),
    warning: rgb(0xe0af68),
    success: rgb(0x9ece6a),
};

static TOKYO_NIGHT_STORM: Theme = Theme {
    name: "tokyo-night-storm",
    bg: rgb(0x24283b),
    surface: rgb(0x1f2335),
    surface_alt: rgb(0x292e42),
    fg: rgb(0xc0caf5),
    muted: rgb(0x565f89),
    divider: rgb(0x414868),
    accent: rgb(0x7aa2f7),
    selection_fg: rgb(0xc0caf5),
    selection_bg: rgb(0x292e42),
    danger: rgb(0xf7768e),
    warning: rgb(0xe0af68),
    success: rgb(0x9ece6a),
};

static TOKYO_NIGHT_DAY: Theme = Theme {
    name: "tokyo-night-day",
    bg: rgb(0xe1e2e7),
    surface: rgb(0xd8dae0),
    surface_alt: rgb(0xc4c8da),
    fg: rgb(0x3760bf),
    muted: rgb(0x8990b3),
    divider: rgb(0xb4b8cc),
    accent: rgb(0x2e7de9),
    selection_fg: rgb(0x3760bf),
    selection_bg: rgb(0xc4c8da),
    danger: rgb(0xf52a65),
    warning: rgb(0xb15c00),
    success: rgb(0x587539),
};

static NORD: Theme = Theme {
    name: "nord",
    bg: rgb(0x2e3440),
    surface: rgb(0x292e39),
    surface_alt: rgb(0x3b4252),
    fg: rgb(0xd8dee9),
    muted: rgb(0x616e88),
    divider: rgb(0x434c5e),
    accent: rgb(0x88c0d0),
    selection_fg: rgb(0xd8dee9),
    selection_bg: rgb(0x3b4252),
    danger: rgb(0xbf616a),
    warning: rgb(0xebcb8b),
    success: rgb(0xa3be8c),
};

static DRACULA: Theme = Theme {
    name: "dracula",
    bg: rgb(0x282a36),
    surface: rgb(0x21222c),
    surface_alt: rgb(0x44475a),
    fg: rgb(0xf8f8f2),
    muted: rgb(0x6272a4),
    divider: rgb(0x44475a),
    accent: rgb(0xbd93f9),
    selection_fg: rgb(0xf8f8f2),
    selection_bg: rgb(0x44475a),
    danger: rgb(0xff5555),
    warning: rgb(0xffb86c),
    success: rgb(0x50fa7b),
};

static GRUVBOX: Theme = Theme {
    name: "gruvbox",
    bg: rgb(0x282828),
    surface: rgb(0x1d2021),
    surface_alt: rgb(0x3c3836),
    fg: rgb(0xebdbb2),
    muted: rgb(0x928374),
    divider: rgb(0x3c3836),
    accent: rgb(0x83a598),
    selection_fg: rgb(0xebdbb2),
    selection_bg: rgb(0x3c3836),
    danger: rgb(0xfb4934),
    warning: rgb(0xfabd2f),
    success: rgb(0xb8bb26),
};

static ROSE_PINE: Theme = Theme {
    name: "rose-pine",
    bg: rgb(0x191724),
    surface: rgb(0x1f1d2e),
    surface_alt: rgb(0x26233a),
    fg: rgb(0xe0def4),
    muted: rgb(0x6e6a86),
    divider: rgb(0x403d52),
    accent: rgb(0xc4a7e7),
    selection_fg: rgb(0xe0def4),
    selection_bg: rgb(0x26233a),
    danger: rgb(0xeb6f92),
    warning: rgb(0xf6c177),
    success: rgb(0x9ccfd8),
};

static ROSE_PINE_MOON: Theme = Theme {
    name: "rose-pine-moon",
    bg: rgb(0x232136),
    surface: rgb(0x2a273f),
    surface_alt: rgb(0x393552),
    fg: rgb(0xe0def4),
    muted: rgb(0x6e6a86),
    divider: rgb(0x44415a),
    accent: rgb(0xc4a7e7),
    selection_fg: rgb(0xe0def4),
    selection_bg: rgb(0x393552),
    danger: rgb(0xeb6f92),
    warning: rgb(0xf6c177),
    success: rgb(0x9ccfd8),
};

static ROSE_PINE_DAWN: Theme = Theme {
    name: "rose-pine-dawn",
    bg: rgb(0xfaf4ed),
    surface: rgb(0xfffaf3),
    surface_alt: rgb(0xf2e9e1),
    fg: rgb(0x575279),
    muted: rgb(0x9893a5),
    divider: rgb(0xdfdad9),
    accent: rgb(0x907aa9),
    selection_fg: rgb(0x575279),
    selection_bg: rgb(0xf2e9e1),
    danger: rgb(0xb4637a),
    warning: rgb(0xea9d34),
    success: rgb(0x56949f),
};

static SOLARIZED_DARK: Theme = Theme {
    name: "solarized-dark",
    bg: rgb(0x002b36),
    surface: rgb(0x073642),
    surface_alt: rgb(0x0e4b56),
    fg: rgb(0x93a1a1),
    muted: rgb(0x586e75),
    divider: rgb(0x12505c),
    accent: rgb(0x268bd2),
    selection_fg: rgb(0x93a1a1),
    selection_bg: rgb(0x0e4b56),
    danger: rgb(0xdc322f),
    warning: rgb(0xb58900),
    success: rgb(0x859900),
};

static SOLARIZED_LIGHT: Theme = Theme {
    name: "solarized-light",
    bg: rgb(0xfdf6e3),
    surface: rgb(0xeee8d5),
    surface_alt: rgb(0xe4ddc8),
    fg: rgb(0x586e75),
    muted: rgb(0x93a1a1),
    divider: rgb(0xd8d2c0),
    accent: rgb(0x268bd2),
    selection_fg: rgb(0x586e75),
    selection_bg: rgb(0xe4ddc8),
    danger: rgb(0xdc322f),
    warning: rgb(0xb58900),
    success: rgb(0x859900),
};

static KANAGAWA: Theme = Theme {
    name: "kanagawa",
    bg: rgb(0x1f1f28),
    surface: rgb(0x16161d),
    surface_alt: rgb(0x2a2a37),
    fg: rgb(0xdcd7ba),
    muted: rgb(0x727169),
    divider: rgb(0x363646),
    accent: rgb(0x7e9cd8),
    selection_fg: rgb(0xdcd7ba),
    selection_bg: rgb(0x2a2a37),
    danger: rgb(0xe46876),
    warning: rgb(0xc0a36e),
    success: rgb(0x98bb6c),
};

static EVERFOREST: Theme = Theme {
    name: "everforest",
    bg: rgb(0x2d353b),
    surface: rgb(0x232a2e),
    surface_alt: rgb(0x3d484d),
    fg: rgb(0xd3c6aa),
    muted: rgb(0x859289),
    divider: rgb(0x475258),
    accent: rgb(0xa7c080),
    selection_fg: rgb(0xd3c6aa),
    selection_bg: rgb(0x3d484d),
    danger: rgb(0xe67e80),
    warning: rgb(0xdbbc7f),
    success: rgb(0x83c092),
};

static ONE_DARK: Theme = Theme {
    name: "one-dark",
    bg: rgb(0x282c34),
    surface: rgb(0x21252b),
    surface_alt: rgb(0x3e4451),
    fg: rgb(0xabb2bf),
    muted: rgb(0x5c6370),
    divider: rgb(0x3e4451),
    accent: rgb(0x61afef),
    selection_fg: rgb(0xabb2bf),
    selection_bg: rgb(0x3e4451),
    danger: rgb(0xe06c75),
    warning: rgb(0xe5c07b),
    success: rgb(0x98c379),
};

static THEMES: &[&Theme] = &[
    &CATPPUCCIN_MOCHA,
    &CATPPUCCIN_MACCHIATO,
    &CATPPUCCIN_FRAPPE,
    &CATPPUCCIN_LATTE,
    &TOKYO_NIGHT,
    &TOKYO_NIGHT_STORM,
    &TOKYO_NIGHT_DAY,
    &ROSE_PINE,
    &ROSE_PINE_MOON,
    &ROSE_PINE_DAWN,
    &SOLARIZED_DARK,
    &SOLARIZED_LIGHT,
    &NORD,
    &DRACULA,
    &GRUVBOX,
    &KANAGAWA,
    &EVERFOREST,
    &ONE_DARK,
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
    if name.eq_ignore_ascii_case("solarized") {
        return Some(&SOLARIZED_DARK);
    }
    if name.eq_ignore_ascii_case("onedark") {
        return Some(&ONE_DARK);
    }
    if name.eq_ignore_ascii_case("rosepine") {
        return Some(&ROSE_PINE);
    }
    if name.eq_ignore_ascii_case("everforest-dark") {
        return Some(&EVERFOREST);
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
        assert_eq!(
            get("solarized").map(|theme| theme.name),
            Some("solarized-dark")
        );
        assert_eq!(get("onedark").map(|theme| theme.name), Some("one-dark"));
        assert_eq!(get("rosepine").map(|theme| theme.name), Some("rose-pine"));
        assert_eq!(
            get("everforest-dark").map(|theme| theme.name),
            Some("everforest")
        );
        assert!(get("nope").is_none());
    }

    #[test]
    fn names_are_unique() {
        let mut seen = Vec::new();
        for theme in all() {
            assert!(
                !seen.contains(&theme.name),
                "duplicate theme: {}",
                theme.name
            );
            seen.push(theme.name);
        }
    }

    /// Relative luminance from the WCAG definition.
    fn luminance(color: Color) -> Option<f64> {
        let Color::Rgb(r, g, b) = color else {
            return None;
        };
        let channel = |value: u8| {
            let value = value as f64 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        Some(0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b))
    }

    fn contrast(a: Color, b: Color) -> Option<f64> {
        let (a, b) = (luminance(a)?, luminance(b)?);
        let (light, dark) = if a > b { (a, b) } else { (b, a) };
        Some((light + 0.05) / (dark + 0.05))
    }

    #[test]
    fn palettes_stay_legible() {
        for theme in all() {
            // `default` rides on the terminal's own ANSI colours, so there is
            // nothing to measure.
            let Some(body) = contrast(theme.fg, theme.bg) else {
                continue;
            };
            assert!(body >= 4.5, "{}: fg on bg is {body:.2}", theme.name);

            let accent = contrast(theme.accent, theme.bg).unwrap_or(0.0);
            assert!(accent >= 3.0, "{}: accent on bg is {accent:.2}", theme.name);
        }
    }
}
