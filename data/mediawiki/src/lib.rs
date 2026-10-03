//! Raw MediaWiki `ZhConversion.php` dataset and parser.
//!
//! License note: the Rust parsing code in this file is licensed under
//! MIT OR Apache-2.0, same as the parent `zhconv` crate. It is independent
//! of the bundled dataset `ZhConversion.php`, which is licensed under
//! GPL-2.0-or-later (upstream MediaWiki) — hence the crate-level license
//! is GPL-2.0-or-later.
//! The parsed tables are merged with OpenCC tables by `zhconv`'s build script
//! into a single automaton; see the parent crate's `build.rs`.

use std::collections::HashMap;

use hex_literal::hex;

// To update the upstream ruleset, run `data/update_basic.py` and `cargo fmt`.
pub const MEDIAWIKI_COMMIT: &str = "ecf4342132cf089ac0c42436827e9038a738bb6f";
pub const MEDIAWIKI_SHA256: [u8; 32] =
    hex!("5cb0019b32bb39ec5c6e662029f90bd166f7a844efb3bc877f9be41fdd511bf2");

/// Raw `ZhConversion.php` text, validated on access via [`raw`].
const RAW: &str = include_str!("../ZhConversion.php");

fn sha256(text: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().into()
}

/// Return the raw dataset text after validating its SHA256 checksum.
pub fn raw() -> &'static str {
    assert_eq!(
        &sha256(RAW),
        &MEDIAWIKI_SHA256,
        "Validating the checksum of ZhConversion.php"
    );
    RAW
}

/// Parse the dataset into per-target conversion pairs.
///
/// Keys are `ZH_TO_HANS`, `ZH_TO_HANT`, `ZH_TO_CN`, `ZH_TO_TW`, `ZH_TO_HK`.
/// Pairs are in upstream order (earlier rules take precedence); sorting and
/// dedup happen in the parent crate's build script after merging.
pub fn parse() -> HashMap<String, Vec<(String, String)>> {
    parse_mediawiki(raw())
}

fn parse_mediawiki(text: &str) -> HashMap<String, Vec<(String, String)>> {
    let patb = regex::Regex::new(r"public const (\w+) = \[([^]]+)\]?;").unwrap();
    let patl = regex::Regex::new(r"'(.+?)' *=> *'(.+?)' *,?\n").unwrap();
    let mut res = HashMap::new();

    for block in patb.captures_iter(text) {
        let name = block.get(1).unwrap().as_str();
        let body = block.get(2).unwrap().as_str();
        let mut pairs = vec![];
        for line in patl.captures_iter(body) {
            let from = line.get(1).unwrap().as_str();
            let to = line.get(2).unwrap().as_str();
            pairs.push((from.to_owned(), to.to_owned()));
        }
        assert!(res.insert(name.to_owned(), pairs).is_none());
    }
    for name in [
        "ZH_TO_HANS",
        "ZH_TO_HANT",
        "ZH_TO_CN",
        "ZH_TO_TW",
        "ZH_TO_HK",
    ] {
        assert!(res.contains_key(name));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_is_validated_and_nonempty() {
        assert!(!raw().is_empty());
    }

    #[test]
    fn parse_covers_all_targets() {
        let tables = parse();
        for name in [
            "ZH_TO_HANS",
            "ZH_TO_HANT",
            "ZH_TO_CN",
            "ZH_TO_TW",
            "ZH_TO_HK",
        ] {
            let pairs = tables
                .get(name)
                .unwrap_or_else(|| panic!("missing {}", name));
            assert!(!pairs.is_empty(), "{} should not be empty", name);
        }
    }
}
