import {
  ClickHouseEngines,
  createMaterializedView,
  createRefreshableMaterializedView,
  dropView,
  populateTable,
} from "../../blocks/helpers";
import { Sql, toStaticQuery } from "../../sqlHelpers";
import { OlapTable } from "./olapTable";
import { SqlResource } from "./sqlResource";
import { View } from "./view";
import { IJsonSchemaCollection } from "typia";
import { Column } from "../../dataModels/dataModelTypes";

/**
 * Represents a time interval for refreshable materialized views.
 */
export interface RefreshInterval {
  value: number;
  unit: "seconds" | "minutes" | "hours" | "days";
}

/**
 * Helper functions for creating duration intervals with better developer experience.
 */
export const duration = {
  seconds: (value: number): RefreshInterval => ({ value, unit: "seconds" }),
  minutes: (value: number): RefreshInterval => ({ value, unit: "minutes" }),
  hours: (value: number): RefreshInterval => ({ value, unit: "hours" }),
  days: (value: number): RefreshInterval => ({ value, unit: "days" }),
};

/**
 * Configuration options for creating a Materialized View.
 * @template T The data type of the records stored in the target table of the materialized view.
 */
export interface MaterializedViewConfig<T> {
  /** The SQL SELECT statement or `Sql` object defining the data to be materialized. Dynamic SQL (with parameters) is not allowed here. */
  selectStatement: string | Sql;
  /** An array of OlapTable or View objects that the `selectStatement` reads from. */
  selectTables: (OlapTable<any> | View)[];

  /** The name for the underlying target OlapTable that stores the materialized data. */
  tableName: string;
  /** The name for the ClickHouse MATERIALIZED VIEW object itself. */
  materializedViewName: string;

  /** Optional ClickHouse engine for the target table (e.g., ReplacingMergeTree). Defaults to MergeTree. */
  engine?: ClickHouseEngines;
  /** Optional ordering fields for the target table. Crucial if using ReplacingMergeTree. */
  orderByFields?: (keyof T & string)[];
}

/**
 * Represents a Materialized View in ClickHouse.
 * This encapsulates both the target OlapTable that stores the data and the MATERIALIZED VIEW definition
 * that populates the table based on inserts into the source tables.
 *
 * @template TargetTable The data type of the records stored in the underlying target OlapTable. The structure of T defines the target table schema.
 */
export class MaterializedView<TargetTable> extends SqlResource {
  /** The target OlapTable instance where the materialized data is stored. */
  targetTable: OlapTable<TargetTable>;

  /**
   * Creates a new MaterializedView instance.
   * Requires the `TargetTable` type parameter to be explicitly provided or inferred,
   * as it's needed to define the schema of the underlying target table.
   *
   * @param options Configuration options for the materialized view.
   */
  constructor(options: MaterializedViewConfig<TargetTable>);

  /** @internal **/
  constructor(
    options: MaterializedViewConfig<TargetTable>,
    targetSchema: IJsonSchemaCollection.IV3_1,
    targetColumns: Column[],
  );
  constructor(
    options: MaterializedViewConfig<TargetTable>,
    targetSchema?: IJsonSchemaCollection.IV3_1,
    targetColumns?: Column[],
  ) {
    let selectStatement = options.selectStatement;
    if (typeof selectStatement !== "string") {
      selectStatement = toStaticQuery(selectStatement);
    }

    if (targetSchema === undefined || targetColumns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    const targetTable = new OlapTable(
      options.tableName,
      {
        orderByFields: options.orderByFields,
      },
      targetSchema,
      targetColumns,
    );
    super(
      options.materializedViewName,
      [
        createMaterializedView({
          name: options.materializedViewName,
          destinationTable: options.tableName,
          select: selectStatement,
        }),
        populateTable({
          destinationTable: options.tableName,
          select: selectStatement,
        }),
      ],
      [dropView(options.materializedViewName)],
      {
        pullsDataFrom: options.selectTables,
        pushesDataTo: [targetTable],
      },
    );

    this.targetTable = targetTable;
  }
}

/**
 * Configuration options for creating a Refreshable Materialized View.
 * @template T The data type of the records stored in the target table of the refreshable materialized view.
 */
export interface RefreshableMaterializedViewConfig<T> {
  /** The SQL SELECT statement or `Sql` object defining the data to be materialized. Dynamic SQL (with parameters) is not allowed here. */
  selectStatement: string | Sql;
  /** An array of OlapTable, View, or RefreshableMaterializedView objects that the `selectStatement` reads from. */
  selectTables: (OlapTable<any> | View | RefreshableMaterializedView<any>)[];

