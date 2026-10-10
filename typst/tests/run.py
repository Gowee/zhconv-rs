#!/usr/bin/env python3
"""Validate the staged package in an isolated package path, outside the checkout."""
import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from build import build  # noqa: E402


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--typst", default="typst")
    args = parser.parse_args()
    stage = build()
    subprocess.run(["cargo", "test", "--locked"], cwd=ROOT, check=True)
    reference = subprocess.check_output(["cargo", "run", "--locked", "--quiet", "--example", "reference"], cwd=ROOT, text=True, encoding="utf-8")
    rows = json.loads(reference)
    negative = [
        ('zhconv("text", "unknown")', "Unsupported target variant"),
        ('zhconv("", "unknown")', "Unsupported target variant"),
        ('zhconv("text", 42)', "expected"),
        ('zhconv("text", "zh-hans", wikitext: "yes")', "expected boolean"),
        ('zhconv-wasm.zhconv(bytes((255,)), bytes("zh-hans"), bytes((0,)))', "Invalid text"),
        ('zhconv-wasm.zhconv(bytes("text"), bytes((255,)), bytes((0,)))', "Invalid target variant"),
        ('zhconv-wasm.zhconv(bytes("text"), bytes("zh-hans"), bytes(()))', "Invalid wikitext flag"),
        ('zhconv-wasm.zhconv(bytes("text"), bytes("zh-hans"), bytes((2,)))', "Invalid wikitext flag"),
        ('zhconv-wasm.zhconv(bytes("text"), bytes("zh-hans"), bytes((0, 1)))', "Invalid wikitext flag"),
        ('zhconv-wasm.is_hans(bytes((255,)))', "Invalid text"),
    ]
    with tempfile.TemporaryDirectory(prefix="zhconv-typst-") as tmp:
        root = Path(tmp)
        packages = root / "packages"
        shutil.copytree(stage, packages / "local" / "zhconv" / "0.4.0")
        (root / "reference.json").write_text(reference, encoding="utf-8")
        shutil.copyfile(ROOT / "tests" / "positive.typ", root / "positive.typ")
        command = [args.typst, "compile", "--root", str(root), "--package-path", str(packages), "--package-cache-path", str(root / "cache")]
        subprocess.run(command + ["positive.typ", "positive.pdf"], cwd=root, check=True)
        for i, (expression, expected) in enumerate(negative):
            name = f"negative-{i}.typ"
            (root / name).write_text('#import "@local/zhconv:0.4.0": zhconv, zhconv-wasm\n#' + expression, encoding="utf-8")
            result = subprocess.run(command + [name, f"negative-{i}.pdf"], cwd=root, capture_output=True, text=True, encoding="utf-8")
            if result.returncode == 0 or expected not in result.stderr or "unreachable" in result.stderr:
                raise AssertionError(f"{expression}: expected {expected!r}, got {result.stderr}")
        example = (ROOT / "example.typ").read_text(encoding="utf-8").replace('#import "zhconv.typ": zhconv', '#import "@local/zhconv:0.4.0": zhconv')
        (root / "example.typ").write_text(example, encoding="utf-8")
        subprocess.run(command + ["example.typ", "example.pdf"], cwd=root, check=True)
    print(f"PASS: {len(rows)} native/WASM comparisons, content/API checks, {len(negative)} error cases, installed example")


if __name__ == "__main__":
    main()
