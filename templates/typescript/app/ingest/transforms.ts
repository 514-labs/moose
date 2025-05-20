import { FooPipeline, BarPipeline, Foo, Bar } from "./models";
import { Type } from "@514labs/moose-lib/dist/dataModels/types";

// Transform Foo events to Bar events
FooPipeline.stream?.addTransform(BarPipeline.stream!, (foo: Foo): Bar => {
  // Create a date-only Date object for processDate
  const processDate = foo.dateOnly ? new Date(foo.dateOnly) : new Date();

  // Remove time component to make it a date-only value
  processDate.setUTCHours(0, 0, 0, 0);

  return {
    primaryKey: foo.primaryKey,
    utcTimestamp: new Date(foo.timestamp * 1000), // Convert timestamp to Date
    hasText: foo.optionalText !== undefined,
    textLength: foo.optionalText?.length ?? 0,
    processDate: processDate as Date & Type<"date">,
  };
});

// Add a streaming consumer to print Foo events
const printFooEvent = (foo: Foo): void => {
  console.log("Received Foo event:");
  console.log(`  Primary Key: ${foo.primaryKey}`);
  console.log(`  Timestamp: ${new Date(foo.timestamp * 1000)}`);
  console.log(`  Optional Text: ${foo.optionalText ?? "None"}`);
  console.log(`  Date Only: ${foo.dateOnly ?? "None"}`);
  console.log("---");
};

FooPipeline.stream?.addConsumer(printFooEvent);
