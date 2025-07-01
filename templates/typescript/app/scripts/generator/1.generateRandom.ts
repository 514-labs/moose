import { cliLog, TaskDefinition, TaskFunction } from "@514labs/moose-lib";
import { Foo } from "../../ingest/models";
import { faker } from "@faker-js/faker";

// Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
const ingest: TaskFunction = async (input: any) => {
  console.log("Starting data generation...");

  // Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
  for (let i = 0; i < 1000; i++) {
    const foo: Foo = {
      primaryKey: faker.string.uuid(),
      timestamp: faker.date.recent({ days: 365 }).getTime(),
      optionalText: Math.random() < 0.5 ? faker.lorem.text() : undefined,
    };

    try {
      const response = await fetch("http://localhost:4000/ingest/Foo", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(foo),
      });

      if (!response.ok) {
        console.log(
          `Failed to ingest record ${i}: ${response.status} ${response.statusText}`,
        );
      }
    } catch (error) {
      console.log(`Error ingesting record ${i}: ${error}`);
    }

    // Add a small delay to avoid overwhelming the server
    if (i % 100 === 0) {
      console.log(`Ingested ${i} records...`);
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }

  console.log("Data generation completed!");
  return {
    task: "ingest",
    data: "success",
  };
};

export default function createTask() {
  return {
    task: ingest,
  } as TaskDefinition;
}
