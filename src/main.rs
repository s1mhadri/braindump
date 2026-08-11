mod config;
mod migration;
mod setup;
mod uninstall;

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("bd: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args() {
        Input::Command(Command::Help) => {
            print!("{USAGE}");
            Ok(())
        }
        Input::Command(Command::Version) => {
            println!("bd {VERSION}");
            Ok(())
        }
        Input::Command(Command::Setup) => {
            let existing = config::load_path()?.map(PathBuf::from);
            run_setup(existing.as_deref())?;
            Ok(())
        }
        Input::Command(Command::Uninstall) => {
            let config_dir = config::braindump_config_dir()?;
            uninstall::run(&config_dir)?;
            Ok(())
        }
        Input::Literal(text) => dump(normalize_note(text)),
        Input::Interactive => dump(read_interactive()?),
    }
}

fn dump(note: String) -> Result<(), String> {
    if note.trim().is_empty() {
        return Ok(());
    }
    let path = resolve_path()?;
    append_entry(&note, &path)
}

fn resolve_path() -> Result<PathBuf, String> {
    match config::load_path()? {
        Some(path) => Ok(PathBuf::from(path)),
        None => run_setup(None),
    }
}

fn run_setup(existing: Option<&Path>) -> Result<PathBuf, String> {
    let result = setup::setup(existing)?;
    let parent = result
        .path
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", result.path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;

    if let setup::MigrationDecision::Migrate { ref source } = result.migration {
        migration::migrate(source, &result.path)?;
    }
    config::save_path(&result.path.display().to_string())?;
    Ok(result.path)
}

enum Input {
    Command(Command),
    Literal(String),
    Interactive,
}

enum Command {
    Help,
    Setup,
    Uninstall,
    Version,
}

const VERSION: &str = match option_env!("BD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

const USAGE: &str = "bd - capture a thought in under a second

USAGE:
    bd [TEXT...]      Append TEXT as a new entry
    bd                Read multi-line input from stdin until end-of-file
    bd -- [TEXT...]   Append TEXT literally, even if it starts with a dash

OPTIONS:
    -h, --help        Print this help
    -v, --version     Print the version
    --setup           Configure where dumps are stored
    --uninstall       Remove the binary and its config

Everything else at the first position is note text, so bd git checkout -f and
bd -important note capture verbatim.
";

fn parse_args() -> Input {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("-h") | Some("--help") => Input::Command(Command::Help),
        Some("-v") | Some("--version") => Input::Command(Command::Version),
        Some("--setup") => Input::Command(Command::Setup),
        Some("--uninstall") => Input::Command(Command::Uninstall),
        Some("--") => Input::Literal(args[1..].join(" ")),
        Some(_) => Input::Literal(args.join(" ")),
        None => Input::Interactive,
    }
}

fn read_interactive() -> Result<String, String> {
    let mut stdin = io::stdin();
    if stdin.is_terminal() {
        let mut stdout = io::stdout();
        stdout
            .write_all(b"bd: dumping, Ctrl+D to save\n")
            .map_err(|error| format!("failed to write to stdout: {error}"))?;
        stdout
            .flush()
            .map_err(|error| format!("failed to write to stdout: {error}"))?;
    }

    let mut input = Vec::new();
    stdin
        .read_to_end(&mut input)
        .map_err(|error| format!("failed to read input: {error}"))?;
    let text =
        String::from_utf8(input).map_err(|error| format!("input is not valid UTF-8: {error}"))?;
    Ok(trim_blank_lines(&text).to_owned())
}

fn append_entry(note: &str, path: &Path) -> Result<(), String> {
    let now = Local::now();
    let parent = path
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", path.display()))?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    let existing = match fs::read_to_string(path) {
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
        .open(path)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn normalize_note(note: String) -> String {
    note.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim_end_matches('\n')
        .to_string()
}

fn trim_blank_lines(text: &str) -> &str {
    let mut start = None;
    let mut end = text.len();
    let mut offset = 0;
    for line in text.split('\n') {
        let line_end = offset + line.len();
        if !line.trim().is_empty() {
            if start.is_none() {
                start = Some(offset);
            }
            end = line_end;
        }
        offset = line_end + 1;
    }
    let Some(start) = start else {
        return "";
    };
    &text[start..end]
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
