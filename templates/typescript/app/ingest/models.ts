import { IngestPipeline, Key } from "@514labs/moose-lib";

//Define the data model "Foo"
export interface Foo {
  primaryKey: Key<string>;
  timestamp: number;
  optionalText?: string;
}

//Automatically generate an ingest pipeline typed to "Foo"
//Includes an API endpoint and a streaming buffer - but no landing table
export const FooPipeline = new IngestPipeline<Foo>("Foo", {
  table: false,
  stream: true,
  ingest: true,
});

//Define the data model "Bar"
export interface Bar {
  primaryKey: Key<string>;
  utcTimestamp: Date;
  hasText: boolean;
  textLength: number;
}

//Automatically generate an ingest pipeline typed to "Bar" (a downstream data model from "Foo")
//Includes a streaming buffer and a landing table, but no API endpoint
export const BarPipeline = new IngestPipeline<Bar>("Bar", {
  table: true,
  stream: true,
  ingest: false,
});
//See functions folder for the streaming transformation Foo->Bar
