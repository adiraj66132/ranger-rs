use crate::config::{self, Config};
use crate::editor;
use crate::fs::{self, Entry};
use crate::info;
use crate::plugin::{Action, PluginHost};
use crate::preview::{self, PreviewMessage};
use crate::theme::{self, Theme};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    ThemePicker,
    Help,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Status {
    pub text: String,
    pub kind: StatusKind,
    pub shown_at: Instant,
}

impl Status {
    fn empty() -> Self {
        Self {
            text: String::new(),
            kind: StatusKind::Info,
            shown_at: Instant::now(),
        }
    }

    fn visible(&self) -> bool {
        !self.text.is_empty() && self.shown_at.elapsed() < STATUS_TTL
    }
}

pub const STATUS_TTL: Duration = Duration::from_secs(4);

pub struct EditorLaunch {
    pub editor: editor::Editor,
    pub path: PathBuf,
}

pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub parent_entries: Vec<Entry>,
    pub parent_selected: Option<usize>,
    pub preview_lines: Vec<String>,
    pub show_info: bool,
    pub info_lines: Vec<(String, String)>,
    pub mode: Mode,
    pub input: String,
    pub prompt: char,
    pub status: Status,
    pub help_scroll: usize,
    pub config: Config,
    pub theme: &'static Theme,
    pub theme_choice: usize,
    theme_original: Option<&'static Theme>,
    editor: Option<editor::Editor>,
    editor_checked: bool,
    editor_launch: Option<EditorLaunch>,
    pub should_quit: bool,
    preview_generation: u64,
    preview_tx: Sender<PreviewMessage>,
    preview_rx: Receiver<PreviewMessage>,
    actions: Arc<Mutex<Vec<Action>>>,
    shared_cwd: Arc<Mutex<PathBuf>>,
    plugin: Option<PluginHost>,
}

impl App {
    pub fn new(cwd: PathBuf, config: Config) -> Self {
        let (preview_tx, preview_rx) = mpsc::channel();
        let actions = Arc::new(Mutex::new(Vec::new()));
        let shared_cwd = Arc::new(Mutex::new(cwd.clone()));
        let (plugin, mut warnings) = match config::plugins_dir() {
            Some(dir) => PluginHost::load(&dir, actions.clone(), shared_cwd.clone()),
            None => (None, Vec::new()),
        };
        let theme = match theme::get(&config.theme) {
            Some(theme) => theme,
            None => {
                warnings.push(format!(
                    "unknown theme '{}', using {}",
                    config.theme,
                    theme::DEFAULT_NAME
                ));
                theme::default_theme()
            }
        };

        let mut app = Self {
            cwd,
            entries: Vec::new(),
            selected: 0,
            parent_entries: Vec::new(),
            parent_selected: None,
            preview_lines: Vec::new(),
            show_info: false,
            info_lines: Vec::new(),
            mode: Mode::Normal,
            input: String::new(),
            prompt: ':',
            status: Status::empty(),
            help_scroll: 0,
            config,
            theme,
            theme_choice: 0,
            theme_original: None,
            editor: None,
            editor_checked: false,
            editor_launch: None,
            should_quit: false,
            preview_generation: 0,
            preview_tx,
            preview_rx,
            actions,
            shared_cwd,
            plugin,
        };
        app.refresh(None);
        for warning in warnings {
            app.add_warning(warning);
        }
        app
    }

    pub fn set_status(&mut self, text: impl Into<String>) {
        self.status.text = text.into();
        self.status.kind = StatusKind::Info;
        self.status.shown_at = Instant::now();
    }

    pub fn set_success(&mut self, text: impl Into<String>) {
        self.status.text = text.into();
        self.status.kind = StatusKind::Success;
        self.status.shown_at = Instant::now();
    }

    pub fn set_warning(&mut self, text: impl Into<String>) {
        self.status.text = text.into();
        self.status.kind = StatusKind::Warning;
        self.status.shown_at = Instant::now();
    }

    pub fn set_error(&mut self, text: impl Into<String>) {
        self.status.text = text.into();
        self.status.kind = StatusKind::Error;
        self.status.shown_at = Instant::now();
    }

