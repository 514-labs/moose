/**
 * Stream-related classes for managing data streams and transformations
 */

import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { TypedBase } from "./typedBase";
import {
  ZeroOrMany,
  SyncOrAsyncTransform,
  Consumer,
  DeadLetterModel,
  DeadLetter,
  TransformConfig,
  ConsumerConfig,
} from "./types";
import { OlapTable } from "./olapTable";

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

export class RoutedMessage {
  destination: Stream<any>;
  values: ZeroOrMany<any>;
  constructor(destination: Stream<any>, values: ZeroOrMany<any>) {
    this.destination = destination;
    this.values = values;
  }
}

function attachTypeGuard<T>(
  dl: DeadLetterModel,
  typeGuard: (input: any) => T,
): asserts dl is DeadLetter<T> {
  (dl as any).asTyped = () => typeGuard(dl.originalRecord);
}
