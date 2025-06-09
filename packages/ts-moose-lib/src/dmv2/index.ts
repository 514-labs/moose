/**
 * @module dmv2
 * This module defines the core Moose v2 data model constructs, including OlapTable, Stream, IngestApi, ConsumptionApi,
 * IngestPipeline, View, and MaterializedView. These classes provide a typed interface for defining and managing
 * data infrastructure components like ClickHouse tables, Redpanda streams, and data processing pipelines.
 */
import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { TypedBase } from "./typedBase";
import {
  ClickHouseEngines,
  createMaterializedView,
  dropView,
  populateTable,
  Sql,
  toStaticQuery,
} from "../index";
import { OlapConfig, OlapTable } from "./sdk/olapTable";
import { Stream, StreamConfig } from "./sdk/stream";
import { IngestApi, IngestConfig } from "./sdk/ingestApi";

/**
 * Configuration options for a complete ingestion pipeline, potentially including an Ingest API, a Stream, and an OLAP Table.
 * @template T The data type of the records being ingested.
 */
export type IngestPipelineConfig<T> = {
  /**
   * Configuration for the OLAP table component of the pipeline.
   * If `true`, a table with default settings is created.
   * If an `OlapConfig` object is provided, it specifies the table's configuration.
   * If `false` no OLAP table is created.
   */
  table: boolean | OlapConfig<T>;
  /**
   * Configuration for the stream component of the pipeline.
   * If `true`, a stream with default settings is created.
   * If a partial `StreamConfig` object (excluding `destination`) is provided, it specifies the stream's configuration.
   * The stream's destination will automatically be set to the pipeline's table if one exists.
   * If `false`, no stream is created.
   */
  stream: boolean | Omit<StreamConfig<T>, "destination">;
  /**
   * Configuration for the ingest API component of the pipeline.
   * If `true`, an ingest API with default settings is created.
   * If a partial `IngestConfig` object (excluding `destination`) is provided, it specifies the API's configuration.
   * The API's destination will automatically be set to the pipeline's stream if one exists. Requires a stream to be configured.
   * If `false`, no ingest API is created.
   */
  ingest: boolean | Omit<IngestConfig<T>, "destination">;
  /**
   * An optional version string applying to all components (table, stream, ingest) created by this pipeline configuration.
   */
  version?: string;
  metadata?: { description?: string };
};

/**
 * Represents a complete ingestion pipeline, potentially combining an Ingest API, a Stream, and an Olap Table
 * under a single name and configuration. Simplifies the setup of common ingestion patterns.
 *
 * @template T The data type of the records flowing through the pipeline. This type defines the schema for the
 *             Ingest API input, the Stream messages, and the Olap Table rows.
 */
export class IngestPipeline<T> extends TypedBase<T, IngestPipelineConfig<T>> {
  metadata?: { description?: string };
  /** The OLAP table component of the pipeline, if configured. */
  table?: OlapTable<T>;
  /** The stream component of the pipeline, if configured. */
  stream?: Stream<T>;
  /** The ingest API component of the pipeline, if configured. */
  ingestApi?: IngestApi<T>;

  /**
   * Creates a new IngestPipeline instance.
   * Based on the configuration, it automatically creates and links the IngestApi, Stream, and OlapTable components.
   *
   * @param name The base name for the pipeline components (e.g., "userData" could create "userData" table, "userData" stream, "userData" ingest API).
   * @param config Configuration specifying which components (table, stream, ingest) to create and their settings.
   */
  constructor(name: string, config: IngestPipelineConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: IngestPipelineConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: IngestPipelineConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);
    this.metadata = config?.metadata;

    if (config.table) {
      const tableConfig = {
        ...(typeof config.table === "object" ? config.table : {}),
        ...(config.version && { version: config.version }),
      };
      this.table = new OlapTable(
        name,
        tableConfig,
        this.schema,
        this.columnArray,
      );
    }

    if (config.stream) {
      const streamConfig = {
        destination: this.table,
        ...(typeof config.stream === "object" ? config.stream : {}),
        ...(config.version && { version: config.version }),
      };
      this.stream = new Stream(
        name,
        streamConfig,
        this.schema,
        this.columnArray,
      );
      (this.stream as any).pipelineParent = this;
    }

    if (config.ingest) {
      if (!this.stream) {
        throw new Error("Ingest API needs a stream to write to.");
      }

      const ingestConfig = {
        destination: this.stream,
        ...(typeof config.ingest === "object" ? config.ingest : {}),
        ...(config.version && { version: config.version }),
      };
      this.ingestApi = new IngestApi(
        name,
        ingestConfig,
        this.schema,
        this.columnArray,
      );
      (this.ingestApi as any).pipelineParent = this;
    }
  }
}

/**
 * A helper type used potentially for indicating aggregated fields in query results or schemas.
 * Captures the aggregation function name and argument types.
 * (Usage context might be specific to query builders or ORM features).
 *
 * @template AggregationFunction The name of the aggregation function (e.g., 'sum', 'avg', 'count').
 * @template ArgTypes An array type representing the types of the arguments passed to the aggregation function.
 */
export type Aggregated<
  AggregationFunction extends string,
  ArgTypes extends any[] = [],
> = {
  _aggregationFunction?: AggregationFunction;
  _argTypes?: ArgTypes;
};

type SqlObject = OlapTable<any> | SqlResource;

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

export { OlapTable, OlapConfig } from "./sdk/olapTable";
export {
  Stream,
  StreamConfig,
  DeadLetterModel,
  DeadLetter,
  DeadLetterQueue,
} from "./sdk/stream";

export { Workflow, Task } from "./sdk/workflow";

export { IngestApi, IngestConfig } from "./sdk/ingestApi";
export { ConsumptionApi, EgressConfig } from "./sdk/consumptionApi";
