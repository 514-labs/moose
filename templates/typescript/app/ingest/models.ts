import {
  IngestPipeline,
  Key,
  OlapTable,
  DeadLetterModel,
} from "@514labs/moose-lib";

// Raw data model
export interface Foo {
  primaryKey: Key<string>;
  timestamp: number;
  optionalText?: string;
}

// Processed data model
export interface Bar {
  primaryKey: Key<string>;
  utcTimestamp: Date;
  hasText: boolean;
  textLength: number;
}

export const deadLetterTable = new OlapTable<DeadLetterModel>("FooDeadLetter", {
  orderByFields: ["failedAt"],
});

/** Raw data ingestion */
export const FooPipeline = new IngestPipeline<Foo>("Foo", {
  table: false, // No table
  stream: true, // Buffer ingested records
  ingest: true, // POST /ingest/Foo
  deadLetterQueue: {
    destination: deadLetterTable,
  },
});

/** Buffering and storing processed records (@see transforms.ts for transformation logic) */
export const BarPipeline = new IngestPipeline<Bar>("Bar", {
  table: true, // Store in ClickHouse table "Bar"
  stream: true, // Buffer processed records
  ingest: false, // No API endpoint
});
