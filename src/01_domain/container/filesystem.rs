use crate::domain::error::DomainError;

/// Kind of a file entry inside a container filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Directory,
    File,
    Symlink,
    Other,
}

/// Unix permission bits (plus file type bits) of a file entry, as reported by
/// the Docker archive API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions(u32);

impl FilePermissions {
    pub fn new(mode: u32) -> Self {
        FilePermissions(mode)
    }

    pub fn permission_bits(&self) -> u32 {
        self.0 & 0o777
    }

    /// `ls`-style display, e.g. `drwxr-xr-x` or `-rw-r--r--`.
    pub fn display(&self) -> String {
        let type_char = match self.0 & 0o170000 {
            0o040000 => 'd',
            0o120000 => 'l',
            _ => '-',
        };
        let mut out = String::with_capacity(10);
        out.push(type_char);
        for shift in [6, 3, 0] {
            let tri = (self.0 >> shift) & 0o7;
            out.push(if tri & 0o4 != 0 { 'r' } else { '-' });
            out.push(if tri & 0o2 != 0 { 'w' } else { '-' });
            out.push(if tri & 0o1 != 0 { 'x' } else { '-' });
        }
        out
    }
}

/// A single file entry of a container filesystem listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    name: String,
    path: String,
    kind: FileKind,
    size: u64,
    modified: Option<i64>,
    permissions: FilePermissions,
    link_target: Option<String>,
}

impl FileEntry {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        kind: FileKind,
        size: u64,
        modified: Option<i64>,
        permissions: FilePermissions,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if name.is_empty() {
            return Err(DomainError::InvalidFileEntry(
                "file entry name cannot be empty".to_string(),
            ));
        }
        Ok(FileEntry {
            name,
            path: path.into(),
            kind,
            size,
            modified,
            permissions,
            link_target: None,
        })
    }

    pub fn with_link_target(mut self, target: impl Into<String>) -> Self {
        self.link_target = Some(target.into());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    /// Unix timestamp (seconds) of the last modification, when reported.
    pub fn modified(&self) -> Option<i64> {
        self.modified
    }

    pub fn permissions(&self) -> FilePermissions {
        self.permissions
    }

    pub fn link_target(&self) -> Option<&str> {
        self.link_target.as_deref()
    }
}

/// Directory listing of a path inside a container filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileListing {
    path: String,
    entries: Vec<FileEntry>,
}

impl FileListing {
    pub fn new(path: impl Into<String>, entries: Vec<FileEntry>) -> Result<Self, DomainError> {
        let path = path.into();
        if path.is_empty() {
            return Err(DomainError::InvalidFileListing(
                "listing path cannot be empty".to_string(),
            ));
        }
        Ok(FileListing { path, entries })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_permissions_display_renders_type_and_rwx_bits() {
        assert_eq!(FilePermissions::new(0o100644).display(), "-rw-r--r--");
        assert_eq!(FilePermissions::new(0o040755).display(), "drwxr-xr-x");
        assert_eq!(FilePermissions::new(0o120777).display(), "lrwxrwxrwx");
        assert_eq!(FilePermissions::new(0o100600).display(), "-rw-------");
    }

    #[test]
    fn file_permissions_permission_bits_mask_type_bits() {
        assert_eq!(FilePermissions::new(0o100644).permission_bits(), 0o644);
        assert_eq!(FilePermissions::new(0o040755).permission_bits(), 0o755);
    }

    #[test]
    fn file_entry_new_rejects_empty_name() {
        let result = FileEntry::new(
            "",
            "/file.txt",
            FileKind::File,
            10,
            None,
            FilePermissions::new(0o644),
        );
        assert!(matches!(result, Err(DomainError::InvalidFileEntry(_))));
    }

    #[test]
    fn file_entry_getters_and_link_target_builder() {
        let entry = FileEntry::new(
            "link",
            "/link",
            FileKind::Symlink,
            0,
            Some(1_700_000_000),
            FilePermissions::new(0o120777),
        )
        .unwrap()
        .with_link_target("/target");
        assert_eq!(entry.name(), "link");
        assert_eq!(entry.path(), "/link");
        assert_eq!(entry.kind(), FileKind::Symlink);
        assert!(!entry.is_dir());
        assert_eq!(entry.modified(), Some(1_700_000_000));
        assert_eq!(entry.link_target(), Some("/target"));
        assert_eq!(entry.permissions().display(), "lrwxrwxrwx");
    }

    #[test]
    fn file_listing_new_rejects_empty_path() {
        let entry = FileEntry::new(
            "f",
            "/f",
            FileKind::File,
            1,
            None,
            FilePermissions::new(0o644),
        )
        .unwrap();
        let result = FileListing::new("", vec![entry]);
        assert!(matches!(result, Err(DomainError::InvalidFileListing(_))));
    }

    #[test]
    fn file_listing_exposes_path_and_entries() {
        let entry = FileEntry::new(
            "f",
            "/dir/f",
            FileKind::File,
            1,
            None,
            FilePermissions::new(0o644),
        )
        .unwrap();
        let listing = FileListing::new("/dir", vec![entry]).unwrap();
        assert_eq!(listing.path(), "/dir");
        assert_eq!(listing.entries().len(), 1);
    }
}
