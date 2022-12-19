#!/usr/bin/env python3
from distutils.command.build import build
import requests
import os
import re
import hashlib

OUT_DIR = os.path.dirname(__file__)
BUILD_RS_PATH = os.path.join(OUT_DIR, "../build.rs")

OPENCC_URL = "https://raw.githubusercontent.com/BYVoid/OpenCC/%s/data/dictionary/%s"

MEDIAWIKI_URL = "https://raw.githubusercontent.com/wikimedia/mediawiki/%s/includes/languages/data/ZhConversion.php"

MEDIAWIKI_ZHCONV = ""

OPENCC_FILES = [
    "HKVariants.txt",
    "HKVariantsRevPhrases.txt",
    # "JPShinjitaiCharacters.txt",
    # "JPShinjitaiPhrases.txt",
    # "JPVariants.txt",
    "STCharacters.txt",
    "STPhrases.txt",
    "TSCharacters.txt",
    "TSPhrases.txt",
    "TWPhrasesIT.txt",
    "TWPhrasesName.txt",
    "TWPhrasesOther.txt",
    "TWVariants.txt",
    "TWVariantsRevPhrases.txt",
]


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


def main():
    with open(BUILD_RS_PATH, "r") as f:
        build_rs = f.read()
    if m := re.search(r'const MEDIAWIKI_COMMIT[^"]+"([0-9a-fA-F]+)"', build_rs):
        mediawiki_commit = m.group(1)
        print("Mediawiki Commit:", mediawiki_commit)
    else:
        raise Exception("Failed to extract MEDIAWIKI_COMMIT from build.rs")
    if m := re.search(r'const OPENCC_COMMIT[^"]+"([0-9a-fA-F]+)"', build_rs):
        opencc_commit = m.group(1)
        print("OpenCC Commit:", opencc_commit)
    else:
        raise Exception("Failed to extract OPENCC_COMMIT from build.rs")

    zhconversion_php_sha256sum = fetch(
        MEDIAWIKI_URL % mediawiki_commit, "ZhConversion.php"
    )

    opencc_sha256sums = []
    for fname in OPENCC_FILES:
        s = fetch(OPENCC_URL % (opencc_commit, fname), fname)
        opencc_sha256sums.append(s)

    old_build_rs = build_rs
    assert re.search(r"const MEDIAWIKI_SHA256[\s\S]+?;$", build_rs, flags=re.MULTILINE)
    build_rs = re.sub(
        r"const MEDIAWIKI_SHA256[\s\S]+?;$",
        f'const MEDIAWIKI_SHA256: [u8; 32] = hex!("{zhconversion_php_sha256sum}");',
        build_rs,
        flags=re.MULTILINE,
    )
    assert re.search(
        r"const OPENCC_SHA256.+?=[\s\S]+?;$", build_rs, flags=re.MULTILINE
    )
    build_rs = re.sub(
        r"const OPENCC_SHA256.+?=[\s\S]+?;$",
        f"const OPENCC_SHA256: [(&str, [u8; 32]); {len(opencc_sha256sums)}] = ["
        + ", ".join(
            f'("{f}", hex!("{s}"))' for f, s in zip(OPENCC_FILES, opencc_sha256sums)
        )
        + "];",
        build_rs,
        flags=re.MULTILINE,
    )
    if old_build_rs == build_rs:
        print("** No update to build.rs **")
    else:
        print("** Updated build.rs **")

    with open(BUILD_RS_PATH, "w") as f:
        f.write(build_rs)


if __name__ == "__main__":
    main()
