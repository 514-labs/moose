import { FooPipeline, BarPipeline, Foo, Bar } from "../datamodels/models";

if (FooPipeline.stream && BarPipeline.stream) {
  FooPipeline.stream.addTransform(BarPipeline.stream, (foo: Foo): Bar => {
    return {
      primaryKey: foo.primaryKey,
      utcTimestamp: new Date(foo.timestamp),
      textLength: foo.optionalText?.length ?? 0,
      hasText: foo.optionalText !== null,
    };
  });
}
