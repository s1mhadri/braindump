use std::env;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationDecision {
    None,
    New,
    Migrate { source: PathBuf },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SetupResult {
    pub path: PathBuf,
    pub migration: MigrationDecision,
}

pub fn setup(existing: Option<&Path>) -> Result<SetupResult, String> {
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
        let bytes_read = io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed to read input: {error}"))?;
        if bytes_read == 0 {
            return Err("setup cancelled".to_string());
        }
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

    let migration = if let Some(old_path) = existing {
        if path != old_path {
            prompt_migration(old_path)?
        } else {
            MigrationDecision::None
        }
    } else {
        MigrationDecision::None
    };

    Ok(SetupResult { path, migration })
}

fn prompt_migration(old_path: &Path) -> Result<MigrationDecision, String> {
    loop {
        print!("Migrate existing braindump file? [Y/n]: ");
        io::stdout()
            .flush()
            .map_err(|error| format!("failed to write to stdout: {error}"))?;
        let mut line = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed to read input: {error}"))?;
        if bytes_read == 0 {
            return Err("setup cancelled".to_string());
        }
        match line.trim() {
            "" | "y" | "Y" | "yes" | "Yes" | "YES" => {
                return Ok(MigrationDecision::Migrate {
                    source: old_path.to_path_buf(),
                });
            }
            "n" | "N" | "no" | "No" | "NO" => {
                return Ok(MigrationDecision::New);
            }
            _ => {
                eprintln!("bd: enter 'y' to migrate or 'n' for new");
            }
        }
    }
}

fn default_path() -> Result<PathBuf, String> {
    home::home_dir()
        .map(|home| home.join("braindump.md"))
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
