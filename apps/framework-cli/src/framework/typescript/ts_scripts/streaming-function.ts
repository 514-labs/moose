import { Consumer, Kafka, KafkaMessage, Producer, SASLOptions } from "kafkajs";
import { Buffer } from "node:buffer";
import process from "node:process";
import { cliLog } from "@514labs/moose-lib";
import http from "http";

const SOURCE_TOPIC = process.argv[1];
const TARGET_TOPIC = process.argv[2];
const TARGET_TOPIC_CONFIG = process.argv[3];
const FUNCTION_FILE_PATH = process.argv[4];
const BROKER = process.argv[5];
const SASL_USERNAME = process.argv[6];
const SASL_PASSWORD = process.argv[7];
const SASL_MECHANISM = process.argv[8];
const SECURITY_PROTOCOL = process.argv[9];

type CliLogData = {
  count: number;
  bytes: number;
  path: string;
  direction: "In" | "Out";
};
export const metricsLog: (log: CliLogData) => void = (log) => {
  const req = http.request({
    port: 4000,
    method: "POST",
    path: "/metrics-logs",
  }); // no callback, fire and forget

  req.write(JSON.stringify({ ...log }));
  req.end();
};

type StreamingFunction = (data: unknown) => unknown | Promise<unknown>;
type SlimKafkaMessage = { value: string };

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

log("Initializing streaming function...");
if (SOURCE_TOPIC === undefined) {
  error("Missing source topic");
  process.exit(1);
}

if (TARGET_TOPIC === undefined) {
  error("Missing target topic");
  process.exit(1);
}

if (FUNCTION_FILE_PATH === undefined) {
  error("Missing streaming function file path");
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

log(`Streaming function configuration loaded with file ${FUNCTION_FILE_PATH}`);

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
  clientId: "streaming-function-consumer",
  brokers: [BROKER],
  ssl: SECURITY_PROTOCOL === "SASL_SSL",
  sasl: getSaslConfig(),
});

const streamingFuncId = `flow-${SOURCE_TOPIC}-${TARGET_TOPIC}`;
const consumer: Consumer = kafka.consumer({
  groupId: streamingFuncId,
});
const producer: Producer = kafka.producer({ transactionalId: streamingFuncId });

const startProducer = async (): Promise<void> => {
  await producer.connect();
  log("Producer is running...");
};

const stopProducer = async (): Promise<void> => {
  await producer.disconnect();
  log("Producer is shutting down...");
};

const stopConsumer = async (): Promise<void> => {
  await consumer.disconnect();
  log("Consumer is shutting down...");
};

