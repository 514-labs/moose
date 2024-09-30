#!/usr/bin/env node

import { register } from "ts-node";

import fs from "fs";
import path from "path";
import * as vm from "vm";

// Register ts-node to interpret TypeScript code
register({
  esm: true,
  experimentalTsImportSpecifiers: true,
});

// Get the script path from the command line arguments
const scriptPath = process.argv[2];

if (!scriptPath) {
  console.error("No script path provided.");
  process.exit(1);
}

// Read the TypeScript file
const tsCode = fs.readFileSync(scriptPath, "utf-8");

// Compile and execute the TypeScript code
const script = new vm.Script(tsCode, {
  filename: path.basename(scriptPath),
});

const context = vm.createContext({
  require,
  module,
  __filename: scriptPath,
  __dirname: path.dirname(scriptPath),
  console,
  process,
  Buffer,
  setTimeout,
  setInterval,
  clearTimeout,
  clearInterval,
});

script.runInContext(context);
