import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal, TypedBase } from "./internal";
import { ConsumptionUtil, IngestionFormat } from "../index";

export type OlapConfig<T> = {
  orderByFields?: (keyof T & string)[];
  deduplicate?: boolean;
};

export interface StreamConfig<T> {
  parallelism?: number;
  retentionPeriod?: number; // seconds
  destination?: OlapTable<T>;
}

export type DataModelConfigV2<T> = {
  table: boolean | OlapConfig<T>;
  stream: boolean | Omit<StreamConfig<T>, "destination">;
  ingest: boolean | Omit<IngestConfig<T>, "destination">;
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

type ZeroOrMany<T> = T | T[] | undefined;
type SyncOrAsyncTransform<T, U> = (
  record: T,
) => ZeroOrMany<U> | Promise<ZeroOrMany<U>>;

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
    [Stream<any>, SyncOrAsyncTransform<T, any>]
  >();
  _multipleTransformations?: (record: T) => [RoutedMessage];

  addTransform = <U>(
    destination: Stream<U>,
    transformation: SyncOrAsyncTransform<T, U>,
  ) => {
    this._transformations.set(destination.name, [destination, transformation]);
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

type ConsumptionHandler<T> = (
  params: T,
  utils: ConsumptionUtil,
) => Promise<any>;

interface EgressConfig<T> {}

export class ConsumptionApi<T> extends TypedBase<T, EgressConfig<T>> {
  _handler: ConsumptionHandler<T>;

  constructor(name: string, handler: ConsumptionHandler<T>, config?: {});

  /** @internal **/
  constructor(
    name: string,
    handler: ConsumptionHandler<T>,
    config: EgressConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    handler: ConsumptionHandler<T>,
    config?: EgressConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config ?? {}, schema, columns);

    this._handler = handler;

    getMooseInternal().egressApis.set(name, this);
  }

  getHandler = (): ConsumptionHandler<T> => {
    return this._handler;
  };
}

export class IngestPipeline<T> extends TypedBase<T, DataModelConfigV2<T>> {
  table?: OlapTable<T>;
  stream?: Stream<T>;
  ingestApi?: IngestApi<T>;

  constructor(name: string, config: DataModelConfigV2<T>);

  /** @internal **/
  constructor(
    name: string,
    config: DataModelConfigV2<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config: DataModelConfigV2<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    super(name, config, schema, columns);

    if (config.table) {
      const tableConfig = config.table === true ? {} : config.table;
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
        ...(config.stream === true ? {} : config.stream),
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
        ...(config.stream === true ? {} : config.stream),
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
