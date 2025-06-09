import { IJsonSchemaCollection } from "typia";
import { TypedBase } from "../typedBase";
import { Column } from "../../dataModels/dataModelTypes";
import { ClickHouseEngines } from "../../blocks/helpers";
import { getMooseInternal } from "../internal";

/**
 * Configuration options for an OLAP (Online Analytical Processing) table.
 * @template T The data type of the records stored in the table.
 */
export type OlapConfig<T> = {
  /**
   * Specifies the fields to use for ordering data within the ClickHouse table.
   * This is crucial for optimizing query performance, especially for ReplacingMergeTree engines.
   */
  orderByFields?: (keyof T & string)[];
  /**
   * If true, uses the ReplacingMergeTree engine for the ClickHouse table, enabling automatic deduplication based on the `orderByFields`.
   * Equivalent to setting `engine: ClickHouseEngines.ReplacingMergeTree`.
   * Defaults to false.
   */
  // equivalent to setting `engine: ClickHouseEngines.ReplacingMergeTree`
  deduplicate?: boolean;
  /**
   * Specifies the ClickHouse table engine to use.
   * Defaults to MergeTree if not specified.
   * @see ClickHouseEngines for available options.
   */
  engine?: ClickHouseEngines;
  /**
   * An optional version string for this configuration. Can be used for tracking changes or managing deployments.
   */
  version?: string;
};

/**
 * Represents an OLAP (Online Analytical Processing) table, typically corresponding to a ClickHouse table.
 * Provides a typed interface for interacting with the table.
 *
 * @template T The data type of the records stored in the table. The structure of T defines the table schema.
 */
export class OlapTable<T> extends TypedBase<T, OlapConfig<T>> {
  /** @internal */
  public readonly kind = "OlapTable";

  /**
   * Creates a new OlapTable instance.
   * @param name The name of the table. This name is used for the underlying ClickHouse table.
   * @param config Optional configuration for the OLAP table.
   */
  constructor(name: string, config?: OlapConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: OlapConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config?: OlapConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config ?? {}, schema, columns);

    getMooseInternal().tables.set(name, this);
  }
}
