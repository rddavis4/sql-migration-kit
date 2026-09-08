use std::path::Path;

/// The version and name pulled out of a migration filename, e.g.
/// `0001_create_accounts.sql` becomes version `"0001"` and name
/// `"create_accounts"`.
///
/// The version is kept as a string rather than parsed into a number so
/// that leading zeros survive round-tripping and so timestamp-style
/// versions (`20240102150405_add_index.sql`) don't need special
/// handling — callers that want numeric ordering can parse it
/// themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filename {
    pub version: String,
    pub name: String,
}

impl Filename {
    /// Parse the version and name out of a migration filename or path.
    /// Only the file stem is considered, so a full path or a bare
    /// filename both work, and any extension is ignored.
    ///
    /// Returns `None` if the stem doesn't start with at least one ASCII
    /// digit — there's no version to extract, so this isn't a
    /// migration filename in the expected shape.
    ///
    /// The separator between version and name (an underscore or a
    /// dash) is optional and, when present, stripped. A version with
    /// no name at all (`0001.sql`) parses to an empty name rather than
    /// failing, since the version is still meaningful on its own.
    pub fn parse<P: AsRef<Path>>(path: P) -> Option<Filename> {
        let path = path.as_ref();
        let stem = path.file_stem()?.to_str()?;
        let digits = stem.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 {
            return None;
        }
        let (version, rest) = stem.split_at(digits);
        let name = rest.trim_start_matches(['_', '-']).to_string();
        Some(Filename {
            version: version.to_string(),
            name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_underscore_separated_name() {
        let f = Filename::parse("0001_create_accounts.sql").unwrap();
        assert_eq!(f.version, "0001");
        assert_eq!(f.name, "create_accounts");
    }

    #[test]
    fn parses_dash_separated_name() {
        let f = Filename::parse("0001-create-accounts.sql").unwrap();
        assert_eq!(f.version, "0001");
        assert_eq!(f.name, "create-accounts");
    }

    #[test]
    fn parses_timestamp_version() {
        let f = Filename::parse("20240102150405_add_index.sql").unwrap();
        assert_eq!(f.version, "20240102150405");
        assert_eq!(f.name, "add_index");
    }

    #[test]
    fn keeps_leading_zeros() {
        let f = Filename::parse("0007_seed.sql").unwrap();
        assert_eq!(f.version, "0007");
    }

    #[test]
    fn version_only_has_empty_name() {
        let f = Filename::parse("0001.sql").unwrap();
        assert_eq!(f.version, "0001");
        assert_eq!(f.name, "");
    }

    #[test]
    fn no_leading_digits_is_none() {
        assert_eq!(Filename::parse("create_accounts.sql"), None);
    }

    #[test]
    fn ignores_directory_components() {
        let f = Filename::parse("migrations/0001_create_accounts.sql").unwrap();
        assert_eq!(f.version, "0001");
        assert_eq!(f.name, "create_accounts");
    }

    #[test]
    fn ignores_extension() {
        let f = Filename::parse("0001_create_accounts.up.sql").unwrap();
        assert_eq!(f.version, "0001");
        assert_eq!(f.name, "create_accounts.up");
    }
}
