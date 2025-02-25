#!/usr/bin/env node

// This file is use to run the proper runners for moose based on the
// the arguments passed to the file.
// It registers ts-node to be able to interpret user code.

import { register } from "ts-node";

// We register ts-node to be able to interpret TS user code.
if (
  process.argv[2] == "consumption-apis" ||
  process.argv[2] == "consumption-type-serializer"
) {
  register({
    esm: true,
    experimentalTsImportSpecifiers: true,
    compiler: "ts-patch/compiler",
    compilerOptions: {
      plugins: [
        {
          transform:
            "./node_modules/@514labs/moose-lib/dist/consumption-apis/insertTypiaValidation.js",
          transformProgram: true,
        },
        {
          transform: "typia/lib/transform",
        },
      ],
      experimentalDecorators: true,
    },
  });
} else {
  register({
    esm: true,
    experimentalTsImportSpecifiers: true,
  });
}
import "./instrumentation";

import { runBlocks } from "./blocks/runner";
import { runConsumptionApis } from "./consumption-apis/runner";
import { runStreamingFunctions } from "./streaming-functions/runner";
import { runExportSerializer } from "./moduleExportSerializer";
import { runConsumptionTypeSerializer } from "./consumption-apis/exportTypeSerializer";
import { runScripts } from "./scripts/runner";

switch (process.argv[2]) {
  case "export-serializer":
    runExportSerializer();
    break;
  case "blocks":
    runBlocks();
    break;
  case "consumption-apis":
    runConsumptionApis();
    break;
  case "streaming-functions":
    runStreamingFunctions();
    break;
  case "consumption-type-serializer":
    runConsumptionTypeSerializer();
    break;
  case "scripts":
    runScripts(process.argv[3], process.argv[4]);
    break;
  default:
    console.error(`Invalid argument: ${process.argv[2]}`);
    process.exit(1);
}
