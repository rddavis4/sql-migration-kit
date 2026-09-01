/// A line that, on its own, separates the "up" SQL from the "down" SQL
/// within a single migration file. Chosen to be a valid SQL comment so
/// the file still runs as-is if fed straight to a client without being
/// split first.
pub const DOWN_MARKER: &str = "-- migrate:down";

/// The two halves of a migration. `down` is `None` when the file never
/// contained a marker line, i.e. the migration is one-directional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub up: String,
    pub down: Option<String>,
}

impl Migration {
    /// Split `input` into up/down sections on the first line that is
    /// exactly `DOWN_MARKER` once surrounding whitespace is trimmed.
    /// Everything before that line is `up`; everything after is `down`.
    /// A second marker line later in the file is treated as ordinary SQL
    /// text, not a second split point.
    pub fn parse(input: &str) -> Migration {
        let lines: Vec<&str> = input.lines().collect();
        match lines.iter().position(|line| line.trim() == DOWN_MARKER) {
            Some(idx) => Migration {
                up: lines[..idx].join("\n").trim().to_string(),
                down: Some(lines[idx + 1..].join("\n").trim().to_string()),
            },
            None => Migration {
                up: input.trim().to_string(),
                down: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_marker() {
        let m = Migration::parse("create table t (id int);\n-- migrate:down\ndrop table t;");
        assert_eq!(m.up, "create table t (id int);");
        assert_eq!(m.down.as_deref(), Some("drop table t;"));
    }

    #[test]
    fn no_marker_means_up_only() {
        let m = Migration::parse("create table t (id int);");
        assert_eq!(m.up, "create table t (id int);");
        assert_eq!(m.down, None);
    }

    #[test]
    fn marker_needs_its_own_line() {
        let m = Migration::parse("select 1; -- migrate:down comment");
        assert_eq!(m.down, None);
    }
}
