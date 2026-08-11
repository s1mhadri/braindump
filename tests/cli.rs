use assert_cmd::Command;
use chrono::Local;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn inline_dump_creates_the_default_braindump_file() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);
    let before = Local::now().date_naive().to_string();

    bd(&temp_home)
        .args(["hello", "world"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let after = Local::now().date_naive().to_string();
    let content = fs::read_to_string(file_path).expect("read braindump file");
    let lines: Vec<_> = content.lines().collect();

    assert_eq!(lines.len(), 4);
    assert!(lines[0] == format!("# {before}") || lines[0] == format!("# {after}"));
    assert_eq!(lines[1], "");
    assert!(is_time_header(lines[2]));
    assert_eq!(lines[3], "hello world");
    assert!(content.ends_with('\n'));
}

#[test]
fn inline_dump_joins_arguments_without_changing_interior_spacing() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["foo", "bar  baz", "qux"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("foo bar  baz qux\n"));
}

#[test]
fn dash_prefixed_arguments_are_literal_note_text() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["git", "checkout", "-f"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("git checkout -f\n"));
}

#[test]
fn repeated_dumps_keep_entries_in_order_and_cleanly_separated() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);
    let before = Local::now().date_naive();

    for note in ["first", "second", "third"] {
        bd(&temp_home)
            .arg(note)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    let after = Local::now().date_naive();
    let content = fs::read_to_string(file_path).expect("read braindump file");
    let day_headers: Vec<_> = content
        .lines()
        .filter(|line| line.starts_with("# ") && !line.starts_with("## "))
        .collect();

    assert!(content.ends_with('\n'));
    assert!(!content.contains('\r'));
    assert!(!content.contains("\n\n\n"));
    assert_eq!(content.matches("## ").count(), 3);
    assert!(content.contains("first\n\n## "));
    assert!(content.contains("second\n\n## "));
    assert!(content.ends_with("third\n"));
    assert!(day_headers.len() <= 2);
    if before == after {
        assert_eq!(day_headers, vec![format!("# {before}")]);
    }
}

#[test]
fn dump_after_a_previous_day_starts_a_new_day_section() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);
    let yesterday = Local::now().date_naive().pred_opt().expect("yesterday");
    let before = Local::now().date_naive().to_string();
    let fixture = format!("# {yesterday}\n\n## 23:59:59\nyesterday entry\n");
    fs::create_dir_all(file_path.parent().expect("braindump parent"))
        .expect("create braindump parent");
    fs::write(&file_path, &fixture).expect("write yesterday fixture");

    bd(&temp_home)
        .arg("today entry")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let after = Local::now().date_naive().to_string();
    let content = fs::read_to_string(file_path).expect("read braindump file");
    let day_headers: Vec<_> = content
        .lines()
        .filter(|line| line.starts_with("# ") && !line.starts_with("## "))
        .collect();

    assert_eq!(day_headers.len(), 2);
    assert_eq!(day_headers[0], format!("# {yesterday}"));
    assert!(content.starts_with(&fixture));
    assert!(content.contains("yesterday entry\n\n# "));
    assert!(content.contains("\n\n## "));
    assert!(content.ends_with("today entry\n"));
    assert!(day_headers[1] == format!("# {before}") || day_headers[1] == format!("# {after}"));
}

#[test]
fn dump_does_not_add_extra_blank_lines_to_a_file_that_already_ends_with_one() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);
    let today = Local::now().date_naive();
    let fixture = format!("# {today}\n\n## 10:00:00\nexisting entry\n\n");
    fs::create_dir_all(file_path.parent().expect("braindump parent"))
        .expect("create braindump parent");
    fs::write(&file_path, &fixture).expect("write fixture");

    bd(&temp_home)
        .arg("next entry")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.contains("existing entry\n\n## "));
    assert!(!content.contains("existing entry\n\n\n## "));
}

#[test]
fn rapid_dumps_persist_both_entries() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    for note in ["duplicate one", "duplicate two"] {
        bd(&temp_home)
            .arg(note)
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert_eq!(content.matches("## ").count(), 2);
    assert!(content.contains("duplicate one\n\n## "));
    assert!(content.ends_with("duplicate two\n"));
}

#[test]
fn blank_inline_dumps_are_silent_no_ops() {
    for arguments in [vec![""], vec!["  "]] {
        let temp_home = tempdir().expect("create temporary home");
        let file_path = braindump_path(&temp_home);

        bd(&temp_home)
            .args(arguments)
            .assert()
            .success()
            .stdout("")
            .stderr("");

        assert!(!file_path.exists());
        assert!(!temp_home.path().join("braindump").exists());
    }
}

#[test]
fn inline_dump_normalizes_line_endings_and_keeps_one_trailing_lf() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .arg("first\r\nsecond\rthird\n")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("first\nsecond\nthird\n"));
    assert!(!content.contains('\r'));
    assert!(!content.ends_with("third\n\n"));
}

