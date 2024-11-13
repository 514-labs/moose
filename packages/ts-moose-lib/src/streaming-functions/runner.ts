import { Readable } from "node:stream";
import { Consumer, Kafka, KafkaMessage, Producer, SASLOptions } from "kafkajs";
import { Buffer } from "node:buffer";
import process from "node:process";
import http from "http";
import { cliLog } from "../commons";

type CliLogData = {
  count_in: number;
  count_out: number;
  bytes: number;
  function_name: string;
  timestamp: Date;
};

type StreamingFunction = (data: unknown) => unknown | Promise<unknown>;
type SlimKafkaMessage = { value: string };

interface StreamingFunctionArgs {
  sourceTopic: string;
  targetTopic: string;
  targetTopicConfig: string;
  functionFilePath: string;
  broker: string;
  saslUsername?: string;
  saslPassword?: string;
  saslMechanism?: string;
  securityProtocol?: string;
}
const has_no_output_topic = (args: StreamingFunctionArgs) =>
  args.targetTopic === "";

interface Logger {
  logPrefix: string;
  log: (message: string) => void;
  error: (message: string) => void;
  warn: (message: string) => void;
}

const MAX_STREAMING_CONCURRENCY = process.env.MAX_STREAMING_CONCURRENCY
  ? parseInt(process.env.MAX_STREAMING_CONCURRENCY)
  : 100;

const parseArgs = (): StreamingFunctionArgs => {
  const SOURCE_TOPIC = process.argv[3];
  const TARGET_TOPIC = process.argv[4];
  const TARGET_TOPIC_CONFIG = process.argv[5];
  const FUNCTION_FILE_PATH = process.argv[6];
  const BROKER = process.argv[7];
  const SASL_USERNAME = process.argv[8];
  const SASL_PASSWORD = process.argv[9];
  const SASL_MECHANISM = process.argv[10];
  const SECURITY_PROTOCOL = process.argv[11];

  return {
    sourceTopic: SOURCE_TOPIC,
    targetTopic: TARGET_TOPIC,
    targetTopicConfig: TARGET_TOPIC_CONFIG,
    functionFilePath: FUNCTION_FILE_PATH,
    broker: BROKER,
    saslUsername: SASL_USERNAME,
    saslPassword: SASL_PASSWORD,
    saslMechanism: SASL_MECHANISM,
    securityProtocol: SECURITY_PROTOCOL,
  };
};

const buildSaslConfig = (
  logger: Logger,
  args: StreamingFunctionArgs,
): SASLOptions | undefined => {
  const mechanism = args.saslMechanism ? args.saslMechanism.toLowerCase() : "";
  switch (mechanism) {
    case "plain":
    case "scram-sha-256":
    case "scram-sha-512":
      return {
        mechanism: mechanism,
        username: args.saslUsername || "",
        password: args.saslPassword || "",
      };
    default:
      logger.warn(`Unsupported SASL mechanism: ${args.saslMechanism}`);
      return undefined;
  }
};

export const metricsLog: (log: CliLogData) => void = (log) => {
  const req = http.request({
    port: 5001,
    method: "POST",
    path: "/metrics-logs",
  }); // no callback, fire and forget

  req.write(JSON.stringify({ ...log }));
  req.end();
};

const jsonDateReviver = (key: string, value: unknown): unknown => {
  const iso8601Format =
    /^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))([T\s]((([01]\d|2[0-3])((:?)[0-5]\d)?|24\:?00)([\.,]\d+(?!:))?)?(\17[0-5]\d([\.,]\d+)?)?([zZ]|([\+-])([01]\d|2[0-3]):?([0-5]\d)?)?)?)?$/;

  if (typeof value === "string" && iso8601Format.test(value)) {
    return new Date(value);
  }

  return value;
};

const startProducer = async (
  logger: Logger,
  producer: Producer,
): Promise<void> => {
  await producer.connect();
  logger.log("Producer is running...");
};

const stopProducer = async (
  logger: Logger,
  producer: Producer,
): Promise<void> => {
  await producer.disconnect();
  logger.log("Producer is shutting down...");
};

const stopConsumer = async (
  logger: Logger,
  consumer: Consumer,
): Promise<void> => {
  await consumer.disconnect();
  logger.log("Consumer is shutting down...");
};

const handleMessage = async (
  logger: Logger,
  streamingFunction: StreamingFunction,
  message: KafkaMessage,
): Promise<SlimKafkaMessage[] | undefined> => {
  if (message.value === undefined || message.value === null) {
    logger.log(`Received message with no value, skipping...`);
    return undefined;
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
    logger.error(`Failed to transform data`);
    if (e instanceof Error) {
      logger.error(e.message);
    }
  }

  return undefined;
};

