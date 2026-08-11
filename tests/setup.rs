use chrono::Local;
use expectrl::session::OsSession;
use expectrl::{ControlCode, Eof, Expect};
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;
mod common;
use common::{bd, braindump_path, config_path, is_time_header, seed_config, seed_config_at};

const NO_TERMINAL_ERROR: &str =
    "bd: no terminal available for setup; run `bd --setup` from a terminal\n";

#[test]
fn setup_requires_a_terminal() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .arg("--setup")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);

    assert!(!config_path(&temp_home).exists());
    assert!(!braindump_path(&temp_home).exists());
}

#[test]
fn first_run_dump_without_terminal_fails_loudly() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .args(["hello", "world"])
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);

    assert!(!config_path(&temp_home).exists());
    assert!(!braindump_path(&temp_home).exists());
}

#[test]
fn first_run_interactive_dump_without_terminal_fails_loudly() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .write_stdin("piped note\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);

    assert!(!config_path(&temp_home).exists());
    assert!(!braindump_path(&temp_home).exists());
}

#[test]
fn parse_broken_and_path_broken_configs_fail_identically() {
    let malformed_home = tempdir().expect("create temporary home");
    seed_config_with_text(&malformed_home, "not: [valid toml");
    let malformed_config =
        fs::read_to_string(config_path(&malformed_home)).expect("read malformed config");

    let dir_home = tempdir().expect("create temporary home");
    fs::create_dir_all(braindump_path(&dir_home)).expect("create directory at braindump path");
    seed_config(&dir_home);
    let dir_config = fs::read_to_string(config_path(&dir_home)).expect("read broken config");

    for temp_home in [&malformed_home, &dir_home] {
        bd(temp_home)
            .arg("note")
            .write_stdin("\n")
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr(NO_TERMINAL_ERROR);
    }

    assert_eq!(
        fs::read_to_string(config_path(&malformed_home)).expect("read config"),
        malformed_config
    );
    assert_eq!(
        fs::read_to_string(config_path(&dir_home)).expect("read config"),
        dir_config
    );
}

#[test]
fn missing_path_key_triggers_setup() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config_with_text(&temp_home, "[braindump]\n");

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);
}

#[test]
fn empty_path_value_triggers_setup() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config_with_text(&temp_home, "braindump_file_path = \"\"\n");

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);
}

#[test]
fn blocked_parent_triggers_setup() {
    let temp_home = tempdir().expect("create temporary home");
    let blocked_parent = temp_home.path().join("braindump");
    fs::write(&blocked_parent, "not a directory").expect("create blocking file");
    seed_config_at(&temp_home, &blocked_parent.join("braindump.md"));

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);

    assert_eq!(
        fs::read_to_string(&blocked_parent).expect("read blocking file"),
        "not a directory"
    );
}

#[cfg(unix)]
#[test]
fn unreadable_config_file_triggers_setup() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    fs::set_permissions(config_path(&temp_home), fs::Permissions::from_mode(0o000))
        .expect("make config unreadable");

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);
}

#[cfg(unix)]
#[test]
fn unwritable_path_triggers_setup() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    let file_path = braindump_path(&temp_home);
    fs::create_dir_all(file_path.parent().expect("braindump parent")).expect("create parent");
    fs::write(&file_path, "existing\n").expect("write braindump file");
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o444))
        .expect("make braindump file read-only");
    seed_config(&temp_home);

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);

    assert_eq!(
        fs::read_to_string(&file_path).expect("read braindump file"),
        "existing\n"
    );
}

#[cfg(unix)]
#[test]
fn uncreatable_parent_triggers_setup() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    let parent = temp_home.path().join("braindump");
    fs::create_dir_all(&parent).expect("create parent");
    fs::set_permissions(&parent, fs::Permissions::from_mode(0o555)).expect("make parent read-only");
    seed_config_at(&temp_home, &parent.join("braindump.md"));

    bd(&temp_home)
        .arg("note")
        .write_stdin("\n")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(NO_TERMINAL_ERROR);
}

