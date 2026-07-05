# `@zhconv/cli` & `@zhconv/cli-opencc`

Native CLI for [zhconv-rs](https://github.com/Gowee/zhconv-rs) — convert Chinese text between Traditional, Simplified and regional variants. Powered by MediaWiki and OpenCC rulesets with an Aho-Corasick automaton for single-pass linear-time conversion.

## Install / Use

```sh
# Default (MediaWiki dicts, smaller binary)
npx @zhconv/cli zh-tw < input.txt

# OpenCC variant (MediaWiki + OpenCC dicts, larger binary)
npx @zhconv/cli-opencc zh-tw < input.txt
```

Requires Node.js 18+. The first run downloads the matching native binary for your platform (linux x64/arm64, darwin x64/arm64, windows x64/arm64).

## Variants

- **`@zhconv/cli`** — uses MediaWiki conversion tables (smaller binary, ~10 MB).
- **`@zhconv/cli-opencc`** — includes BOTH MediaWiki and OpenCC tables (larger binary, ~20 MB). This matches Python's `zhconv-rs-opencc` package: it adds OpenCC dicts to the default MediaWiki dicts rather than replacing them. Use this if you need OpenCC-specific phrases (e.g., TW colloquialisms).

## Usage

```sh
zhconv VARIANT [FILE...]
zhconv zh-Hant                 # stdin → stdout
zhconv zh-tw file.txt          # in-place edit
zhconv --rule "X => Y" zh-cn   # custom rule
zhconv --rules_file rules.txt zh-tw *.md
zhconv --wikitext zh-mo article.txt
zhconv --dump-table zh-hk      # print built-in table
```

Supported variants: `zh`, `zh-Hant`, `zh-Hans`, `zh-TW`, `zh-HK`, `zh-MO`, `zh-CN`, `zh-SG`, `zh-MY`.

## How it works

`@zhconv/cli` is a tiny launcher (Node.js) that resolves a platform-specific `@zhconv/cli-<platform>` sub-package containing the actual Rust binary, then spawns it. npm's `optionalDependencies` mechanism auto-installs only the binary for your platform — no download scripts, no `curl | bash`.

## License

GPL-2.0-or-later (same as the underlying Rust crate). Conversion tables sourced from MediaWiki (GPLv2.0+) and OpenCC (Apache-2.0).

## Related

- [`zhconv`](https://www.npmjs.com/package/zhconv) — JS/wasm library for browser/Node usage
- [`zhconv-web`](https://www.npmjs.com/package/zhconv-web) — browser ESM build
- [`zhconv-rs`](https://pypi.org/project/zhconv-rs/) — Python bindings
- [GitHub repository](https://github.com/Gowee/zhconv-rs)