use assert_cmd::Command;
use chrono::Local;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn inline_dump_creates_the_default_braindump_file() {
    let temp_home = tempdir().expect("create temporary home");
    let file_path = temp_home.path().join("braindump/braindump.md");
    let before = Local::now().date_naive().to_string();

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");
    let before = Local::now().date_naive();

    for note in ["first", "second", "third"] {
        Command::cargo_bin("bd")
            .expect("locate bd binary")
            .env("HOME", temp_home.path())
            .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");
    let yesterday = Local::now().date_naive().pred_opt().expect("yesterday");
    let before = Local::now().date_naive().to_string();
    let fixture = format!("# {yesterday}\n\n## 23:59:59\nyesterday entry\n");
    fs::create_dir_all(file_path.parent().expect("braindump parent"))
        .expect("create braindump parent");
    fs::write(&file_path, &fixture).expect("write yesterday fixture");

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");
    let today = Local::now().date_naive();
    let fixture = format!("# {today}\n\n## 10:00:00\nexisting entry\n\n");
    fs::create_dir_all(file_path.parent().expect("braindump parent"))
        .expect("create braindump parent");
    fs::write(&file_path, &fixture).expect("write fixture");

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");

    for note in ["duplicate one", "duplicate two"] {
        Command::cargo_bin("bd")
            .expect("locate bd binary")
            .env("HOME", temp_home.path())
            .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
        let file_path = temp_home.path().join("braindump/braindump.md");

        Command::cargo_bin("bd")
            .expect("locate bd binary")
            .env("HOME", temp_home.path())
            .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let file_path = temp_home.path().join("braindump/braindump.md");
    fs::create_dir_all(&file_path).expect("create directory at braindump path");
    let expected_error = format!("bd: failed to write {}:", file_path.display());

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
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
    let expected_error = format!("bd: failed to write {}:", file_path.display());

    Command::cargo_bin("bd")
        .expect("locate bd binary")
        .env("HOME", temp_home.path())
        .env("XDG_CONFIG_HOME", temp_home.path().join("config"))
        .arg("cannot create parent")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(predicate::str::starts_with(expected_error));
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
