use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("bd: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let note = normalize_note(std::env::args().skip(1).collect::<Vec<_>>().join(" "));
    if note.trim().is_empty() {
        return Ok(());
    }

    let now = Local::now();
    let path = default_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", path.display()))?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("failed to write {}: {error}", path.display())),
    };
    let day_header = format!("# {}", now.date_naive());
    let separator = separator_after(&existing);
    let prefix = if last_day_header(&existing) == Some(day_header.as_str()) {
        format!("{separator}## {}\n", now.format("%H:%M:%S"))
    } else {
        format!("{separator}{day_header}\n\n## {}\n", now.format("%H:%M:%S"))
    };
    let content = format!("{prefix}{note}\n");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn default_path() -> Result<PathBuf, String> {
    home::home_dir()
        .map(|home| home.join("braindump/braindump.md"))
        .ok_or_else(|| "failed to determine home directory".to_string())
}

fn normalize_note(note: String) -> String {
    note.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn last_day_header(content: &str) -> Option<&str> {
    content.lines().rev().find(|line| is_day_header(line))
}

fn separator_after(content: &str) -> &'static str {
    if content.is_empty() {
        return "";
    }

    let trailing_line_feeds = content
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\n')
        .count();

    match trailing_line_feeds {
        0 => "\n\n",
        1 => "\n",
        _ => "",
    }
}

fn is_day_header(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.len() == 12
        && bytes[0] == b'#'
        && bytes[1] == b' '
        && bytes[6] == b'-'
        && bytes[9] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 0 | 1 | 6 | 9) || byte.is_ascii_digit())
}