const handleMessage = async (
  streamingFunction: StreamingFunction,
  message: KafkaMessage,
): Promise<SlimKafkaMessage[] | null> => {
  if (message.value === undefined || message.value === null) {
    log(`Received message with no value, skipping...`);
    return null;
  }

  try {
    const transformedData = await streamingFunction(
      JSON.parse(message.value.toString(), jsonDateReviver),
    );

    if (transformedData) {
      if (Array.isArray(transformedData)) {
        return transformedData.map((item) => ({ value: JSON.stringify(item) }));
      } else {
        return [{ value: JSON.stringify(transformedData) }];
      }
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

const sendMessages = async (
  targetTopic: string,
  messages: SlimKafkaMessage[],
  maxMessageSize: number,
): Promise<void> => {
  try {
    let chunks: SlimKafkaMessage[] = [];
    let chunkSize = 0;

    for (const message of messages) {
      const messageSize = Buffer.byteLength(message.value, "utf8");

      if (chunkSize + messageSize > maxMessageSize) {
        // Send the current chunk before adding the new message
        await producer.send({ topic: targetTopic, messages: chunks });
        log(`Sent ${chunks.length} transformed data to ${targetTopic}`);

        // Start a new chunk
        chunks = [message];
        chunkSize = messageSize;
      } else {
        // Add the new message to the current chunk
        chunks.push(message);
        chunks.forEach(
          (chunk) => (bytes += Buffer.byteLength(chunk.value, "utf8")),
        );
        chunkSize += messageSize;
      }
    }
    count_out += chunks.length;

    // Send the last chunk
    if (chunks.length > 0) {
      await producer.send({ topic: targetTopic, messages: chunks });
      log(`Sent final ${chunks.length} transformed data to ${targetTopic}`);
    }
  } catch (e) {
    error(`Failed to send transformed data`);
    if (e instanceof Error) {
      error(e.message);
    }
  }
};

setTimeout(() => sendMessageMetricsOut(), 1000);

var count_in = 0;
var count_out = 0;
var bytes = 0;

const sendMessageMetricsIn = () => {
  metricsLog({
    count: count_in,
    path: logPrefix,
    bytes: bytes,
    direction: "In",
  });
  setTimeout(() => sendMessageMetricsIn(), 1000);
};

const sendMessageMetricsOut = () => {
  metricsLog({
    count: count_out,
    bytes: bytes,
    path: logPrefix,
    direction: "Out",
  });
  setTimeout(() => sendMessageMetricsOut(), 1000);
};

const startConsumer = async (
  sourceTopic: string,
  targetTopic: string,
  maxMessageSize: number,
): Promise<void> => {
  await consumer.connect();

  log(
    `Starting consumer group '${streamingFuncId}' with source topic: ${sourceTopic} and target topic: ${targetTopic}`,
  );

  const streamingFunctionImport = await import(
    FUNCTION_FILE_PATH.substring(0, FUNCTION_FILE_PATH.length - 3)
  );
  const streamingFunction: StreamingFunction = streamingFunctionImport.default;

  await consumer.subscribe({
    topics: [sourceTopic],
    fromBeginning: false,
  });
  await consumer.run({
    eachBatchAutoResolve: true,
    eachBatch: async ({ batch }) => {
      count_in += batch.messages.length;

      cliLog({
        action: "Received",
        message: `${logPrefix} ${batch.messages.length} message(s)`,
      });
      const messages = (
        await Promise.all(
          batch.messages.map((message) =>
            handleMessage(streamingFunction, message),
          ),
        )
      ).flat();

      const filteredMessages = messages.filter((msg) => msg !== null);

      if (filteredMessages.length > 0) {
        await sendMessages(
          targetTopic,
          filteredMessages as SlimKafkaMessage[],
          maxMessageSize,
        );
      }
    },
  });

  log("Consumer is running...");
};

setTimeout(() => sendMessageMetricsIn(), 1000);

/**
 * message.max.bytes is a broker setting that applies to all topics.
 * max.message.bytes is a per-topic setting.
 *
 * In general, max.message.bytes should be less than or equal to message.max.bytes.
 * If max.message.bytes is larger than message.max.bytes, the broker will still reject
 * any message that is larger than message.max.bytes, even if it's sent to a topic
 * where max.message.bytes is larger. So we take the minimum of the two values,
 * or default to 1MB if either value is not set. 1MB is the server's default.
 */
const getMaxMessageSize = (config: Record<string, unknown>): number => {
  const maxMessageBytes =
    (config["max.message.bytes"] as number) || 1024 * 1024;
  const messageMaxBytes =
    (config["message.max.bytes"] as number) || 1024 * 1024;

  return Math.min(maxMessageBytes, messageMaxBytes);
};

const startStreaming = async (
  sourceTopic: string,
  targetTopic: string,
  targetTopicConfigJson: string,
): Promise<void> => {
  try {
    await startProducer();

    try {
      const targetTopicConfig = JSON.parse(targetTopicConfigJson) as Record<
        string,
        unknown
      >;
      const maxMessageSize = getMaxMessageSize(targetTopicConfig);

      await startConsumer(sourceTopic, targetTopic, maxMessageSize);
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

process.on("SIGTERM", async () => {
  log("Received SIGTERM, shutting down...");
  await stopConsumer();
  await stopProducer();
  process.exit(0);
});

startStreaming(SOURCE_TOPIC, TARGET_TOPIC, TARGET_TOPIC_CONFIG);
