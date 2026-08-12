use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

fn host_target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        (os, arch) => panic!("unsupported test platform {os} {arch}"),
    }
}

fn make_executable(path: &Path) {
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("set permissions");
}

fn sha256_of(path: &Path) -> String {
    let tool = if Command::new("sha256sum")
        .arg(path)
        .output()
        .is_ok_and(|output| output.status.success())
    {
        "sha256sum"
    } else {
        "shasum"
    };
    let mut command = Command::new(tool);
    if tool == "shasum" {
        command.args(["-a", "256"]);
    }
    let output = command
        .arg(path)
        .output()
        .expect("failed to run checksum tool");
    assert!(
        output.status.success(),
        "{tool} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("checksum output is not UTF-8")
        .split_whitespace()
        .next()
        .expect("checksum output has no hash")
        .to_string()
}

fn build_archive(work_dir: &Path, archive_path: &Path) {
    fs::create_dir_all(work_dir).unwrap();
    let dummy_bd = work_dir.join("bd");
    fs::write(&dummy_bd, "#!/bin/sh\necho bd 0.1.0\n").unwrap();
    make_executable(&dummy_bd);

    let status = Command::new("tar")
        .args([
            "-czf",
            archive_path.to_str().unwrap(),
            "-C",
            work_dir.to_str().unwrap(),
            "bd",
        ])
        .status()
        .expect("failed to create test tar archive");
    assert!(status.success());
}

fn archive_name(tag: &str) -> String {
    format!("bd-{tag}-{target}.tar.gz", target = host_target())
}

fn mock_release_root(
    temp: &TempDir,
    tag: &str,
    archive_bytes: &[u8],
    sha_bytes: &[u8],
) -> PathBuf {
    let root = temp.path().join("mock_release");
    let release_dir = root.join(tag);
    fs::create_dir_all(&release_dir).unwrap();

    fs::write(release_dir.join(archive_name(tag)), archive_bytes).unwrap();
    fs::write(release_dir.join(format!("{}.sha256", archive_name(tag))), sha_bytes).unwrap();
    root
}

fn run_installer(
    temp: &TempDir,
    mock_root: &Path,
    install_dir: &Path,
    version: Option<&str>,
    api_url: Option<&str>,
) -> Output {
    let wrapper = temp.path().join("run_install.sh");
    let version_line = match version {
        Some(version) => format!(r#"export VERSION="{version}""#),
        None => String::new(),
    };
    let api_line = match api_url {
        Some(api_url) => format!(r#"export BD_API_URL="{api_url}""#),
        None => String::new(),
    };
    let content = format!(
        r#"#!/bin/sh
set -eu
export BD_INSTALLER_SKIP_MAIN=1
. ./install.sh
{version_line}
{api_line}
export INSTALL_DIR="{install_dir}"
export BD_INSTALL_BASE_URL="file://{mock_root}"
main
"#,
        version_line = version_line,
        api_line = api_line,
        install_dir = install_dir.display(),
        mock_root = mock_root.display()
    );
    fs::write(&wrapper, content).unwrap();
    make_executable(&wrapper);
    Command::new("sh")
        .arg(&wrapper)
        .output()
        .expect("execute install.sh")
}

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
fn install_script_downloads_verifies_and_installs_the_binary() {
    let temp = tempdir().expect("create temp dir");
    let tag = "v0.1.0";
    let version = "0.1.0";

    let archive_path = temp.path().join(archive_name(tag));
    build_archive(&temp.path().join("work"), &archive_path);

    let archive_bytes = fs::read(&archive_path).unwrap();
    let sha = sha256_of(&archive_path);
    let sha_bytes = format!("{sha}  {name}\n", name = archive_name(tag));
    let mock_root = mock_release_root(&temp, tag, &archive_bytes, sha_bytes.as_bytes());

    let install_dir = temp.path().join("bin");
    let output = run_installer(&temp, &mock_root, &install_dir, Some(version), None);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let installed = install_dir.join("bd");
    assert!(installed.exists());

    let version_output = Command::new(&installed)
        .output()
        .expect("run installed binary");
    assert_eq!(String::from_utf8_lossy(&version_output.stdout).trim(), "bd 0.1.0");
}

#[test]
fn install_script_aborts_when_the_archive_checksum_mismatches() {
    let temp = tempdir().expect("create temp dir");
    let tag = "v0.1.0";
    let version = "0.1.0";

    let archive_path = temp.path().join(archive_name(tag));
    build_archive(&temp.path().join("work"), &archive_path);

    let sha = sha256_of(&archive_path);
    let sha_bytes = format!("{sha}  {name}\n", name = archive_name(tag));

    let mut corrupted = fs::read(&archive_path).unwrap();
    corrupted[0] ^= 0xFF;
    let mock_root = mock_release_root(&temp, tag, &corrupted, sha_bytes.as_bytes());

    let install_dir = temp.path().join("bin");
    let output = run_installer(&temp, &mock_root, &install_dir, Some(version), None);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .to_lowercase()
            .contains("checksum"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!install_dir.join("bd").exists());
}

#[test]
fn install_script_discovers_latest_release_when_version_unset() {
    let temp = tempdir().expect("create temp dir");
    let tag = "v0.1.0";

    let archive_path = temp.path().join(archive_name(tag));
    build_archive(&temp.path().join("work"), &archive_path);
    let archive_bytes = fs::read(&archive_path).unwrap();
    let sha = sha256_of(&archive_path);
    let sha_bytes = format!("{sha}  {name}\n", name = archive_name(tag));
    let release_dir = mock_release_root(&temp, tag, &archive_bytes, sha_bytes.as_bytes());

    fs::write(
        release_dir.join("releases-latest.json"),
        format!("{{\"tag_name\": \"{tag}\"}}\n"),
    )
    .unwrap();

    let install_dir = temp.path().join("bin");
    let api_url = format!("file://{}/releases-latest.json", release_dir.display());
    let output = run_installer(&temp, &release_dir, &install_dir, None, Some(&api_url));
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let installed = install_dir.join("bd");
    assert!(installed.exists());
    let version_output = Command::new(&installed)
        .output()
        .expect("run installed binary");
    assert_eq!(String::from_utf8_lossy(&version_output.stdout).trim(), "bd 0.1.0");
}