#[test]
fn setup_accepts_the_default_path() {
    let temp_home = tempdir().expect("create temporary home");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("setup exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!(
            "braindump_file_path = \"{}\"\n",
            braindump_path(&temp_home).display()
        )
    );
    assert!(
        braindump_path(&temp_home)
            .parent()
            .expect("default parent")
            .exists()
    );
}

#[test]
fn setup_accepts_a_custom_path_and_creates_parents() {
    let temp_home = tempdir().expect("create temporary home");
    let custom = temp_home.path().join("custom/dir/notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(custom.display().to_string())
        .expect("enter custom path");
    session.expect(Eof).expect("setup exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", custom.display())
    );
    assert!(custom.parent().expect("custom parent").exists());
}

#[test]
fn setup_reprompts_when_the_choice_is_a_directory() {
    let temp_home = tempdir().expect("create temporary home");
    let directory = temp_home.path().join("adir");
    fs::create_dir_all(&directory).expect("create directory");
    let custom = temp_home.path().join("custom.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(directory.display().to_string())
        .expect("enter a directory");
    session.expect("is a directory").expect("see reprompt");
    session
        .send_line(custom.display().to_string())
        .expect("enter custom path");
    session.expect(Eof).expect("setup exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", custom.display())
    );
}

#[test]
fn setup_expands_a_tilde_in_the_custom_path() {
    let temp_home = tempdir().expect("create temporary home");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session.send_line("~/notes.md").expect("enter tilde path");
    session.expect(Eof).expect("setup exits");

    let expected = temp_home.path().join("notes.md");
    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", expected.display())
    );
}

#[test]
fn setup_rewrites_an_existing_config() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let new_path = temp_home.path().join("elsewhere/notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("setup exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
}

#[test]
fn first_run_inline_dump_survives_setup() {
    let temp_home = tempdir().expect("create temporary home");
    let before = Local::now().date_naive().to_string();
    let mut session = bd_pty(&temp_home, &["hello", "world"]);

    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("process exits");

    let after = Local::now().date_naive().to_string();
    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!(
            "braindump_file_path = \"{}\"\n",
            braindump_path(&temp_home).display()
        )
    );
    let content = fs::read_to_string(braindump_path(&temp_home)).expect("read braindump file");
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0] == format!("# {before}") || lines[0] == format!("# {after}"));
    assert_eq!(lines[1], "");
    assert!(is_time_header(lines[2]));
    assert_eq!(lines[3], "hello world");
}

#[test]
fn first_run_interactive_dump_survives_setup() {
    let temp_home = tempdir().expect("create temporary home");
    let mut session = bd_pty(&temp_home, &[]);

    session.expect("dumping, Ctrl+D to save").expect("see hint");
    session.send_line("multi line note").expect("type note");
    session
        .send(ControlCode::EndOfTransmission)
        .expect("end input");
    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("process exits");

    let content = fs::read_to_string(braindump_path(&temp_home)).expect("read braindump file");
    assert!(content.ends_with("multi line note\n"));
}

#[test]
fn path_is_a_directory_reenters_setup() {
    let temp_home = tempdir().expect("create temporary home");
    fs::create_dir_all(braindump_path(&temp_home)).expect("create directory at braindump path");
    seed_config(&temp_home);
    let new_path = temp_home.path().join("recovered/notes.md");
    let mut session = bd_pty(&temp_home, &["note", "after", "defect"]);

    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session.expect(Eof).expect("process exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
    let content = fs::read_to_string(&new_path).expect("read new braindump file");
    assert!(content.ends_with("note after defect\n"));
}

#[test]
fn blocked_parent_reenters_setup() {
    let temp_home = tempdir().expect("create temporary home");
    let blocked_parent = temp_home.path().join("braindump");
    fs::write(&blocked_parent, "not a directory").expect("create blocking file");
    seed_config_at(&temp_home, &blocked_parent.join("braindump.md"));
    let new_path = temp_home.path().join("recovered/notes.md");
    let mut session = bd_pty(&temp_home, &["cannot", "create", "parent"]);

    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session.expect(Eof).expect("process exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
    let content = fs::read_to_string(&new_path).expect("read new braindump file");
    assert!(content.ends_with("cannot create parent\n"));
}

#[test]
fn malformed_config_reenters_setup() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config_with_text(&temp_home, "not: [valid toml");
    let mut session = bd_pty(&temp_home, &["survives", "defect"]);

    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("process exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!(
            "braindump_file_path = \"{}\"\n",
            braindump_path(&temp_home).display()
        )
    );
    let content = fs::read_to_string(braindump_path(&temp_home)).expect("read braindump file");
    assert!(content.ends_with("survives defect\n"));
}

