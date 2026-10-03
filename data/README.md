# Dataset

## Layout

- `mediawiki/` is the `zhconv-data-mediawiki` crate (GPL-2.0-or-later): `ZhConversion.php` from [MediaWiki](https://github.com/wikimedia/mediawiki/blob/master/includes/Languages/Data/ZhConversion.php) plus an internal parser. It returns structured tables; the parent `build.rs` merges them.
- `opencc/` is the `zhconv-data-opencc` crate (Apache-2.0): `*.txt` from [OpenCC](https://github.com/BYVoid/OpenCC/tree/master/data) plus internal staging/flattening. Same contract.
- `../build.rs` merges both sources into a **single** automaton per target (MediaWiki first, OpenCC appended; earlier rules win), then sorts, dedups and emits `.conv`/`.daac`.

## Files

- `update_basic.py` updates `mediawiki/ZhConversion.php`, `opencc/*.txt` from upstream and relevant references in `mediawiki/src/lib.rs` / `opencc/src/lib.rs` automatically.

- `update_cgroups.py` pulls and formats CGroups (common conversion groups) from Chinese Wikipedia into `cgroups/`. ([source](https://zh.wikipedia.org/w/index.php?search=CGroup&title=Special:%E6%90%9C%E7%B4%A2&profile=advanced&fulltext=1&ns10=1&ns828=1))

- `cgroups/merge_for_web.py` combines `cgroups/*.json` into a monolithic `cgroups.json` to be placed in `:/web/public/cgroups.json`, which is included in the web app. The libs and the cli are not referencing CGroups for now. 

## Licenses

Rulesets are neither maintained nor licensed by zhconv-rs. Check their sources for licenses.

Code vs data: the Rust parsing code in `mediawiki/src/lib.rs` and `opencc/src/lib.rs` is licensed under MIT OR Apache-2.0, same as the parent `zhconv` crate, independent of the datasets. The bundled `ZhConversion.php` is GPL-2.0-or-later (upstream MediaWiki, hence the mediawiki crate-level license); the bundled `*.txt` dictionaries are Apache-2.0 (upstream OpenCC).
