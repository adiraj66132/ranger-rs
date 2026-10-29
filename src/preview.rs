use crate::config::{Config, ScopeRule};
use crate::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub struct PreviewMessage {
    pub generation: u64,
    pub lines: Vec<String>,
}

pub fn request(
    generation: u64,
    path: std::path::PathBuf,
    config: Config,
    tx: Sender<PreviewMessage>,
) {
    // shortcut: one thread per selection change; debounce if huge trees make this expensive.
    thread::spawn(move || {
        let lines = generate(&path, &config);
        let _ = tx.send(PreviewMessage { generation, lines });
    });
}

/// Cheap text sniff used to decide between the editor and the system opener.
pub fn looks_text(path: &Path) -> bool {
    // Opening a FIFO blocks until a writer appears, which would freeze the UI
    // thread. Only sniff regular files (metadata follows symlinks).
    let is_file = std::fs::metadata(path).is_ok_and(|meta| meta.file_type().is_file());
    if !is_file {
        return false;
    }
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut sample = Vec::with_capacity(8192);
    if file.take(8192).read_to_end(&mut sample).is_err() {
        return false;
    }
    !sample.contains(&0)
}

fn generate(path: &Path, config: &Config) -> Vec<String> {
    let Ok(meta) = std::fs::metadata(path) else {
        return vec!["(unreadable)".into()];
    };
    if meta.is_dir() {
        return directory_preview(path, config);
    }
    if !meta.file_type().is_file() {
        // FIFOs, sockets and devices can block forever when opened.
        return vec!["(special file)".into()];
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    for rule in &config.scope {
        if !glob_match(&rule.pattern, &name) {
            continue;
        }
        match run_scope(rule, path, config.preview_lines) {
            Ok(lines) if !lines.is_empty() => return lines,
            Ok(_) => break,
            Err(err) => {
                let mut lines = vec![format!("[scope: {err}]")];
                lines.extend(file_preview(path, config));
                return lines;
            }
        }
    }
    file_preview(path, config)
}

fn directory_preview(path: &Path, config: &Config) -> Vec<String> {
    let entries = match fs::read_dir(path, config.show_hidden) {
        Ok(entries) => entries,
        Err(err) => return vec![format!("(cannot read directory: {err})")],
    };
    let mut lines = Vec::with_capacity(entries.len().min(config.preview_lines));
    for entry in entries.iter().take(config.preview_lines) {
        lines.push(if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        });
    }
    if entries.len() > config.preview_lines {
        lines.push(format!("... {} more", entries.len() - config.preview_lines));
    }
    if lines.is_empty() {
        lines.push("(empty directory)".into());
    }
    lines
}

fn file_preview(path: &Path, config: &Config) -> Vec<String> {
    let Ok(file) = File::open(path) else {
        return vec!["(unreadable)".into()];
    };
    let mut data = Vec::with_capacity(config.preview_bytes.min(64 * 1024));
    if file
        .take(config.preview_bytes as u64)
        .read_to_end(&mut data)
        .is_err()
    {
        return vec!["(unreadable)".into()];
    }
    if data.is_empty() {
        return vec!["(empty file)".into()];
    }
    if data.contains(&0) {
        return vec!["(binary file)".into()];
    }
    let text = String::from_utf8_lossy(&data);
    let mut lines: Vec<String> = text
        .lines()
        .take(config.preview_lines)
        .map(sanitize)
        .collect();
    if lines.is_empty() {
        lines.push("(empty file)".into());
    }
    lines
}

fn run_scope(rule: &ScopeRule, path: &Path, max_lines: usize) -> Result<Vec<String>, String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&rule.command)
        .env("RANGER_RS_PATH", path)
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;

    let mut stdout = child.stdout.take().ok_or("no stdout")?;
    let reader = thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stdout.read_to_end(&mut data);
        data
    });

    let timeout = Duration::from_millis(rule.timeout_ms.max(50));
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("timed out after {}ms", rule.timeout_ms));
            }
            Err(err) => return Err(err.to_string()),
        }
    }

    let data = reader.join().unwrap_or_default();
    let text = String::from_utf8_lossy(&data);
    let lines: Vec<String> = text.lines().take(max_lines).map(sanitize).collect();
    Ok(lines)
}

/// Keep terminal control sequences out of the preview pane.
fn sanitize(line: &str) -> String {
    line.replace('\t', "    ")
        .chars()
        .filter(|c| !c.is_control())
        .take(500)
        .collect()
}

fn glob_match(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    let (mut pi, mut ni) = (0, 0);
    let mut star = None;
    let mut resumed = 0;

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == '?' || pattern[pi] == name[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < pattern.len() && pattern[pi] == '*' {
            star = Some(pi);
            pi += 1;
            resumed = ni;
        } else if let Some(pos) = star {
            pi = pos + 1;
            resumed += 1;
            ni = resumed;
        } else {
            return false;
        }
    }
    while pi < pattern.len() && pattern[pi] == '*' {
        pi += 1;
    }
    pi == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_match_names() {
        assert!(glob_match("*.pdf", "manual.pdf"));
        assert!(glob_match("*.pdf", ".pdf"));
        assert!(!glob_match("*.pdf", "manual.txt"));
        assert!(glob_match("README?md", "README.md"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("a*b*c", "axxbyyc"));
    }

    #[test]
    fn previews_never_emit_escape_codes() {
        assert_eq!(sanitize("\u{1b}[31mred"), "[31mred");
        assert_eq!(sanitize("a\tb"), "a    b");
    }

    #[test]
    fn special_files_are_not_sniffed_or_opened() {
        use std::sync::mpsc;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-fifo-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let fifo = dir.join("pipe");
        let made = Command::new("mkfifo").arg(&fifo).status();
        if !made.is_ok_and(|status| status.success()) {
            std::fs::remove_dir_all(dir).unwrap();
            return;
        }

        let (tx, rx) = mpsc::channel();
        let path = fifo.clone();
        thread::spawn(move || {
            let _ = tx.send(looks_text(&path));
        });
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)), Ok(false));
        assert_eq!(generate(&fifo, &Config::default()), vec!["(special file)"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn text_sniff_separates_text_from_binary() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-sniff-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let text = dir.join("text.txt");
        let binary = dir.join("binary.bin");
        std::fs::write(&text, "hello\nworld\n").unwrap();
        std::fs::write(&binary, b"abc\0def").unwrap();
        assert!(looks_text(&text));
        assert!(!looks_text(&binary));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
