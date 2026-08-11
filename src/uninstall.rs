use std::env;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;

pub fn run(config_dir: &Path) -> Result<(), String> {
    let binary = env::current_exe()
        .map_err(|error| format!("failed to locate the running binary: {error}"))?;

    println!("This will uninstall bd:");
    println!("  binary: {}", binary.display());
    println!("  config: {}", config_dir.display());
    println!("Your braindump file is not deleted.");
    print!("Continue? [y/N]: ");
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to write to stdout: {error}"))?;

    let mut line = String::new();
    let bytes_read = io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("failed to read input: {error}"))?;
    if bytes_read == 0 || !matches!(line.trim(), "y" | "Y") {
        return Err("uninstall cancelled".to_string());
    }

    remove_config(config_dir)?;
    remove_binary(&binary)?;
    Ok(())
}

fn remove_config(config_dir: &Path) -> Result<(), String> {
    if let Some(braindump) = config::load_path()? {
        let braindump = PathBuf::from(braindump);
        if braindump.starts_with(config_dir) {
            let config_file = config_dir.join("config.toml");
            if config_file.exists() {
                fs::remove_file(&config_file).map_err(|error| {
                    format!("failed to remove {}: {error}", config_file.display())
                })?;
                println!("Removed {}.", config_file.display());
            }
            println!(
                "Braindump file {} is inside {}; keeping the directory.",
                braindump.display(),
                config_dir.display()
            );
            return Ok(());
        }
    }
    if !config_dir.exists() {
        println!("Config not found at {}; nothing to remove.", config_dir.display());
        return Ok(());
    }
    fs::remove_dir_all(config_dir)
        .map_err(|error| format!("failed to remove {}: {error}", config_dir.display()))?;
    println!("Removed {}.", config_dir.display());
    Ok(())
}

fn remove_binary(binary: &Path) -> Result<(), String> {
    if !binary.exists() {
        println!("Binary not found at {}; nothing to remove.", binary.display());
        return Ok(());
    }
    match fs::remove_file(binary) {
        Ok(_) => {
            println!("Removed {}.", binary.display());
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            remove_binary_with_sudo(binary)
        }
        Err(error) => Err(format!("failed to remove {}: {error}", binary.display())),
    }
}

fn remove_binary_with_sudo(binary: &Path) -> Result<(), String> {
    let sudo = env::var("BD_SUDO").unwrap_or_else(|_| "sudo".to_string());
    let status = Command::new(&sudo)
        .arg("rm")
        .arg("-f")
        .arg(binary)
        .status()
        .map_err(|error| format!("failed to run {sudo}: {error}"))?;
    if status.success() {
        println!("Removed {} (with {sudo}).", binary.display());
        Ok(())
    } else {
        Err(format!(
            "failed to remove {} even with {sudo}; remove it manually",
            binary.display()
        ))
    }
}