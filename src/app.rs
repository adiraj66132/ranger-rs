use crate::config::{self, Config};
use crate::fs::{self, Entry};
use crate::plugin::{Action, PluginHost};
use crate::preview::{self, PreviewMessage};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
}

pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub parent_entries: Vec<Entry>,
    pub parent_selected: Option<usize>,
    pub preview_lines: Vec<String>,
    pub mode: Mode,
    pub input: String,
    pub status: String,
    pub config: Config,
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

        let mut app = Self {
            cwd,
            entries: Vec::new(),
            selected: 0,
            parent_entries: Vec::new(),
            parent_selected: None,
            preview_lines: Vec::new(),
            mode: Mode::Normal,
            input: String::new(),
            status: String::new(),
            config,
            should_quit: false,
            preview_generation: 0,
            preview_tx,
            preview_rx,
            actions,
            shared_cwd,
            plugin,
        };
        app.refresh(None);
        if !warnings.is_empty() {
            if !app.status.is_empty() {
                warnings.push(app.status.clone());
            }
            app.status = warnings.join(" | ");
        }
        app
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
                self.status = format!("{}: {err}", self.cwd.display());
            }
        }
        self.refresh_parent();
        self.request_preview();
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
                self.status.clear();
                self.sync_cwd();
                self.refresh(None);
            }
            Ok(_) => self.status = format!("not a directory: {}", path.display()),
            Err(err) => self.status = format!("{}: {err}", path.display()),
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
        self.status.clear();
        self.sync_cwd();
        self.refresh(previous.as_deref());
    }

    fn toggle_hidden(&mut self) {
        self.config.show_hidden = !self.config.show_hidden;
        self.status = if self.config.show_hidden {
            "hidden files: on".into()
        } else {
            "hidden files: off".into()
        };
        self.refresh(None);
    }

    fn open_selected(&mut self) {
        let Some(entry) = self.selection().cloned() else {
            return;
        };
        if entry.is_dir {
            self.change_dir(entry.path);
        } else {
            self.open_path(&entry.path);
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
            Ok(_) => self.status = format!("opened {}", path.display()),
            Err(err) => self.status = format!("{opener}: {err}"),
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
            self.status = "usage: :mkdir <path>".into();
            return;
        }
        let path = self.resolve(arg);
        match std::fs::create_dir_all(&path) {
            Ok(()) => {
                self.status = format!("created {}", path.display());
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                self.refresh(name.as_deref());
            }
            Err(err) => self.status = format!("mkdir {}: {err}", path.display()),
        }
    }

    fn touch_file(&mut self, arg: &str) {
        if arg.is_empty() {
            self.status = "usage: :touch <path>".into();
            return;
        }
        let path = self.resolve(arg);
        match std::fs::File::create(&path) {
            Ok(_) => {
                self.status = format!("created {}", path.display());
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
                self.refresh(name.as_deref());
            }
            Err(err) => self.status = format!("touch {}: {err}", path.display()),
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
            "pwd" => self.status = self.cwd.display().to_string(),
            "help" => self.status = HINTS.into(),
            _ => self.run_plugin_command(command, args),
        }
    }

    fn run_plugin_command(&mut self, command: &str, args: &str) {
        let result = self
            .plugin
            .as_ref()
            .map(|plugin| plugin.run_command(command, args));
        match result {
            Some(Ok(())) => {}
            Some(Err(err)) => self.status = err,
            None => self.status = format!("unknown command: {command}"),
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
                Action::Notify(message) => self.status = message,
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
            KeyCode::Char('r') => self.refresh(None),
            KeyCode::Char('~') => {
                if let Ok(home) = std::env::var("HOME") {
                    self.change_dir(PathBuf::from(home));
                }
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.input.clear();
            }
            KeyCode::Char('?') => self.status = HINTS.into(),
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

pub const HINTS: &str =
    "j/k move  h/l dirs  g/G top/bottom  enter open  . hidden  r reload  : command  q quit";
