use crate::config;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

pub fn setup() -> Result<PathBuf, String> {
    if !io::stdin().is_terminal() {
        return Err(
            "no terminal available for setup; run `bd --setup` from a terminal".to_string(),
        );
    }
    let default = default_path()?;
    let path = loop {
        print!(
            "Braindump file path [default: {}]: ",
            display_path(&default)
        );
        io::stdout()
            .flush()
            .map_err(|error| format!("failed to write to stdout: {error}"))?;
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed to read input: {error}"))?;
        let candidate = resolve_input(line.trim(), &default)?;
        if candidate.is_dir() {
            eprintln!(
                "bd: {} is a directory; enter a file path",
                candidate.display()
            );
            continue;
        }
        break candidate;
    };

    let parent = path
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;

    config::save_path(&path.display().to_string())?;
    Ok(path)
}

fn default_path() -> Result<PathBuf, String> {
    home::home_dir()
        .map(|home| home.join("braindump/braindump.md"))
        .ok_or_else(|| "failed to determine home directory".to_string())
}

fn resolve_input(input: &str, default: &Path) -> Result<PathBuf, String> {
    if input.is_empty() {
        return Ok(default.to_path_buf());
    }
    let home = home::home_dir().ok_or_else(|| "failed to determine home directory".to_string())?;
    let candidate = if input == "~" {
        home
    } else if let Some(rest) = input.strip_prefix("~/") {
        home.join(rest)
    } else {
        PathBuf::from(input)
    };
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        let cwd = env::current_dir()
            .map_err(|error| format!("failed to determine current directory: {error}"))?;
        Ok(cwd.join(candidate))
    }
}

fn display_path(path: &Path) -> String {
    match home::home_dir() {
        Some(home) => match path.strip_prefix(&home) {
            Ok(relative) if relative.components().next().is_some() => {
                format!("~/{}", relative.display())
            }
            _ => path.display().to_string(),
        },
        None => path.display().to_string(),
    }
}
