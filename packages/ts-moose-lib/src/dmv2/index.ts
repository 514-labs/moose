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
  ConsumptionUtil,
  createMaterializedView,
  dropView,
  populateTable,
  Sql,
  toQuery,
  toStaticQuery,
} from "../index";

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
 * Configuration options for a data stream (e.g., a Redpanda topic).
 * @template T The data type of the messages in the stream.
 */
export interface StreamConfig<T> {
  /**
   * Specifies the number of partitions for the stream. Affects parallelism and throughput.
   */
  parallelism?: number;
  /**
   * Specifies the data retention period for the stream in seconds. Messages older than this may be deleted.
   */
  retentionPeriod?: number; // seconds
  /**
   * An optional destination OLAP table where messages from this stream should be automatically ingested.
   */
  destination?: OlapTable<T>;
  /**
   * An optional version string for this configuration. Can be used for tracking changes or managing deployments.
   */
  version?: string;
  metadata?: { description?: string };
}

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

type ZeroOrMany<T> = T | T[] | undefined | null;
type SyncOrAsyncTransform<T, U> = (
  record: T,
) => ZeroOrMany<U> | Promise<ZeroOrMany<U>>;
type Consumer<T> = (record: T) => Promise<void> | void;

export interface DeadLetterModel {
  originalRecord: Record<string, any>;
  errorMessage: string;
  errorType: string;
  failedAt: Date;
  source: "api" | "transform" | "table";
}

export interface DeadLetter<T> extends DeadLetterModel {
  asTyped: () => T;
}

function attachTypeGuard<T>(
  dl: DeadLetterModel,
  typeGuard: (input: any) => T,
): asserts dl is DeadLetter<T> {
  (dl as any).asTyped = () => typeGuard(dl.originalRecord);
}

export interface TransformConfig<T> {
  version?: string;
  metadata?: { description?: string };
  deadLetterQueue?: DeadLetterQueue<T>;
}

export interface ConsumerConfig<T> {
  version?: string;
  deadLetterQueue?: DeadLetterQueue<T>;
}

/**
 * Represents a data stream, typically corresponding to a Redpanda topic.
 * Provides a typed interface for producing to and consuming from the stream, and defining transformations.
 *
 * @template T The data type of the messages flowing through the stream. The structure of T defines the message schema.
 */
export class Stream<T> extends TypedBase<T, StreamConfig<T>> {
  metadata?: { description?: string };
  /**
   * Creates a new Stream instance.
   * @param name The name of the stream. This name is used for the underlying Redpanda topic.
   * @param config Optional configuration for the stream.
   */
  constructor(name: string, config?: StreamConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: StreamConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config?: StreamConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config ?? {}, schema, columns);
    this.metadata = config?.metadata;
    getMooseInternal().streams.set(name, this);
  }

  _transformations = new Map<
    string,
    [Stream<any>, SyncOrAsyncTransform<T, any>, TransformConfig<T>][]
  >();
  _multipleTransformations?: (record: T) => [RoutedMessage];
  _consumers = new Array<{
    consumer: Consumer<T>;
    config: ConsumerConfig<T>;
  }>();

  /**
   * Adds a transformation step that processes messages from this stream and sends the results to a destination stream.
   * Multiple transformations to the same destination stream can be added if they have distinct `version` identifiers in their config.
   *
   * @template U The data type of the messages in the destination stream.
   * @param destination The destination stream for the transformed messages.
   * @param transformation A function that takes a message of type T and returns zero or more messages of type U (or a Promise thereof).
   *                       Return `null` or `undefined` or an empty array `[]` to filter out a message. Return an array to emit multiple messages.
   * @param config Optional configuration for this specific transformation step, like a version.
   */
  addTransform<U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<T, U>,
    config?: TransformConfig<T>,
  ) {
    const transformConfig = config ?? {};

    if (this._transformations.has(destination.name)) {
      const existingTransforms = this._transformations.get(destination.name)!;
      const hasVersion = existingTransforms.some(
        ([_, __, cfg]) => cfg.version === transformConfig.version,
      );

      if (!hasVersion) {
        existingTransforms.push([destination, transformation, transformConfig]);
      }
    } else {
      this._transformations.set(destination.name, [
        [destination, transformation, transformConfig],
      ]);
    }
  }

  /**
   * Adds a consumer function that processes messages from this stream.
   * Multiple consumers can be added if they have distinct `version` identifiers in their config.
   *
   * @param consumer A function that takes a message of type T and performs an action (e.g., side effect, logging). Should return void or Promise<void>.
   * @param config Optional configuration for this specific consumer, like a version.
   */
  addConsumer(consumer: Consumer<T>, config?: ConsumerConfig<T>) {
    const consumerConfig = config ?? {};
    const hasVersion = this._consumers.some(
      (existing) => existing.config.version === consumerConfig.version,
    );

    if (!hasVersion) {
      this._consumers.push({ consumer, config: consumerConfig });
    }
  }

  /**
   * Helper method for `addMultiTransform` to specify the destination and values for a routed message.
   * @param values The value or values to send to this stream.
   * @returns A `RoutedMessage` object associating the values with this stream.
   */
  routed = (values: ZeroOrMany<T>) => new RoutedMessage(this, values);

  /**
   * Adds a single transformation function that can route messages to multiple destination streams.
   * This is an alternative to adding multiple individual `addTransform` calls.
   * Only one multi-transform function can be added per stream.
   *
   * @param transformation A function that takes a message of type T and returns an array of `RoutedMessage` objects,
   *                       each specifying a destination stream and the message(s) to send to it.
   */
  addMultiTransform(transformation: (record: T) => [RoutedMessage]) {
    this._multipleTransformations = transformation;
  }
}