#[cfg(unix)]
#[test]
fn unwritable_path_reenters_setup() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    let file_path = braindump_path(&temp_home);
    fs::create_dir_all(file_path.parent().expect("braindump parent")).expect("create parent");
    fs::write(&file_path, "existing\n").expect("write braindump file");
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o444))
        .expect("make braindump file read-only");
    seed_config(&temp_home);
    let new_path = temp_home.path().join("recovered/notes.md");
    let mut session = bd_pty(&temp_home, &["note"]);

    session
        .expect("Braindump file path")
        .expect("setup prompts");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session.expect(Eof).expect("process exits");

    assert_eq!(
        fs::read_to_string(&file_path).expect("read braindump file"),
        "existing\n"
    );
    let content = fs::read_to_string(&new_path).expect("read new braindump file");
    assert!(content.ends_with("note\n"));
}

#[cfg(unix)]
#[test]
fn ctrl_c_during_setup_aborts_with_nothing_written() {
    use expectrl::process::unix::{Signal, WaitStatus};

    let temp_home = tempdir().expect("create temporary home");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session.send(ControlCode::EndOfText).expect("send Ctrl+C");
    let pid = session.get_process().pid();
    let status = session.get_process().wait().expect("wait for bd");
    assert_eq!(status, WaitStatus::Signaled(pid, Signal::SIGINT, false));
    assert!(!config_path(&temp_home).exists());
    assert!(!braindump_path(&temp_home).exists());
}

#[test]
fn path_change_prompts_migrate_or_new_default_migrate() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old note content\n").unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("new_dir/notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session
        .expect("Braindump file path")
        .expect("see path prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session.send_line("").expect("accept default migrate");
    session.expect(Eof).expect("setup exits");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
    assert_eq!(
        fs::read_to_string(&new_path).expect("read new braindump file"),
        "old note content\n"
    );
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old braindump file"),
        "old note content\n"
    );
}

#[test]
fn migration_into_missing_target_copies_markdown_bytes_in_order() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    let old_content = "# 2026-08-11\n\n## 10:00:00\nfirst entry\n\n## 11:00:00\nsecond entry\n";
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, old_content).unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("new_braindump.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see prompt");
    session.send_line("y").expect("explicit yes");
    session.expect(Eof).expect("setup exits");

    assert_eq!(
        fs::read_to_string(&new_path).expect("read new file"),
        old_content
    );
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old file"),
        old_content
    );
}

#[test]
fn migration_into_target_containing_entries_appends_in_order() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old entries\n").unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("target.md");
    fs::write(&new_path, "existing target entries\n").unwrap();

    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see prompt");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("setup exits");

    assert_eq!(
        fs::read_to_string(&new_path).expect("read target"),
        "existing target entries\nold entries\n"
    );
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old"),
        "old entries\n"
    );
}

#[test]
fn choosing_new_creates_or_uses_target_without_copying_old_entries() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old entries\n").unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("target.md");
    fs::write(&new_path, "existing target entries\n").unwrap();

    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see prompt");
    session.send_line("n").expect("choose new");
    session.expect(Eof).expect("setup exits");

    assert_eq!(
        fs::read_to_string(&new_path).expect("read target"),
        "existing target entries\n"
    );
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old"),
        "old entries\n"
    );
}

#[test]
fn choosing_same_path_does_not_show_migration_and_does_not_duplicate() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "existing entry\n").unwrap();
    seed_config(&temp_home);

    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(old_path.display().to_string())
        .expect("enter same path");
    session
        .expect(Eof)
        .expect("setup exits without migration prompt");

    assert_eq!(
        fs::read_to_string(&old_path).expect("read file"),
        "existing entry\n"
    );
}

