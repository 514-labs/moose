import { watch } from "npm:chokidar@3.6.0";
import {
  CompressionCodecs,
  CompressionTypes,
  Consumer,
  Kafka,
  KafkaMessage,
  Producer,
  SASLOptions,
} from "npm:kafkajs@2.2.4";
import SnappyCodec from "npm:kafkajs-snappy@1.1.0";
import { antiCachePath } from "./ts-helpers.ts";

CompressionCodecs[CompressionTypes.Snappy] = SnappyCodec;

const cwd = Deno.args[0] || Deno.cwd();
const FLOWS_DIR_PATH = `${cwd}/app/flows`;
const FLOW_FILE = "flow.ts";
const CONSUMER_ID = "deno-group";

const FILEWATCHER_ENABLED = Deno.env.get("FILEWATCHER_ENABLED") || false;
const BROKER = Deno.env.get("MOOSE_REDPANDA_CONFIG__BROKER");
const SASL_USERNAME = Deno.env.get("MOOSE_REDPANDA_CONFIG__SASL_USERNAME");
const SASL_PASSWORD = Deno.env.get("MOOSE_REDPANDA_CONFIG__SASL_PASSWORD");
const SASL_MECHANISM = Deno.env.get("MOOSE_REDPANDA_CONFIG__SASL_MECHANISM");
const SECURITY_PROTOCOL = Deno.env.get(
  "MOOSE_REDPANDA_CONFIG__SECURITY_PROTOCOL",
);
console.log(
  `FLOWS_DIR_PATH: ${FLOWS_DIR_PATH} | BROKER: ${BROKER} | FILEWATCHER_ENABLED: ${FILEWATCHER_ENABLED} | SECURITY_PROTOCOL: ${SECURITY_PROTOCOL} | SASL_MECHANISM: ${SASL_MECHANISM}`,
);

if (!BROKER) {
  console.error("Missing environment variable MOOSE_REDPANDA_CONFIG__BROKER");
  Deno.exit(1);
}

const getVersion = (): string => {
  const version = JSON.parse(Deno.readTextFileSync(`${cwd}/package.json`))
    .version as string;
  return version.replace(/\./g, "_");
};
const version = getVersion();

const getSaslConfig = (): SASLOptions | undefined => {
  const mechanism = SASL_MECHANISM ? SASL_MECHANISM.toLowerCase() : "";
  switch (mechanism) {
    case "plain":
    case "scram-sha-256":
    case "scram-sha-512":
      return {
        mechanism: mechanism,
        username: SASL_USERNAME || "",
        password: SASL_PASSWORD || "",
      };
    default:
      console.log(`Unsupported SASL mechanism: ${SASL_MECHANISM}`);
      return undefined;
  }
};

const kafka = new Kafka({
  clientId: "deno-consumer",
  brokers: [BROKER],
  ssl: SECURITY_PROTOCOL === "SASL_SSL",
  sasl: getSaslConfig(),
});

const consumer: Consumer = kafka.consumer({ groupId: CONSUMER_ID });
const producer: Producer = kafka.producer({
  transactionalId: "deno-producer",
  maxInFlightRequests: 1,
  idempotent: true,
});

interface VersionSync {
  sourceTable: string;
  targetTable: string;

  filePath: string;
}

const migrationRegex =
  /^([a-zA-Z0-9_]+)_migrate__([0-9_]+)__(([a-zA-Z0-9_]+)__)?([0-9_]+).ts$/;

const getMigrations = (): Map<string, VersionSync> => {
  const flowsDir = Deno.readDirSync(FLOWS_DIR_PATH);
  const output = new Map<string, VersionSync>();
  for (const entry of flowsDir) {
    if (entry.isDirectory) continue;
    const names = migrationRegex.exec(entry.name);
    if (names === null) continue;

    const sourceModel = names[1];
    const targetModel = names[4] ?? sourceModel;
    const sourceVersion = names[2];
    const targetVersion = names[5];

    const filePath = `${FLOWS_DIR_PATH}/${entry.name}`;

    const sourceTable = `${sourceModel}_${sourceVersion}`;
    const targetTable = `${targetModel}_${targetVersion}`;

    output.set(sourceTable, {
      sourceTable,
      targetTable,
      filePath,
    });
  }

  return output;
};

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
        `${FLOWS_DIR_PATH}/${source.name}/${destination.name}`,
      );
      for (const destinationFile of destinationFiles) {
        if (
          destinationFile.isFile &&
          destinationFile.name.toLowerCase() === FLOW_FILE
        ) {
          flows.set(
            `${destination.name}_${version}`,
            `${FLOWS_DIR_PATH}/${source.name}/${destination.name}/${destinationFile.name}`,
          );
        }
      }
    }

    if (flows.size > 0) {
      output.set(`${source.name}_${version}`, flows);
    }
  }

  return output;
};

