import process from "process";

const target_model = process.argv[3];

export async function runConsumptionTypeSerializer() {
  const func = require(`${process.cwd()}/app/apis/${target_model}.ts`).default;
  const inputSchema = func["moose_input_schema"] || null;
  const outputSchema = func["moose_output_schema"] || null;
  console.log(
    JSON.stringify({
      inputSchema,
      outputSchema,
    }),
  );
}
