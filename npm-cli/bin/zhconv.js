#!/usr/bin/env node
const { platform, arch, env } = process;
const { spawnSync } = require("child_process");

const PLATFORMS = {
  win32:  { x64: "@zhconv/cli-windows-x64/zhconv.exe",   arm64: "@zhconv/cli-windows-arm64/zhconv.exe" },
  darwin: { x64: "@zhconv/cli-darwin-x64/zhconv",        arm64: "@zhconv/cli-darwin-arm64/zhconv" },
  linux:  { x64: "@zhconv/cli-linux-x64/zhconv",         arm64: "@zhconv/cli-linux-arm64/zhconv" },
};

let subpath;
try {
  subpath = require.resolve(PLATFORMS[platform]?.[arch]);
} catch {
  // require.resolve failed — only error out if no override is provided
  if (!env.ZHCONV_BINARY) {
    console.error(`zhconv-cli: no prebuilt binary for ${platform}-${arch}.`);
    console.error(`Build from source: cargo install zhconv --features bin-build`);
    console.error(`Or use the Python package: pip install zhconv-rs`);
    process.exit(1);
  }
  subpath = null;
}

const bin = env.ZHCONV_BINARY || subpath;
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit", shell: false });
process.exit(r.status ?? 1);