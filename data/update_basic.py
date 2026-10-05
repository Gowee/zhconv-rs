#!/usr/bin/env python3
import requests
import os
import re
import hashlib
from pathlib import Path

OUT_DIR = Path(__file__).parent
MEDIAWIKI_DIR = OUT_DIR / "mediawiki"
OPENCC_DIR = OUT_DIR / "opencc"
MEDIAWIKI_LIB_PATH = MEDIAWIKI_DIR / "src/lib.rs"
OPENCC_LIB_PATH = OPENCC_DIR / "src/lib.rs"

MEDIAWIKI_COMMIT_API = "https://api.github.com/repos/mediawiki/mediawiki/commits/master"
MEDIAWIKI_ZHCONV_URL = "https://raw.githubusercontent.com/wikimedia/mediawiki/%s/includes/Languages/Data/ZhConversion.php"

OPENCC_COMMIT_API = "https://api.github.com/repos/BYVoid/OpenCC/commits/master"
OPENCC_DICTS_URL = (
    "https://raw.githubusercontent.com/BYVoid/OpenCC/%s/data/dictionary/%s"
)

OPENCC_FILES = [
    "HKVariants.txt",
    "HKVariantsRevPhrases.txt",
    "STCharacters.txt",
    "STPhrases.txt",
    "TSCharacters.txt",
    "TSPhrases.txt",
    "TWPhrases.txt",
    "TWVariants.txt",
    "TWVariantsRevPhrases.txt",
    "CJK_Compatibility_Ideographs.txt",
    "TWVariantsPhrases.txt",
    "HKVariantsPhrases.txt",
    "HKPhrases.txt",
    "HKPhrasesRev.txt",
    "TWPhrasesRev.txt",
]
# JP dictionaries deliberately skipped (out of scope for zh-Hans/Hant/TW/HK/CN).
# "JPShinjitaiCharacters.txt",
# "JPShinjitaiPhrases.txt",


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def fetch(url, dest_path):
    print("Downloading", url)
    try:
        with open(dest_path, "rb") as f:
            olds = sha256(f.read())
    except FileNotFoundError:
        olds = None
    resp = requests.get(url)
    resp.raise_for_status()
    assert len(resp.content) != 0, "Got empty file"
    if "Content-Length" in resp.headers:
        # https://blog.petrzemek.net/2018/04/22/on-incomplete-http-reads-and-the-requests-library-in-python/
        expected_size = int(resp.headers["Content-Length"])
        actual_size = resp.raw.tell()
        assert (
            expected_size == actual_size
        ), f"Incomplete download: {actual_size}/{expected_size}"
    with open(dest_path, "wb") as f:
        f.write(resp.content)
    s = sha256(resp.content)
    if olds == s:
        print("(Unchanged)")
    elif olds:
        print(f"(Updated {olds} -> {s})")
    else:
        print(f"(Created {s})")
    return s


def update_const(path, name_pattern, replacement):
    with open(path, "r") as f:
        content = f.read()
    old = content
    content = re.sub(name_pattern, replacement, content, flags=re.MULTILINE)
    assert content != old or replacement in content
    if old != content:
        print(f"** Updated {path} **")
    with open(path, "w") as f:
        f.write(content)
    return old != content


def main():
    with open(MEDIAWIKI_LIB_PATH, "r") as f:
        mw_lib = f.read()
    old_mw_lib = mw_lib
    with open(OPENCC_LIB_PATH, "r") as f:
        opencc_lib = f.read()
    old_opencc_lib = opencc_lib

    if m := re.search(r'pub const MEDIAWIKI_COMMIT[^"]+"([0-9a-fA-F]+)"', mw_lib):
        old_mediawiki_commit = m.group(1).lower()
        if old_mediawiki_commit != (
            mediawiki_commit := requests.get(MEDIAWIKI_COMMIT_API).json()["sha"].lower()
        ):
            print(f"MediaWiki Commit: {old_mediawiki_commit} -> {mediawiki_commit}")
            mw_lib = re.sub(
                r"pub const MEDIAWIKI_COMMIT.+?=[\s\S]+?;$",
                f'pub const MEDIAWIKI_COMMIT: &str = "{mediawiki_commit}";',
                mw_lib,
                flags=re.MULTILINE,
            )
        else:
            print(f"MediaWiki Commit: {mediawiki_commit}")
    else:
        raise Exception("Failed to extract MEDIAWIKI_COMMIT from data/mediawiki/src/lib.rs")
    if m := re.search(r'pub const OPENCC_COMMIT[^"]+"([0-9a-fA-F]+)"', opencc_lib):
        old_opencc_commit = m.group(1).lower()
        if old_opencc_commit != (
            opencc_commit := requests.get(OPENCC_COMMIT_API).json()["sha"].lower()
        ):
            print(f"OpenCC Commit: {old_opencc_commit} -> {opencc_commit}")
            opencc_lib = re.sub(
                r"pub const OPENCC_COMMIT.+?=[\s\S]+?;$",
                f'pub const OPENCC_COMMIT: &str = "{opencc_commit}";',
                opencc_lib,
                flags=re.MULTILINE,
            )
        else:
            print(f"OpenCC Commit: {opencc_commit}")
    else:
        raise Exception("Failed to extract OPENCC_COMMIT from data/opencc/src/lib.rs")

    zhconversion_php_sha256sum = fetch(
        MEDIAWIKI_ZHCONV_URL % mediawiki_commit, MEDIAWIKI_DIR / "ZhConversion.php"
    )

    opencc_sha256sums = []
    for fname in OPENCC_FILES:
        s = fetch(OPENCC_DICTS_URL % (opencc_commit, fname), OPENCC_DIR / fname)
        opencc_sha256sums.append(s)

    assert re.search(r"pub const MEDIAWIKI_SHA256[\s\S]+?;$", mw_lib, flags=re.MULTILINE)
    mw_lib = re.sub(
        r"pub const MEDIAWIKI_SHA256[\s\S]+?;$",
        f'pub const MEDIAWIKI_SHA256: [u8; 32] = hex!("{zhconversion_php_sha256sum}");',
        mw_lib,
        flags=re.MULTILINE,
    )
    assert re.search(r"pub const OPENCC_SHA256.+?=[\s\S]+?;$", opencc_lib, flags=re.MULTILINE)
    opencc_lib = re.sub(
        r"pub const OPENCC_SHA256.+?=[\s\S]+?;$",
        f"pub const OPENCC_SHA256: [(&str, [u8; 32]); {len(opencc_sha256sums)}] = ["
        + ", ".join(
            f'("{f}", hex!("{s}"))' for f, s in zip(OPENCC_FILES, opencc_sha256sums)
        )
        + "];",
        opencc_lib,
        flags=re.MULTILINE,
    )
    if old_mw_lib == mw_lib:
        print("** No update to data/mediawiki/src/lib.rs **")
    else:
        print("** Updated data/mediawiki/src/lib.rs **")
        with open(MEDIAWIKI_LIB_PATH, "w") as f:
            f.write(mw_lib)

    if old_opencc_lib == opencc_lib:
        print("** No update to data/opencc/src/lib.rs **")
    else:
        print("** Updated data/opencc/src/lib.rs **")
        with open(OPENCC_LIB_PATH, "w") as f:
            f.write(opencc_lib)


if __name__ == "__main__":
    main()
