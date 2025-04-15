import { cliLog, TaskDefinition, TaskFunction } from "@514labs/moose-lib";
import { Foo } from "../../ingest/models";
import fs from "fs";
import { randomUUID } from "crypto";

// Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
const ingest: TaskFunction = async (input: any) => {
  // Read the Unix word list
  const unixWords = fs
    .readFileSync("/usr/share/dict/words", "utf8")
    .split("\n");

  // Get a recent timestamp within the last n_days
  const getRecentTimestamp = (n_days: number) => {
    const millisecondsInDays = n_days * 24 * 60 * 60 * 1000;
    const intervalStartDate = Date.now() - millisecondsInDays;
    return intervalStartDate + Math.random() * millisecondsInDays;
  };

  // Get a random word from the word list
  const getRandomWord = (words: string[]) => {
    return words[Math.floor(Math.random() * words.length)];
  };

  // Generate 1000 rows of random Foo data and ingest it into the Foo Data Model
  for (let i = 0; i < 1000; i++) {
    const foo: Foo = {
      primaryKey: randomUUID(),
      timestamp: getRecentTimestamp(365),
      optionalText: Math.random() < 0.5 ? getRandomWord(unixWords) : undefined,
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
