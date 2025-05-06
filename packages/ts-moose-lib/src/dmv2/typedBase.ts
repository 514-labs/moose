import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { Column } from "../dataModels/dataModelTypes";

/**
 * Base class for all typed Moose dmv2 resources (OlapTable, Stream, etc.).
 * Handles the storage and injection of schema information (JSON schema and Column array)
 * provided by the Moose compiler plugin.
 *
 * @template T The data type (interface or type alias) defining the schema of the resource.
 * @template C The specific configuration type for the resource (e.g., OlapConfig, StreamConfig).
 */
export class TypedBase<T, C> {
  /** The JSON schema representation of type T. Injected by the compiler plugin. */
  schema: IJsonSchemaCollection.IV3_1;
  /** The name assigned to this resource instance. */
  name: string;

  /** A dictionary mapping column names (keys of T) to their Column definitions. */
  columns: {
    [columnName in keyof T]: Column;
  };
  /** An array containing the Column definitions for this resource. Injected by the compiler plugin. */
  columnArray: Column[];

  /** The configuration object specific to this resource type. */
  config: C;

  /**
   * @internal Constructor intended for internal use by subclasses and the compiler plugin.
   * It expects the schema and columns to be provided, typically injected by the compiler.
   *
   * @param name The name for the resource instance.
   * @param config The configuration object for the resource.
   * @param schema The JSON schema for the resource's data type T (injected).
   * @param columns The array of Column definitions for T (injected).
   */
  constructor(
    name: string,
    config: C,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    if (schema === undefined || columns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    this.schema = schema;
    this.columnArray = columns;
    const columnsObj = {} as any;
    columns.forEach((column) => {
      columnsObj[column.name] = column;
    });
    this.columns = columnsObj;

    this.name = name;
    this.config = config;
  }
}
