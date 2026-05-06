#!/usr/bin/env node
// Build the napi-rs Node.js native module by invoking cargo with the
// `napi` feature, then copying the resulting cdylib into ./build/ as a
// .node file Node can require().
//
// This is the dev path — the published wheel will use prebuilt binaries
// uploaded as release artifacts (handled in phase 5).

import { execSync } from "node:child_process";
import { existsSync, mkdirSync, copyFileSync } from "node:fs";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform, arch } from "node:os";

const here = dirname(fileURLToPath(import.meta.url));
const jsDir = resolve(here, "..");
const repoRoot = resolve(jsDir, "..");
const targetDir = resolve(repoRoot, "target", "release");
const outDir = resolve(jsDir, "build");

console.log("→ Compiling Rust + napi feature (this takes ~30s on a clean build)...");
execSync("cargo build --release --features napi", {
  cwd: repoRoot,
  stdio: "inherit",
});

const platform_arch = `${platform()}-${arch()}`;
const ext =
  platform() === "darwin" ? ".dylib" : platform() === "win32" ? ".dll" : ".so";
const prefix = platform() === "win32" ? "" : "lib";
const sourceLib = join(targetDir, `${prefix}jailguard${ext}`);

if (!existsSync(sourceLib)) {
  console.error(`✗ cdylib not found at ${sourceLib}`);
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });
const targetNode = join(outDir, `jailguard.${platform_arch}.node`);
copyFileSync(sourceLib, targetNode);

// Also write a generic jailguard.node symlink for dev convenience.
const genericNode = join(outDir, "jailguard.node");
copyFileSync(sourceLib, genericNode);

console.log(`✓ Wrote ${targetNode}`);
console.log(`✓ Wrote ${genericNode} (dev convenience copy)`);
