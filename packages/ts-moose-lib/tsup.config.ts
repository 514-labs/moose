import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/index.ts",
    "src/dataModels/toDataModels.ts",
    "src/moose-tspc.ts",
    "src/moose-runner.ts",
    "src/moose-exec.ts",
  ],
  format: ["cjs", "esm"], // Build for commonJS and ESmodules
  dts: true, // Generate declaration file (.d.ts)
  splitting: false,
  sourcemap: true,
  clean: true,
});
