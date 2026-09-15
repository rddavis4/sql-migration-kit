use crate::statement::split_statements;

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

    /// The `up` section broken into individual statements. See
    /// `split_statements` for what counts as a separator.
    pub fn up_statements(&self) -> Vec<String> {
        split_statements(&self.up)
    }

    /// The `down` section broken into individual statements, or an
    /// empty list if the migration has no down section.
    pub fn down_statements(&self) -> Vec<String> {
        match &self.down {
            Some(down) => split_statements(down),
            None => Vec::new(),
        }
    }

    /// A checksum over the parsed `up` and `down` sections, as 16
    /// lowercase hex digits. Two migrations with the same checksum have
    /// the same effective content; a mismatch against a previously
    /// recorded checksum means the file changed after it was applied.
    /// Not cryptographically secure — see `checksum` module docs.
    pub fn checksum(&self) -> String {
        crate::checksum::checksum(&self.up, self.down.as_deref())
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

    #[test]
    fn up_and_down_statements_are_split() {
        let m = Migration::parse(
            "create table t (id int);\ninsert into t values (1);\n-- migrate:down\ndrop table t;",
        );
        assert_eq!(
            m.up_statements(),
            vec!["create table t (id int)", "insert into t values (1)"]
        );
        assert_eq!(m.down_statements(), vec!["drop table t"]);
    }

    #[test]
    fn down_statements_empty_when_no_down_section() {
        let m = Migration::parse("select 1;");
        assert!(m.down_statements().is_empty());
    }

    #[test]
    fn checksum_is_stable_across_reparses() {
        let text = "create table t (id int);\n-- migrate:down\ndrop table t;";
        assert_eq!(Migration::parse(text).checksum(), Migration::parse(text).checksum());
    }

    #[test]
    fn checksum_changes_when_content_changes() {
        let a = Migration::parse("create table t (id int);");
        let b = Migration::parse("create table t (id integer);");
        assert_ne!(a.checksum(), b.checksum());
    }
}
