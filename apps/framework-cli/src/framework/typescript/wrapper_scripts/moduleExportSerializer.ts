#!/usr/bin/env ts-node

import process from "process";

const target_model = process.argv[1];

async function read_exports() {
  const exports_list = await import(target_model);
  console.log(JSON.stringify(exports_list));
}

read_exports();
