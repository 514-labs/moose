import { IJsonSchemaCollection } from "typia";
import { RoutedMessage, Stream, StreamConfig } from "./stream";
import { Consumer, ConsumerConfig, DeadLetter, DeadLetterModel, SyncOrAsyncTransform, TransformConfig } from "./types";
import { Column } from "../dataModels/dataModelTypes";
import { getMooseInternal } from "./internal";

function attachTypeGuard<T>(
    dl: DeadLetterModel,
    typeGuard: (input: any) => T,
  ): asserts dl is DeadLetter<T> {
    (dl as any).asTyped = () => typeGuard(dl.originalRecord);
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