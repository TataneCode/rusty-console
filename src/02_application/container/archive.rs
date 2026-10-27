//! Extraction of Docker container archives (tar) onto the host filesystem.
//!
//! The application layer calls [`extract_archive`] with the destination
//! directory; every entry is written below it, with a guard against path
//! traversal (`..`, absolute paths).

use crate::application::error::AppError;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Extracts a tar archive (as produced by the Docker `/archive` endpoint)
/// into `dest_dir`. Returns the path of the first top-level entry extracted.
///
/// Entries whose path escapes `dest_dir` are rejected with an error.
pub fn extract_archive(archive: &[u8], dest_dir: &Path) -> Result<PathBuf, AppError> {
    let mut tar = tar::Archive::new(io::Cursor::new(archive));
    let mut first_entry: Option<PathBuf> = None;

    for entry in tar.entries().map_err(|e| archive_error(&e))? {
        let mut entry = entry.map_err(|e| archive_error(&e))?;
        let entry_path = entry.path().map_err(|e| archive_error(&e))?.into_owned();
        let target = resolve_target(dest_dir, &entry_path)?;

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::operation_failed(format!("cannot create {parent:?}: {e}"))
            })?;
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| {
                AppError::operation_failed(format!("cannot create {target:?}: {e}"))
            })?;
        } else if entry.header().entry_type().is_symlink() {
            #[cfg(unix)]
            {
                let link_target = entry
                    .link_name()
                    .map_err(|e| archive_error(&e))?
                    .ok_or_else(|| {
                        AppError::operation_failed("symlink entry without link target")
                    })?;
                std::os::unix::fs::symlink(&link_target, &target).map_err(|e| {
                    AppError::operation_failed(format!("cannot create symlink {target:?}: {e}"))
                })?;
            }
            #[cfg(not(unix))]
            {
                let _ = target;
                return Err(AppError::operation_failed(
                    "symlinks are not supported on this platform",
                ));
            }
        } else {
            entry
                .unpack(&target)
                .map_err(|e| AppError::operation_failed(format!("cannot write {target:?}: {e}")))?;
        }

        if first_entry.is_none() {
            first_entry = Some(target);
        }
    }

    first_entry.ok_or_else(|| AppError::operation_failed("archive is empty"))
}

/// Maps an archive member path onto a destination path, rejecting any entry
/// that would escape `dest_dir` (absolute paths, `..`, prefix components).
fn resolve_target(dest_dir: &Path, entry_path: &Path) -> Result<PathBuf, AppError> {
    let mut target = dest_dir.to_path_buf();
    for component in entry_path.components() {
        match component {
            Component::Normal(part) => target.push(part),
            Component::CurDir => {}
            _ => {
                return Err(AppError::operation_failed(format!(
                    "unsafe path in archive: {entry_path:?}"
                )))
            }
        }
    }
    Ok(target)
}

fn archive_error(error: &io::Error) -> AppError {
    AppError::operation_failed(format!("cannot read container archive: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Appends a raw tar entry. Unlike `tar::Builder::append_data`, this does
    /// not sanitize the path, so fixtures can contain `..` and absolute paths
    /// to exercise the traversal guard.
    fn append_raw_entry(builder: &mut tar::Builder<Vec<u8>>, path: &str, data: &[u8], mode: u32) {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(mode);
        let name = header.as_old_mut();
        name.name[..path.len()].copy_from_slice(path.as_bytes());
        header.set_cksum();
        builder.append(&header, data).expect("append raw entry");
    }

    fn build_archive(entries: &[(&str, &[u8], u32)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, data, mode) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(*mode);
            header.set_cksum();
            builder
                .append_data(&mut header, path, *data)
                .expect("append entry");
        }
        builder.into_inner().expect("finish archive")
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rusty_console_archive_test_{}_{}",
            tag,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extract_archive_writes_files_and_returns_first_entry() {
        let dir = temp_dir("extract");
        let archive = build_archive(&[("web/logs/access.log", b"line1\n", 0o644)]);

        let first = extract_archive(&archive, &dir).unwrap();
        assert_eq!(first, dir.join("web/logs/access.log"));
        assert_eq!(
            fs::read(dir.join("web/logs/access.log")).unwrap(),
            b"line1\n"
        );

        fs::remove_dir_all(&dir).ok();
    }

    /// Builds a valid tar archive containing a single entry with the given
    /// raw (unsanitized) path.
    fn build_raw_archive(path: &str, data: &[u8]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        append_raw_entry(&mut builder, path, data, 0o644);
        builder.into_inner().expect("finish archive")
    }

    #[test]
    fn extract_archive_rejects_path_traversal() {
        let dir = temp_dir("traversal");
        let archive = build_raw_archive("../evil.txt", b"boom");

        let result = extract_archive(&archive, &dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unsafe path in archive"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extract_archive_rejects_absolute_paths() {
        let dir = temp_dir("absolute");
        let archive = build_raw_archive("/etc/passwd", b"root");

        assert!(extract_archive(&archive, &dir).is_err());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extract_archive_rejects_empty_archive() {
        let dir = temp_dir("empty");
        let archive = build_archive(&[]);

        let result = extract_archive(&archive, &dir);
        assert!(result.unwrap_err().to_string().contains("archive is empty"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extract_archive_rejects_invalid_tar() {
        let dir = temp_dir("invalid");
        let result = extract_archive(b"not a tar archive at all", &dir);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn extract_archive_preserves_permissions() {
        let dir = temp_dir("perms");
        let archive = build_archive(&[("script.sh", b"#!/bin/sh\n", 0o755)]);

        extract_archive(&archive, &dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.join("script.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755);
        }

        fs::remove_dir_all(&dir).ok();
    }
}