#[test]
fn dump_reports_an_error_when_the_braindump_path_is_a_directory() {
    let temp_home = tempdir().expect("create temporary home");
    let file_path = braindump_path(&temp_home);
    fs::create_dir_all(&file_path).expect("create directory at braindump path");
    seed_config(&temp_home);
    let expected_error = format!("bd: failed to write {}:", file_path.display());

    bd(&temp_home)
        .arg("cannot write")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(predicate::str::starts_with(expected_error));
}

#[test]
fn dump_reports_an_error_when_a_required_parent_is_a_file() {
    let temp_home = tempdir().expect("create temporary home");
    let blocked_parent = temp_home.path().join("braindump");
    fs::write(&blocked_parent, "not a directory").expect("create blocking file");
    let file_path = blocked_parent.join("braindump.md");
    seed_config_at(&temp_home, &file_path);
    let expected_error = format!("bd: failed to write {}:", file_path.display());

    bd(&temp_home)
        .arg("cannot create parent")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(predicate::str::starts_with(expected_error));
}

#[test]
fn no_argument_multiline_stdin_is_appended() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);
    let before = Local::now().date_naive().to_string();

    bd(&temp_home)
        .write_stdin("first line\nsecond line\n")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let after = Local::now().date_naive().to_string();
    let content = fs::read_to_string(file_path).expect("read braindump file");
    let lines: Vec<_> = content.lines().collect();

    assert_eq!(lines.len(), 5);
    assert!(lines[0] == format!("# {before}") || lines[0] == format!("# {after}"));
    assert_eq!(lines[1], "");
    assert!(is_time_header(lines[2]));
    assert_eq!(lines[3], "first line");
    assert_eq!(lines[4], "second line");
    assert!(content.ends_with('\n'));
}

#[test]
fn interactive_input_trims_leading_and_trailing_blank_lines() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .write_stdin("\n\nfirst line\nsecond line\n\n  \n")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("first line\nsecond line\n"));
    assert!(!content.contains("\n\nfirst line"));
}

#[test]
fn interactive_input_preserves_interior_blank_lines_and_formatting() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .write_stdin("  indented\n\n\tTabbed\n\n\nspaced    out\n\nfinal\n")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("  indented\n\n\tTabbed\n\n\nspaced    out\n\nfinal\n"));
}

#[test]
fn all_blank_interactive_input_is_a_silent_no_op() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .write_stdin("\n\n   \n\t\n\n")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!braindump_path(&temp_home).exists());
    assert!(!temp_home.path().join("braindump").exists());
}

#[test]
fn inline_arguments_take_precedence_over_stdin() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["inline note"])
        .write_stdin(&b"sentinel from stdin\xffSTDIN"[..])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("inline note\n"));
    assert!(!content.contains("STDIN"));
}

#[test]
fn piped_interactive_input_prints_no_hint() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);

    bd(&temp_home)
        .write_stdin("piped note\n")
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn version_flag_prints_the_version_without_writing() {
    for flag in ["-v", "--version"] {
        let temp_home = tempdir().expect("create temporary home");

        bd(&temp_home)
            .arg(flag)
            .assert()
            .success()
            .stdout(format!("bd {}\n", env!("CARGO_PKG_VERSION")))
            .stderr("");

        assert!(!temp_home.path().join("braindump").exists());
    }
}

#[test]
fn help_flag_prints_usage_without_writing() {
    for flag in ["-h", "--help"] {
        let temp_home = tempdir().expect("create temporary home");

        bd(&temp_home)
            .arg(flag)
            .assert()
            .success()
            .stdout(predicate::str::contains("USAGE"))
            .stderr("");

        assert!(!temp_home.path().join("braindump").exists());
    }
}

#[test]
fn dash_prefixed_first_argument_is_literal_note_text() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["-important", "note"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("-important note\n"));
}

#[test]
fn command_like_tokens_outside_first_position_are_literal_text() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["call", "-h", "support"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    assert!(content.ends_with("call -h support\n"));
}

#[test]
fn double_dash_at_first_position_forces_literal_text() {
    let temp_home = tempdir().expect("create temporary home");
    seed_config(&temp_home);
    let file_path = braindump_path(&temp_home);

    bd(&temp_home)
        .args(["--", "--search", "foo"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    let content = fs::read_to_string(file_path).expect("read braindump file");
    let last_line = content.lines().last().expect("content has a note");
    assert_eq!(last_line, "--search foo");
    assert!(!content.contains("-- --search"));
}

#[test]
fn double_dash_makes_even_command_like_tokens_literal() {
    for arguments in [["--", "-h"], ["--", "--setup"]] {
        let temp_home = tempdir().expect("create temporary home");
        seed_config(&temp_home);
        let file_path = braindump_path(&temp_home);

        bd(&temp_home)
            .args(arguments)
            .assert()
            .success()
            .stdout("")
            .stderr("");

        let content = fs::read_to_string(file_path).expect("read braindump file");
        let last_line = content.lines().last().expect("content has a note");
        assert_eq!(last_line, arguments[1]);
        assert!(!content.contains(&format!("-- {}", arguments[1])));
    }
}

#[test]
fn bare_double_dash_is_a_silent_no_op() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .arg("--")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!braindump_path(&temp_home).exists());
}

