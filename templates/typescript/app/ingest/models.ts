import {
  IngestPipeline,
  IngestApi,
  Stream,
  Key,
  OlapTable,
  DeadLetterModel,
} from "@514labs/moose-lib";

/**
 * Data Pipeline: Raw Record (Foo) → Processed Record (Bar)
 * Raw (Foo) → HTTP → Raw Stream → Transform → Derived (Bar) → Processed Stream → DB Table
 *
 * Additionally demonstrates versioned ingest APIs for User data
 */

/** =======Data Models========= */

export interface Foo {
  id: Key<string>;
  timestamp: number;
  message: string;
  category?: string;
  priority?: number;
}

export interface FooV1 {
  primaryKey: Key<string>; // Unique ID
  timestamp: number; // Unix timestamp
  optionalText?: string; // Text to analyze
}

/** Analyzed text metrics derived from Foo */
export interface Bar {
  primaryKey: Key<string>; // From Foo.primaryKey
  utcTimestamp: Date; // From Foo.timestamp
  hasText: boolean; // From Foo.optionalText?
  textLength: number; // From Foo.optionalText.length
}

/** =======Pipeline Configuration========= */

export const deadLetterTable = new OlapTable<DeadLetterModel>("FooDeadLetter", {
  orderByFields: ["failedAt"],
});

export const FooPipeline = new IngestPipeline<Foo>("Foo", {
  table: true,
  stream: true,
  ingest: true,
});

export const FooPipelineV1 = new IngestPipeline<FooV1>("Foo", {
  table: false, // No table; only stream raw records
  stream: true, // Buffer ingested records
  ingest: true, // POST /ingest/Foo
  version: "1", // Version string for schema versioning
  deadLetterQueue: {
    destination: deadLetterTable,
  },
});

/** Buffering and storing processed records (@see transforms.ts for transformation logic) */
export const BarPipeline = new IngestPipeline<Bar>("Bar", {
  table: true, // Persist in ClickHouse table "Bar"
  stream: true, // Buffer processed records
  ingest: false, // No API; only derive from processed Foo records
});
