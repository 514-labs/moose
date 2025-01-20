#!/usr/bin/env node

import { register } from "ts-node";
import "ts-patch/register";

// Register ts-node to interpret TypeScript code
register({
  esm: true,
  experimentalSpecifierResolution: "node",
  transpileOnly: true,
  compilerOptions: {
    plugins: [{ transform: "typia/lib/transform" }],
  },
});

// Get the script path from the command line arguments
let scriptPath = process.argv[2];
if (!scriptPath) {
  console.error("moose-exec: No script path provided.");
  process.exit(1);
}

scriptPath = scriptPath.substring(0, scriptPath.length - 3);

// Use dynamic import to load and execute the TypeScript file
try {
  require(scriptPath);
} catch (e) {
  console.error("moose-exec: Error executing the script:", e);
  process.exit(1);
}
