use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Vim,
    Nvim,
    Vi,
    Nano,
    Emacs,
}

#[derive(Clone, Copy, Debug)]
pub struct Editor {
    pub program: &'static str,
    pub kind: Kind,
}

impl Editor {
    pub fn is_fallback(self) -> bool {
        matches!(self.kind, Kind::Nano | Kind::Emacs)
    }

    pub fn display_name(self) -> &'static str {
        match self.kind {
            Kind::Vim => "vim",
            Kind::Nvim => "nvim",
            Kind::Vi => "vi",
            Kind::Nano => "nano",
            Kind::Emacs => "Emacs",
        }
    }
}

const CANDIDATES: [(&str, Kind); 5] = [
    ("vim", Kind::Vim),
    ("nvim", Kind::Nvim),
    ("vi", Kind::Vi),
    ("nano", Kind::Nano),
    ("emacs", Kind::Emacs),
];

/// Finds the first editor in the user's PATH. Pure filesystem checks, no shell.
pub fn detect() -> Option<Editor> {
    let path = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    detect_in(&dirs)
}

fn detect_in(dirs: &[PathBuf]) -> Option<Editor> {
    CANDIDATES.iter().find_map(|(program, kind)| {
        dirs.iter()
            .any(|dir| is_executable(&dir.join(program)))
            .then_some(Editor {
                program,
                kind: *kind,
            })
    })
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-{label}-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_executable(path: &Path) {
        std::fs::write(path, "#!/bin/sh\n").unwrap();
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn priority_is_vim_then_nvim_then_vi_then_nano_then_emacs() {
        let dir = temp_dir("priority");
        make_executable(&dir.join("nano"));
        make_executable(&dir.join("vi"));
        make_executable(&dir.join("nvim"));
        make_executable(&dir.join("vim"));
        assert_eq!(
            detect_in(std::slice::from_ref(&dir)).unwrap().kind,
            Kind::Vim
        );

        std::fs::remove_file(dir.join("vim")).unwrap();
        assert_eq!(
            detect_in(std::slice::from_ref(&dir)).unwrap().kind,
            Kind::Nvim
        );

        std::fs::remove_file(dir.join("nvim")).unwrap();
        assert_eq!(
            detect_in(std::slice::from_ref(&dir)).unwrap().kind,
            Kind::Vi
        );

        std::fs::remove_file(dir.join("vi")).unwrap();
        assert_eq!(
            detect_in(std::slice::from_ref(&dir)).unwrap().kind,
            Kind::Nano
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn later_path_entries_still_count_and_non_executables_are_skipped() {
        let first = temp_dir("path-first");
        let second = temp_dir("path-second");
        std::fs::write(first.join("vim"), "not executable").unwrap();
        make_executable(&second.join("vim"));
        let found = detect_in(&[first.clone(), second.clone()]).unwrap();
        assert_eq!(found.program, "vim");
        std::fs::remove_dir_all(first).unwrap();
        std::fs::remove_dir_all(second).unwrap();
    }

    #[test]
    fn no_candidates_returns_none() {
        let dir = temp_dir("empty");
        assert!(detect_in(std::slice::from_ref(&dir)).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
