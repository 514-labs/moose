pub static TRANSFORM_FILE: &str = r#"
import {
  CompressionCodecs,
  CompressionTypes,
  Consumer,
  Kafka,
  KafkaMessage,
  Producer,
} from "npm:kafkajs";
import SnappyCodec from "npm:kafkajs-snappy";

CompressionCodecs[CompressionTypes.Snappy] = SnappyCodec;

const cwd = Deno.args[0] || Deno.cwd();

const getVersion = (): string => {
  const version = JSON.parse(Deno.readTextFileSync(`${cwd}/package.json`))
    .version as string;
  return version.replace(/\./g, "_");
};
const scrubVersionFromTopic = (topic: string): string => {
  return topic.replace(/(_\d+)+$/, "");
};
const version = getVersion();

const kafka = new Kafka({
  clientId: "deno-consumer",
  brokers: ["redpanda:9092"],
});

const consumer: Consumer = kafka.consumer({ groupId: "deno-group" });
const producer: Producer = kafka.producer({
  transactionalId: "deno-producer",
  maxInFlightRequests: 1,
  idempotent: true,
});

const getFlows = (): Map<string, Map<string, string>> => {
  const dirPath = `${cwd}/app/flows`;
  const flowsDir = Deno.readDirSync(dirPath);
  const output = new Map<string, Map<string, string>>();

  for (const source of flowsDir) {
    if (!source.isDirectory) continue;

    const flows = new Map<string, string>();
    const destinations = Deno.readDirSync(`${cwd}/app/flows/${source.name}`);
    for (const destination of destinations) {
      if (!destination.isDirectory) continue;

      const destinationFiles = Deno.readDirSync(
        `${cwd}/app/flows/${source.name}/${destination.name}`
      );
      for (const destinationFile of destinationFiles) {
        if (
          destinationFile.isFile &&
          destinationFile.name.toLowerCase() === "flow.ts"
        ) {
          flows.set(
            destination.name,
            `${dirPath}/${source.name}/${destination.name}/${destinationFile.name}`
          );
        }
      }
    }
    output.set(source.name, flows);
  }

  return output;
};

const handleMessage = async (
  flows: Map<string, string>,
  message: KafkaMessage,
  resolveOffset: (offset: string) => void
): Promise<void> => {
  for (let [destination, flowFilePath] of flows) {
    const transaction = await producer.transaction();
    try {
      const transform = await import(flowFilePath);
      const output = JSON.stringify(
        transform.default(JSON.parse(message.value.toString()))
      );
      await transaction.send({
        topic: `${destination}_${version}`,
        messages: [{ value: output }],
      });
      await transaction.commit();
      resolveOffset(message.offset);
      console.log(`Sent transformed data to ${destination}`);
    } catch (error) {
      await transaction.abort();
      console.error(`Failed to send transformed data to ${destination}: `, error);
    }
  }
};

const startConsumer = async (): Promise<void> => {
  const flows = getFlows();
  const flowTopics = Array.from(flows.keys()).map(
    (flow) => `${flow}_${version}`
  );

  await consumer.connect();
  await consumer.subscribe({ topics: flowTopics });

  await consumer.run({
    eachBatchAutoResolve: false,
    eachBatch: async ({ batch, resolveOffset, isRunning, isStale }) => {
      const topic = scrubVersionFromTopic(batch.topic);
      for (const message of batch.messages) {
        if (!isRunning() || isStale()) break;
        else if (!flows.has(topic)) continue;

        await handleMessage(flows.get(topic)!, message, resolveOffset);
      }
    },
  });
};

const startProducer = async (): Promise<void> => {
  await producer.connect();
};

startProducer()
  .then(() => {
    startConsumer()
      .then(() => {
        console.log("Consumer is running...");
      })
      .catch((error) => {
        console.error("Failed to start kafka consumer: ", error);
      });
  })
  .catch((error) => {
    console.error("Failed to start kafka producer: ", error);
  });
"#;
