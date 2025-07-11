import { FooPipeline, BarPipeline, Foo, Bar } from "./models";
import { DeadLetterQueue, MooseCache } from "@514labs/moose-lib";

// Transform Foo events to Bar events
FooPipeline.stream!.addTransform(
  BarPipeline.stream!,
  async (foo: Foo): Promise<Bar> => {
    /**
     * Checks cache for event -> Transform Foo to Bar -> cache result -> return Bar
     * If errors occur during transformation, the event is sent to DLQ
    */

    // Initialize cache
    const cache = await MooseCache.get();
    const cacheKey = `processed:${foo.primaryKey}`;

    // Check if we have processed this event before
    const cached = await cache.get<Bar>(cacheKey);
    if (cached) {
      console.log(`Using cached result for ${foo.primaryKey}`);
      return cached;
    }

    // Magic value to test the dead letter queue
    if (foo.timestamp === 1728000000.0) {
      throw new Error(`Test DLQ trigger for timestamp ${foo.timestamp}`);
    }

    // Transform Foo to Bar
    const result: Bar = {
      primaryKey: foo.primaryKey,
      utcTimestamp: new Date(foo.timestamp * 1000), // Convert timestamp to Date
      hasText: foo.optionalText !== undefined,
      textLength: foo.optionalText?.length ?? 0,
    };

    // Cache the result (1 hour retention)
    await cache.set(cacheKey, result, 3600);

    return result;
  },
  {
    deadLetterQueue: FooPipeline.deadLetterQueue,
  },
);

// DLQ consumer for handling failed events (alternate flow)
FooPipeline.deadLetterQueue!.addConsumer((deadLetter) => {
  console.log(deadLetter);
  const foo: Foo = deadLetter.asTyped();
  console.log(foo);
});

// Add a streaming consumer to print Foo events
const printFooEvent = (foo: Foo): void => {
  console.log("Received Foo event:");
  console.log(`  Primary Key: ${foo.primaryKey}`);
  console.log(`  Timestamp: ${new Date(foo.timestamp * 1000)}`);
  console.log(`  Optional Text: ${foo.optionalText ?? "None"}`);
  console.log("---");
};

FooPipeline.stream!.addConsumer(printFooEvent);
