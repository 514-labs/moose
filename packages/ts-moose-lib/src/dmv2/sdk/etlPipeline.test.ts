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

class MockSource {
  private count: number;
  private namePrefix: string;
  private valuePrefix: string;

  constructor(
    count: number,
    namePrefix: string = "item",
    valuePrefix: string = "value",
  ) {
    this.count = count;
    this.namePrefix = namePrefix;
    this.valuePrefix = valuePrefix;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<SourceData> {
    for (let i = 0; i < this.count; i++) {
      const sourceData = {
        name: `${this.namePrefix}_${i}`,
        value: `${this.valuePrefix}_${i}`,
      };
      console.log(`\nExtracted data: ${JSON.stringify(sourceData)}`);
      yield sourceData;
    }
  }
}

describe("ETLPipeline", () => {
  describe("run", () => {
    it("should handle empty data source gracefully", async () => {
      // Use inline anonymous class for simple empty source
      const source = {
        async *[Symbol.asyncIterator](): AsyncIterator<SourceData> {
          return;
        },
      };

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
      const source = new MockSource(50);
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
