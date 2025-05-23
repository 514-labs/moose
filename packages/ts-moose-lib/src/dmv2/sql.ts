/**
 * SQL-related classes for managing database views and materialized views
 */

import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { OlapTable } from "./olapTable";
import { SqlObject } from "./types";
import {
  ClickHouseEngines,
  createMaterializedView,
  dropView,
  populateTable,
  Sql,
  toStaticQuery,
} from "../index";

/**
 * Represents a generic SQL resource that requires setup and teardown commands.
 * Base class for constructs like Views and Materialized Views. Tracks dependencies.
 */
export class SqlResource {
  /** @internal */
  public readonly kind = "SqlResource";

  /** Array of SQL statements to execute for setting up the resource. */
  setup: readonly string[];
  /** Array of SQL statements to execute for tearing down the resource. */
  teardown: readonly string[];
  /** The name of the SQL resource (e.g., view name, materialized view name). */
  name: string;

  /** List of OlapTables or Views that this resource reads data from. */
  pullsDataFrom: SqlObject[];
  /** List of OlapTables or Views that this resource writes data to. */
  pushesDataTo: SqlObject[];

  /**
   * Creates a new SqlResource instance.
   * @param name The name of the resource.
   * @param setup An array of SQL DDL statements to create the resource.
   * @param teardown An array of SQL DDL statements to drop the resource.
   * @param options Optional configuration for specifying data dependencies.
   * @param options.pullsDataFrom Tables/Views this resource reads from.
   * @param options.pushesDataTo Tables/Views this resource writes to.
   */
  constructor(
    name: string,
    setup: readonly string[],
    teardown: readonly string[],
    options?: {
      pullsDataFrom?: SqlObject[];
      pushesDataTo?: SqlObject[];
    },
  ) {
    getMooseInternal().sqlResources.set(name, this);

    this.name = name;
    this.setup = setup;
    this.teardown = teardown;
    this.pullsDataFrom = options?.pullsDataFrom ?? [];
    this.pushesDataTo = options?.pushesDataTo ?? [];
  }
}

/**
 * Represents a database View, defined by a SQL SELECT statement based on one or more base tables or other views.
 * Inherits from SqlResource, providing setup (CREATE VIEW) and teardown (DROP VIEW) commands.
 */
export class View extends SqlResource {
  /**
   * Creates a new View instance.
   * @param name The name of the view to be created.
   * @param selectStatement The SQL SELECT statement that defines the view's logic.
   * @param baseTables An array of OlapTable or View objects that the `selectStatement` reads from. Used for dependency tracking.
   */
  constructor(
    name: string,
    selectStatement: string | Sql,
    baseTables: (OlapTable<any> | View)[],
  ) {
    if (typeof selectStatement !== "string") {
      selectStatement = toStaticQuery(selectStatement);
    }

    super(
      name,
      [
        `CREATE VIEW IF NOT EXISTS ${name} 
        AS ${selectStatement}`.trim(),
      ],
      [dropView(name)],
      {
        pullsDataFrom: baseTables,
      },
    );
  }
}

/**
 * Configuration options for creating a Materialized View.
 * @template T The data type of the records stored in the target table of the materialized view.
 */
interface MaterializedViewOptions<T> {
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
  constructor(options: MaterializedViewOptions<TargetTable>);

  /** @internal **/
  constructor(
    options: MaterializedViewOptions<TargetTable>,
    targetSchema: IJsonSchemaCollection.IV3_1,
    targetColumns: Column[],
  );
  constructor(
    options: MaterializedViewOptions<TargetTable>,
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