  /** The name for the underlying target OlapTable that stores the materialized data. */
  tableName: string;
  /** The name for the ClickHouse MATERIALIZED VIEW object itself. */
  materializedViewName: string;

  /** Refresh interval using duration helpers */
  refreshEvery: RefreshInterval;

  /** Optional: Use APPEND mode to add new rows instead of replacing the view */
  appendMode?: boolean;

  /** Optional ClickHouse engine for the target table (e.g., ReplacingMergeTree). Defaults to MergeTree. */
  engine?: ClickHouseEngines;
  /** Optional ordering fields for the target table. Crucial if using ReplacingMergeTree. */
  orderByFields?: (keyof T & string)[];
}

/**
 * Represents a Refreshable Materialized View in ClickHouse.
 * Unlike regular materialized views which update incrementally, refreshable materialized views
 * periodically execute a query over the full dataset and replace the result set.
 * This is useful for complex queries involving JOINs that don't support incremental updates.
 *
 * @template TargetTable The data type of the records stored in the underlying target OlapTable.
 */
export class RefreshableMaterializedView<TargetTable> extends SqlResource {
  /** The target OlapTable instance where the materialized data is stored. */
  targetTable: OlapTable<TargetTable>;

  /** The configuration options for this refreshable materialized view. */
  config: RefreshableMaterializedViewConfig<TargetTable>;

  /**
   * Creates a new RefreshableMaterializedView instance.
   * Requires the `TargetTable` type parameter to be explicitly provided or inferred,
   * as it's needed to define the schema of the underlying target table.
   *
   * @param options Configuration options for the refreshable materialized view.
   */
  constructor(options: RefreshableMaterializedViewConfig<TargetTable>);

  /** @internal **/
  constructor(
    options: RefreshableMaterializedViewConfig<TargetTable>,
    targetSchema: IJsonSchemaCollection.IV3_1,
    targetColumns: Column[],
  );
  constructor(
    options: RefreshableMaterializedViewConfig<TargetTable>,
    targetSchema?: IJsonSchemaCollection.IV3_1,
    targetColumns?: Column[],
  ) {
    let selectStatement = options.selectStatement;
    if (typeof selectStatement !== "string") {
      selectStatement = toStaticQuery(selectStatement);
    }

    if (targetSchema === undefined || targetColumns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    // Extract dependencies from selectTables
    const dependsOn: string[] = [];
    const pullsDataFrom: (
      | OlapTable<any>
      | View
      | RefreshableMaterializedView<any>
    )[] = [];

    for (const table of options.selectTables) {
      pullsDataFrom.push(table);
      if (table instanceof RefreshableMaterializedView) {
        dependsOn.push(table.config.materializedViewName);
      }
    }

    const targetTable = new OlapTable(
      options.tableName,
      {
        orderByFields: options.orderByFields,
        engine: options.engine,
      },
      targetSchema,
      targetColumns,
    );

    super(
      options.materializedViewName,
      [
        createRefreshableMaterializedView({
          name: options.materializedViewName,
          destinationTable: options.tableName,
          select: selectStatement,
          refreshInterval: options.refreshEvery,
          appendMode: options.appendMode,
          dependsOn: dependsOn.length > 0 ? dependsOn : undefined,
        }),
      ],
      [dropView(options.materializedViewName)],
      {
        pullsDataFrom,
        pushesDataTo: [targetTable],
      },
    );

    this.targetTable = targetTable;
    this.config = options;
  }
}
