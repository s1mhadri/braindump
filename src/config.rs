use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
struct Config {
    braindump_file_path: String,
}

pub fn load_path() -> Result<Option<String>, String> {
    let path = config_path()?;
    match fs::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Config>(&text) {
            Ok(config) => {
                let braindump_file_path = config.braindump_file_path.trim();
                if braindump_file_path.is_empty() {
                    Ok(None)
                } else if usable_path(Path::new(braindump_file_path)) {
                    Ok(Some(braindump_file_path.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        },
        Err(_) => Ok(None),
    }
}

fn usable_path(path: &Path) -> bool {
    match fs::OpenOptions::new().append(true).open(path) {
        Ok(_) => true,
        Err(error) if error.kind() == ErrorKind::NotFound => creatable_ancestor(path),
        Err(_) => false,
    }
}

fn creatable_ancestor(path: &Path) -> bool {
    let mut ancestor = path;
    while let Some(parent) = ancestor.parent() {
        if parent.as_os_str().is_empty() {
            return env::current_dir().is_ok();
        }
        match fs::metadata(parent) {
            Ok(metadata) => return metadata.is_dir() && dir_accepts_files(parent),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                ancestor = parent;
            }
            Err(_) => return false,
        }
    }
    false
}

#[cfg(unix)]
fn dir_accepts_files(dir: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(dir)
        .map(|metadata| metadata.permissions().mode() & 0o222 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn dir_accepts_files(_dir: &Path) -> bool {
    true
}

pub fn save_path(braindump_file_path: &str) -> Result<(), String> {
    let config = Config {
        braindump_file_path: braindump_file_path.to_string(),
    };
    let path = config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    let text =
        toml::to_string(&config).map_err(|error| format!("failed to serialize config: {error}"))?;
    fs::write(&path, text).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(braindump_config_dir()?.join("config.toml"))
}

pub fn braindump_config_dir() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("braindump"))
}

fn config_dir() -> Result<PathBuf, String> {
    #[cfg(not(windows))]
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME").filter(|xdg| !xdg.is_empty()) {
        return Ok(PathBuf::from(xdg));
    }
    #[cfg(windows)]
    if let Some(appdata) = env::var_os("APPDATA").filter(|appdata| !appdata.is_empty()) {
        return Ok(PathBuf::from(appdata));
    }
    if let Some(home) = home::home_dir() {
        return Ok(home.join(".config"));
    }
    Err("failed to determine config directory".to_string())
}
