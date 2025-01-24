import { QueryParamMetadata } from "./types";
import { logToConsole } from "./hlogger";
import typia from "typia";

export class QueryParamParser {
  static parseValue(value: string, type: string): any {
    logToConsole(
      `QueryParamParser.parseValue: Parsing value: ${value} of type: ${type}`,
    );
    switch (type) {
      case "number": {
        if (!/^-?\d+(\.\d+)?$/.test(value)) {
          const error = new Error(`Invalid number format: ${value}`);
          (error as any).type = "VALIDATION_ERROR";
          throw error;
        }
        const num = Number(value);
        if (isNaN(num)) {
          const error = new Error(`Invalid number value: ${value}`);
          (error as any).type = "VALIDATION_ERROR";
          throw error;
        }
        return Number(num);
      }
      case "boolean":
        return value.toLowerCase() === "true";
      case "date":
        return new Date(value);
      default:
        return value;
    }
  }

  static mapParamsToType(
    params: Record<string, string[]>,
    metadata: QueryParamMetadata,
  ): Record<string, any> {
    const result: Record<string, any> = {};

    for (const [key, values] of Object.entries(params)) {
      if (!values || values.length === 0) continue;

      // Take the first value and pass it through directly
      result[key] = values[0];
    }

    if (metadata.typiaValidator) {
      try {
        metadata.typiaValidator(result);
      } catch (error) {
        const validationError = new Error(
          `Validation error: Invalid type for parameters`,
        );
        (validationError as any).type = "VALIDATION_ERROR";
        logToConsole(
          `mapParamsToType: Validation error: Invalid type for parameters`,
        );
        throw validationError;
      }
    }

    logToConsole(`mapParamsToType: Result: ${JSON.stringify(result)}`);
    return result;
  }
}
