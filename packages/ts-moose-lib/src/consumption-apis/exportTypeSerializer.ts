import process from "process";

const target_model = process.argv[3];

export async function runConsumptionTypeSerializer() {
  const func = require(`${process.cwd()}/app/apis/${target_model}.ts`).default;
  const schema = func["moose_input_schema"];
  console.log(schema === undefined ? "null" : JSON.stringify(schema));
}
