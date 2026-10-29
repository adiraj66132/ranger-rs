use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub show_hidden: bool,
    pub preview_lines: usize,
    pub preview_bytes: usize,
    pub preview_media: bool,
    pub scope: Vec<ScopeRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScopeRule {
    #[serde(rename = "match")]
    pub pattern: String,
    pub command: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    1500
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: crate::theme::DEFAULT_NAME.into(),
            show_hidden: false,
            preview_lines: 200,
            preview_bytes: 64 * 1024,
            preview_media: true,
            scope: Vec::new(),
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Missing file means defaults. A bad file means defaults plus a warning.
    pub fn load() -> (Self, Vec<String>) {
        let Some(path) = Self::path() else {
            return (Self::default(), Vec::new());
        };
        if !path.exists() {
            return (Self::default(), Vec::new());
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(config) => (config, Vec::new()),
                Err(err) => (
                    Self::default(),
                    vec![format!("config {}: {err}", path.display())],
                ),
            },
            Err(err) => (
                Self::default(),
                vec![format!("config {}: {err}", path.display())],
            ),
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir).join("ranger-rs"));
        }
    }
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/ranger-rs"))
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

/// Persist a theme without discarding comments or the rest of the config.
pub fn save_theme(name: &str) -> Result<(), String> {
    let Some(path) = Config::path() else {
        return Err("no config directory".into());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    let existing = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(format!("{}: {err}", path.display())),
    };
    std::fs::write(&path, update_theme_text(&existing, name))
        .map_err(|err| format!("{}: {err}", path.display()))
}

fn update_theme_text(existing: &str, name: &str) -> String {
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();
    let assignment = format!("theme = \"{name}\"");
    let mut replaced = false;
    for line in &mut lines {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("theme") {
            if rest.trim_start().starts_with('=') {
                *line = assignment.clone();
                replaced = true;
                break;
            }
        }
    }
    if !replaced {
        lines.insert(0, assignment);
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

pub fn plugins_dir() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("plugins"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_update_preserves_the_rest_of_the_config() {
        let existing =
            "# my config\ntheme = \"nord\"\nshow_hidden = true\n\n[[scope]]\nmatch = \"*.pdf\"\n";
        let updated = update_theme_text(existing, "dracula");
        assert!(updated.starts_with("# my config"));
        assert!(updated.contains("theme = \"dracula\""));
        assert!(!updated.contains("nord"));
        assert!(updated.contains("show_hidden = true"));
        assert!(updated.contains("[[scope]]"));
        assert_eq!(updated.matches("theme =").count(), 1);
    }

    #[test]
    fn theme_update_adds_missing_key() {
        let updated = update_theme_text("show_hidden = true\n", "gruvbox");
        assert_eq!(updated, "theme = \"gruvbox\"\nshow_hidden = true\n");
    }
}
