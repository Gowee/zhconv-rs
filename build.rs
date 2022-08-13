use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use const_format::formatcp;
use hex_literal::hex;
use itertools::Itertools;
use regex::Regex;
use reqwest::blocking as reqwest;
use sha2::{Digest, Sha256};
use vergen::{vergen, Config as VergenConfig};

const COMMIT: &str = "56417313aa08801ef4b737b40bb7e436c2160d0a";
const SHA256: [u8; 32] = hex!("2c61d46f4412f883d324defcc7447acb69929ad3502d2edde3a1ac0261d03a99");
const URL: &str = formatcp!("https://raw.githubusercontent.com/wikimedia/mediawiki/{}/includes/languages/data/ZhConversion.php", COMMIT);

fn main() {
    let zhconv = fetch_zhconv();

    let out_dir = env::var_os("OUT_DIR").unwrap();

    for (name, mut pairs) in parse(&zhconv).into_iter() {
        let dest_path_from = Path::new(&out_dir).join(format!("{}.from.conv", name));
        let dest_path_to = Path::new(&out_dir).join(format!("{}.to.conv", name));
        let mut ffrom = File::create(&dest_path_from).unwrap();
        let mut fto = File::create(&dest_path_to).unwrap();

        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let olen = pairs.len();
        pairs.dedup();
        assert_eq!(olen, pairs.len(), "deduping pairs of {}", name);
        assert_eq!(
            pairs.len(),
            pairs
                .iter()
                .map(|(from, _to_)| from)
                .collect::<HashSet<_>>()
                .len(),
            "deduping keys of {}",
            name
        );

        for e in Itertools::intersperse(pairs.iter().map(|(from, _to)| from.as_str()), "|") {
            write!(ffrom, "{}", e).unwrap();
        }
        for e in Itertools::intersperse(pairs.iter().map(|(_from, to)| to.as_str()), "|") {
            write!(fto, "{}", e).unwrap();
        }
    }

    if std::env::var("DOCS_RS").is_err() {
        // vergen panics in docs.rs. It is only used by wasm.rs for now.
        // So it is ok to disable it in docs.rs.

        // Note: conditional compilation tricks won't be effective since it is cross compiling here.
        // Ref:
        //   https://kazlauskas.me/entries/writing-proper-buildrs-scripts
        //   https://github.com/rust-lang/cargo/issues/4302
        // #[cfg(target_arch = "wasm32")] #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        if env::var("CARGO_CFG_TARGET_ARCH") == Ok("wasm32".to_owned()) {
            vergen(VergenConfig::default())
                .unwrap_or_else(|e| println!("cargo:warning=vergen failed: {:?}", e));
        }
    }
    println!("cargo:rustc-env=MEDIAWIKI_COMMIT_HASH={}", COMMIT);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/zhConversion.php");
}

fn fetch_zhconv() -> String {
    let out_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data/zhConversion.php");

    if let Some(content) = fs::read(&dest_path)
        .ok()
        .and_then(|s| String::from_utf8(s).ok())
        .and_then(|s| if sha256(&s) == SHA256 { Some(s) } else { None })
    {
        content
    } else {
        let content = reqwest::get(URL).unwrap().text().unwrap();
        assert_eq!(
            sha256(&content),
            SHA256,
            "Validating the checksum of zhconv"
        );
        fs::write(&dest_path, &content).unwrap();
        content
    }
}

fn parse(text: &str) -> HashMap<String, Vec<(String, String)>> {
    let patb = Regex::new(r"public static \$(\w+) = \[([^]]+)\]?;").unwrap();
    let patl = Regex::new(r"'(.+?)' *=> *'(.+?)' *,?\n").unwrap();
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
    res
}

fn sha256(text: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().try_into().unwrap()
}