#[test]
fn setup_accepts_the_default_path() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .arg("--setup")
        .write_stdin("\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Braindump file path"))
        .stderr("");

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

    bd(&temp_home)
        .arg("--setup")
        .write_stdin(format!("{}\n", custom.display()))
        .assert()
        .success()
        .stderr("");

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

    bd(&temp_home)
        .arg("--setup")
        .write_stdin(format!("{}\n{}\n", directory.display(), custom.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Braindump file path"))
        .stderr(predicate::str::contains("is a directory"));

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", custom.display())
    );
}

#[test]
fn setup_expands_a_tilde_in_the_custom_path() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .arg("--setup")
        .write_stdin("~/notes.md\n")
        .assert()
        .success()
        .stderr("");

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

    bd(&temp_home)
        .arg("--setup")
        .write_stdin(format!("{}\n", new_path.display()))
        .assert()
        .success()
        .stderr("");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!("braindump_file_path = \"{}\"\n", new_path.display())
    );
}

#[test]
fn first_run_without_config_auto_triggers_setup_and_dumps() {
    let temp_home = tempdir().expect("create temporary home");
    let before = Local::now().date_naive().to_string();

    bd(&temp_home)
        .args(["hello", "world"])
        .write_stdin("\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Braindump file path"))
        .stderr("");

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
fn first_run_interactive_stdin_dump_lands_in_the_default_path() {
    let temp_home = tempdir().expect("create temporary home");

    bd(&temp_home)
        .write_stdin("piped note\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Braindump file path"))
        .stderr("");

    let config = fs::read_to_string(config_path(&temp_home)).expect("read config");
    assert_eq!(
        config,
        format!(
            "braindump_file_path = \"{}\"\n",
            braindump_path(&temp_home).display()
        )
    );
    let content = fs::read_to_string(braindump_path(&temp_home)).expect("read braindump file");
    assert!(content.ends_with("piped note\n"));
}

#[test]
fn inline_dumps_use_the_configured_path() {
    let temp_home = tempdir().expect("create temporary home");
    let custom = temp_home.path().join("custom/notes.md");
    seed_config_at(&temp_home, &custom);

    bd(&temp_home)
        .args(["note", "here"])
        .assert()
        .success()
        .stdout("")
        .stderr("");

    assert!(!braindump_path(&temp_home).exists());
    let content = fs::read_to_string(&custom).expect("read braindump file");
    assert!(content.ends_with("note here\n"));
}

#[cfg(unix)]
#[test]
fn ctrl_c_during_setup_writes_nothing() {
    use std::os::unix::process::ExitStatusExt;
    use std::process::{Command as ProcessCommand, Stdio};

    let temp_home = tempdir().expect("create temporary home");
    let mut child = ProcessCommand::new(assert_cmd::cargo::cargo_bin!("bd"))
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg("--setup")
        .spawn()
        .expect("spawn bd");

    let _keep_stdin_open = child.stdin.take().expect("piped stdin");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let settled_at = std::time::Instant::now() + std::time::Duration::from_millis(200);
    loop {
        if let Some(status) = child.try_wait().expect("poll bd") {
            panic!("bd exited before SIGINT: {status:?}");
        }
        if std::time::Instant::now() >= settled_at || std::time::Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let killed = ProcessCommand::new("kill")
        .args(["-s", "INT", &child.id().to_string()])
        .status()
        .expect("send SIGINT");
    assert!(killed.success());

    let status = child.wait().expect("wait for bd");
    assert_eq!(status.signal(), Some(2));
    assert!(!config_path(&temp_home).exists());
    assert!(!braindump_path(&temp_home).exists());
}

fn bd(temp_home: &tempfile::TempDir) -> assert_cmd::Command {
    let mut command = Command::cargo_bin("bd").expect("locate bd binary");
    command
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"));
    command
}

fn braindump_path(temp_home: &tempfile::TempDir) -> PathBuf {
    temp_home.path().join("braindump/braindump.md")
}

fn config_path(temp_home: &tempfile::TempDir) -> PathBuf {
    temp_home.path().join("config/braindump/config.toml")
}

fn seed_config(temp_home: &tempfile::TempDir) {
    seed_config_at(temp_home, &braindump_path(temp_home));
}

fn seed_config_at(temp_home: &tempfile::TempDir, path: &Path) {
    let config_dir = temp_home.path().join("config/braindump");
    fs::create_dir_all(&config_dir).expect("create config directory");
    fs::write(
        config_dir.join("config.toml"),
        format!("braindump_file_path = \"{}\"\n", path.display()),
    )
    .expect("write config");
}

fn is_time_header(line: &str) -> bool {
    let time = line.strip_prefix("## ").unwrap_or_default();
    time.len() == 8
        && time.as_bytes()[2] == b':'
        && time.as_bytes()[5] == b':'
        && time
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5) || byte.is_ascii_digit())
}
