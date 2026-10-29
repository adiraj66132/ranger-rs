mod app;
mod config;
mod editor;
mod fs;
mod info;
mod media;
mod plugin;
mod preview;
mod theme;
mod ui;

use app::App;
use config::Config;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(err) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(err);
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(err) => {
                // No Tui exists yet, so Drop cannot undo the terminal setup.
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
                Err(err)
            }
        }
    }

    fn suspend(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)
    }

    fn resume(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        self.terminal.clear()
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "ranger-rs {}\n\nUsage: ranger-rs [PATH]\n\nVim-powered terminal file manager.",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("ranger-rs {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let start = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
    let start = std::fs::canonicalize(&start).unwrap_or(start);
    if !start.is_dir() {
        eprintln!("ranger-rs: not a directory: {}", start.display());
        std::process::exit(1);
    }

    let (config, warnings) = Config::load();
    let mut app = App::new(start, config);
    if !warnings.is_empty() {
        app.add_warning(warnings.join(" | "));
    }

    let mut tui = Tui::new()?;
    let mut state = ui::UiState::default();
    // Must run after entering the alternate screen and before reading events.
    state.set_picker(ui::detect_picker());

    while !app.should_quit {
        tui.terminal
            .draw(|frame| ui::draw(frame, &app, &mut state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }

        if let Some(launch) = app.take_editor_launch() {
            tui.suspend()?;
            let result = Command::new(launch.editor.program)
                .arg("--")
                .arg(&launch.path)
                .status();
            tui.resume()?;
            let error = result.err().map(|err| err.to_string());
            app.refresh(None);
            app.editor_finished(launch.editor, error);
        }

        if let Some(path) = app.take_video_launch() {
            tui.suspend()?;
            let result = media::play_video(&path);
            tui.resume()?;
            app.refresh(None);
            if let Err(err) = result {
                app.set_error(format!("mpv: {err}"));
            }
        }

        app.drain_preview();
        app.tick();
    }

    Ok(())
}