const jsonDateReviver = (key: string, value: unknown): unknown => {
  const iso8601Format =
    /^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))([T\s]((([01]\d|2[0-3])((:?)[0-5]\d)?|24\:?00)([\.,]\d+(?!:))?)?(\17[0-5]\d([\.,]\d+)?)?([zZ]|([\+-])([01]\d|2[0-3]):?([0-5]\d)?)?)?)?$/;

  if (typeof value === "string" && iso8601Format.test(value)) {
    return new Date(value);
  }

  return value;
};

const handleMessage = async (
  topic: string,
  partition: number,
  flows: Map<string, string>,
  versionSync: VersionSync | undefined,
  message: KafkaMessage,
): Promise<void> => {
  const transaction = await producer.transaction();
  let didTransform = false;

  try {
    if (versionSync !== undefined) {
      didTransform = true;
      await transaction.send({
        topic: `${versionSync.sourceTable}_${versionSync.targetTable}_input`,
        messages: [{ value: message.value }],
      });
    }
    for (const [destinationTopic, flowFilePath] of flows) {
      const transform = await import(antiCachePath(flowFilePath));
      const transformedData = await transform.default(
        JSON.parse(message.value.toString(), jsonDateReviver),
      );

      if (transformedData) {
        await transaction.send({
          topic: destinationTopic,
          messages: [{ value: JSON.stringify(transformedData) }],
        });
        didTransform = true;
        console.log(`Sent transformed data to ${destinationTopic}`);
      }
    }

    if (didTransform) {
      await transaction.sendOffsets({
        consumerGroupId: CONSUMER_ID,
        topics: [
          { topic, partitions: [{ partition, offset: message.offset }] },
        ],
      });
      await transaction.commit();
    } else {
      await transaction.abort();
    }

    // https://github.com/tulios/kafkajs/issues/540#issuecomment-601443828
    // Without it, the consumer receives last message on restart.
    await consumer.commitOffsets([
      { topic, partition, offset: (Number(message.offset) + 1).toString() },
    ]);
  } catch (error) {
    await transaction.abort();
    console.error(`Failed to send transformed data`, error);
  }
};

const startConsumer = async (): Promise<void> => {
  const flows = getFlows();
  const migrations = getMigrations();

  for (const [_, vs] of migrations) {
    if (!flows.has(vs.sourceTable)) {
      flows.set(vs.sourceTable, new Map());
    }
    let targets = new Map<string, string>();
    targets.set(`${vs.sourceTable}_${vs.targetTable}_output`, vs.filePath);
    flows.set(`${vs.sourceTable}_${vs.targetTable}_input`, targets);
  }

  const flowTopics = Array.from(flows.keys());

  await consumer.connect();
  await consumer.subscribe({ topics: flowTopics, fromBeginning: false });
  console.log("Subscribed to topics: ", flowTopics.join(", "));

  await consumer.run({
    autoCommit: false,
    eachMessage: async ({ topic, partition, message }) => {
      if (flows.has(topic) || migrations.has(topic)) {
        await handleMessage(
          topic,
          partition,
          flows.get(topic),
          migrations.get(topic),
          message,
        );
      }
    },
  });
  console.log("Consumer is running...");
};

const startProducer = async (): Promise<void> => {
  await producer.connect();
  console.log("Producer is running...");
};

const startFlowsFilewatcher = (): void => {
  if (!FILEWATCHER_ENABLED) {
    console.log("Filewatcher disabled");
    return;
  }

  const pathToWatch = `${FLOWS_DIR_PATH}/**/${FLOW_FILE}`;
  watch(pathToWatch, { usePolling: true }).on("all", async (event, path) => {
    if (path.endsWith(FLOW_FILE)) {
      console.log("Flows updated, restarting kafka group...");
      await consumer.disconnect();
      await producer.disconnect();
      await startKafkaGroup();
    }
  });
  console.log(`Watching for changes to ${pathToWatch}...`);
};

const startKafkaGroup = async (): Promise<void> => {
  try {
    await startProducer();

    try {
      await startConsumer();
    } catch (error) {
      console.error("Failed to start kafka consumer: ", error);
    }
  } catch (error) {
    console.error("Failed to start kafka producer: ", error);
  }
};

startKafkaGroup().then(() => startFlowsFilewatcher());
