//! Parsing of Docker container archives (tar streams) into domain entities.
//!
//! `download_from_container` returns a tar archive of the requested path:
//! for a directory the archive contains one entry per direct child (plus the
//! directory entries themselves); for a single file it contains just that
//! file. [`parse_listing`] keeps only the direct children of the listed
//! directory, skipping `.`, `..`, and the directory entry itself.

use crate::application::error::AppError;
use crate::domain::container::{FileEntry, FileKind, FileListing, FilePermissions};
use std::io::{self, Read};
use std::path::Path;

/// Parses a directory listing archive into a [`FileListing`] for `path`.
///
/// Malformed entries are skipped silently (same convention as the other
/// infrastructure mappers).
pub fn parse_listing(archive: &[u8], path: &str) -> Result<FileListing, AppError> {
    ensure_valid_tar(archive)?;
    let base_depth = path_depth(path);
    let mut tar = tar::Archive::new(io::Cursor::new(archive));
    let mut entries = Vec::new();

    for entry in tar.entries().map_err(archive_error)? {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(entry_path) = entry.path() else {
            continue;
        };
        let Some(name) = direct_child_name(&entry_path, base_depth) else {
            continue;
        };
        if name == "." || name == ".." {
            continue;
        }

        let header = entry.header();
        let kind = if header.entry_type().is_dir() {
            FileKind::Directory
        } else if header.entry_type().is_symlink() {
            FileKind::Symlink
        } else if header.entry_type().is_file() {
            FileKind::File
        } else {
            FileKind::Other
        };

        let link_target = if kind == FileKind::Symlink {
            entry
                .link_name()
                .ok()
                .flatten()
                .map(|target| target.to_string_lossy().into_owned())
        } else {
            None
        };

        let full_path = if path == "/" {
            format!("/{name}")
        } else {
            format!("{}/{name}", path.trim_end_matches('/'))
        };

        let file_entry = FileEntry::new(
            name,
            full_path,
            kind,
            header.size().unwrap_or(0),
            header.mtime().ok().map(|ts| ts as i64),
            FilePermissions::new(header.mode().unwrap_or(0)),
        );
        match file_entry {
            Ok(mut file_entry) => {
                if let Some(target) = link_target {
                    file_entry = file_entry.with_link_target(target);
                }
                entries.push(file_entry);
            }
            Err(_) => continue,
        }
    }

    FileListing::new(path, entries).map_err(AppError::from)
}

/// Reads up to `max_bytes` of a single-file archive and returns the bytes
/// together with the real file size declared by the tar header.
pub fn parse_file_content(archive: &[u8], max_bytes: u64) -> Result<(Vec<u8>, u64), AppError> {
    ensure_valid_tar(archive)?;
    let mut tar = tar::Archive::new(io::Cursor::new(archive));
    for entry in tar.entries().map_err(archive_error)? {
        let Ok(mut entry) = entry else {
            continue;
        };
        let header = entry.header();
        if !header.entry_type().is_file() {
            continue;
        }
        let total_size = header.size().unwrap_or(0);
        let mut data = Vec::new();
        io::Read::take(&mut entry, max_bytes)
            .read_to_end(&mut data)
            .map_err(archive_error)?;
        return Ok((data, total_size));
    }
    Err(AppError::not_found("file not found in container archive"))
}

/// Number of name components of a container path: `/` -> 0, `/etc` -> 1,
/// `/etc/nginx` -> 2. Only `Normal` components count, so the leading `/`
/// (RootDir) is ignored.
fn path_depth(path: &str) -> usize {
    Path::new(path)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .count()
}

/// Returns the entry name when `entry_path` is a direct child of the listed
/// directory. Docker archives a directory relative to its parent, so a
/// listing of `/etc` contains entries like `etc/nginx` (parent depth 1), and a
/// listing of `/` contains entries like `etc/` (parent depth 0).
fn direct_child_name(entry_path: &Path, base_depth: usize) -> Option<String> {
    let normalized = entry_path.to_str()?.trim_end_matches('/');
    let normalized = Path::new(normalized);
    let name = normalized.file_name()?.to_str()?.to_string();
    let parent_depth = normalized.parent().map(path_depth_of).unwrap_or(0);
    if parent_depth == base_depth {
        Some(name)
    } else {
        None
    }
}

fn path_depth_of(path: &Path) -> usize {
    path.components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .count()
}

/// Minimal tar sanity check: the archive must hold at least one 512-byte
/// block whose checksum is valid. The `tar` crate silently yields zero entries
/// for short garbage inputs, which would otherwise surface as an empty
/// listing instead of an error.
fn ensure_valid_tar(archive: &[u8]) -> Result<(), AppError> {
    if archive.len() < 512 {
        return Err(AppError::operation_failed("invalid container archive"));
    }
    let block = &archive[..512];
    let stored = parse_octal(&block[148..156]);
    let computed: u64 = block
        .iter()
        .enumerate()
        .map(|(i, b)| {
            if (148..156).contains(&i) {
                32
            } else {
                u64::from(*b)
            }
        })
        .sum();
    match stored {
        Some(stored) if stored == computed => Ok(()),
        _ => Err(AppError::operation_failed("invalid container archive")),
    }
}

fn parse_octal(field: &[u8]) -> Option<u64> {
    let text = std::str::from_utf8(field).ok()?;
    let trimmed = text.trim_matches(|c| c == ' ' || c == '\0');
    if trimmed.is_empty() {
        return None;
    }
    u64::from_str_radix(trimmed, 8).ok()
}