export class DeadLetterQueue<T> extends Stream<DeadLetterModel> {
  constructor(name: string, config?: StreamConfig<DeadLetterModel>);

  /** @internal **/
  constructor(
    name: string,
    config: StreamConfig<DeadLetterModel>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    validate: (originalRecord: any) => T,
  );

  constructor(
    name: string,
    config?: StreamConfig<DeadLetterModel>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    typeGuard?: (originalRecord: any) => T,
  ) {
    if (
      schema === undefined ||
      columns === undefined ||
      typeGuard === undefined
    ) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    super(name, config ?? {}, schema, columns);
    this.typeGuard = typeGuard;
    getMooseInternal().streams.set(name, this);
  }
  private typeGuard: (originalRecord: any) => T;

  addTransform<U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<DeadLetter<T>, U>,
    config?: TransformConfig<DeadLetterModel>,
  ) {
    const withValidate: SyncOrAsyncTransform<DeadLetterModel, U> = (
      deadLetter,
    ) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return transformation(deadLetter);
    };
    super.addTransform(destination, withValidate, config);
  }
  addConsumer(
    consumer: Consumer<DeadLetter<T>>,
    config?: ConsumerConfig<DeadLetterModel>,
  ) {
    const withValidate: Consumer<DeadLetterModel> = (deadLetter) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return consumer(deadLetter);
    };
    super.addConsumer(withValidate, config);
  }

  addMultiTransform(
    transformation: (record: DeadLetter<T>) => [RoutedMessage],
  ) {
    const withValidate: (record: DeadLetterModel) => [RoutedMessage] = (
      deadLetter,
    ) => {
      attachTypeGuard<T>(deadLetter, this.typeGuard);
      return transformation(deadLetter);
    };
    super.addMultiTransform(withValidate);
  }
}

class RoutedMessage {
  destination: Stream<any>;
  values: ZeroOrMany<any>;
  constructor(destination: Stream<any>, values: ZeroOrMany<any>) {
    this.destination = destination;
    this.values = values;
  }
}

/**
 * @template T The data type of the messages expected by the destination stream.
 */
