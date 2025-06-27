import { expect } from "chai";
import { ETLPipeline } from "./etlPipeline";

interface SourceData {
  name: string;
  value: string;
}

interface TransformedData {
  id: string;
  value: string;
}

export class InMemorySource {
  async *[Symbol.asyncIterator](): AsyncIterator<SourceData> {
    for (let i = 0; i < 3; i++) {
      const sourceData = { name: `source_${i}`, value: `value_${i}` };
      console.log(`\nExtracted data: ${JSON.stringify(sourceData)}`);
      yield sourceData;
    }
  }
}

describe("ETLPipeline", () => {
  describe("run", () => {
    it("should handle empty data source gracefully", async () => {
      class EmptySource {
        async *[Symbol.asyncIterator](): AsyncIterator<SourceData> {
          // Yield nothing
          return;
        }
      }

      const source = new EmptySource();
      const loadedData: TransformedData[] = [];

      const pipeline = new ETLPipeline<SourceData, TransformedData>(
        "emptyPipeline",
        {
          extract: source,
          transform: async (sourceData: SourceData) => ({
            id: `transformed_${sourceData.name}`,
            value: `transformed_${sourceData.value}`,
          }),
          load: async (data: TransformedData[]) => {
            loadedData.push(...data);
          },
        },
      );

      await pipeline.run();

      // Should complete without errors and process no data
      expect(loadedData).to.have.length(0);
    });

    it("should process large batches correctly", async () => {
      class LargeSource {
        async *[Symbol.asyncIterator](): AsyncIterator<SourceData> {
          for (let i = 0; i < 50; i++) {
            yield { name: `item_${i}`, value: `value_${i}` };
          }
        }
      }

      const source = new LargeSource();
      const loadedData: TransformedData[] = [];

      const pipeline = new ETLPipeline<SourceData, TransformedData>(
        "largePipeline",
        {
          extract: source,
          transform: async (sourceData: SourceData) => ({
            id: `transformed_${sourceData.name}`,
            value: `transformed_${sourceData.value}`,
          }),
          load: async (data: TransformedData[]) => {
            loadedData.push(...data);
          },
        },
      );

      await pipeline.run();

      // Should process all 50 items across multiple batches
      expect(loadedData).to.have.length(50);
      expect(loadedData[0].id).to.equal("transformed_item_0");
      expect(loadedData[49].id).to.equal("transformed_item_49");
    });
  });
});
