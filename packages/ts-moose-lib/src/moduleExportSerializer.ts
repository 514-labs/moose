import process from "process";

const target_model = process.argv[3];

export async function runExportSerializer() {
  const exports_list = require(target_model);
  console.log(JSON.stringify(exports_list));
}
