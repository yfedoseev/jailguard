#!/usr/bin/env node
// Local-development build helper for the napi addon.
//
// Builds the napi-rs Node.js native module by invoking cargo with the
// `napi` feature, then copies the resulting cdylib into the prebuilds/
// directory at the path the loader expects:
//
//   prebuilds/<platform>-<arch>/jailguard.node
//
// CI does NOT use this script — the release pipeline downloads per-target
// build artifacts and stages them into prebuilds/ directly (see
// `.github/workflows/release.yml` → publish-npm). This script exists so
// contributors can do `npm run build:native && npm test` from a clean
// checkout without configuring CGO_* env vars or running cargo manually.

import { execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { arch, platform } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const jsDir = resolve(here, "..");
const repoRoot = resolve(jsDir, "..");
const targetDir = resolve(repoRoot, "target", "release");

// Map Node's os.arch() to the directory naming the loader uses.
// (Node uses 'x64', 'arm64'; same convention as the prebuilds layout.)
const triple = `${platform()}-${arch()}`;
const prebuildDir = resolve(jsDir, "prebuilds", triple);

console.log("→ Compiling Rust + napi feature (this takes ~30s on a clean build)...");
// Use `rustup run stable cargo` to bypass any broken cargo shim on
// macos-latest runners where ~/.cargo/bin/cargo can land in a state
// that resolves to rustup-init instead of the active toolchain's cargo.
// On dev machines without rustup, this falls back to plain `cargo`.
const cargoCmd = process.env.CI
  ? "rustup run stable cargo build --locked --release --features napi"
  : "cargo build --locked --release --features napi";
execSync(cargoCmd, {
  cwd: repoRoot,
  stdio: "inherit",
});

const ext = platform() === "darwin" ? ".dylib" : platform() === "win32" ? ".dll" : ".so";
const prefix = platform() === "win32" ? "" : "lib";
const sourceLib = join(targetDir, `${prefix}jailguard${ext}`);

if (!existsSync(sourceLib)) {
  console.error(`✗ cdylib not found at ${sourceLib}`);
  process.exit(1);
}

mkdirSync(prebuildDir, { recursive: true });
const dest = join(prebuildDir, "jailguard.node");
copyFileSync(sourceLib, dest);

console.log(`✓ Wrote ${dest}`);
console.log(`  Loader will find this automatically at runtime.`);
