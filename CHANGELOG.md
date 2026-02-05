# Changelog

All notable changes to this project will be documented in this file. 
(CAUTION: primarily vibed by Gemini)

## v0.4.1 (2026-02-05)

### Core & CLI
- **Simplified Conversion Optimizations**: Omit "!HKVariants" and "!TWVariants" from OpenCC cofnig when targeting Simplified Chinese to avoid misconversion.
- **Improved Variant Inference**: Enhanced accuracy by omitting no-op pairs during character counting.
- **Fine-grained Feature Gating**: Refactored conversion table features for granular control over ruleset inclusion.
- **Dependency Cleanup**: Removed `once_cell` in favor of standard library features.
- **Data Sync**: Synchronized conversion tables with the latest upstream rules.

### Web Application
- **Frontend Modernization**: Upgraded the stack to **React 19**, **Material UI v7**, and **Vite**.
- **Enhanced Ruleset Selection**: Added support for toggling between MediaWiki and OpenCC rulesets, or using both concurrently.
- **Dynamic Variant Filtering**: Automatically hides target variants unsupported by the current ruleset.
- **File Conversion**: Added the ability to convert local text files directly in the browser.
- **Refactor Codebase**: Refactored the codebase with ContextProvider to improve performance and maintainability.
- **Infrastructure**: Migrated to **Yarn Berry (v4)** and enforced stricter ESLint/Prettier rules.

## v0.4.0 (2025-12-02)

- **Feature Gating**: Disabled WASM feature by default to be friendly as a library.
- **OpenCC Integration**: Bundled OpenCC binaries in releases and fixed CI build issues.
- **Benchmarking**: Added comprehensive benchmark comparisons with other conversion crates.
- **Maintenance**: Updated rulesets from upstream and fixed outdated CI configurations.

## v0.3.3 (2025-01-15)

- **MediaWiki Alignment**: Fixed nested rules handling to better match MediaWiki's behavior.
- **Worker Enhancements**: Made request body limit configurable and optimized API query parameter representation.
- **Wasm/Web**: Added conversion triggers upon option changes.

## v0.3.2 (2024-11-27)

- **New Platform**: Initialized **Typst** integration with kebab-case function naming.
- **PyO3**: Refined CI and module naming for Python bindings.
- **Bug Fixes**: Resolved panics in `convert_to_with` when skipping characters at boundaries.

## v0.3.1 (2024-03-22)

- **Optional Rulesets**: Made OpenCC optional in WASM builds.
- **Conversion Safety**: Applied more conservative rules for one-to-many conversions.
- **Performance**: Documented build size changes and performance improvements in the v0.3 series.

## v0.3.0 (2023-07-05)

- **Performance Engine**: Switched to `daachorse` (Aho-Corasick) for significantly faster linear-time conversions.
- **Compression**: Implemented ruleset compression to reduce binary size.
- **Variant Inference**: Introduced a more intuitive method for `infer_variant_confidence`.
- **MediaWiki Support**: Added a secondary automaton to handle complex global rules in wikitext.

## v0.2.0 (2023-05-15)

- **Ruleset Expansion**: Introduced OpenCC rulesets in addition to MediaWiki rules.
- **Regional Support**: Added ruleset precedence for regional variants (TW/HK/CN).
- **Web App**: Implemented localStorage persistence for selected variants.

## v0.1.0 (2022-09-08)

- **Initial Release**: First stable release of `zhconv-rs`.
- **Multi-Platform**: Support for CLI, WASM, and Python (PyO3).
- **MediaWiki Core**: Integrated core conversion rules from MediaWiki.
- **Relicensing**: Switched project license to GPL-2.0-or-later for better compatibility.