#[test]
fn setup_shows_migration_only_when_selected_path_differs() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);

    // Same path: no migration prompt
    let mut session1 = bd_pty(&temp_home, &["--setup"]);
    session1.expect("Braindump file path").expect("see prompt");
    session1.send_line("").expect("accept default (same path)");
    session1
        .expect(Eof)
        .expect("exits without migration prompt");

    // Different path: migration prompt shown
    let new_path = temp_home.path().join("different.md");
    let mut session2 = bd_pty(&temp_home, &["--setup"]);
    session2.expect("Braindump file path").expect("see prompt");
    session2
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session2
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session2.send_line("n").expect("choose new");
    session2.expect(Eof).expect("exits");
}

#[test]
fn invalid_migrate_or_new_input_reprompts_and_follows_choice() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old note\n").unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("new_notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session
        .send_line("invalid_choice")
        .expect("send invalid choice");
    session
        .expect("enter 'y' to migrate or 'n' for new")
        .expect("see reprompt");
    session.send_line("y").expect("send valid yes");
    session.expect(Eof).expect("exits");

    assert_eq!(
        fs::read_to_string(&new_path).expect("read new file"),
        "old note\n"
    );
}

#[cfg(unix)]
#[test]
fn ctrl_c_at_migration_prompt_aborts_with_nothing_written() {
    use expectrl::process::unix::{Signal, WaitStatus};

    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old content\n").unwrap();
    seed_config(&temp_home);

    let new_path = temp_home.path().join("new_notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);

    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter new path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session.send(ControlCode::EndOfText).expect("send Ctrl+C");

    let pid = session.get_process().pid();
    let status = session.get_process().wait().expect("wait for bd");
    assert_eq!(status, WaitStatus::Signaled(pid, Signal::SIGINT, false));

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", old_path.display())
    );
    assert!(!new_path.exists());
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old file"),
        "old content\n"
    );
}

#[test]
fn missing_source_behaves_as_empty_migration_source() {
    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    seed_config(&temp_home);

    let new_path = temp_home.path().join("new_notes.md");
    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(new_path.display().to_string())
        .expect("enter path");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see prompt");
    session.send_line("").expect("accept default migrate");
    session.expect(Eof).expect("exits");

    assert_eq!(
        fs::read_to_string(config_path(&temp_home)).expect("read config"),
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
    assert_eq!(fs::read_to_string(&new_path).expect("read new file"), "");
    assert!(!old_path.exists());
}

#[cfg(unix)]
#[test]
fn migration_failure_leaves_configured_path_and_old_file_intact() {
    use std::os::unix::fs::PermissionsExt;

    let temp_home = tempdir().expect("create temporary home");
    let old_path = braindump_path(&temp_home);
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, "old entries\n").unwrap();
    seed_config(&temp_home);

    let read_only_dir = temp_home.path().join("read_only");
    fs::create_dir_all(&read_only_dir).unwrap();
    fs::set_permissions(&read_only_dir, fs::Permissions::from_mode(0o555)).unwrap();
    let unwritable_target = read_only_dir.join("notes.md");

    let mut session = bd_pty(&temp_home, &["--setup"]);
    session.expect("Braindump file path").expect("see prompt");
    session
        .send_line(unwritable_target.display().to_string())
        .expect("enter target in unwritable dir");
    session
        .expect("Migrate existing braindump file? [Y/n]: ")
        .expect("see migration prompt");
    session.send_line("").expect("accept default");
    session.expect(Eof).expect("process exits with error");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", old_path.display())
    );
    assert_eq!(
        fs::read_to_string(&old_path).expect("read old braindump file"),
        "old entries\n"
    );
}

fn bd_pty(temp_home: &tempfile::TempDir, args: &[&str]) -> OsSession {
    let mut command = ProcessCommand::new(assert_cmd::cargo::cargo_bin!("bd"));
    command
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
        .args(args);
    OsSession::spawn(command).expect("spawn bd under a pty")
}

fn seed_config_with_text(temp_home: &tempfile::TempDir, text: &str) {
    let config_dir = temp_home.path().join("config/braindump");
    fs::create_dir_all(&config_dir).expect("create config directory");
    fs::write(config_dir.join("config.toml"), text).expect("write config");
}
