import * as cheerio from "cheerio";

/**
 * @module connectors
 *
 * @description
 * This module is used to create the connectors for MooseJS based applications.
 * These connectors are used to extract data from different applications, cloud
 * native systems and databases or to write data into specific analytical systems
 * such as a data warehouse or stream processing system.
 *
 *
 *
 *
 */

abstract class Connector {
  /**
   * Connectors manage the connection to the data source or destination.
   * They are used to extract data from different applications, cloud native
   * systems and databases or to write data into specific analytical systems
   * such as a data warehouse or stream processing system.
   *
   * Connectors are aware of the datamodels and the data structures of the
   * data source or destination. They allow developers to easily extract or
   * write data without having to infer the data that will be coming from a
   * specific source or going to a specific destination.
   *
   */

  abstract connect(): void;
  abstract disconnect(): void;
}

const processPageHTML = ($: cheerio.CheerioAPI) => {
  // Remove all the meta tags
  $("meta").remove();

  // Remove all the script tags
  $("script").remove();

  // Remove the head tag
  $("head").remove();

  // Remove svgs
  $("svg").remove();

  // Remove all the classes from all elements
  $("[class]").removeAttr("class");

  // Remove all the ids from all elements
  $("[id]").removeAttr("id");

  // Remove all the styles from all elements
  $("[style]").removeAttr("style");

  return $.html();
};

class ConnectorCreatorHelper {
  /**
   * The ConnectorCreatorHelper class is used to help developers and LLMs
   * create new connectors for MooseJS based applications. It provides
   * instrospection capabilities to the developer to understand the data
   * models and data structures of the data source or destination ahead of
   * actually writing the connector.
   */
  documenation: string;
  scanPage = async (url: string) => {
    const html = await fetch(url).then((res) => res.text());

    const $ = cheerio.load(html);
    processPageHTML($);

    return Response.json({ html: $.html() });
  };
}

// Models will all be collected in a single folder

abstract class Connection<MT> {
  /**
   * The connection class is used to extract or write a specific data model
   * from a data source or destination.
   */
}

abstract class ReadablelMethods<MT> {
  abstract readBatch(): MT[];
  abstract readOne(): MT;
}

abstract class ReadableRangedMethods<MT> extends ReadablelMethods<MT> {
  abstract readRange(start: any, end: any): MT[];
}

abstract class WritableMethods<MT> {
  abstract writeOne(data: MT): Response;
  abstract writeBatch(data: MT[]): Response;
}

abstract class OverWritableMethods<MT> {
  abstract overwriteOne(data: MT): Response;
  abstract overwriteBatch(data: MT[]): Response;
}

interface AbstractField {
  name: string;
  type: string;
  required: boolean;
  unique?: boolean;
  indexed?: boolean;
  defaultValue?: string;
  description?: string;
}

interface AbstractDataModel {
  name: string;
  fields: AbstractField[];
}

function mergeDataModels(
  base: AbstractDataModel,
  toMerge: AbstractDataModel[],
): AbstractDataModel {
  return {
    name: base.name,
    fields: base.fields.concat(toMerge.flatMap((m) => m.fields)),
  };
}

abstract class ConnectorCreatorScanner {
  /**
   * The ConnectorCreatorScanner class is used to scan the data models and
   * data structures of the data source or destination. It provides
   * instrospection capabilities to the developer to understand the data
   * models and data structures of the data source or destination ahead of
   * actually writing the connector.
   */
  abstract scanAll(): AbstractDataModel[];
  abstract scanOne(): AbstractDataModel[];
  abstract scanAllRecusively(): AbstractDataModel[];
}

// An ideal connector would have methods to read, write, update and delete data
// from the data source or destination. The connector would also have methods
// to connect and disconnect from the data source or destination. Methods would
// be consistent named across all connectors to make it easier for developers
// to switch between connectors.

// Define the common interface for all connectors
interface IConnector {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  read(): Promise<any>;
  write(data: any): Promise<void>;
}

// Define the abstract class for SaaS connectors
abstract class SaaSConnector implements IConnector {
  abstract connect(): Promise<void>;
  abstract disconnect(): Promise<void>;
  abstract read(): Promise<any>;
  abstract write(data: any): Promise<void>;
}

// Define the abstract class for Database connectors
abstract class DatabaseConnector implements IConnector {
  abstract connect(): Promise<void>;
  abstract disconnect(): Promise<void>;
  abstract read(): Promise<any>;
  abstract write(data: any): Promise<void>;
}

// Define the abstract class for FileSystem connectors
abstract class FileSystemConnector implements IConnector {
  abstract connect(): Promise<void>;
  abstract disconnect(): Promise<void>;
  abstract read(): Promise<any>;
  abstract write(data: any): Promise<void>;
}
