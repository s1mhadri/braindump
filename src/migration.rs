use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Path;
use tempfile::NamedTempFile;

pub fn migrate(source: &Path, target: &Path) -> Result<(), String> {
    if source == target {
        return Ok(());
    }

    let source_bytes = match fs::read(source) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return Err(format!(
                "failed to read source {}: {error}",
                source.display()
            ));
        }
    };

    let target_bytes = match fs::read(target) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            return Err(format!(
                "failed to read target {}: {error}",
                target.display()
            ));
        }
    };

    let needs_lf = !target_bytes.is_empty()
        && !source_bytes.is_empty()
        && !target_bytes.ends_with(b"\n")
        && !source_bytes.starts_with(b"\n");

    let target_parent = target
        .parent()
        .ok_or_else(|| format!("failed to determine parent of {}", target.display()))?;
    fs::create_dir_all(target_parent)
        .map_err(|error| format!("failed to create {}: {error}", target_parent.display()))?;

    let mut temp_file = NamedTempFile::new_in(target_parent).map_err(|error| {
        format!(
            "failed to create temporary file in {}: {error}",
            target_parent.display()
        )
    })?;

    if !target_bytes.is_empty() {
        temp_file
            .write_all(&target_bytes)
            .map_err(|error| format!("failed to write temporary file: {error}"))?;
    }

    if needs_lf {
        temp_file
            .write_all(b"\n")
            .map_err(|error| format!("failed to write temporary file: {error}"))?;
    }

    if !source_bytes.is_empty() {
        temp_file
            .write_all(&source_bytes)
            .map_err(|error| format!("failed to write temporary file: {error}"))?;
    }

    temp_file
        .flush()
        .map_err(|error| format!("failed to flush temporary file: {error}"))?;

    temp_file.persist(target).map_err(|error| {
        format!(
            "failed to rename temporary file to {}: {error}",
            target.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migrate_missing_source_and_missing_target_creates_empty_target() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        migrate(&source, &target).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read(&target).unwrap(), b"");
    }

    #[test]
    fn migrate_into_missing_target_copies_source_verbatim_and_leaves_source_intact() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");
        let source_content = b"# 2026-08-11\n\n## 12:00:00\nhello world\n";
        fs::write(&source, source_content).unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&source).unwrap(), source_content);
        assert_eq!(fs::read(&target).unwrap(), source_content);
    }

    #[test]
    fn migrate_into_existing_target_with_trailing_lf_inserts_no_join_byte() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        fs::write(&source, b"source note\n").unwrap();
        fs::write(&target, b"target note\n").unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&source).unwrap(), b"source note\n");
        assert_eq!(fs::read(&target).unwrap(), b"target note\nsource note\n");
    }

    #[test]
    fn migrate_into_existing_target_without_trailing_lf_inserts_exactly_one_lf() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        fs::write(&source, b"source note\n").unwrap();
        fs::write(&target, b"target note").unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"target note\nsource note\n");
    }

    #[test]
    fn migrate_into_existing_target_where_source_starts_with_lf_inserts_no_join_byte() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        fs::write(&source, b"\nsource note\n").unwrap();
        fs::write(&target, b"target note").unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"target note\nsource note\n");
    }

    #[test]
    fn migrate_where_both_target_ends_with_lf_and_source_starts_with_lf_inserts_no_join_byte() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        fs::write(&source, b"\nsource note\n").unwrap();
        fs::write(&target, b"target note\n").unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"target note\n\nsource note\n");
    }

    #[test]
    fn migrate_empty_source_into_existing_target_leaves_target_unchanged() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.md");
        let target = dir.path().join("target.md");

        fs::write(&source, b"").unwrap();
        fs::write(&target, b"existing target\n").unwrap();

        migrate(&source, &target).unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"existing target\n");
    }
}
