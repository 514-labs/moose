import {
  CompressionCodecs,
  CompressionTypes,
  Consumer,
  Kafka,
  KafkaMessage,
  Producer,
} from "npm:kafkajs@2.2.4";
import SnappyCodec from "npm:kafkajs-snappy@1.1.0";
import { debounce } from "https://deno.land/std@0.220.0/async/debounce.ts";

CompressionCodecs[CompressionTypes.Snappy] = SnappyCodec;

const cwd = Deno.args[0] || Deno.cwd();
const FLOWS_DIR_PATH = `${cwd}/app/flows`;
const FLOW_FILE = "flow.ts";

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
  const flowsDir = Deno.readDirSync(FLOWS_DIR_PATH);
  const output = new Map<string, Map<string, string>>();

  for (const source of flowsDir) {
    if (!source.isDirectory) continue;

    const flows = new Map<string, string>();
    const destinations = Deno.readDirSync(`${FLOWS_DIR_PATH}/${source.name}`);
    for (const destination of destinations) {
      if (!destination.isDirectory) continue;

      const destinationFiles = Deno.readDirSync(
        `${FLOWS_DIR_PATH}/${source.name}/${destination.name}`
      );
      for (const destinationFile of destinationFiles) {
        if (
          destinationFile.isFile &&
          destinationFile.name.toLowerCase() === FLOW_FILE
        ) {
          flows.set(
            destination.name,
            `${FLOWS_DIR_PATH}/${source.name}/${destination.name}/${destinationFile.name}`
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
  for (const [destination, flowFilePath] of flows) {
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
      console.error(
        `Failed to send transformed data to ${destination}: `,
        error
      );
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
    // TODO: disable autoCommit and do transaction.sendOffsets
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
  console.log("Consumer is running...");
};

const startProducer = async (): Promise<void> => {
  await producer.connect();
  console.log("Producer is running...");
};

const startFlowsFilewatcher = async (): Promise<void> => {
  // https://examples.deno.land/watching-files
  const updateKafkaGroup = debounce(async (event: Deno.FsEvent) => {
    console.log("Flows updated, restarting kafka group...");
    await consumer.disconnect();
    await producer.disconnect();
    await startKafkaGroup();
  }, 200);

  const watcher = Deno.watchFs(`${FLOWS_DIR_PATH}`, { recursive: true });
  console.log(`Watching for changes to ${FLOWS_DIR_PATH}...`);

  for await (const event of watcher) {
    updateKafkaGroup(event);
  }
};

const startKafkaGroup = async (): Promise<void> => {
  try {
    await startProducer();

    try {
      await startConsumer();
      await startFlowsFilewatcher();
    } catch (error) {
      console.error("Failed to start kafka consumer: ", error);
    }
  } catch (error) {
    console.error("Failed to start kafka producer: ", error);
  }
};

startKafkaGroup();
