import { cliLog, TaskDefinition, TaskFunction } from "@514labs/moose-lib";
import { Foo } from "../../ingest/models";
import { faker } from "@faker-js/faker";

// Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
const ingest: TaskFunction = async (input: any) => {
  // Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
  for (let i = 0; i < 1000; i++) {
    const foo: Foo = {
      primaryKey: faker.string.uuid(),
      timestamp: faker.date.recent({ days: 365 }).getTime(),
      optionalText: Math.random() < 0.5 ? faker.lorem.text() : undefined,
    };

    await fetch("http://localhost:4000/ingest/Foo", {
      method: "POST",
      body: JSON.stringify(foo),
    });
  }

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
