import { FooPipeline, BarPipeline, Foo, Bar } from "./models";
import { DeadLetterQueue } from "@514labs/moose-lib";

// Transform Foo events to Bar events
FooPipeline.stream!.addTransform(
  BarPipeline.stream!,
  (foo: Foo): Bar => {
    if (foo.timestamp === 1728000000.0) {
      // magic value to test the dead letter queue
      throw new Error("blah");
    }

    return {
      primaryKey: foo.primaryKey,
      utcTimestamp: new Date(foo.timestamp * 1000), // Convert timestamp to Date
      hasText: foo.optionalText !== undefined,
      textLength: foo.optionalText?.length ?? 0,
    };
  },
  {
    deadLetterQueue: FooPipeline.deadLetterQueue,
  },
);

// Add a streaming consumer to print Foo events
const printFooEvent = (foo: Foo): void => {
  console.log("Received Foo event:");
  console.log(`  Primary Key: ${foo.primaryKey}`);
  console.log(`  Timestamp: ${new Date(foo.timestamp * 1000)}`);
  console.log(`  Optional Text: ${foo.optionalText ?? "None"}`);
  console.log("---");
};

FooPipeline.stream!.addConsumer(printFooEvent);

FooPipeline.stream!.addConsumer(printFooEvent);
FooPipeline.deadLetterQueue!.addConsumer((deadLetter) => {
  console.log(deadLetter);
  const foo: Foo = deadLetter.asTyped();
  console.log(foo);
});
