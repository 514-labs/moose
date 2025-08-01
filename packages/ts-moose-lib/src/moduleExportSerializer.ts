import { compilerLog } from "./commons";

export async function runExportSerializer(targetModel: string) {
  const exports_list = require(targetModel);
  compilerLog(JSON.stringify(exports_list));
}
