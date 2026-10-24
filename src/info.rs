use crate::fs::{self, Entry};
use crate::preview;
use std::os::unix::fs::MetadataExt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Built only when the info pane is visible or the selection changes, never per frame.
pub fn describe(entry: &Entry) -> Vec<(String, String)> {
    let metadata = std::fs::metadata(&entry.path).ok();
    let mut lines = Vec::with_capacity(10);

    lines.push(("Name".into(), entry.name.clone()));
    lines.push(("Type".into(), mime_for(entry)));
    if entry.is_dir {
        let count = std::fs::read_dir(&entry.path)
            .map(|entries| entries.count())
            .unwrap_or(0);
        lines.push(("Items".into(), format!("{count} entries")));
    } else {
        lines.push((
            "Size".into(),
            format!("{} ({} bytes)", fs::format_size(entry.size), entry.size),
        ));
    }

    if let Some(metadata) = metadata {
        lines.push((
            "Owner".into(),
            format!(
                "{}:{} ({}:{})",
                lookup_name("/etc/passwd", metadata.uid())
                    .unwrap_or_else(|| metadata.uid().to_string()),
                lookup_name("/etc/group", metadata.gid())
                    .unwrap_or_else(|| metadata.gid().to_string()),
                metadata.uid(),
                metadata.gid()
            ),
        ));
        if let Ok(created) = metadata.created() {
            lines.push(("Created".into(), format_time(created)));
        }
        if let Ok(modified) = metadata.modified() {
            lines.push(("Modified".into(), format_time(modified)));
        }
    }

    lines.push((
        "Permissions".into(),
        fs::mode_string(entry.mode, entry.is_dir, entry.is_symlink),
    ));
    if entry.is_symlink {
        if let Ok(target) = std::fs::read_link(&entry.path) {
            lines.push(("Target".into(), target.display().to_string()));
        }
    }
    lines.push(("Path".into(), entry.path.display().to_string()));
    lines
}

fn mime_for(entry: &Entry) -> String {
    if entry.is_symlink {
        return "inode/symlink".into();
    }
    if entry.is_dir {
        return "inode/directory".into();
    }
    let guessed = mime_guess::from_path(&entry.path).first_or_octet_stream();
    if guessed == mime_guess::mime::APPLICATION_OCTET_STREAM && preview::looks_text(&entry.path) {
        "text/plain".into()
    } else {
        guessed.to_string()
    }
}

fn lookup_name(file: &str, id: u32) -> Option<String> {
    std::fs::read_to_string(file)
        .ok()?
        .lines()
        .find_map(|line| {
            let mut fields = line.split(':');
            let name = fields.next()?;
            fields.next()?;
            let value: u32 = fields.next()?.parse().ok()?;
            (value == id).then(|| name.to_string())
        })
}

pub fn format_time(time: SystemTime) -> String {
    let seconds = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(err) => -(err.duration().as_secs() as i64),
    };
    let days = seconds.div_euclid(86_400);
    let clock = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        clock / 3_600,
        (clock % 3_600) / 60,
        clock % 60
    )
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn formats_epoch_and_known_timestamp() {
        use std::time::Duration;
        assert_eq!(format_time(UNIX_EPOCH), "1970-01-01 00:00:00 UTC");
        assert_eq!(
            format_time(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            "2023-11-14 22:13:20 UTC"
        );
    }

    #[test]
    fn guesses_common_mime_types() {
        use std::path::Path;
        assert_eq!(
            mime_guess::from_path(Path::new("notes.txt"))
                .first_or_octet_stream()
                .to_string(),
            "text/plain"
        );
        assert_eq!(
            mime_guess::from_path(Path::new("photo.png"))
                .first_or_octet_stream()
                .to_string(),
            "image/png"
        );
    }

    #[test]
    fn describes_a_file_with_expected_fields() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-info-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("note.txt");
        std::fs::write(&path, "hello\n").unwrap();
        let entry = Entry {
            name: "note.txt".into(),
            path: path.clone(),
            is_dir: false,
            is_symlink: false,
            size: 6,
            mode: 0o644,
        };
        let lines = describe(&entry);
        let labels: Vec<&str> = lines.iter().map(|(label, _)| label.as_str()).collect();
        assert!(labels.contains(&"Type"));
        assert!(labels.contains(&"Size"));
        assert!(labels.contains(&"Owner"));
        assert!(labels.contains(&"Modified"));
        assert!(labels.contains(&"Path"));
        assert!(lines
            .iter()
            .any(|(label, value)| label == "Type" && value == "text/plain"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
