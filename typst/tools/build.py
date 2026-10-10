#!/usr/bin/env python3
"""Build and stage the OpenCC-only Typst package; Python 3.11+, no dependencies."""
import hashlib
import json
import shutil
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = "wasm32-unknown-unknown"


def run(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True, encoding="utf-8")


def check_imports(data):
    """Read Wasm import declarations without depending on an external Wasm tool."""
    if data[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("Not a WebAssembly 1.0 module")
    pos = 8

    def uint():
        nonlocal pos
        value = 0
        for shift in range(0, 35, 7):
            byte = data[pos]
            pos += 1
            value |= (byte & 127) << shift
            if byte < 128:
                return value
        raise ValueError("Invalid Wasm integer")

    def name():
        nonlocal pos
        size = uint()
        value = data[pos:pos + size].decode("utf-8")
        pos += size
        return value

    imports = []
    while pos < len(data):
        section = data[pos]
        pos += 1
        size = uint()
        end = pos + size
        if end > len(data):
            raise ValueError("Truncated Wasm section")
        if section == 2:
            for _ in range(uint()):
                module, field = name(), name()
                kind = data[pos]
                pos += 1
                if kind != 0:
                    raise ValueError(f"Unexpected non-function import: {module}.{field}")
                uint()  # Function type index; actual signatures are checked by Typst.
                imports.append((module, field))
            if pos != end:
                raise ValueError("Invalid Wasm import section")
        pos = end
    expected = {
        ("typst_env", "wasm_minimal_protocol_write_args_to_buffer"),
        ("typst_env", "wasm_minimal_protocol_send_result_to_host"),
    }
    if set(imports) != expected or len(imports) != 2:
        raise ValueError(f"Unexpected Wasm imports: {imports}")
    return imports


def build():
    metadata = json.loads(run("cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET))
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = metadata["resolve"]["nodes"]
    for node in nodes:
        if "mediawiki" in packages[node["id"]]["name"] or any("mediawiki" in f for f in node["features"]):
            raise RuntimeError("MediaWiki dependency/features found in the resolved build graph")
    core = next(n for n in nodes if packages[n["id"]]["name"] == "zhconv")
    # Regional phrase dictionaries (OpenCC TWPhrases/HKPhrases, Apache-2.0) are
    # part of the full zh-TW / zh-HK semantics; script-only conversion remains
    # available via zh-Hant. Keep them required so the package cannot silently
    # regress to region-less output.
    required = {"opencc", "opencc-twp", "opencc-hkp", "compress", "cjk-compat"}
    if not required.issubset(core["features"]) or {"wasm"}.intersection(core["features"]):
        raise RuntimeError(f"Unexpected core feature configuration: {core['features']}")
    # The Typst package version tracks the core version (single source of truth:
    # the workspace version in the repository root manifest).
    core_version = packages[core["id"]]["version"]
    pkg_manifest = tomllib.loads((ROOT / "typst.toml").read_text(encoding="utf-8"))["package"]
    if pkg_manifest["version"] != core_version:
        raise RuntimeError(
            f"Package version {pkg_manifest['version']} does not track the core version {core_version}; "
            "update typst.toml to match the workspace version"
        )
    messages = [json.loads(line) for line in run(
        "cargo", "build", "--locked", "--release", "--target", TARGET,
        "--message-format=json",
    ).splitlines()]
    target_dir = Path(metadata["target_directory"])
    wasm = target_dir / TARGET / "release" / "zhconv_typst.wasm"
    data = wasm.read_bytes()
    imports = check_imports(data)
    # Cargo identifies the build script used by this invocation, even on a
    # cache hit. Do not guess from timestamps of stale target/ directories.
    core_build = next(m for m in messages if m.get("reason") == "build-script-executed" and m["package_id"] == core["id"])
    report = (Path(core_build["out_dir"]) / "zhconv-diagnostics.txt").read_text(encoding="utf-8")
    if "MEDIAWIKI_COMMIT=" in report:
        raise RuntimeError("MediaWiki data found in build diagnostics; use a clean target directory")
    opencc = next(line.split("=", 1)[1] for line in report.splitlines() if line.startswith("OPENCC_COMMIT="))
    stage = ROOT / "dist" / "package"
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    for filename in ["zhconv.typ", "typst.toml", "README.md", "LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "THIRD-PARTY-NOTICES.md"]:
        shutil.copyfile(ROOT / filename, stage / filename)
    shutil.copyfile(wasm, stage / wasm.name)
    # Preserve dependency notices, including build tools, conservatively. Root
    # license files are sufficient for these locked crates; zstd's additional
    # upstream notice is included explicitly.
    licenses = []
    for node in nodes:
        package = packages[node["id"]]
        if package["source"] is None:
            continue  # Workspace code/data are covered by the notices above.
        directory = Path(package["manifest_path"]).parent
        files = [p for p in directory.iterdir() if p.is_file() and p.name.lower().startswith(("license", "copying", "notice"))]
        if package["name"] == "zstd-sys":
            files.append(directory / "zstd" / "LICENSE")
        if not files:
            raise RuntimeError(f"No license text found for {package['name']}")
        destination = stage / "licenses" / f"{package['name']}-{package['version']}"
        destination.mkdir(parents=True)
        for path in files:
            relative = path.relative_to(directory)
            output = destination / relative
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, output)
        licenses.append({"name": package["name"], "version": package["version"], "license": package["license"], "license_file": package["license_file"]})
    (stage / "licenses" / "index.json").write_text(json.dumps(licenses, indent=2) + "\n", encoding="utf-8")
    info = {
        "rustc": run("rustc", "--version").strip(),
        "source_commit": run("git", "rev-parse", "HEAD").strip(),
        "source_dirty": bool(run("git", "status", "--porcelain").strip()),
        "target": TARGET,
        "core_version": packages[core["id"]]["version"],
        "core_features": sorted(core["features"]),
        "opencc_commit": opencc,
        "wasm_bytes": len(data),
        "wasm_sha256": hashlib.sha256(data).hexdigest(),
        "lockfile_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
        "imports": imports,
    }
    (stage / "build-info.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    manifest = tomllib.loads((stage / "typst.toml").read_text(encoding="utf-8"))["package"]
    print(f"Staged {manifest['name']}:{manifest['version']}: {len(data):,} WASM bytes at {stage}")
    return stage


if __name__ == "__main__":
    build()