const sendMessages = async (
  logger: Logger,
  args: StreamingFunctionArgs,
  producer: Producer,
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
        await producer.send({ topic: args.targetTopic, messages: chunks });
        logger.log(
          `Sent ${chunks.length} transformed data to ${args.targetTopic}`,
        );

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
      await producer.send({ topic: args.targetTopic, messages: chunks });
      logger.log(
        `Sent final ${chunks.length} transformed data to ${args.targetTopic}`,
      );
    }
  } catch (e) {
    logger.error(`Failed to send transformed data`);
    if (e instanceof Error) {
      logger.error(e.message);
    }
  }
};

let count_in = 0;
let count_out = 0;
let bytes = 0;

const sendMessageMetrics = (logger: Logger) => {
  if (count_in > 0 || count_out > 0 || bytes > 0) {
    metricsLog({
      count_in: count_in,
      count_out: count_out,
      function_name: logger.logPrefix,
      bytes: bytes,
      timestamp: new Date(),
    });
  }
  count_in = 0;
  bytes = 0;
  count_out = 0;
  setTimeout(() => sendMessageMetrics(logger), 1000);
};

const startConsumer = async (
  logger: Logger,
  args: StreamingFunctionArgs,
  consumer: Consumer,
  producer: Producer,
  streamingFuncId: string,
  maxMessageSize: number,
): Promise<void> => {
  await consumer.connect();

  logger.log(
    `Starting consumer group '${streamingFuncId}' with source topic: ${args.sourceTopic} and target topic: ${args.targetTopic}`,
  );

  let streamingFunctionImport;
  try {
    streamingFunctionImport = require(
      args.functionFilePath.substring(0, args.functionFilePath.length - 3),
    );
  } catch (e) {
    cliLog({ action: "Function", message: `${e}`, message_type: "Error" });
    throw e;
  }
  const streamingFunction: StreamingFunction = streamingFunctionImport.default;

  await consumer.subscribe({
    topics: [args.sourceTopic],
    // to read records sent before subscriber is created
    fromBeginning: true,
  });
  await consumer.run({
    eachBatchAutoResolve: true,
    eachBatch: async ({ batch }) => {
      count_in += batch.messages.length;

      cliLog({
        action: "Received",
        message: `${logger.logPrefix} ${batch.messages.length} message(s)`,
      });
      logger.log(`Received ${batch.messages.length} message(s)`);
      const messages = await Readable.from(batch.messages)
        .map((message) => handleMessage(logger, streamingFunction, message), {
          concurrency: MAX_STREAMING_CONCURRENCY,
        })
        .toArray();

      if (has_no_output_topic(args)) {
        return;
      }

      // readable.map is used for parallel map with a concurrency limit,
      // but Readable does not accept null values
      // so the return type in handleMessage cannot contain null
      const filteredMessages = messages
        .flat()
        .filter((msg) => msg !== undefined);

      if (filteredMessages.length > 0) {
        await sendMessages(
          logger,
          args,
          producer,
          filteredMessages as SlimKafkaMessage[],
          maxMessageSize,
        );
      }
    },
  });

  logger.log("Consumer is running...");
};

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

const buildLogger = (args: StreamingFunctionArgs): Logger => {
  const logPrefix = `${args.sourceTopic} -> ${args.targetTopic}`;
  const logger: Logger = {
    logPrefix: logPrefix,
    log: (message: string): void => {
      console.log(`${logPrefix}: ${message}`);
    },
    error: (message: string): void => {
      console.error(`${logPrefix}: ${message}`);
    },
    warn: (message: string): void => {
      console.warn(`${logPrefix}: ${message}`);
    },
  };
  return logger;
};

export const runStreamingFunctions = async (): Promise<void> => {
  const args = parseArgs();
  const logger = buildLogger(args);

  setTimeout(() => sendMessageMetrics(logger), 1000);

  const kafka = new Kafka({
    clientId: "streaming-function-consumer",
    brokers: [args.broker],
    ssl: args.securityProtocol === "SASL_SSL",
    sasl: buildSaslConfig(logger, args),
  });

  const streamingFuncId = `flow-${args.sourceTopic}-${args.targetTopic}`;
  const consumer: Consumer = kafka.consumer({
    groupId: streamingFuncId,
  });
  const producer: Producer = kafka.producer({
    transactionalId: streamingFuncId,
  });

  process.on("SIGTERM", async () => {
    logger.log("Received SIGTERM, shutting down...");
    await stopConsumer(logger, consumer);
    await stopProducer(logger, producer);
    process.exit(0);
  });

  try {
    if (!has_no_output_topic(args)) {
      await startProducer(logger, producer);
    }

    try {
      const targetTopicConfig = JSON.parse(args.targetTopicConfig) as Record<
        string,
        unknown
      >;
      const maxMessageSize = getMaxMessageSize(targetTopicConfig);

      await startConsumer(
        logger,
        args,
        consumer,
        producer,
        streamingFuncId,
        maxMessageSize,
      );
    } catch (e) {
      logger.error("Failed to start kafka consumer: ");
      if (e instanceof Error) {
        logger.error(e.message);
      }
    }
  } catch (e) {
    logger.error("Failed to start kafka producer: ");
    if (e instanceof Error) {
      logger.error(e.message);
    }
  }
};