interface IngestConfig<T> {
  /**
   * The destination stream where the ingested data should be sent.
   */
  destination: Stream<T>;
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * Represents an Ingest API endpoint, used for sending data into a Moose system, typically writing to a Stream.
 * Provides a typed interface for the expected data format.
 *
 * @template T The data type of the records that this API endpoint accepts. The structure of T defines the expected request body schema.
 */
export class IngestApi<T> extends TypedBase<T, IngestConfig<T>> {
  metadata?: { description?: string };
  /**
   * Creates a new IngestApi instance.
   * @param name The name of the ingest API endpoint.
   * @param config Configuration for the ingest API, including the destination stream.
   */
  constructor(name: string, config?: IngestConfig<T>);

  /** @internal **/
  constructor(
    name: string,
    config: IngestConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: IngestConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);
    this.metadata = config?.metadata;
    getMooseInternal().ingestApis.set(name, this);
  }
}

/**
 * Defines the signature for a handler function used by a Consumption API.
 * @template T The expected type of the request parameters or query parameters.
 * @template R The expected type of the response data.
 * @param params An object containing the validated request parameters, matching the structure of T.
 * @param utils Utility functions provided to the handler, e.g., for database access (`runSql`).
 * @returns A Promise resolving to the response data of type R.
 */
type ConsumptionHandler<T, R> = (
  params: T,
  utils: ConsumptionUtil,
) => Promise<R>;

/**
 * @template T The data type of the request parameters.
 */
interface EgressConfig<T> {
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * Represents a Consumption API endpoint (Egress API), used for querying data from a Moose system.
 * Exposes data, often from an OlapTable or derived through a custom handler function.
 *
 * @template T The data type defining the expected structure of the API's query parameters.
 * @template R The data type defining the expected structure of the API's response body. Defaults to `any`.
 */
export class ConsumptionApi<T, R = any> extends TypedBase<T, EgressConfig<T>> {
  metadata?: { description?: string };
  /** @internal The handler function that processes requests and generates responses. */
  _handler: ConsumptionHandler<T, R>;
  /** @internal The JSON schema definition for the response type R. */
  responseSchema: IJsonSchemaCollection.IV3_1;

  /**
   * Creates a new ConsumptionApi instance.
   * @param name The name of the consumption API endpoint.
   * @param handler The function to execute when the endpoint is called. It receives validated query parameters and utility functions.
   * @param config Optional configuration for the consumption API.
   */
  constructor(name: string, handler: ConsumptionHandler<T, R>, config?: {});

  /** @internal **/
  constructor(
    name: string,
    handler: ConsumptionHandler<T, R>,
    config: EgressConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
    responseSchema: IJsonSchemaCollection.IV3_1,
  );

  constructor(
    name: string,
    handler: ConsumptionHandler<T, R>,
    config?: EgressConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
    responseSchema?: IJsonSchemaCollection.IV3_1,
  ) {
    super(name, config ?? {}, schema, columns);
    this.metadata = config?.metadata;
    this._handler = handler;
    this.responseSchema = responseSchema ?? {
      version: "3.1",
      schemas: [{ type: "array", items: { type: "object" } }],
      components: { schemas: {} },
    };
    getMooseInternal().egressApis.set(name, this);
  }

  /**
   * Retrieves the handler function associated with this Consumption API.
   * @returns The handler function.
   */
  getHandler = (): ConsumptionHandler<T, R> => {
    return this._handler;
  };
}

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

type TaskHandler<T, R> = (input: T) => Promise<R | void>;

export interface TaskConfig<T, R> {
  run: TaskHandler<T, R>;
  onComplete?: Task<R, any>[];
  timeout?: string;
  retries?: number;
}

export class Task<T, R = any> extends TypedBase<T, TaskConfig<T, R>> {
  constructor(name: string, config: TaskConfig<T, R>);

  /** @internal **/
  constructor(
    name: string,
    config: TaskConfig<T, R>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: TaskConfig<T, R>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);
  }
}

export interface WorkflowConfig {
  startingTask: Task<any, any>;
  retries?: number;
  timeout?: string;
  schedule?: string;
}

export class Workflow {
  constructor(
    readonly name: string,
    readonly config: WorkflowConfig,
  ) {
    getMooseInternal().workflows.set(name, this);
  }
}
