#!/usr/bin/env node
const { platform, arch, env } = process;
const { spawnSync } = require("child_process");

const pkg = require("../package.json");
const prefix = pkg.name && pkg.name.includes("opencc") ? "@zhconv/cli-opencc-" : "@zhconv/cli-";

const PLATFORMS = {
  win32:  { x64: `${prefix}windows-x64/zhconv.exe`,   arm64: `${prefix}windows-arm64/zhconv.exe` },
  darwin: { x64: `${prefix}darwin-x64/zhconv`,        arm64: `${prefix}darwin-arm64/zhconv` },
  linux:  { x64: `${prefix}linux-x64/zhconv`,         arm64: `${prefix}linux-arm64/zhconv` },
};

let subpath;
const target = PLATFORMS[platform]?.[arch];
if (target) {
  try {
    subpath = require.resolve(target);
  } catch {
    // fallback to alternate variant if present
    const altPrefix = prefix === "@zhconv/cli-" ? "@zhconv/cli-opencc-" : "@zhconv/cli-";
    try {
      subpath = require.resolve(target.replace(prefix, altPrefix));
    } catch {}
  }
}

if (!subpath && !env.ZHCONV_BINARY) {
  console.error(`zhconv-cli: no prebuilt binary for ${platform}-${arch}.`);
  console.error(`Build from source: cargo install zhconv --features bin-build`);
  console.error(`Or use the Python package: pip install zhconv-rs`);
  process.exit(1);
}

const bin = env.ZHCONV_BINARY || subpath;
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit", shell: false });
process.exit(r.status ?? 1);