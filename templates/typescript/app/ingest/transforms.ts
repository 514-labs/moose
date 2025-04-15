import { FooPipeline, BarPipeline, Foo, Bar } from "./models";

// Transform Foo events to Bar events
FooPipeline.stream?.addTransform(
  BarPipeline.stream!,
  (foo: Foo): Bar => ({
    primaryKey: foo.primaryKey,
    utcTimestamp: new Date(foo.timestamp * 1000), // Convert timestamp to Date
    hasText: foo.optionalText !== undefined,
    textLength: foo.optionalText?.length ?? 0,
  }),
);

// Add a streaming consumer to print Foo events
const printFooEvent = (foo: Foo): void => {
  console.log("Received Foo event:");
  console.log(`  Primary Key: ${foo.primaryKey}`);
  console.log(`  Timestamp: ${new Date(foo.timestamp * 1000)}`);
  console.log(`  Optional Text: ${foo.optionalText ?? "None"}`);
  console.log("---");
};

FooPipeline.stream?.addConsumer(printFooEvent);
