import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    environment: "node",
    testTimeout: 30000, // first call downloads the ONNX model
    hookTimeout: 30000,
  },
});
