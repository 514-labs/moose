import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal, TypedBase } from "./internal";
import {
  ClickHouseEngines,
  ConsumptionUtil,
  createMaterializedView,
  dropView,
  IngestionFormat,
  populateTable,
  Sql,
  toQuery,
} from "../index";

export type OlapConfig<T> = {
  orderByFields?: (keyof T & string)[];
  // equivalent to setting `engine: ClickHouseEngines.ReplacingMergeTree`
  deduplicate?: boolean;
  engine?: ClickHouseEngines;
  version?: string;
};

export interface StreamConfig<T> {
  parallelism?: number;
  retentionPeriod?: number; // seconds
  destination?: OlapTable<T>;
  version?: string;
}

export type IngestPipelineConfig<T> = {
  table: boolean | OlapConfig<T>;
  stream: boolean | Omit<StreamConfig<T>, "destination">;
  ingest: boolean | Omit<IngestConfig<T>, "destination">;
  version?: string;
};

export class OlapTable<T> extends TypedBase<T, OlapConfig<T>> {
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

export interface TransformConfig {
  version?: string;
}

export interface ConsumerConfig {
  version?: string;
}

export class Stream<T> extends TypedBase<T, StreamConfig<T>> {
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

    getMooseInternal().streams.set(name, this);
  }

  _transformations = new Map<
    string,
    [Stream<any>, SyncOrAsyncTransform<T, any>, TransformConfig][]
  >();
  _multipleTransformations?: (record: T) => [RoutedMessage];
  _consumers = new Array<{ consumer: Consumer<T>; config: ConsumerConfig }>();

  addTransform = <U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<T, U>,
    config?: TransformConfig,
  ) => {
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
  };

  addConsumer = (consumer: Consumer<T>, config?: ConsumerConfig) => {
    const consumerConfig = config ?? {};
    const hasVersion = this._consumers.some(
      (existing) => existing.config.version === consumerConfig.version,
    );

    if (!hasVersion) {
      this._consumers.push({ consumer, config: consumerConfig });
    }
  };

  routed = (values: ZeroOrMany<T>) => new RoutedMessage(this, values);

  addMultiTransform = (transformation: (record: T) => [RoutedMessage]) => {
    this._multipleTransformations = transformation;
  };
}

class RoutedMessage {
  destination: Stream<any>;
  values: ZeroOrMany<any>;
  constructor(destination: Stream<any>, values: ZeroOrMany<any>) {
    this.destination = destination;
    this.values = values;
  }
}

interface IngestConfig<T> {
  destination: Stream<T>;
  format?: IngestionFormat; // TODO: we may not need this
  version?: string;
}

export class IngestApi<T> extends TypedBase<T, IngestConfig<T>> {
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

    getMooseInternal().ingestApis.set(name, this);
  }
}

type ConsumptionHandler<T, R> = (
  params: T,
  utils: ConsumptionUtil,
) => Promise<R>;

interface EgressConfig<T> {
  version?: string;
}

export class ConsumptionApi<T, R = any> extends TypedBase<T, EgressConfig<T>> {
  _handler: ConsumptionHandler<T, R>;
  responseSchema: IJsonSchemaCollection.IV3_1;

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
    this._handler = handler;
    this.responseSchema = responseSchema ?? {
      version: "3.1",
      schemas: [{ type: "array", items: { type: "object" } }],
      components: { schemas: {} },
    };
    getMooseInternal().egressApis.set(name, this);
  }

  getHandler = (): ConsumptionHandler<T, R> => {
    return this._handler;
  };
}

export class IngestPipeline<T> extends TypedBase<T, IngestPipelineConfig<T>> {
  table?: OlapTable<T>;
  stream?: Stream<T>;
  ingestApi?: IngestApi<T>;

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
    }
  }
}

export type Aggregated<
  AggregationFunction extends string,
  ArgTypes extends any[] = [],
> = {
  _aggregationFunction?: AggregationFunction;
  _argTypes?: ArgTypes;
};
interface MaterializedViewOptions<T> {
  selectStatement: string | Sql;

  tableName: string;
  materializedViewName: string;

  engine?: ClickHouseEngines;
  orderByFields?: (keyof T & string)[];
}

export class SqlResource {
  setup: readonly string[];
  teardown: readonly string[];
  name: string;

  constructor(
    name: string,
    setup: readonly string[],
    teardown: readonly string[],
  ) {
    getMooseInternal().sqlResources.set(name, this);

    this.name = name;
    this.setup = setup;
    this.teardown = teardown;
  }
}

export class View extends SqlResource {
  constructor(name: string, selectStatement: string) {
    super(
      name,
      [
        `CREATE MATERIALIZED VIEW IF NOT EXISTS ${name} 
        AS ${selectStatement}`.trim(),
      ],
      [dropView(name)],
    );
  }
}

export class MaterializedView<TargetTable> extends SqlResource {
  targetTable: OlapTable<TargetTable>;

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
      const [query, params] = toQuery(selectStatement);
      if (Object.keys(params).length !== 0) {
        throw new Error(
          "Dynamic SQL is not allowed in the select statement in materialized view creation.",
        );
      }
      selectStatement = query;
    }

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
    );

    if (targetSchema === undefined || targetColumns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }
    this.targetTable = new OlapTable(
      options.tableName,
      {
        orderByFields: options.orderByFields,
      },
      targetSchema,
      targetColumns,
    );
  }
}
