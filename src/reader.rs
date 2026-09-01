use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::migration::Migration;

/// Read and parse a migration from anything that implements `Read`. Both
/// file-backed and stdin-backed reads go through this so the parsing
/// logic only has to be written once.
fn parse_reader<R: Read>(mut reader: R) -> io::Result<Migration> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    Ok(Migration::parse(&buf))
}

/// Read a migration from a file on disk.
pub fn read_from_path<P: AsRef<Path>>(path: P) -> io::Result<Migration> {
    let file = File::open(path)?;
    parse_reader(file)
}

/// Read a migration from stdin. Useful for pre-commit hooks and editor
/// integrations that pipe in buffer contents that were never saved.
pub fn read_from_stdin() -> io::Result<Migration> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    parse_reader(handle)
}

/// Read a migration from `path` if given, otherwise fall back to stdin.
/// This is the shape most callers want: a CLI flag like `--file` that is
/// optional, with stdin as the default when it's left off.
pub fn read_source(path: Option<&Path>) -> io::Result<Migration> {
    match path {
        Some(p) => read_from_path(p),
        None => read_from_stdin(),
    }
}
