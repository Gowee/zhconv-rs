#!/usr/bin/env node
const { platform, arch, env } = require("node:process");
const { spawnSync } = require("node:child_process");

// Detect which meta package is running by inspecting this launcher's
// file path:
//   npx @zhconv/cli        -> path contains "/@zhconv/cli/"
//   npx @zhconv/cli-opencc -> path contains "/@zhconv/cli-opencc/"
// The launcher file is shipped by both meta packages, but each pulls
// in a different set of optionalDependencies (e.g. cli-opencc-linux-x64
// vs cli-linux-x64), so the launcher has to resolve the right
// subpackage.
const VARIANT = (() => {
  const m = __filename.match(/@zhconv\/(cli(?:-opencc)?)\//);
  if (!m) return "cli";
  return m[1];
})();

const PREFIX = VARIANT === "cli-opencc" ? "@zhconv/cli-opencc-" : "@zhconv/cli-";

const PLATFORMS = {
  win32:  { x64: `${PREFIX}windows-x64/zhconv.exe`, arm64: `${PREFIX}windows-arm64/zhconv.exe` },
  darwin: { x64: `${PREFIX}darwin-x64/zhconv`,      arm64: `${PREFIX}darwin-arm64/zhconv` },
  linux:  { x64: `${PREFIX}linux-x64/zhconv`,       arm64: `${PREFIX}linux-arm64/zhconv` },
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
