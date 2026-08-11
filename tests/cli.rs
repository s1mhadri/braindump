use chrono::Local;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{bd, braindump_path, is_time_header, seed_config, seed_config_at};

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
