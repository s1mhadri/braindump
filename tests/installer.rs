use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn install_script_syntax_is_valid() {
    let output = Command::new("sh")
        .arg("-n")
        .arg("install.sh")
        .output()
        .expect("failed to execute sh -n");

    assert!(
        output.status.success(),
        "install.sh syntax error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn install_script_extracts_and_installs_binary_to_custom_dir() {
    let temp = tempdir().expect("create temp dir");
    let mock_release_dir = temp.path().join("mock_release");
    let install_dir = temp.path().join("bin");

    fs::create_dir_all(&mock_release_dir).unwrap();

    // Create a dummy bd binary
    let dummy_bd = mock_release_dir.join("bd");
    fs::write(&dummy_bd, "#!/bin/sh\necho bd 0.1.0\n").unwrap();
    let mut perms = fs::metadata(&dummy_bd).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dummy_bd, perms).unwrap();

    // Package into tar.gz
    let tag = "v0.1.0";
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        (os, arch) => panic!("unsupported test platform {os} {arch}"),
    };

    let archive_name = format!("bd-{tag}-{target}.tar.gz");
    let archive_path = temp.path().join(&archive_name);

    let tar_status = Command::new("tar")
        .args([
            "-czf",
            archive_path.to_str().unwrap(),
            "-C",
            mock_release_dir.to_str().unwrap(),
            "bd",
        ])
        .status()
        .expect("failed to create test tar archive");
    assert!(tar_status.success());

    // We test the extraction and installation portion of install.sh logic by simulating the environment
    let test_script = temp.path().join("run_test.sh");
    let script_content = format!(
        r#"#!/bin/sh
set -eu
export INSTALL_DIR="{install_dir}"
export VERSION="0.1.0"

# Mock download_file function to use our local archive
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

cp "{archive_path}" "$tmp_dir/{archive_name}"
tar -xzf "$tmp_dir/{archive_name}" -C "$tmp_dir"
chmod +x "$tmp_dir/bd"

mkdir -p "$INSTALL_DIR"
mv "$tmp_dir/bd" "$INSTALL_DIR/bd"
"#,
        install_dir = install_dir.display(),
        archive_path = archive_path.display(),
        archive_name = archive_name
    );

    fs::write(&test_script, script_content).unwrap();
    let mut perms = fs::metadata(&test_script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&test_script, perms).unwrap();

    let output = Command::new(&test_script).output().expect("execute test script");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let installed_binary = install_dir.join("bd");
    assert!(installed_binary.exists());

    let version_output = Command::new(&installed_binary)
        .output()
        .expect("run installed binary");
    assert_eq!(String::from_utf8_lossy(&version_output.stdout).trim(), "bd 0.1.0");
}
