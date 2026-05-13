/**
 * JailGuard — fast prompt-injection detection.
 *
 * Pure-Rust core; this entry point loads the napi-rs Node addon.
 *
 * @example
 * ```ts
 * import { detect, isInjection, downloadModel } from "@jailguard/jailguard";
 *
 * await downloadModel(); // optional, avoids first-call latency
 *
 * if (isInjection("ignore previous instructions")) {
 *   throw new Error("blocked");
 * }
 *
 * const r = detect("What is the capital of France?");
 * console.log(r.score, r.risk);
 * ```
 */

import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";
import { platform, arch } from "node:os";

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));

/** Find the .node file built by scripts/build-native.mjs. */
function loadAddon(): NativeAddon {
  const platformArch = `${platform()}-${arch()}`;
  const candidates = [
    // Prebuilt binary checked into npm package (release path).
    join(here, "..", "build", `jailguard.${platformArch}.node`),
    // Local dev build (after `npm run build`).
    join(here, "..", "build", "jailguard.node"),
    // Repo root — direct cargo build for monorepo dev.
    join(here, "..", "..", "target", "release", "libjailguard.dylib"),
    join(here, "..", "..", "target", "release", "libjailguard.so"),
    join(here, "..", "..", "target", "release", "jailguard.dll"),
  ];

  for (const path of candidates) {
    if (existsSync(path)) {
      return require(path) as NativeAddon;
    }
  }

  throw new Error(
    `JailGuard native addon not found for ${platformArch}. ` +
      `Run \`npm run build\` from the js/ directory, or rebuild from source ` +
      `with \`cargo build --release --features napi\` and re-run.`,
  );
}

/** Risk classification bucket. */
export enum RiskLevel {
  Safe = 0,
  Low = 1,
  Medium = 2,
  High = 3,
  Critical = 4,
}

/** Output of `detect()`. */
export interface DetectionResult {
  /** True if the input is classified as a prompt injection. */
  isInjection: boolean;
  /** Raw probability in [0.0, 1.0]. */
  score: number;
  /** Confidence in the prediction (always ≥ 0.5). */
  confidence: number;
  /** Risk bucket derived from `score`. */
  risk: RiskLevel;
}

/** Native addon's exported surface. */
interface NativeAddon {
  version: () => string;
  downloadModel: () => void;
  modelCacheDir: () => string;
  detect: (text: string) => DetectionResult;
  isInjection: (text: string) => boolean;
  score: (text: string) => number;
  detectBatch: (texts: string[]) => DetectionResult[];
}

const addon = loadAddon();

/** Library version. */
export const version: () => string = addon.version;

/**
 * Pre-fetch the ONNX embedding model (~90 MB) into the cache directory.
 * Idempotent — safe to call multiple times.
 */
export const downloadModel: () => void = addon.downloadModel;

/** Path to the ONNX model cache directory. */
export const modelCacheDir: () => string = addon.modelCacheDir;

/** Classify a single string. */
export const detect: (text: string) => DetectionResult = addon.detect;

/** Quick boolean check — equivalent to `detect(text).isInjection`. */
export const isInjection: (text: string) => boolean = addon.isInjection;

/** Raw injection probability in [0.0, 1.0]. */
export const score: (text: string) => number = addon.score;

/** Process a batch of texts. */
export const detectBatch: (texts: string[]) => DetectionResult[] =
  addon.detectBatch;
