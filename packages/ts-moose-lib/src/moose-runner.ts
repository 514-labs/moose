#!/usr/bin/env node

// This file is use to run the proper runners for moose based on the
// the arguments passed to the file.
// It registers ts-node to be able to interpret user code.

import { register } from "ts-node";

// We register ts-node to be able to interpret TS user code.
if (
  process.argv[2] == "consumption-apis" ||
  process.argv[2] == "consumption-type-serializer" ||
  process.argv[2] == "dmv2-serializer"
) {
  const transformFile =
    process.argv[2] !== "dmv2-serializer"
      ? "consumption-apis/insertTypiaValidation.js"
      : "dmv2/compilerPlugin.js";
  register({
    esm: true,
    experimentalTsImportSpecifiers: true,
    compiler: "ts-patch/compiler",
    compilerOptions: {
      plugins: [
        {
          transform: `./node_modules/@514labs/moose-lib/dist/${transformFile}`,
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
import { loadIndex } from "./dmv2/internal";
import process from "process";

switch (process.argv[2]) {
  case "dmv2-serializer":
    loadIndex();
    break;
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
    runScripts();
    break;
  default:
    console.error(`Invalid argument: ${process.argv[2]}`);
    process.exit(1);
}
