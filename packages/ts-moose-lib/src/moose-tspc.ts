#!/usr/bin/env node

import { getLiveModule } from "ts-patch";
import { runInThisContext } from "vm";
import path from "path";
import fs from "fs";

// Updated helper to find ts-patch installation
function findTsPatchRoot() {
  const possiblePaths = [
    path.join(__dirname, "ts-patch"),
    path.join(__dirname, "../ts-patch"),
    path.join(__dirname, "../../ts-patch"),
    path.join(__dirname, "../node_modules/ts-patch"),
    path.join(process.cwd(), "node_modules/ts-patch"),
    // Add workspace root for monorepo support
    path.join(process.cwd(), "../../node_modules/ts-patch"),
  ];

  for (const p of possiblePaths) {
    if (fs.existsSync(p)) {
      return p;
    }
  }

  // If we can't find the path, try to use the module resolution
  try {
    return path.dirname(require.resolve("ts-patch"));
  } catch (e) {
    throw new Error(
      "Could not find ts-patch installation. Please ensure ts-patch is installed.",
    );
  }
}

// Set ts-patch root before getting live module
const tsPatchRoot = findTsPatchRoot();
process.env.TS_PATCH_ROOT = tsPatchRoot;

const { js, tsModule } = getLiveModule("tsc.js");

const script = runInThisContext(`
    (function (exports, require, module, __filename, __dirname) {
      ${js}
    });
`);

script.call(
  exports,
  exports,
  require,
  module,
  tsModule.modulePath,
  path.dirname(tsModule.modulePath),
);
