/// Internal scanner state while walking a block of SQL looking for
/// statement-separating semicolons.
enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
    LineComment,
    BlockComment,
}

/// Split a block of SQL into individual statements on top-level `;`
/// characters. A semicolon inside a quoted string, a quoted identifier,
/// a `--` line comment, or a `/* */` block comment does not count as a
/// separator — it's just part of that text. Quote escaping via a
/// doubled quote (`''` inside a string, `""` inside an identifier) is
/// understood so a semicolon right after one isn't misread as ending
/// the quoted text early.
///
/// Empty statements (blank lines, stray semicolons) are dropped, and a
/// trailing statement with no closing semicolon is still included.
pub fn split_statements(sql: &str) -> Vec<String> {
    let chars: Vec<char> = sql.chars().collect();
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut state = State::Normal;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match state {
            State::Normal => match c {
                '\'' => {
                    current.push(c);
                    state = State::SingleQuoted;
                }
                '"' => {
                    current.push(c);
                    state = State::DoubleQuoted;
                }
                '-' if chars.get(i + 1) == Some(&'-') => {
                    current.push('-');
                    current.push('-');
                    i += 1;
                    state = State::LineComment;
                }
                '/' if chars.get(i + 1) == Some(&'*') => {
                    current.push('/');
                    current.push('*');
                    i += 1;
                    state = State::BlockComment;
                }
                ';' => {
                    push_if_non_empty(&mut statements, &current);
                    current.clear();
                }
                _ => current.push(c),
            },
            State::SingleQuoted => {
                current.push(c);
                if c == '\'' {
                    if chars.get(i + 1) == Some(&'\'') {
                        current.push('\'');
                        i += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::DoubleQuoted => {
                current.push(c);
                if c == '"' {
                    if chars.get(i + 1) == Some(&'"') {
                        current.push('"');
                        i += 1;
                    } else {
                        state = State::Normal;
                    }
                }
            }
            State::LineComment => {
                current.push(c);
                if c == '\n' {
                    state = State::Normal;
                }
            }
            State::BlockComment => {
                current.push(c);
                if c == '*' && chars.get(i + 1) == Some(&'/') {
                    current.push('/');
                    i += 1;
                    state = State::Normal;
                }
            }
        }
        i += 1;
    }

    push_if_non_empty(&mut statements, &current);
    statements
}

fn push_if_non_empty(statements: &mut Vec<String>, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_semicolons() {
        let got = split_statements("select 1; select 2;");
        assert_eq!(got, vec!["select 1", "select 2"]);
    }

    #[test]
    fn keeps_trailing_statement_without_semicolon() {
        let got = split_statements("select 1;\nselect 2");
        assert_eq!(got, vec!["select 1", "select 2"]);
    }

    #[test]
    fn ignores_semicolon_inside_single_quoted_string() {
        let got = split_statements("insert into t (v) values ('a;b');");
        assert_eq!(got, vec!["insert into t (v) values ('a;b')"]);
    }

    #[test]
    fn understands_escaped_single_quote() {
        let got = split_statements("insert into t (v) values ('it''s; fine');");
        assert_eq!(got, vec!["insert into t (v) values ('it''s; fine')"]);
    }

    #[test]
    fn ignores_semicolon_inside_quoted_identifier() {
        let got = split_statements("select \"weird;column\" from t;");
        assert_eq!(got, vec!["select \"weird;column\" from t"]);
    }

    #[test]
    fn ignores_semicolon_inside_line_comment() {
        let got = split_statements("select 1; -- trailing; comment\nselect 2;");
        assert_eq!(
            got,
            vec!["select 1", "-- trailing; comment\nselect 2"]
        );
    }

    #[test]
    fn ignores_semicolon_inside_block_comment() {
        let got = split_statements("select 1; /* a; b */ select 2;");
        assert_eq!(got, vec!["select 1", "/* a; b */ select 2"]);
    }

    #[test]
    fn drops_empty_statements() {
        let got = split_statements(";;select 1;;;");
        assert_eq!(got, vec!["select 1"]);
    }

    #[test]
    fn empty_input_yields_no_statements() {
        let got = split_statements("   \n  ");
        assert!(got.is_empty());
    }
}
