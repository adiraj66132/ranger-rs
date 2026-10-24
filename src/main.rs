mod app;
mod config;
mod fs;
mod plugin;
mod preview;
mod ui;

use app::App;
use config::Config;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
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
        let prefix = warnings.join(" | ");
        app.status = if app.status.is_empty() {
            prefix
        } else {
            format!("{prefix} | {}", app.status)
        };
    }

    let mut tui = Tui::new()?;
    let mut state = ui::UiState::default();

    while !app.should_quit {
        tui.terminal
            .draw(|frame| ui::draw(frame, &app, &mut state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
        app.drain_preview();
    }

    Ok(())
}
