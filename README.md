# sql-migration-kit

A small Rust library for working with SQL migration files that keep the
"up" and "down" SQL in one file, separated by a marker line:

```sql
create table accounts (
    id integer primary key,
    email text not null unique
);

-- migrate:down
drop table accounts;
```

Most migration runners want the file on disk, but plenty of other tools
that touch migrations don't have a stable path to hand over: a
pre-commit hook validating the staged diff, an editor plugin linting the
current buffer, a CI step that assembles a migration from a template and
pipes it straight into the checker. This library treats "read from a
file" and "read from stdin" as the same operation, so callers don't have
to special-case either one.

## Usage

Read from a path:

```rust
use std::path::Path;
use sql_migration_kit::read_from_path;

let migration = read_from_path(Path::new("migrations/0001_accounts.sql"))?;
println!("{}", migration.up);
if let Some(down) = &migration.down {
    println!("{}", down);
}
# Ok::<(), std::io::Error>(())
```

Read from stdin:

```rust
use sql_migration_kit::read_from_stdin;

let migration = read_from_stdin()?;
# Ok::<(), std::io::Error>(())
```

Let the caller decide, with stdin as the fallback (the common shape for
a CLI flag like `--file <PATH>` that's optional):

```rust
use std::path::PathBuf;
use sql_migration_kit::read_source;

fn load(file: Option<PathBuf>) -> std::io::Result<()> {
    let migration = read_source(file.as_deref())?;
    println!("{} bytes of up SQL", migration.up.len());
    Ok(())
}
```

Or parse text you already have, no I/O involved:

```rust
use sql_migration_kit::Migration;

let migration = Migration::parse("create table t (id int);");
assert_eq!(migration.down, None);
```

Break a section into individual statements, for runners that execute
one at a time instead of handing the whole block to the driver:

```rust
use sql_migration_kit::Migration;

let migration = Migration::parse("create table t (id int);\ninsert into t values (1);");
for statement in migration.up_statements() {
    println!("{statement}");
}
```

The splitter tracks single-quoted strings, double-quoted identifiers,
`--` line comments, and `/* */` block comments, so a semicolon inside
any of those doesn't end the statement early.

Pull the version and name out of a migration filename:

```rust
use sql_migration_kit::Filename;

let f = Filename::parse("0001_create_accounts.sql").unwrap();
assert_eq!(f.version, "0001");
assert_eq!(f.name, "create_accounts");
```

The version is left as a string rather than a number, so leading
zeros survive and timestamp-style versions
(`20240102150405_add_index.sql`) work the same way as small integer
ones. A bare version with no name (`0001.sql`) parses fine and just
has an empty name. Filenames with no leading digits return `None`.

Scan a directory for migration files, in order:

```rust
use sql_migration_kit::scan_dir;

let files = scan_dir("migrations")?;
for file in &files {
    println!("{} {}", file.filename.version, file.filename.name);
}
# Ok::<(), std::io::Error>(())
```

Files are ordered by version, treating each version as a number rather
than as text, so a directory can mix short sequential versions
(`0001`, `0002`, ...) with timestamp-style ones without the two
schemes fighting over sort order. Entries that don't parse as a
migration filename (a README, a schema dump) are skipped, and
subdirectories aren't descended into.

## Status

Early skeleton. The parser does the up/down split, statement-level
splitting within each section, filename/version parsing, and ordered
directory scanning. Still missing: checksums and a richer error type.
See the crate for what's implemented so far.

## License

MIT
