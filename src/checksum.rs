//! Content checksums for drift detection: a migration runner that
//! records the checksum at apply time can later reread the file and
//! flag it if the checksum no longer matches, catching edits made to a
//! migration after it already ran somewhere.
//!
//! FNV-1a is used instead of a cryptographic hash because this crate
//! stays dependency-free and drift detection only needs to notice
//! accidental changes, not resist someone deliberately crafting a
//! collision.

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Hash `up` and `down` together into a single checksum, formatted as
/// 16 lowercase hex digits.
///
/// A NUL separator is inserted between the two sections so that, say,
/// `up = "a", down = Some("b")` and `up = "ab", down = None` hash
/// differently instead of colliding on the concatenated bytes.
pub fn checksum(up: &str, down: Option<&str>) -> String {
    let mut bytes = Vec::with_capacity(up.len() + down.map_or(0, str::len) + 1);
    bytes.extend_from_slice(up.as_bytes());
    bytes.push(0);
    if let Some(down) = down {
        bytes.extend_from_slice(down.as_bytes());
    }
    format!("{:016x}", fnv1a_64(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_checksum() {
        assert_eq!(checksum("a", Some("b")), checksum("a", Some("b")));
    }

    #[test]
    fn different_up_different_checksum() {
        assert_ne!(checksum("a", Some("b")), checksum("aa", Some("b")));
    }

    #[test]
    fn none_vs_empty_string_down_differ() {
        assert_ne!(checksum("a", None), checksum("a", Some("")));
    }

    #[test]
    fn separator_prevents_boundary_collision() {
        assert_ne!(checksum("a", Some("b")), checksum("ab", None));
    }

    #[test]
    fn output_is_sixteen_hex_chars() {
        let c = checksum("select 1;", Some("select 2;"));
        assert_eq!(c.len(), 16);
        assert!(c.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}
