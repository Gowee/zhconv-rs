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

const COMMIT: &str = "2682461394ee5c631b48f1cec8c0328634152558";
const SHA256: [u8; 32] = hex!("0f0979dc3041c68884a31d3bbd181d30d3b95ad77cfa110404c59e794bf7df4b");
const URL: &str = formatcp!("https://raw.githubusercontent.com/wikimedia/mediawiki/{}/includes/languages/data/ZhConversion.php", COMMIT);

fn main() {
    // let URL = format!("https://raw.githubusercontent.com/wikimedia/mediawiki/{}/includes/languages/data/ZhConversion.php", COMMIT);

    let zhconv = fetch_zhconv();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    // let dest_path = Path::new(&out_dir).join("convs.rs");
    // let mut f = File::create(&dest_path).unwrap();
    // writeln!(f, "// Generated from zhConversion.php at {}\n", COMMIT).unwrap();

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

        // writeln!(f, "fn gen_{}() -> ZhConverter {{", name).unwrap();
        // writeln!(f, "let mut m = HashMap::with_capacity({});", pairs.len()).unwrap();

        // write!(f, r#"let p = Regex::new("("#).unwrap();
        // let mut it = pairs.iter().peekable();
        // while let Some((from, _to)) = it.next() {
        //     assert!(!from.contains('|'));
        //     write!(f, "{}", from).unwrap();
        //     if it.peek().is_some() {
        //         write!(f, "|").unwrap();
        //     }
        // }
        // write!(f, r#")").unwrap();"#).unwrap();

        // let mut it = pairs.iter(); //.peekable();
        // while let Some((from, to)) = it.next() {
        //     assert!(!from.contains('"') && !to.contains('"'));
        //     writeln!(f, r#"m.insert("{}".to_owned(), "{}".to_owned());"#, from, to).unwrap();
        //     // if it.peek().is_some() {
        //     //     write!(f, ", ").unwrap();
        //     // }
        // }
        // writeln!(f, "\nZhConverter::new(p, m)\n}}").unwrap();

        // writeln!(f, "lazy_static! {{").unwrap();
        // writeln!(f, r"pub static ref {name}Converter: ZhConverter = gen_{name}();", name=name).unwrap();

        // writeln!(f, "}}").unwrap();
    }

    // fs::write(
    //     &dest_path,
    //     &zhconv
    // ).unwrap();
    vergen(VergenConfig::default()).expect("vergen");
    println!("cargo:rustc-env=MEDIAWIKI_COMMIT_HASH={}", COMMIT);
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=zhConversion.php");
}

fn fetch_zhconv() -> String {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("zhConversion.php");

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
