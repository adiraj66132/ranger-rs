use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub show_hidden: bool,
    pub preview_lines: usize,
    pub preview_bytes: usize,
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
            show_hidden: false,
            preview_lines: 200,
            preview_bytes: 64 * 1024,
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

pub fn plugins_dir() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("plugins"))
}
