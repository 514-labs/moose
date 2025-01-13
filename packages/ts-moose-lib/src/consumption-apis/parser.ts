import { QueryFieldDefinition } from "./types";

import { QueryParamMetadata } from "./types";

import fs from "fs";

const logToFile = (message: string) => {
  const logPath = "/Users/cjus/moose-parser-debug.log";
  const timestamp = new Date().toISOString();
  try {
    if (!fs.existsSync(logPath)) {
      fs.writeFileSync(logPath, ""); // Create file if it doesn't exist
    }
    fs.appendFileSync(logPath, `${timestamp}: ${message}\n`);
  } catch (error) {
    console.error("Failed to write to log file:", error);
  }
};

export class QueryParamParser {
  static parseValue(value: string, fieldDef: QueryFieldDefinition): any {
    logToFile(
      `Parsing value: ${value} for field: ${fieldDef.name} of type: ${fieldDef.type}`,
    );
    switch (fieldDef.type) {
      case "number": {
        if (!/^-?\d+(\.\d+)?$/.test(value)) {
          const error = new Error(
            `Invalid number format for ${fieldDef.name}: ${value}`,
          );
          (error as any).type = "VALIDATION_ERROR";
          logToFile(`Throwing validation error: ${error.message}`);
          throw error;
        }
        const num = Number(value);
        if (isNaN(num)) {
          const error = new Error(
            `Invalid number value for ${fieldDef.name}: ${value}`,
          );
          (error as any).type = "VALIDATION_ERROR";
          logToFile(`Throwing validation error: ${error.message}`);
          throw error;
        }
        return num;
      }
      case "boolean":
        return value.toLowerCase() === "true";
      case "date":
        return new Date(value);
      case "array":
        return value
          .split(",")
          .map((v) =>
            fieldDef.elementType ? this.parseValue(v, fieldDef.elementType) : v,
          );
      default:
        return value;
    }
  }

  static mapParamsToType(
    params: Record<string, string[]>,
    metadata: QueryParamMetadata,
  ): Record<string, any> {
    logToFile(`Starting parameter validation...`);
    logToFile(`Raw params received: ${JSON.stringify(params)}`);

    const result: Record<string, any> = {};

    // Add required field validation
    for (const field of metadata.fields) {
      if (field.required && !(field.name in params)) {
        const error = new Error(`Missing required field: ${field.name}`);
        (error as any).type = "VALIDATION_ERROR";
        logToFile(`Throwing validation error: ${error.message}`);
        throw error;
      }
    }

    // First pass: Convert basic types
    for (const field of metadata.fields) {
      logToFile(`Validating field: ${field.name} (${field.type})`);
      if (field.name in params) {
        const values = params[field.name];
        logToFile(
          `Processing field ${field.name} with values: ${JSON.stringify(values)}`,
        );

        if (!values || values.length === 0) {
          continue;
        }

        if (field.type === "array") {
          result[field.name] = values.map((v) =>
            this.parseValue(
              v,
              field.elementType || { name: "", type: "string", required: true },
            ),
          );
        } else {
          if (values.length !== 1) {
            const error = new Error(`Expected single value for ${field.name}`);
            (error as any).type = "VALIDATION_ERROR";
            throw error;
          }
          result[field.name] = this.parseValue(values[0], field);
        }
      } else {
        logToFile(`Field ${field.name} not found in params`);
      }
    }

    // Second pass: Validate with Typia if validator exists
    if (metadata.typiaValidator) {
      const isValid = metadata.typiaValidator(result);
      if (!isValid) {
        const error = new Error(
          `Validation error: Invalid type for parameters`,
        );
        (error as any).type = "VALIDATION_ERROR";
        logToFile(`Throwing typia validation error: ${error.message}`);
        throw error;
      }
    }

    logToFile(`Final validated result: ${JSON.stringify(result)}`);
    return result;
  }
}
