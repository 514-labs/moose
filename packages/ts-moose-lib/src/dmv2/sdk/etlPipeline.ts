import { IJsonSchemaCollection } from "typia";
import { Workflow, Task } from "./workflow";
import { OlapTable } from "./olapTable";

interface BatchResult<T> {
  items: T[];
  hasMore: boolean;
}

interface TransformedResult<U> {
  items: U[];
}

class InternalBatcher<T> {
  private iterator: AsyncIterator<T>;
  private batchSize: number;

  constructor(asyncIterable: AsyncIterable<T>, batchSize = 20) {
    this.iterator = asyncIterable[Symbol.asyncIterator]();
    this.batchSize = batchSize;
  }

  async getNextBatch(): Promise<BatchResult<T>> {
    const items: T[] = [];

    for (let i = 0; i < this.batchSize; i++) {
      const { value, done } = await this.iterator.next();

      if (done) {
        return { items, hasMore: false };
      }

      items.push(value);
    }

    return { items, hasMore: true };
  }
}

export interface ETLPipelineConfig<T, U> {
  extract: AsyncIterable<T> | (() => AsyncIterable<T>);
  transform: (sourceData: T) => Promise<U>;
  load: ((data: U[]) => Promise<void>) | OlapTable<U>;
}

export class ETLPipeline<T, U> {
  private batcher: InternalBatcher<T>;

  constructor(
    readonly name: string,
    readonly config: ETLPipelineConfig<T, U>,
  ) {
    // Create batcher from user's extract config
    const iterable =
      typeof config.extract === "function" ? config.extract() : config.extract;

    this.batcher = new InternalBatcher(iterable);
    const nullSchema: IJsonSchemaCollection.IV3_1 = {
      version: "3.1",
      components: {
        schemas: {},
      },
      schemas: [
        {
          type: "null",
        },
      ],
    };

    // Create Extract Task
    const extractTask = new Task<null, BatchResult<T>>(
      `${name}_extract`,
      {
        run: async () => {
          console.log(`Running extract task for ${name}...`);
          const batch = await this.batcher.getNextBatch();
          console.log(
            `Extract task completed with ${batch.items.length} items`,
          );
          return batch;
        },
        retries: 1,
        timeout: "30m",
      },
      nullSchema,
      [],
    );

    // Create Transform Task
    const transformTask = new Task<BatchResult<T>, TransformedResult<U>>(
      `${name}_transform`,
      {
        run: async (batch: { items: T[]; hasMore: boolean }) => {
          console.log(
            `Running transform task for ${name} with ${batch.items.length} items...`,
          );
          const transformedItems: U[] = [];

          for (const item of batch.items) {
            const transformed = await this.config.transform(item);
            transformedItems.push(transformed);
          }

          console.log(
            `Transform task completed with ${transformedItems.length} items`,
          );
          return { items: transformedItems };
        },
        retries: 1,
        timeout: "30m",
      },
      nullSchema,
      [],
    );

    // Create Load Task
    const loadTask = new Task<TransformedResult<U>, void>(
      `${name}_load`,
      {
        run: async (transformedItems: TransformedResult<U>) => {
          console.log(
            `Running load task for ${name} with ${transformedItems.items.length} items...`,
          );

          // Handle both function and OlapTable
          if ("insert" in this.config.load) {
            // It's an OlapTable - insert entire batch
            await this.config.load.insert(transformedItems.items);
          } else {
            // It's a function - call with entire array
            await this.config.load(transformedItems.items);
          }

          console.log(`Load task completed`);
        },
        retries: 1,
        timeout: "30m",
      },
      nullSchema,
      [],
    );

    // Wire tasks together
    extractTask.config.onComplete = [transformTask];
    transformTask.config.onComplete = [loadTask];

    new Workflow(name, {
      startingTask: extractTask,
      retries: 1,
      timeout: "30m",
    });
  }

  // Execute the entire ETL pipeline
  async run(): Promise<void> {
    console.log(`Starting ETL Pipeline: ${this.name}`);

    let batchNumber = 1;
    do {
      console.log(`Processing batch ${batchNumber}...`);
      const batch = await this.batcher.getNextBatch();

      if (batch.items.length === 0) {
        break;
      }

      // Transform all items in the batch
      const transformedItems: U[] = [];
      for (const extractedData of batch.items) {
        const transformedData = await this.config.transform(extractedData);
        transformedItems.push(transformedData);
      }

      // Load the entire batch
      if ("insert" in this.config.load) {
        // It's an OlapTable - insert entire batch
        await this.config.load.insert(transformedItems);
      } else {
        // It's a function - call with entire array
        await this.config.load(transformedItems);
      }

      console.log(
        `Completed batch ${batchNumber} with ${batch.items.length} items`,
      );
      batchNumber++;

      if (!batch.hasMore) {
        break;
      }
    } while (true);

    console.log(`Completed ETL Pipeline: ${this.name}`);
  }
}
