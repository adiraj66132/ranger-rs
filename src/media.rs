use image::DynamicImage;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Longest edge of a decoded preview. The pane is a few hundred pixels wide,
/// so anything larger is wasted memory and transfer.
const MAX_PIXELS: u32 = 1024;

/// A malformed video must not wedge the preview thread.
const FRAME_TIMEOUT: Duration = Duration::from_millis(4000);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Video,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Video => "video",
        }
    }
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "qoi",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "m4v", "avi", "mpg", "mpeg", "wmv", "flv", "ogv", "ts", "3gp",
];

pub fn kind_of(path: &Path) -> Option<Kind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        Some(Kind::Image)
    } else if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        Some(Kind::Video)
    } else {
        None
    }
}

pub struct Frame {
    pub image: DynamicImage,
    pub caption: String,
}

/// Decode an image, or pull a single frame out of a video with ffmpeg.
pub fn preview(path: &Path, kind: Kind) -> Option<Frame> {
    match kind {
        Kind::Image => image_frame(path),
        Kind::Video => video_frame(path),
    }
}

fn image_frame(path: &Path) -> Option<Frame> {
    let reader = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?;
    let image = reader.decode().ok()?;
    Some(finish(image, ""))
}

fn video_frame(path: &Path) -> Option<Frame> {
    // One second in is a better thumbnail than a black opening frame, but very
    // short clips have nothing there, so fall back to the first frame.
    let bytes = ffmpeg_frame(path, "1").or_else(|| ffmpeg_frame(path, "0"))?;
    let image = image::load_from_memory(&bytes).ok()?;
    Some(finish(image, " · video"))
}

fn finish(image: DynamicImage, suffix: &str) -> Frame {
    let caption = format!("{}×{}{suffix}", image.width(), image.height());
    let image = if image.width().max(image.height()) > MAX_PIXELS {
        image.thumbnail(MAX_PIXELS, MAX_PIXELS)
    } else {
        image
    };
    Frame { image, caption }
}

fn ffmpeg_frame(path: &Path, seek: &str) -> Option<Vec<u8>> {
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin", "-ss", seek, "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-an",
            "-sn",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let reader = thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stdout.read_to_end(&mut data);
        data
    });

    let deadline = Instant::now() + FRAME_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }

    let data = reader.join().ok()?;
    (!data.is_empty()).then_some(data)
}

/// Play a video in the terminal, using mpv's kitty output when the terminal
/// speaks the kitty graphics protocol and truecolor cells otherwise.
pub fn play_video(path: &Path) -> std::io::Result<()> {
    let vo = if std::env::var_os("KITTY_WINDOW_ID").is_some()
        || std::env::var("TERM")
            .map(|term| term.contains("kitty"))
            .unwrap_or(false)
    {
        "kitty"
    } else {
        "tct"
    };
    let status = Command::new("mpv")
        .arg(format!("--vo={vo}"))
        .arg("--really-quiet")
        .arg("--")
        .arg(path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("mpv exited with {status}")))
    }
}

/// Whether a program is available on `PATH`, without spawning anything.
pub fn has_command(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
            && candidate
                .metadata()
                .map(|meta| {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                })
                .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-media-{label}-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extensions_map_to_kinds_case_insensitively() {
        assert_eq!(kind_of(Path::new("a.PNG")), Some(Kind::Image));
        assert_eq!(kind_of(Path::new("a.jpeg")), Some(Kind::Image));
        assert_eq!(kind_of(Path::new("clip.MKV")), Some(Kind::Video));
        assert_eq!(kind_of(Path::new("clip.webm")), Some(Kind::Video));
        assert_eq!(kind_of(Path::new("notes.md")), None);
        assert_eq!(kind_of(Path::new("no-extension")), None);
        assert_eq!(kind_of(Path::new("png")), None);
    }

    #[test]
    fn images_decode_and_caption_their_size() {
        let dir = temp_dir("image");
        let path = dir.join("pixel.png");
        let image = DynamicImage::new_rgb8(4, 3);
        image.save(&path).unwrap();

        let frame = preview(&path, Kind::Image).unwrap();
        assert_eq!(frame.caption, "4×3");
        assert_eq!((frame.image.width(), frame.image.height()), (4, 3));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn oversized_images_are_thumbnailed() {
        let dir = temp_dir("large");
        let path = dir.join("large.png");
        DynamicImage::new_rgb8(4000, 2000).save(&path).unwrap();

        let frame = preview(&path, Kind::Image).unwrap();
        assert_eq!(frame.caption, "4000×2000");
        assert!(frame.image.width().max(frame.image.height()) <= MAX_PIXELS);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn video_frames_come_from_ffmpeg_when_available() {
        if !has_command("ffmpeg") {
            return;
        }
        let dir = temp_dir("video");
        let path = dir.join("clip.mp4");
        let made = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=64x48:d=1",
                "-pix_fmt",
                "yuv420p",
                "-y",
            ])
            .arg(&path)
            .status();
        if !made.is_ok_and(|status| status.success()) {
            std::fs::remove_dir_all(dir).unwrap();
            return;
        }

        let frame = preview(&path, Kind::Video).unwrap();
        assert!(frame.caption.ends_with("· video"), "{}", frame.caption);
        assert_eq!((frame.image.width(), frame.image.height()), (64, 48));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
