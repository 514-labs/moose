import { defineConfig } from "tsup";

export default defineConfig({
  entry: [
    "src/index.ts",
    "src/dataModels/toDataModels.ts",
    "src/consumption-apis/insertTypiaValidation.ts",
    "src/scripts/workflow.ts",
    "src/moose-tspc.ts",
    "src/moose-runner.ts",
    "src/moose-exec.ts",
    "src/dmv2/compilerPlugin.ts",
    "src/dmv2/index.ts",
  ],
  format: ["cjs", "esm"],
  dts: true, // Generate declaration file (.d.ts)
  outDir: "dist",
  splitting: false,
  sourcemap: true,
  clean: true,
});
