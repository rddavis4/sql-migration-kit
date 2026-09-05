//! Parsing for single-file SQL migrations that carry both an "up" and a
//! "down" section, split on a marker line.
//!
//! The two things a caller needs are here: `Migration::parse` for anything
//! that already has the text in hand, and the `read_*` helpers in
//! `reader` for the common case of getting that text from either a file
//! path or stdin.

mod migration;
mod reader;
mod statement;

pub use migration::{Migration, DOWN_MARKER};
pub use reader::{read_from_path, read_from_stdin, read_source};
pub use statement::split_statements;
