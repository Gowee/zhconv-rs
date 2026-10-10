//! Produce same-configuration native results for the Typst integration tests.
use serde_json::{json, Value};
use zhconv::{zhconv, zhconv_mw, Variant};

fn main() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/corpus.json")).unwrap();
    let mut results = Vec::new();
    for case in cases {
        let text = case["text"].as_str().unwrap();
        let wikitext = case["wikitext"].as_bool().unwrap_or(false);
        for target in [
            "zh-Hans", "zh-Hant", "zh-TW", "zh-HK", "zh-MO", "zh-CN", "zh-SG", "zh-MY",
        ] {
            let variant: Variant = target.parse().unwrap();
            let output = if wikitext {
                zhconv_mw(text, variant)
            } else {
                zhconv(text, variant)
            };
            if let Some(expected) = case["expected"].get(target) {
                assert_eq!(output, expected.as_str().unwrap(), "{text:?} -> {target}");
            }
            results.push(
                json!({"text": text, "target": target, "wikitext": wikitext, "expected": output}),
            );
        }
    }
    println!("{}", serde_json::to_string(&results).unwrap());
}
