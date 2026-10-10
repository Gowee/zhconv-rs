# Source and dictionary licenses

The binding and the zhconv conversion code are MIT OR Apache-2.0, as clarified
by the maintainer in [issue #15](https://github.com/Gowee/zhconv-rs/issues/15).
The core crate's Cargo license metadata may still describe its default
MediaWiki-based distribution; this package explicitly disables those defaults.

The standard build uses OpenCC dictionaries (Apache-2.0), with compression and
CJK compatibility normalization enabled. It does not embed MediaWiki conversion
tables. The `wikitext` option processes inline rules supplied by the caller; it
does not enable the MediaWiki dataset.

OpenCC: <https://github.com/BYVoid/OpenCC>, copyright the OpenCC contributors.
The pinned source revision and dictionary checksums are maintained by
`data/opencc/src/lib.rs` in the source repository. The package's generated
`build-info.json` records the OpenCC revision actually used, source commit,
resolved core features, Rust compiler, lockfile hash and WASM hash.

Full MIT and Apache-2.0 texts accompany this package. Dependency license texts
(including MIT, Apache-2.0, BSD-3-Clause and Unicode-3.0 notices) are staged in
`licenses/`, conservatively including build dependencies as well. Review
the locked runtime dependency graph before each release; the dictionary-only
feature check does not replace a dependency license review.

This package's manifest describes the binding/code choice together with the
Apache-2.0 dictionaries: `(MIT OR Apache-2.0) AND Apache-2.0`. Neither this notice
nor a new release changes the license of previously published packages.
