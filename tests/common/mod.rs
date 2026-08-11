use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};

pub fn bd(temp_home: &tempfile::TempDir) -> Command {
    let mut command = Command::cargo_bin("bd").expect("locate bd binary");
    command
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"));
    command
}

pub fn braindump_path(temp_home: &tempfile::TempDir) -> PathBuf {
    temp_home.path().join("braindump.md")
}

#[allow(dead_code)]
pub fn config_path(temp_home: &tempfile::TempDir) -> PathBuf {
    temp_home.path().join("config/braindump/config.toml")
}

pub fn seed_config(temp_home: &tempfile::TempDir) {
    seed_config_at(temp_home, &braindump_path(temp_home));
}

pub fn seed_config_at(temp_home: &tempfile::TempDir, path: &Path) {
    let config_dir = temp_home.path().join("config/braindump");
    fs::create_dir_all(&config_dir).expect("create config directory");
    fs::write(
        config_dir.join("config.toml"),
        format!("braindump_file_path = \"{}\"\n", path.display()),
    )
    .expect("write config");
}

pub fn is_time_header(line: &str) -> bool {
    let time = line.strip_prefix("## ").unwrap_or_default();
    time.len() == 8
        && time.as_bytes()[2] == b':'
        && time.as_bytes()[5] == b':'
        && time
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5) || byte.is_ascii_digit())
}