fn archive_error(error: io::Error) -> AppError {
    AppError::operation_failed(format!("cannot read container archive: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Appends one entry to the archive. The path is written into the header
    /// verbatim (`Builder::append` performs no path sanitization), which keeps
    /// the fixtures independent of the `tar` crate's validation rules.
    fn append_entry(
        builder: &mut tar::Builder<Vec<u8>>,
        path: &str,
        data: &[u8],
        mode: u32,
        is_dir: bool,
    ) {
        let mut header = tar::Header::new_gnu();
        if is_dir {
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
        } else {
            header.set_size(data.len() as u64);
        }
        header.set_mode(mode);
        header.set_mtime(1_700_000_000);
        let name = header.as_old_mut();
        name.name[..path.len()].copy_from_slice(path.as_bytes());
        header.set_cksum();
        builder.append(&header, data).unwrap();
    }

    fn build_archive(entries: &[(&str, &[u8], u32, bool)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, data, mode, is_dir) in entries {
            append_entry(&mut builder, path, data, *mode, *is_dir);
        }
        builder.into_inner().unwrap()
    }

    /// Builds a tar archive the way the Docker daemon does: a directory is
    /// archived relative to its parent, i.e. a listing of `/etc` contains
    /// entries `etc/`, `etc/nginx/`, `etc/nginx/nginx.conf`, ...
    fn build_docker_dir_archive(dir_path: &str, entries: &[(&str, &[u8], u32, bool)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        let prefix = dir_path.trim_start_matches('/');
        if !prefix.is_empty() {
            append_entry(&mut builder, &format!("{prefix}/"), &[], 0o040755, true);
        }
        for (path, data, mode, is_dir) in entries {
            let full = if prefix.is_empty() {
                path.to_string()
            } else {
                format!("{prefix}/{path}")
            };
            append_entry(&mut builder, &full, data, *mode, *is_dir);
        }
        builder.into_inner().unwrap()
    }

    #[test]
    fn parse_listing_keeps_only_direct_children() {
        let archive = build_docker_dir_archive(
            "etc",
            &[
                ("nginx/", &[], 0o040755, true),
                ("nginx/nginx.conf", b"server {}", 0o100644, false),
                ("hostname", b"abc", 0o100644, false),
            ],
        );

        let listing = parse_listing(&archive, "/etc").unwrap();
        assert_eq!(listing.path(), "/etc");
        let names: Vec<&str> = listing.entries().iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["nginx", "hostname"]);

        let dir = &listing.entries()[0];
        assert_eq!(dir.kind(), FileKind::Directory);
        assert!(dir.is_dir());
        assert_eq!(dir.path(), "/etc/nginx");
        assert_eq!(dir.permissions().display(), "drwxr-xr-x");
        assert_eq!(dir.modified(), Some(1_700_000_000));

        let file = &listing.entries()[1];
        assert_eq!(file.kind(), FileKind::File);
        assert_eq!(file.size(), 3);
        assert_eq!(file.permissions().display(), "-rw-r--r--");
    }

    #[test]
    fn parse_listing_at_root_uses_single_leading_slash() {
        // A listing of `/` is archived as `var/`, `etc/`, ... (parent depth 0).
        let archive = build_docker_dir_archive("", &[("var/", &[], 0o040755, true)]);
        let listing = parse_listing(&archive, "/").unwrap();
        assert_eq!(listing.entries()[0].path(), "/var");
    }

    #[test]
    fn parse_listing_skips_dot_entries_and_deep_paths() {
        let archive = build_docker_dir_archive(
            "a",
            &[
                ("b/c/deep.txt", b"x", 0o100644, false),
                ("top.txt", b"x", 0o100644, false),
            ],
        );
        let listing = parse_listing(&archive, "/a").unwrap();
        let names: Vec<&str> = listing.entries().iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["top.txt"]);
    }

    #[test]
    fn parse_listing_rejects_empty_path() {
        let archive = build_archive(&[("f.txt", b"x", 0o100644, false)]);
        let result = parse_listing(&archive, "");
        assert!(result.is_err());
    }

    #[test]
    fn parse_listing_invalid_tar_returns_error() {
        let result = parse_listing(b"not a tar", "/etc");
        assert!(result.is_err());
    }

    #[test]
    fn parse_file_content_reads_first_file_with_total_size() {
        let archive = build_archive(&[("logs/app.log", b"hello world", 0o100644, false)]);
        let (data, total) = parse_file_content(&archive, 1024).unwrap();
        assert_eq!(data, b"hello world");
        assert_eq!(total, 11);
    }

    #[test]
    fn parse_file_content_truncates_to_max_bytes() {
        let archive = build_archive(&[("big.bin", &[0u8; 4096], 0o100644, false)]);
        let (data, total) = parse_file_content(&archive, 100).unwrap();
        assert_eq!(data.len(), 100);
        assert_eq!(total, 4096);
    }

    #[test]
    fn parse_file_content_skips_directories() {
        let archive = build_archive(&[
            ("dir/", &[], 0o040755, true),
            ("dir/file.txt", b"content", 0o100644, false),
        ]);
        let (data, total) = parse_file_content(&archive, 1024).unwrap();
        assert_eq!(data, b"content");
        assert_eq!(total, 7);
    }

    #[test]
    fn parse_file_content_without_file_returns_not_found() {
        let archive = build_archive(&[("dir/", &[], 0o040755, true)]);
        let result = parse_file_content(&archive, 1024);
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
