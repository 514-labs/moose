import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/index.ts",
    "src/dataModels/toDataModels.ts",
    "src/moose-tspc.ts",
    "src/moose-runner.ts",
    "src/moose-exec.ts",
  ],
  format: ["cjs", "esm"],
  dts: true,
  outDir: "dist",
  splitting: false,
  sourcemap: true,
  clean: true,
});
