import {
  Column,
  DataType,
  Nested,
  ArrayType,
  DataEnum,
} from "../dataModels/dataModelTypes";
import { IJsonSchemaCollection } from "typia/src/schemas/json/IJsonSchemaCollection";
import { getMooseInternal } from "./internal";
import { DataModelConfig } from "../index";
export class OlapTable<T> {
  schema: IJsonSchemaCollection.IV3_1;
  name: string;

  columns: {
    [columnName in keyof T]: Column;
  };
  columnArray: Column[];

  config: DataModelConfig<T>;

  constructor(name: string, config?: DataModelConfig<T>);

  // internal
  constructor(
    name: string,
    config: DataModelConfig<T>,
    schema: IJsonSchemaCollection.IV3_1,
    columns: Column[],
  );

  constructor(
    name: string,
    config?: DataModelConfig<T>,
    schema?: IJsonSchemaCollection.IV3_1,
    columns?: Column[],
  ) {
    if (schema === undefined || columns === undefined) {
      throw new Error(
        "Supply the type param T so that the schema is inserted by the compiler plugin.",
      );
    }

    this.schema = schema;
    this.columnArray = columns;
    const columnsObj = {} as any;
    columns.forEach((column) => {
      columnsObj[column.name] = column;
    });
    this.columns = columnsObj;

    this.name = name;
    this.config = config ?? {};

    getMooseInternal().tables.set(name, this);
  }
}
