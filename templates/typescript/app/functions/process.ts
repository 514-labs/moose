import { FooPipeline, BarPipeline, Foo, Bar } from "../ingest/models";

//Function to transform Foo->Bar
//Automatically generates a streaming transformation that is orchestrated on each event from the Foo stream and writes to the Bar stream
FooPipeline.stream!.addTransform(BarPipeline.stream!, (foo: Foo): Bar => {
  return {
    primaryKey: foo.primaryKey,
    utcTimestamp: new Date(foo.timestamp),
    textLength: foo.optionalText?.length ?? 0,
    hasText: foo.optionalText !== null,
  };
});
