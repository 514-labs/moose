#!/usr/bin/env node

import { getLiveModule } from "ts-patch";
import { runInThisContext } from "vm";
import path from "path";

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
