# zhconv-typst

Convert Chinese strings and nested content between simplified, traditional and
regional variants using [zhconv-rs](https://github.com/Gowee/zhconv-rs).

## Published package and development version

The currently published Universe package is `@preview/zhconv:0.3.1`. This source
tree prepares an updated package; it is **not** that published release. The
manifest's `0.4.0` version is the proposed Typst package version, independent of
the Rust core's version. The core dependency comes from this checkout (currently
0.5 development), not a crates.io 0.5 release.

After building and installing this development package locally:

```typst
#import "@local/zhconv:0.4.0": zhconv

#zhconv("汉字转换", "zh-Hant") // 漢字轉換
#zhconv([柳外輕雷池上雨], "zh-Hans")
#zhconv("-{zh-hans:甲;zh-hant:乙;}-", "zh-Hant", wikitext: true) // 乙
```

## API and conversion behavior

`zhconv(document, target, wikitext: false)` keeps the existing API. Strings return
strings. Content is traversed using the existing `children`, `text` and `body`
handling; other values pass through unchanged. The lower-level helpers
`zhconv-str(text, target, wikitext: false)` and `is-hans-str(text)` remain available.

Targets are case-insensitive: `zh-Hans`, `zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`,
`zh-CN`, `zh-SG`, `zh-MY`. The upstream `zh` target is also accepted (no script
conversion in plain mode). Unknown targets and invalid UTF-8 at the byte-level
plugin boundary produce errors. Wikitext processing is opt-in; its wire flag
must be exactly one byte, either 0 or 1.

The standard build explicitly enables **OpenCC, compression and CJK compatibility
normalization**, with default core features disabled. It embeds no MediaWiki
conversion tables. This differs from the older published package: some outputs
will change, even though the function signatures remain compatible.

- Hans/Hant select script conversion rules; TW/HK additionally select their
  regional character rules. MO shares HK's rules; SG/MY share CN's rules.
- The optional `opencc-twp` and `opencc-hkp` phrase-replacement features are not
  enabled. Selecting TW is not a promise to localize every technical term.
  For example, the test corpus expects `阿拉伯联合酋长国` → `阿拉伯聯合酋長國`
  for TW, without the alternative regional phrase substitution.
- These are zhconv's flattened OpenCC automata, not OpenCC's original multi-stage
  pipeline. Individual conversions can differ. Conversion is neither generally
  reversible nor a guarantee of linguistic correctness in every context.
- Wikitext syntax remains supported without embedding MediaWiki's dataset.
- Content traversal is a convenience, not a lossless rewrite of arbitrary Typst
  content. In particular, explicit text styling can be lost, and words split
  across content nodes are converted separately. Convert a complete string
  before applying formatting when phrase context or styling matters.
- This API does not parse placeholders, URLs, source code or identifiers. Chinese
  text inside them can be converted. Protect such syntax in the calling package.

## Migration examples and known limitations

The following differences were observed against the published 0.3.1 WASM under
Typst 0.15.1. They reflect both dictionary selection and changes in the core,
not an API change. A 12-case, eight-target smoke corpus is not a quality score.

| Input / target | Published 0.3.1 | This OpenCC-only build |
| --- | --- | --- |
| `软件在服务器上运行，鼠标和内存。` / TW | 軟體在伺服器上運行，滑鼠和內存。 | 軟件在服務器上運行，鼠標和內存。 |
| `阿拉伯联合酋长国` / TW | 阿拉伯聯合大公國 | 阿拉伯聯合酋長國 |
| `神 神 﨑 崎` / Hans | 神 神 﨑 崎 | 神 神 﨑 崎 |

The current core also has known conversion limitations exposed by these tests:

- For TW, `簡繁混排：后台與後臺，里面和裡面。` becomes
  `簡繁混排：後臺與後臺，裡面和裡麵。`, incorrectly changing the last `面`.
- For HK, `理发以后发现头发干了，干活的人在后台。` becomes
  `理髮以後發現頭髮幹了，幹活的人在後枱。`. The contextual readings of
  `干了` and `后台` are not reliably resolved.

Native/WASM agreement on these cases is only an integration check, not an
endorsement of the output. Review these examples when the core changes, and
resolve or explicitly accept the limitations before publishing. This binding
does not apply ad hoc replacements to hide core/dictionary behavior.

## Build and test

Requirements: Git checkout of this repository, rustup, Python 3.11 or later,
and Typst 0.14.0 or later. `rust-toolchain.toml` pins the Rust toolchain and Wasm
target; `Cargo.lock` pins dependency resolution. No WASI stubber, wasm-bindgen CLI,
Node.js or Python dependencies are needed. Run from `typst/`:

```sh
python3 tools/build.py
python3 tests/run.py
# Test with another compiler executable:
python3 tests/run.py --typst /path/to/typst
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
```

The builder checks resolved dependency features for MediaWiki, verifies that the
Wasm imports only the two Typst protocol functions, and stages an explicit file
list in `dist/package/`. It also copies dependency license texts and writes
`build-info.json` with provenance and hashes. The test runner checks malformed
plugin inputs, manually reviewed semantic fixtures and 96 comparisons with the
same-configuration native Rust implementation. It installs only the staged
package into a temporary package path and compiles outside the source checkout,
including content examples and expected-error cases. Native agreement validates
the binding, not linguistic accuracy; the small semantic corpus is separate.

To install manually, copy `dist/package/` to
`<Typst data directory>/typst/packages/local/zhconv/0.4.0/` (or the package path
reported by `typst info`). Then use the `@local` import above. The test runner
requires no persistent installation. `example.typ` can also be compiled after
copying the built WASM next to `zhconv.typ` for source-tree development.

## Release checklist

1. Review changes to the core/data revision and the semantic fixtures. Investigate
   changed outputs rather than blindly regenerating expected values. Check both
   the minimum and current Typst versions in CI.
2. Review dependency licenses and the generated provenance. Keep the manifest,
   LICENSE files and dictionary configuration consistent. Source-level feature
   changes can affect both output and binary licensing.
3. Choose the package version with the maintainer, update the manifest and local
   imports/tests together, and rebuild from a clean committed checkout. Repeat
   with a clean Cargo target directory under the pinned toolchain and compare
   WASM hashes. The hash is not promised to be stable across toolchain versions.
4. Submit only the contents of `dist/package/` to the corresponding version
   directory in `typst/packages`, including the WASM, notices and license texts.
   Never copy a stale WASM from an earlier build. Published versions are unchanged.

The scripts do not publish packages. The maintainer reviews and approves the
release separately. See [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) for the
code/data licensing split and [issue #15](https://github.com/Gowee/zhconv-rs/issues/15)
for the maintenance discussion.
