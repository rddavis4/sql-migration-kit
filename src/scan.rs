use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::filename::Filename;

/// A migration discovered on disk: the path to read it from, plus the
/// version and name already pulled out of the filename so callers don't
/// have to re-parse it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationFile {
    pub path: PathBuf,
    pub filename: Filename,
}

/// Scan `dir` for migration files and return them in migration order
/// (ascending by version).
///
/// Only direct entries are considered — subdirectories are not
/// descended into. Entries whose name doesn't parse as a migration
/// filename (no leading version digits) are skipped rather than
/// treated as an error, since a migrations directory commonly also
/// holds a README or a schema dump alongside the numbered files.
pub fn scan_dir<P: AsRef<Path>>(dir: P) -> io::Result<Vec<MigrationFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if let Some(filename) = Filename::parse(&path) {
            files.push(MigrationFile { path, filename });
        }
    }
    files.sort_by(|a, b| {
        compare_versions(&a.filename.version, &b.filename.version)
            .then_with(|| a.filename.name.cmp(&b.filename.name))
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(files)
}

/// Compare two version strings numerically rather than lexicographically.
/// `Filename::parse` guarantees both are composed entirely of ASCII
/// digits, so a shorter string is always numerically smaller regardless
/// of leading zeros, and equal-length strings sort the same way as
/// strings and as numbers. This is what lets `0002` and
/// `20240102150405`-style timestamp versions coexist in one directory
/// without either scheme breaking the other's order.
fn compare_versions(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> TempDir {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("sql-migration-kit-test-{nanos}"));
            fs::create_dir(&dir).unwrap();
            TempDir(dir)
        }

        fn write(&self, name: &str, contents: &str) {
            fs::write(self.0.join(name), contents).unwrap();
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn orders_by_version_ascending() {
        let dir = TempDir::new();
        dir.write("0003_add_index.sql", "select 1;");
        dir.write("0001_create_accounts.sql", "select 1;");
        dir.write("0002_add_column.sql", "select 1;");

        let files = scan_dir(dir.path()).unwrap();
        let versions: Vec<&str> = files.iter().map(|f| f.filename.version.as_str()).collect();
        assert_eq!(versions, vec!["0001", "0002", "0003"]);
    }

    #[test]
    fn orders_short_and_long_versions_numerically() {
        let dir = TempDir::new();
        dir.write("9_late_small_number.sql", "select 1;");
        dir.write("20240102150405_add_index.sql", "select 1;");
        dir.write("10_next.sql", "select 1;");

        let files = scan_dir(dir.path()).unwrap();
        let versions: Vec<&str> = files.iter().map(|f| f.filename.version.as_str()).collect();
        assert_eq!(versions, vec!["9", "10", "20240102150405"]);
    }

    #[test]
    fn skips_non_migration_files() {
        let dir = TempDir::new();
        dir.write("0001_create_accounts.sql", "select 1;");
        dir.write("README.md", "not a migration");

        let files = scan_dir(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename.name, "create_accounts");
    }

    #[test]
    fn ignores_subdirectories() {
        let dir = TempDir::new();
        dir.write("0001_create_accounts.sql", "select 1;");
        fs::create_dir(dir.path().join("0002_looks_like_a_migration")).unwrap();

        let files = scan_dir(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn empty_directory_yields_empty_set() {
        let dir = TempDir::new();
        let files = scan_dir(dir.path()).unwrap();
        assert!(files.is_empty());
    }
}
