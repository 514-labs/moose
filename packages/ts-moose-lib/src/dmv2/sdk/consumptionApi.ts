import { IJsonSchemaCollection } from "typia";
import { TypedBase } from "../typedBase";
import { Column } from "../../dataModels/dataModelTypes";
import { getMooseInternal } from "../internal";
import type { ConsumptionUtil } from "../../consumption-apis/helpers";

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
export interface EgressConfig<T> {
  /**
   * An optional version string for this configuration.
   */
  version?: string;
  metadata?: { description?: string };
}

/**
 * Class representing a Consumption API that can be registered in Moose.
 * @template T The data type of the request parameters.
 * @template R The data type of the response.
 */
export class ConsumptionApi<T = any, R = any> extends TypedBase<
  T,
  EgressConfig<T>
> {
  protected _handler: ConsumptionHandler<T, R>;
  public responseSchema: IJsonSchemaCollection.IV3_1;

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
    this._handler = handler;
    this.responseSchema = responseSchema ?? {
      version: "3.1",
      schemas: [{ type: "array", items: { type: "object" } }],
      components: { schemas: {} },
    };
    const egressApis = getMooseInternal().egressApis;

    // Create a unique key that includes version information if available
    const version = config?.version;
    let apiKey: string;

    if (version) {
      // Use version-based key when version is specified
      apiKey = `v${version}/${name}`;
    } else {
      // For unversioned APIs, use the plain name for routing compatibility
      apiKey = name;
    }

    if (egressApis.has(apiKey)) {
      throw new Error(
        `Consumption API with name ${name}${version ? ` version ${version}` : ""} already exists`,
      );
    }
    egressApis.set(apiKey, this);
  }

  /**
   * Retrieves the handler function associated with this Consumption API.
   * @returns The handler function.
   */
  getHandler = (): ConsumptionHandler<T, R> => {
    return this._handler;
  };

  async call(baseUrl: string, queryParams: T): Promise<R> {
    // Construct the API endpoint URL
    const url = new URL(
      `${baseUrl.replace(/\/$/, "")}/consumption/${this.name}`,
    );

    const searchParams = url.searchParams;

    for (const [key, value] of Object.entries(queryParams as any)) {
      if (Array.isArray(value)) {
        // For array values, add each item as a separate query param
        for (const item of value) {
          if (item !== null && item !== undefined) {
            searchParams.append(key, String(item));
          }
        }
      } else if (value !== null && value !== undefined) {
        searchParams.append(key, String(value));
      }
    }

    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json",
      },
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const data = await response.json();
    return data as R;
  }
}
