import process from "process";

export async function runExportSerializer(targetModel: string) {
  const exports_list = require(targetModel);
  console.log(JSON.stringify(exports_list));
}
