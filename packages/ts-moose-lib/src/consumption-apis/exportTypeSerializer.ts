import process from "process";

export async function runConsumptionTypeSerializer(targetModel: string) {
  const module = require(`${process.cwd()}/app/apis/${targetModel}.ts`);
  const func = module.default;
  const inputSchema = func["moose_input_schema"] || null;
  const outputSchema = func["moose_output_schema"] || null;

  // Extract version from all exported ConsumptionApi instances
  let version = null;
  for (const exportName in module) {
    const exportedValue = module[exportName];
    if (
      exportedValue &&
      typeof exportedValue === "object" &&
      exportedValue.config &&
      exportedValue.config.version
    ) {
      version = exportedValue.config.version;
      break;
    }
  }

  console.log(
    JSON.stringify({
      inputSchema,
      outputSchema,
      version,
    }),
  );
}
