#![allow(dead_code)]

use assert_cmd::Command;
use expectrl::session::OsSession;
use expectrl::{Eof, Expect};
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::{tempdir, TempDir};

mod common;
use common::{braindump_path, config_path, seed_config, seed_config_at};

fn installed(temp_home: &TempDir) -> PathBuf {
    let bin_dir = temp_home.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin directory");
    let source = assert_cmd::cargo::cargo_bin!("bd");
    let destination = bin_dir.join("bd");
    fs::copy(source, &destination).expect("copy bd binary");
    destination
}

fn command(binary: &Path, temp_home: &TempDir) -> Command {
    let mut command = Command::new(binary);
    command
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"));
    command
}

fn pty(binary: &Path, temp_home: &TempDir, args: &[&str]) -> OsSession {
    let mut command = ProcessCommand::new(binary);
    command
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
        .args(args);
    OsSession::spawn(command).expect("spawn bd under a pty")
}

#[test]
fn uninstall_without_confirmation_aborts_and_deletes_nothing() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let binary = installed(&temp_home);
    let braindump = braindump_path(&temp_home);
    fs::write(&braindump, "# 2026-08-12\n\n## 10:00:00\nprecious notes\n").expect("write notes");

    command(&binary, &temp_home)
        .arg("--uninstall")
        .write_stdin("")
        .assert()
        .failure()
        .code(1);

    assert!(binary.exists());
    assert!(config_path(&temp_home).exists());
    assert_eq!(
        fs::read_to_string(&braindump).expect("read braindump file"),
        "# 2026-08-12\n\n## 10:00:00\nprecious notes\n"
    );
}

#[test]
fn uninstall_aborts_on_enter_and_none_inputs() {
    for input in ["\n", "n\n", "maybe\n"] {
        let temp_home = tempdir().expect("create temporary home");
        seed_config(&temp_home);
        let binary = installed(&temp_home);

        command(&binary, &temp_home)
            .arg("--uninstall")
            .write_stdin(input)
            .assert()
            .failure()
            .code(1);

        assert!(binary.exists());
        assert!(config_path(&temp_home).exists());
    }
}

#[test]
fn confirmed_uninstall_removes_binary_and_config_but_not_the_braindump_file() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let binary = installed(&temp_home);
    let braindump = braindump_path(&temp_home);
    fs::write(&braindump, "precious notes\n").expect("write notes");

    command(&binary, &temp_home)
        .arg("--uninstall")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("This will uninstall bd"))
        .stdout(predicate::str::contains(format!(
            "Removed {}.",
            config_path(&temp_home).parent().expect("config parent").display()
        )))
        .stdout(predicate::str::contains(format!("Removed {}.", binary.display())));

    assert!(!binary.exists());
    assert!(!config_path(&temp_home).exists());
    assert!(!temp_home.path().join("config/braindump").exists());
    assert_eq!(
        fs::read_to_string(&braindump).expect("read braindump file"),
        "precious notes\n"
    );
}

#[test]
fn uninstall_when_config_is_missing_reports_nothing_to_remove_and_succeeds() {
    let temp_home = tempdir().expect("create temporary home");
    let binary = installed(&temp_home);

    command(&binary, &temp_home)
        .arg("--uninstall")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config not found at"))
        .stdout(predicate::str::contains(format!("Removed {}.", binary.display())));

    assert!(!binary.exists());
    assert!(!temp_home.path().join("config/braindump").exists());
}

#[cfg(unix)]
#[test]
fn uninstall_when_the_binary_is_already_gone_reports_nothing_to_remove() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let binary = installed(&temp_home);
    let mut session = pty(&binary, &temp_home, &["--uninstall"]);

    session
        .expect("Continue? [y/N]: ")
        .expect("see confirmation prompt");
    fs::remove_file(&binary).expect("remove binary while it is running");
    session.send_line("y").expect("confirm uninstall");
    session.expect("Binary not found at").expect("report missing binary");
    session.expect(Eof).expect("uninstall exits");

    assert!(!binary.exists());
    assert!(!config_path(&temp_home).exists());
}

#[cfg(unix)]
#[test]
fn uninstall_when_nothing_is_installed_reports_success() {
    let temp_home = tempdir().expect("create temporary home");
    let binary = installed(&temp_home);
    let mut session = pty(&binary, &temp_home, &["--uninstall"]);

    session
        .expect("Continue? [y/N]: ")
        .expect("see confirmation prompt");
    fs::remove_file(&binary).expect("remove binary while it is running");
    session.send_line("y").expect("confirm uninstall");
    session.expect("Config not found at").expect("report missing config");
    session.expect("Binary not found at").expect("report missing binary");
    session.expect(Eof).expect("uninstall exits");
}

#[test]
fn uninstall_keeps_the_config_directory_when_the_braindump_file_lives_inside_it() {
    let temp_home = tempdir().expect("create temporary home");
    let braindump = temp_home.path().join("config/braindump/notes.md");
    seed_config_at(&temp_home, &braindump);
    fs::write(&braindump, "nested notes\n").expect("write nested notes");
    let binary = installed(&temp_home);

    command(&binary, &temp_home)
        .arg("--uninstall")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("keeping the directory"));

    assert!(!binary.exists());
    assert!(!config_path(&temp_home).exists());
    assert!(braindump.parent().expect("config parent").exists());
    assert_eq!(
        fs::read_to_string(&braindump).expect("read braindump file"),
        "nested notes\n"
    );
}

#[cfg(unix)]
#[test]
fn uninstall_escalates_to_sudo_when_the_binary_directory_is_not_writable() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let binary = installed(&temp_home);
    let bin_dir = binary.parent().expect("binary directory");

    let fake_sudo = temp_home.path().join("fake-sudo");
    fs::write(
        &fake_sudo,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$BD_SUDO_LOG\"\n",
    )
    .expect("write fake sudo");
    let mut perms = fs::metadata(&fake_sudo).expect("fake sudo metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&fake_sudo, perms).expect("make fake sudo executable");
    let sudo_log = temp_home.path().join("sudo.log");

    fs::set_permissions(bin_dir, fs::Permissions::from_mode(0o555))
        .expect("make binary directory read-only");

    command(&binary, &temp_home)
        .env("BD_SUDO", &fake_sudo)
        .env("BD_SUDO_LOG", &sudo_log)
        .arg("--uninstall")
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("(with"));

    let log = fs::read_to_string(&sudo_log).expect("read sudo log");
    assert!(log.contains("rm -f"), "fake sudo invoked rm: {log}");
    assert!(
        log.contains(&binary.display().to_string()),
        "fake sudo removed the binary path: {log}"
    );

    fs::set_permissions(bin_dir, fs::Permissions::from_mode(0o755))
        .expect("restore binary directory permissions");

    assert!(!config_path(&temp_home).exists());
}