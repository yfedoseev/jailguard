/**
 * JailGuard — fast prompt-injection detection.
 *
 * Pure-Rust core; this entry point loads the prebuilt napi-rs Node addon
 * shipped under `prebuilds/<platform>-<arch>/jailguard.node`.
 *
 * @example
 * ```ts
 * import { detect, isInjection, downloadModel } from "@yfedoseev/jailguard";
 *
 * downloadModel(); // optional — avoids first-call latency
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
import { arch, platform } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));

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

/**
 * Map (platform, arch) to the prebuilt `.node` file shipped in the npm
 * tarball. The keys correspond to Node's `os.platform()` and
 * `os.arch()` return values; we deliberately ship the same set of
 * triples the Rust release.yml builds for.
 */
const PREBUILD_PATHS: Record<string, Record<string, string>> = {
  darwin: {
    x64: "../prebuilds/darwin-x64/jailguard.node",
    arm64: "../prebuilds/darwin-arm64/jailguard.node",
  },
  linux: {
    x64: "../prebuilds/linux-x64/jailguard.node",
    arm64: "../prebuilds/linux-arm64/jailguard.node",
  },
  win32: {
    x64: "../prebuilds/win32-x64/jailguard.node",
  },
};

function prebuildPath(): string {
  const plat = platform();
  const a = arch();
  const rel = PREBUILD_PATHS[plat]?.[a];
  if (!rel) {
    throw new Error(
      `@yfedoseev/jailguard: unsupported platform ${plat}/${a}. ` +
        "Supported: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64. " +
        "File an issue at https://github.com/yfedoseev/jailguard/issues if you need a new target.",
    );
  }
  return join(here, rel);
}

/**
 * Load the native addon. Tries the prebuilt path first; falls back to a
 * developer build at `../build/jailguard.node` when JAILGUARD_NAPI_DEV=1
 * is set (used by `npm run build:native` during local development).
 */
function loadAddon(): NativeAddon {
  const prebuilt = prebuildPath();
  try {
    return require(prebuilt) as NativeAddon;
  } catch (err) {
    if (process.env.JAILGUARD_NAPI_DEV === "1" || process.env.NODE_ENV === "development") {
      try {
        return require(join(here, "..", "build", "jailguard.node")) as NativeAddon;
      } catch {
        /* fall through to original error */
      }
    }
    throw new Error(
      `@yfedoseev/jailguard: failed to load native addon at ${prebuilt}. ` +
        "This usually means the prebuilt binary for your platform is missing " +
        "from the npm tarball — please file an issue.\n\n" +
        `Underlying error: ${(err as Error).message}`,
    );
  }
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
export const detectBatch: (texts: string[]) => DetectionResult[] = addon.detectBatch;
