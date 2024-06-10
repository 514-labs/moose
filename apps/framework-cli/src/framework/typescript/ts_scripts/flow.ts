import { Consumer, Kafka, KafkaMessage, Producer, SASLOptions } from "kafkajs";
import process from "node:process";

const SOURCE_TOPIC = process.argv[1];
const TARGET_TOPIC = process.argv[2];
const FLOW_FILE_PATH = process.argv[3];
const BROKER = process.argv[4];
const SASL_USERNAME = process.argv[5];
const SASL_PASSWORD = process.argv[6];
const SASL_MECHANISM = process.argv[7];
const SECURITY_PROTOCOL = process.argv[8];

type FlowFunction = (data: unknown) => unknown | Promise<unknown>;

const logPrefix = `${SOURCE_TOPIC} -> ${TARGET_TOPIC}`;
const log = (message: string): void => {
  console.log(`${logPrefix}: ${message}`);
};

const error = (message: string): void => {
  console.error(`${logPrefix}: ${message}`);
};

const warn = (message: string): void => {
  console.warn(`${logPrefix}: ${message}`);
};

log("Initializing flow...");
if (SOURCE_TOPIC === undefined) {
  error("Missing source topic");
  process.exit(1);
}

if (TARGET_TOPIC === undefined) {
  error("Missing target topic");
  process.exit(1);
}

if (FLOW_FILE_PATH === undefined) {
  error("Missing flow file path");
  process.exit(1);
}

if (BROKER === undefined) {
  error("Missing argument: BROKER");
  process.exit(1);
}

// TODO - when having optional arguments, we should use a proper
// command line parser to handle this. As is a user would not be able to
// only provide a SASL_USERNAME and not a SASL_PASSWORD.
if (SASL_USERNAME === undefined) {
  warn("No argument: SASL_USERNAME");
}

if (SASL_PASSWORD === undefined) {
  warn("No argument: SASL_USERNAME");
}

if (SASL_MECHANISM === undefined) {
  warn("No argument: SASL_MECHANISM");
}

if (SECURITY_PROTOCOL === undefined) {
  warn("No argument: SECURITY_PROTOCOL");
}

log(`Flow configuration loaded with file ${FLOW_FILE_PATH}`);

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
      log(`Unsupported SASL mechanism: ${SASL_MECHANISM}`);
      return undefined;
  }
};

const jsonDateReviver = (key: string, value: unknown): unknown => {
  const iso8601Format =
    /^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))([T\s]((([01]\d|2[0-3])((:?)[0-5]\d)?|24\:?00)([\.,]\d+(?!:))?)?(\17[0-5]\d([\.,]\d+)?)?([zZ]|([\+-])([01]\d|2[0-3]):?([0-5]\d)?)?)?)?$/;

  if (typeof value === "string" && iso8601Format.test(value)) {
    return new Date(value);
  }

  return value;
};

const kafka = new Kafka({
  clientId: "flow-consumer",
  brokers: [BROKER],
  ssl: SECURITY_PROTOCOL === "SASL_SSL",
  sasl: getSaslConfig(),
});

const flowIdentifier = `flow-${SOURCE_TOPIC}-${TARGET_TOPIC}`;
const consumer: Consumer = kafka.consumer({ groupId: flowIdentifier });
const producer: Producer = kafka.producer({ transactionalId: flowIdentifier });

const startProducer = async (): Promise<void> => {
  await producer.connect();
  log("Producer is running...");
};

const handleMessage = async (
  flowFn: FlowFunction,
  message: KafkaMessage,
): Promise<{ value: string } | null> => {
  if (message.value === undefined || message.value === null) {
    log(`Received message with no value, skipping...`);
    return null;
  }

  try {
    const transformedData = await flowFn(
      JSON.parse(message.value.toString(), jsonDateReviver),
    );

    if (transformedData) {
      return { value: JSON.stringify(transformedData) };
    }
  } catch (e) {
    // TODO: Track failure rate
    error(`Failed to transform data`);
    if (e instanceof Error) {
      error(e.message);
    }
  }

  return null;
};

const startConsumer = async (
  sourceTopic: string,
  targetTopic: string,
): Promise<void> => {
  await consumer.connect();

  log(
    `Starting consumer group '${flowIdentifier}' with source topic: ${sourceTopic} and target topic: ${targetTopic}`,
  );

  const flowModuleImport = await import(
    FLOW_FILE_PATH.substring(0, FLOW_FILE_PATH.length - 3)
  );
  const flowFunction: FlowFunction = flowModuleImport.default;

  // We limit consumption to 900KB to hiting the batch limit of 1MB on the producer side.
  // In order to increase this we should increase the accepting size on the topic itself.
  await consumer.subscribe({
    topics: [sourceTopic],
    fromBeginning: false,
    maxBytes: 900 * 1024,
  });
  await consumer.run({
    eachBatchAutoResolve: true,
    eachBatch: async ({ batch }) => {
      const messages = await Promise.all(
        batch.messages.map((message) => handleMessage(flowFunction, message)),
      );

      const filteredMessages = messages.filter((msg) => msg !== null);

      if (filteredMessages.length > 0) {
        await producer.send({
          topic: targetTopic,
          messages: filteredMessages as { value: string }[],
        });
        log(
          `Sent ${filteredMessages.length} transformed data to ${targetTopic}`,
        );
      }
    },
  });

  log("Consumer is running...");
};

const startFlow = async (
  sourceTopic: string,
  targetTopic: string,
): Promise<void> => {
  try {
    await startProducer();

    try {
      await startConsumer(sourceTopic, targetTopic);
    } catch (e) {
      error("Failed to start kafka consumer: ");
      if (e instanceof Error) {
        error(e.message);
      }
    }
  } catch (e) {
    error("Failed to start kafka producer: ");
    if (e instanceof Error) {
      error(e.message);
    }
  }
};

startFlow(SOURCE_TOPIC, TARGET_TOPIC);
