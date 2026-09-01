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

## Status

Early skeleton. The parser currently does the up/down split and nothing
else — no statement-level splitting, no filename/version parsing, no
directory scanning. See the crate for what's implemented so far.

## License

MIT
