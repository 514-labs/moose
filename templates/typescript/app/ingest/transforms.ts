import { FooPipelineV1, BarPipeline, FooV1, Bar } from "./models";
import { DeadLetterQueue, MooseCache } from "@514labs/moose-lib";

// Transform Foo events to Bar events
FooPipelineV1.stream!.addTransform(
  BarPipeline.stream!,
  async (foo: FooV1): Promise<Bar> => {
    /**
     * Transform Foo events to Bar events with error handling and caching.
     *
     * Normal flow:
     * 1. Check cache for previously processed events
     * 2. Transform Foo to Bar
     * 3. Cache the result
     * 4. Return transformed Bar event
     *
     * Alternate flow (DLQ):
     * - If errors occur during transformation, the event is sent to DLQ
     * - This enables separate error handling, monitoring, and retry strategies
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

    if (foo.timestamp === 1728000000.0) {
      // magic value to test the dead letter queue
      throw new Error("blah");
    }

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
    deadLetterQueue: FooPipelineV1.deadLetterQueue,
  },
);

// Add a streaming consumer to print Foo events
const printFooEvent = (foo: FooV1): void => {
  console.log("Received Foo event:");
  console.log(`  Primary Key: ${foo.primaryKey}`);
  console.log(`  Timestamp: ${new Date(foo.timestamp * 1000)}`);
  console.log(`  Optional Text: ${foo.optionalText ?? "None"}`);
  console.log("---");
};

FooPipelineV1.stream!.addConsumer(printFooEvent);

// DLQ consumer for handling failed events (alternate flow)
FooPipelineV1.deadLetterQueue!.addConsumer((deadLetter) => {
  console.log(deadLetter);
  const foo: FooV1 = deadLetter.asTyped();
  console.log(foo);
});
