/**
 * API-related classes for ingestion and consumption endpoints
 */

import { Column } from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { TypedBase } from "./typedBase";
import { ConsumptionUtil } from "./types";

/**
 * @template T The data type of the messages expected by the destination stream.
 */
export interface IngestConfig<T> {
  /**
   * The destination stream where the ingested data should be sent.
   */
  destination: any; // Stream<T> - avoiding circular dependency
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
 * @template T The data type of the request parameters.
 */
export interface EgressConfig<T> {
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
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
