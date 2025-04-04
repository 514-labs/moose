import { FooPipeline, BarPipeline, Foo, Bar } from "../ingest/models";

FooPipeline.stream!.addTransform(BarPipeline.stream!, (foo: Foo): Bar => {
  return {
    primaryKey: foo.primaryKey,
    utcTimestamp: new Date(foo.timestamp),
    textLength: foo.optionalText?.length ?? 0,
    hasText: foo.optionalText !== null,
  };
});