    pub fn add_warning(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.status.text.is_empty() {
            self.set_warning(text);
        } else {
            self.status.text = format!("{} | {text}", self.status.text);
            self.status.kind = StatusKind::Warning;
            self.status.shown_at = Instant::now();
        }
    }

    pub fn clear_status(&mut self) {
        self.status.text.clear();
    }

    /// Expire transient messages so the key hints come back on their own.
    pub fn tick(&mut self) {
        if !self.status.text.is_empty() && !self.status.visible() {
            self.clear_status();
        }
    }

    pub fn refresh(&mut self, keep: Option<&str>) {
        let previous = keep.map(str::to_string).or_else(|| {
            self.entries
                .get(self.selected)
                .map(|entry| entry.name.clone())
        });

        match fs::read_dir(&self.cwd, self.config.show_hidden) {
            Ok(entries) => {
                self.entries = entries;
                if let Some(name) = previous {
                    if let Some(index) = self.entries.iter().position(|entry| entry.name == name) {
                        self.selected = index;
                    }
                }
                if self.selected >= self.entries.len() {
                    self.selected = self.entries.len().saturating_sub(1);
                }
            }
            Err(err) => {
                self.entries.clear();
                self.selected = 0;
                self.set_error(format!("{}: {err}", self.cwd.display()));
            }
        }
        self.refresh_parent();
        self.request_preview();
        self.refresh_info();
    }

    fn refresh_parent(&mut self) {
        self.parent_selected = None;
        self.parent_entries.clear();
        let Some(parent) = self.cwd.parent() else {
            return;
        };
        if let Ok(entries) = fs::read_dir(parent, self.config.show_hidden) {
            self.parent_selected = entries.iter().position(|entry| entry.path == self.cwd);
            self.parent_entries = entries;
        }
    }

    fn request_preview(&mut self) {
        self.preview_generation = self.preview_generation.wrapping_add(1);
        match self.entries.get(self.selected) {
            Some(entry) => {
                self.preview_lines = vec!["Loading...".into()];
                preview::request(
                    self.preview_generation,
                    entry.path.clone(),
                    self.config.clone(),
                    self.preview_tx.clone(),
                );
            }
            None => self.preview_lines = vec!["(empty directory)".into()],
        }
    }

    pub fn drain_preview(&mut self) {
        while let Ok(message) = self.preview_rx.try_recv() {
            if message.generation == self.preview_generation {
                self.preview_lines = message.lines;
            }
        }
    }

    fn refresh_info(&mut self) {
        if !self.show_info {
            return;
        }
        self.info_lines = match self.entries.get(self.selected) {
            Some(entry) => info::describe(entry),
            None => Vec::new(),
        };
    }

    fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
        if self.show_info {
            self.refresh_info();
        }
    }

    fn sync_cwd(&self) {
        *self.shared_cwd.lock().unwrap() = self.cwd.clone();
    }

    fn selection(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    fn select(&mut self, index: usize) {
        if self.entries.is_empty() {
            return;
        }
        let index = index.min(self.entries.len() - 1);
        if index != self.selected {
            self.selected = index;
            self.request_preview();
            self.refresh_info();
        }
    }

    fn move_down(&mut self, amount: usize) {
        self.select(self.selected.saturating_add(amount));
    }

    fn move_up(&mut self, amount: usize) {
        self.select(self.selected.saturating_sub(amount));
    }

    fn go_top(&mut self) {
        self.select(0);
    }

    fn go_bottom(&mut self) {
        self.select(self.entries.len().saturating_sub(1));
    }

    fn change_dir(&mut self, path: PathBuf) {
        let path = if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        };
        match std::fs::canonicalize(&path) {
            Ok(path) if path.is_dir() => {
                self.cwd = path;
                self.entries.clear();
                self.selected = 0;
                self.clear_status();
                self.sync_cwd();
                self.refresh(None);
            }
            Ok(_) => self.set_error(format!("not a directory: {}", path.display())),
            Err(err) => self.set_error(format!("{}: {err}", path.display())),
        }
    }

    fn go_parent(&mut self) {
        let Some(parent) = self.cwd.parent().map(PathBuf::from) else {
            return;
        };
        let previous = self
            .cwd
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        self.cwd = parent;
        self.selected = 0;
        self.clear_status();
        self.sync_cwd();
        self.refresh(previous.as_deref());
    }

    fn toggle_hidden(&mut self) {
        self.config.show_hidden = !self.config.show_hidden;
        self.set_status(if self.config.show_hidden {
            "hidden files: on"
        } else {
            "hidden files: off"
        });
        self.refresh(None);
    }

    fn open_selected(&mut self) {
        let Some(entry) = self.selection().cloned() else {
            return;
        };
        if entry.is_dir {
            self.change_dir(entry.path);
        } else if preview::looks_text(&entry.path) {
            self.open_in_editor(entry.path);
        } else {
            self.open_path(&entry.path);
        }
    }

    fn open_in_editor(&mut self, path: PathBuf) {
        if !self.editor_checked {
            self.editor = editor::detect();
            self.editor_checked = true;
        }
        let Some(editor) = self.editor else {
            self.set_error("No editor found. Install vim or nvim (or nano).");
            return;
        };
        self.editor_launch = Some(EditorLaunch { editor, path });
    }

    pub fn take_editor_launch(&mut self) -> Option<EditorLaunch> {
        self.editor_launch.take()
    }

    pub fn editor_finished(&mut self, editor: editor::Editor, error: Option<String>) {
        match error {
            Some(err) => self.set_error(format!("{}: {err}", editor.program)),
            None if editor.is_fallback() => self.set_warning(format!(
                "Opened with {}. Install vim or nvim for the native experience.",
                editor.display_name()
            )),
            None => self.set_success(format!("Opened with {}", editor.display_name())),
        }
    }

    fn open_path(&mut self, path: &std::path::Path) {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        match Command::new(opener)
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => self.set_success(format!("opened {}", path.display())),
            Err(err) => self.set_error(format!("{opener}: {err}")),
        }
    }

    fn resolve(&self, arg: &str) -> PathBuf {
        let path = config::expand_tilde(arg);
        if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        }
    }

    fn make_dir(&mut self, arg: &str) {
        if arg.is_empty() {
            self.set_warning("usage: :mkdir <path>");
            return;
        }
        let path = self.resolve(arg);
        match std::fs::create_dir_all(&path) {
            Ok(()) => {
                self.set_success(format!("created {}", path.display()));
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                self.refresh(name.as_deref());
            }
            Err(err) => self.set_error(format!("mkdir {}: {err}", path.display())),
        }
    }

    fn touch_file(&mut self, arg: &str) {
        if arg.is_empty() {
            self.set_warning("usage: :touch <path>");
            return;
        }
        let path = self.resolve(arg);
        match touch_path(&path) {
            Ok(_) => {
                self.set_success(format!("created {}", path.display()));
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                self.refresh(name.as_deref());
            }
            Err(err) => self.set_error(format!("touch {}: {err}", path.display())),
        }
    }

    fn execute_input(&mut self) {
        let line = std::mem::take(&mut self.input);
        self.mode = Mode::Normal;
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        let (command, args) = match line.split_once(char::is_whitespace) {
            Some((command, args)) => (command, args.trim()),
            None => (line, ""),
        };
        match command {
            "q" | "quit" => self.should_quit = true,
            "cd" => {
                if args.is_empty() {
                    if let Ok(home) = std::env::var("HOME") {
                        self.change_dir(PathBuf::from(home));
                    }
                } else {
                    self.change_dir(self.resolve(args));
                }
            }
            "mkdir" => self.make_dir(args),
            "touch" => self.touch_file(args),
            "reload" => self.refresh(None),
            "hidden" | "toggle_hidden" => self.toggle_hidden(),
            "open" => self.open_selected(),
            "pwd" => self.set_status(self.cwd.display().to_string()),
            "help" => self.open_help(),
            "theme" => self.set_theme(args),
            "themes" => self.set_status(theme::names().join(", ")),
            _ => self.run_plugin_command(command, args),
        }
    }

    fn set_theme(&mut self, name: &str) {
        if name.is_empty() {
            self.open_theme_picker();
            return;
        }
        match theme::get(name) {
            Some(theme) => {
                self.theme = theme;
                self.config.theme = theme.name.into();
                match config::save_theme(theme.name) {
                    Ok(()) => self.set_success(format!("theme: {} (saved)", theme.name)),
                    Err(err) => {
                        self.set_warning(format!("theme: {} (not saved: {err})", theme.name))
                    }
                }
            }
            None => self.set_error(format!("unknown theme: {name}")),
        }
    }

    fn open_theme_picker(&mut self) {
        self.theme_choice = theme::all()
            .iter()
            .position(|theme| theme.name == self.theme.name)
            .unwrap_or(0);
        self.theme_original = Some(self.theme);
        self.mode = Mode::ThemePicker;
        self.input.clear();
        self.clear_status();
    }

    fn open_help(&mut self) {
        self.mode = Mode::Help;
        self.help_scroll = 0;
        self.input.clear();
    }

    fn close_help(&mut self) {
        self.mode = Mode::Normal;
        self.help_scroll = 0;
    }

    fn scroll_help(&mut self, delta: isize) {
        let max = HELP_KEYS.len() + HELP_COMMANDS.len() + 4;
        self.help_scroll = if delta < 0 {
            self.help_scroll.saturating_sub(delta.unsigned_abs())
        } else {
            self.help_scroll.saturating_add(delta as usize).min(max)
        };
    }

    fn move_theme_choice(&mut self, delta: isize) {
        let themes = theme::all();
        if themes.is_empty() {
            return;
        }
        let index = (self.theme_choice as isize + delta).rem_euclid(themes.len() as isize);
        self.theme_choice = index as usize;
        self.theme = themes[self.theme_choice];
    }

    fn jump_theme_choice(&mut self, last: bool) {
        let themes = theme::all();
        if themes.is_empty() {
            return;
        }
        self.theme_choice = if last { themes.len() - 1 } else { 0 };
        self.theme = themes[self.theme_choice];
    }

    fn finish_theme_picker(&mut self, save: bool) {
        if save {
            let theme = theme::all()[self.theme_choice];
            self.theme = theme;
            self.config.theme = theme.name.into();
            match config::save_theme(theme.name) {
                Ok(()) => self.set_success(format!("theme: {} (saved)", theme.name)),
                Err(err) => self.set_warning(format!("theme: {} (not saved: {err})", theme.name)),
            }
        } else if let Some(original) = self.theme_original {
            self.theme = original;
            self.clear_status();
        }
        self.theme_original = None;
        self.mode = Mode::Normal;
    }

    fn run_plugin_command(&mut self, command: &str, args: &str) {
        let result = self
            .plugin
            .as_ref()
            .map(|plugin| plugin.run_command(command, args));
        match result {
            Some(Ok(())) => {}
            Some(Err(err)) => self.set_error(err),
            None => self.set_error(format!("unknown command: {command}")),
        }
        self.drain_actions();
    }

    fn drain_actions(&mut self) {
        let actions = {
            let mut queue = self.actions.lock().unwrap();
            std::mem::take(&mut *queue)
        };
        for action in actions {
            match action {
                Action::Cd(path) => self.change_dir(path),
                Action::Notify(message) => self.set_status(message),
                Action::Quit => self.should_quit = true,
                Action::Reload => self.refresh(None),
                Action::ToggleHidden => self.toggle_hidden(),
                Action::Mkdir(path) => self.make_dir(&path),
                Action::Touch(path) => self.touch_file(&path),
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match self.mode {
            Mode::Command => self.handle_command_key(key),
            Mode::Normal => self.handle_normal_key(key),
            Mode::ThemePicker => self.handle_theme_picker_key(key),
            Mode::Help => self.handle_help_key(key),
        }
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => self.close_help(),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_help(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_help(-1),
            KeyCode::PageDown => self.scroll_help(10),
            KeyCode::PageUp => self.scroll_help(-10),
            KeyCode::Char('g') | KeyCode::Home => self.help_scroll = 0,
            KeyCode::Char('G') | KeyCode::End => self.scroll_help(isize::MAX / 2),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.close_help()
            }
            _ => {}
        }
    }

    fn handle_theme_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.finish_theme_picker(false),
            KeyCode::Enter => self.finish_theme_picker(true),
            KeyCode::Char('j') | KeyCode::Down => self.move_theme_choice(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_theme_choice(-1),
            KeyCode::Char('g') | KeyCode::Home => self.jump_theme_choice(false),
            KeyCode::Char('G') | KeyCode::End => self.jump_theme_choice(true),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.finish_theme_picker(false);
            }
            _ => {}
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input.clear();
            }
            KeyCode::Enter => self.execute_input(),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.clear();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = Mode::Normal;
                self.input.clear();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.input.push(ch);
            }
            _ => {}
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(1),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_down(10);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_up(10);
            }
            KeyCode::Char('g') | KeyCode::Home => self.go_top(),
            KeyCode::Char('G') | KeyCode::End => self.go_bottom(),
            KeyCode::Char('h') | KeyCode::Left => self.go_parent(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => self.open_selected(),
            KeyCode::Char('.') => self.toggle_hidden(),
            KeyCode::Tab => self.toggle_info(),
            KeyCode::Char('r') => self.refresh(None),
            KeyCode::Char('~') => {
                if let Ok(home) = std::env::var("HOME") {
                    self.change_dir(PathBuf::from(home));
                }
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.prompt = ':';
                self.input.clear();
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Command;
                self.prompt = '/';
                self.input.clear();
            }
            KeyCode::Char('T') => self.open_theme_picker(),
            KeyCode::Char('?') => self.open_help(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char(ch) => {
                let command = self
                    .plugin
                    .as_ref()
                    .and_then(|plugin| plugin.key_command(ch))
                    .map(str::to_string);
                if let Some(command) = command {
                    self.run_plugin_command(&command, "");
                }
            }
            _ => {}
        }
    }
}

