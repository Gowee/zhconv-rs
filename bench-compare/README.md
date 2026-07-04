# `zhconv-bench-compare`

Comparative benchmarks for [`zhconv-rs`](../) against other Chinese conversion
crates (`opencc-rust`, `ferrous-opencc`, `fast2s`, `opencc-fmmseg`).

Standalone crate; not a workspace member. Requires OpenCC C library ≤ 1.2.

Run with: `cd bench-compare && cargo bench --features bench,mediawiki,opencc`.