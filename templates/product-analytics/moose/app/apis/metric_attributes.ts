import { ConsumptionUtil, ConsumptionHelpers } from "@514labs/moose-lib";
import { typeToValidOperators } from "../helpers/types";
export interface QueryParams {
  metricName: string;
}
type ReturnType = {
  name: string;
  type: keyof typeof typeToValidOperators;
};

export default async function handle(
  { metricName }: QueryParams,
  { client, sql }: ConsumptionUtil,
) {
  const response = client.query(
    sql`SELECT name, type 
      FROM system.columns 
      WHERE table = ${metricName}
  `,
  );
  const columns = (await (await response).json()) as ReturnType[];
  return columns.reduce(
    (acc, { name, type }) => {
      acc[name] = typeToValidOperators[type];
      return acc;
    },
    {} as Record<string, string[]>,
  );
}