/// Create `path` if needed without touching the contents of an existing file.
fn touch_path(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path)
        .map(|_| ())
}

pub const HINTS: &str = "j/k move  h/l dirs  g/G ends  . hidden  tab info  : command  ? help";

pub const HELP_KEYS: &[(&str, &str)] = &[
    ("j / k", "move down / up"),
    ("Ctrl-d / Ctrl-u", "jump 10 down / up"),
    ("g / G", "first / last entry"),
    ("h / l", "parent directory / open"),
    ("Enter", "enter directory or open file"),
    (".", "toggle hidden files"),
    ("r", "reload directory"),
    ("~", "go to home"),
    ("Tab", "info panel"),
    ("T", "theme picker"),
    (": or /", "command line"),
    ("?", "open or close this help"),
    ("q / Ctrl-c", "quit"),
];

pub const HELP_COMMANDS: &[(&str, &str)] = &[
    (":q, :quit", "quit"),
    (":cd [path]", "change directory (no path: home)"),
    (":mkdir PATH", "create a directory"),
    (":touch PATH", "create a file"),
    (":open", "open the selection"),
    (":reload", "reload the current directory"),
    (":hidden", "toggle hidden files"),
    (":pwd", "print the working directory"),
    (":help", "show this help"),
    (":theme [NAME]", "switch theme (no name: picker)"),
    (":themes", "list built-in themes"),
    (":NAME", "run a plugin command"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn touch_does_not_truncate_an_existing_file() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-touch-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("keep.txt");
        std::fs::write(&path, "important contents\n").unwrap();

        touch_path(&path).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "important contents\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn statuses_expire_after_the_ttl() {
        let mut status = Status::empty();
        status.text = "saved".into();
        status.kind = StatusKind::Error;
        status.shown_at = Instant::now() - STATUS_TTL - Duration::from_millis(1);
        assert!(!status.visible());

        status.shown_at = Instant::now();
        assert!(status.visible());
    }

    #[test]
    fn touch_creates_missing_files() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-touch-new-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("new.txt");

        touch_path(&path).unwrap();

        assert!(path.is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
