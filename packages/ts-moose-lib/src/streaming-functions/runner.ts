import { Readable } from "node:stream";
import { Consumer, Kafka, KafkaMessage, Producer, SASLOptions } from "kafkajs";
import { Buffer } from "node:buffer";
import process from "node:process";
import http from "http";
import { cliLog } from "../commons";
import { Cluster } from "../cluster-utils";
import { getStreamingFunctions } from "../dmv2/internal";

const HOSTNAME = process.env.HOSTNAME;
const AUTO_COMMIT_INTERVAL_MS = 5000;
const PARTITIONS_CONSUMED_CONCURRENTLY = 3;
const MAX_RETRIES = 150;
const MAX_RETRY_TIME_MS = 1000;
const MAX_RETRIES_PRODUCER = 150;
const MAX_RETRIES_CONSUMER = 150;
const SESSION_TIMEOUT_CONSUMER = 30000;
const HEARTBEAT_INTERVAL_CONSUMER = 3000;
const RETRY_FACTOR_PRODUCER = 0.2;
const DEFAULT_MAX_STREAMING_CONCURRENCY = 100;
const RETRY_INITIAL_TIME_MS = 100;
// https://github.com/apache/kafka/blob/trunk/clients/src/main/java/org/apache/kafka/common/record/AbstractRecords.java#L124
// According to the above, the overhead should be 12 + 22 bytes - 34 bytes.
// We put 500 to be safe.
const KAFKAJS_BYTE_MESSAGE_OVERHEAD = 500;

//Dummy change

/**
 * Data structure for metrics logging containing counts and metadata
 */
type CliLogData = {
  count_in: number;
  count_out: number;
  bytes: number;
  function_name: string;
  timestamp: Date;
};

/**
 * Interface for tracking message processing metrics
 */
interface Metrics {
  count_in: number;
  count_out: number;
  bytes: number;
}

/**
 * Type definition for streaming transformation function
 */
type StreamingFunction = (data: unknown) => unknown | Promise<unknown>;

/**
 * Simplified Kafka message type containing only value
 */
type SlimKafkaMessage = { value: string };

export interface TopicConfig {
  name: string;
  partitions: number;
  retentionPeriod: number;
  maxMessageBytes: number;
}

/**
 * Configuration interface for streaming function arguments
 */
export interface StreamingFunctionArgs {
  sourceTopic: TopicConfig;
  targetTopic?: TopicConfig;
  functionFilePath: string;
  broker: string;
  maxSubscriberCount: number;
  isDmv2: boolean;
  saslUsername?: string;
  saslPassword?: string;
  saslMechanism?: string;
  securityProtocol?: string;
}

/**
 * Interface for logging functionality
 */
interface Logger {
  logPrefix: string;
  log: (message: string) => void;
  error: (message: string) => void;
  warn: (message: string) => void;
}

/**
 * Maximum number of concurrent streaming operations, configurable via environment
 */
const MAX_STREAMING_CONCURRENCY = process.env.MAX_STREAMING_CONCURRENCY
  ? parseInt(process.env.MAX_STREAMING_CONCURRENCY)
  : DEFAULT_MAX_STREAMING_CONCURRENCY;

/**
 * Builds SASL configuration for Kafka client authentication
 */
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

/**
 * Logs metrics data to HTTP endpoint
 */
export const metricsLog: (log: CliLogData) => void = (log) => {
  const req = http.request({
    port: 5001,
    method: "POST",
    path: "/metrics-logs",
  }); // no callback, fire and forget

  req.write(JSON.stringify({ ...log }));
  req.end();
};

/**
 * Revives ISO 8601 date strings into Date objects during JSON parsing
 */
const jsonDateReviver = (key: string, value: unknown): unknown => {
  const iso8601Format =
    /^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))([T\s]((([01]\d|2[0-3])((:?)[0-5]\d)?|24\:?00)([\.,]\d+(?!:))?)?(\17[0-5]\d([\.,]\d+)?)?([zZ]|([\+-])([01]\d|2[0-3]):?([0-5]\d)?)?)?)$/;

  if (typeof value === "string" && iso8601Format.test(value)) {
    return new Date(value);
  }

  return value;
};

/**
 * Initializes and connects Kafka producer
 */
const startProducer = async (
  logger: Logger,
  producer: Producer,
): Promise<void> => {
  await producer.connect();
  logger.log("Producer is running...");
};

/**
 * Disconnects a Kafka producer and logs the shutdown
 *
 * @param logger - Logger instance for outputting producer status
 * @param producer - KafkaJS Producer instance to disconnect
 * @returns Promise that resolves when producer is disconnected
 * @example
 * ```ts
 * await stopProducer(logger, producer); // Disconnects producer and logs shutdown
 * ```
 */
const stopProducer = async (
  logger: Logger,
  producer: Producer,
): Promise<void> => {
  await producer.disconnect();
  logger.log("Producer is shutting down...");
};

/**
 * Disconnects a Kafka consumer and logs the shutdown
 *
 * @param logger - Logger instance for outputting consumer status
 * @param consumer - KafkaJS Consumer instance to disconnect
 * @returns Promise that resolves when consumer is disconnected
 * @example
 * ```ts
 * await stopConsumer(logger, consumer); // Disconnects consumer and logs shutdown
 * ```
 */
const stopConsumer = async (
  logger: Logger,
  consumer: Consumer,
): Promise<void> => {
  await consumer.disconnect();
  logger.log("Consumer is shutting down...");
};

/**
 * Processes a single Kafka message through a streaming function and returns transformed message(s)
 *
 * @param logger - Logger instance for outputting message processing status and errors
 * @param streamingFunction - Function that transforms input message data
 * @param message - Kafka message to be processed
 * @returns Promise resolving to array of transformed messages or undefined if processing fails
 *
 * The function will:
 * 1. Check for null/undefined message values
 * 2. Parse the message value as JSON with date handling
 * 3. Pass parsed data through the streaming function
 * 4. Convert transformed data back to string format
 * 5. Handle both single and array return values
 * 6. Log any processing errors
 */
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

/**
 * Sends processed messages to a target Kafka topic in chunks to respect max message size limits
 *
 * @param logger - Logger instance for outputting send status and errors
 * @param metrics - Metrics object for tracking message counts and bytes sent
 * @param args - Configuration arguments containing target topic
 * @param producer - KafkaJS Producer instance for sending messages
 * @param messages - Array of processed messages to send
 * @param maxMessageSize - Maximum allowed size in bytes for a message chunk
 * @returns Promise that resolves when all messages are sent
 *
 * The function will:
 * 1. Split messages into chunks that fit within maxMessageSize
 * 2. Send each chunk to the target topic
 * 3. Track metrics for bytes sent and message counts
 * 4. Log success/failure of sends
 */
const sendMessages = async (
  logger: Logger,
  metrics: Metrics,
  targetTopic: TopicConfig,
  producer: Producer,
  messages: SlimKafkaMessage[],
): Promise<void> => {
  try {
    let chunks: SlimKafkaMessage[] = [];
    let chunkSize = 0;

    for (const message of messages) {
      const messageSize =
        Buffer.byteLength(message.value, "utf8") +
        KAFKAJS_BYTE_MESSAGE_OVERHEAD;

      if (chunkSize + messageSize > targetTopic.maxMessageBytes) {
        logger.log(
          `Sending ${chunkSize} bytes of a transformed record batch to ${targetTopic.name}`,
        );
        // Send the current chunk before adding the new message
        // We are not setting the key, so that should not take any size in the payload
        await producer.send({ topic: targetTopic.name, messages: chunks });
        logger.log(
          `Sent ${chunks.length} transformed records to ${targetTopic.name}`,
        );

        // Start a new chunk
        chunks = [message];
        chunkSize = messageSize;
      } else {
        // Add the new message to the current chunk
        chunks.push(message);
        chunks.forEach(
          (chunk) => (metrics.bytes += Buffer.byteLength(chunk.value, "utf8")),
        );
        chunkSize += messageSize;
      }
    }

    metrics.count_out += chunks.length;

    // Send the last chunk
    if (chunks.length > 0) {
      logger.log(
        `Sending ${chunkSize} bytes of a transformed record batch to ${targetTopic.name}`,
      );
      await producer.send({ topic: targetTopic.name, messages: chunks });
      logger.log(
        `Sent final ${chunks.length} transformed data to ${targetTopic.name}`,
      );
    }
  } catch (e) {
    logger.error(`Failed to send transformed data`);
    if (e instanceof Error) {
      logger.error(e.message);
    }
    // This is needed for retries
    throw e;
  }
};

/**
 * Periodically sends metrics about message processing to a metrics logging endpoint.
 * Resets metrics counters after each send. Runs every second via setTimeout.
 *
 * @param logger - Logger instance containing the function name prefix
 * @param metrics - Metrics object tracking message counts and bytes processed
 * @example
 * ```ts
 * const metrics = { count_in: 10, count_out: 8, bytes: 1024 };
 * sendMessageMetrics(logger, metrics); // Sends metrics and resets counters
 * ```
 */
const sendMessageMetrics = (logger: Logger, metrics: Metrics) => {
  if (metrics.count_in > 0 || metrics.count_out > 0 || metrics.bytes > 0) {
    metricsLog({
      count_in: metrics.count_in,
      count_out: metrics.count_out,
      function_name: logger.logPrefix,
      bytes: metrics.bytes,
      timestamp: new Date(),
    });
  }
  metrics.count_in = 0;
  metrics.bytes = 0;
  metrics.count_out = 0;
  setTimeout(() => sendMessageMetrics(logger, metrics), 1000);
};

/**
 * Dynamically loads a streaming function from a file path
 *
 * @param args - The streaming function arguments containing the function file path
 * @returns The default export of the streaming function module
 * @throws Will throw and log an error if the function file cannot be loaded
 * @example
 * ```ts
 * const fn = loadStreamingFunction({functionFilePath: './transform.js'});
 * const result = await fn(data);
 * ```
 */
function loadStreamingFunction(functionFilePath: string) {
  let streamingFunctionImport;
  try {
    streamingFunctionImport = require(
      functionFilePath.substring(0, functionFilePath.length - 3),
    );
  } catch (e) {
    cliLog({ action: "Function", message: `${e}`, message_type: "Error" });
    throw e;
  }
  return streamingFunctionImport.default;
}

async function loadStreamingFunctionV2(
  sourceTopic: TopicConfig,
  targetTopic?: TopicConfig,
) {
  const transformFunctions = await getStreamingFunctions();
  const transformFunctionKey = `${sourceTopic.name}_${targetTopic?.name}`;
  return transformFunctions.get(transformFunctionKey);
}

/**
 * Initializes and starts a Kafka consumer that processes messages using a streaming function
 *
 * @param logger - Logger instance for outputting consumer status and errors
 * @param metrics - Metrics object for tracking message counts and bytes processed
 * @param parallelism - Number of parallel workers processing messages
 * @param args - Configuration arguments for source/target topics and streaming function
 * @param consumer - KafkaJS Consumer instance
 * @param producer - KafkaJS Producer instance for sending processed messages
 * @param streamingFuncId - Unique identifier for this consumer group
 * @param maxMessageSize - Maximum message size in bytes allowed by Kafka broker
 * @returns Promise that resolves when consumer is started
 *
 * The consumer will:
 * 1. Connect to Kafka
 * 2. Subscribe to the source topic
 * 3. Process messages in batches using the streaming function
 * 4. Send processed messages to target topic (if configured)
 * 5. Commit offsets after successful processing
 */
const startConsumer = async (
  isDmv2: boolean,
  logger: Logger,
  metrics: Metrics,
  parallelism: number,
  functionFilePath: string,
  sourceTopic: TopicConfig,
  consumer: Consumer,
  producer: Producer,
  streamingFuncId: string,
  targetTopic?: TopicConfig,
): Promise<void> => {
  await consumer.connect();

  logger.log(
    `Starting consumer group '${streamingFuncId}' with source topic: ${sourceTopic.name} and target topic: ${targetTopic?.name}`,
  );

  // We preload the function to not have to load it for each message
  const streamingFunction: StreamingFunction = isDmv2
    ? await loadStreamingFunctionV2(sourceTopic, targetTopic)
    : loadStreamingFunction(functionFilePath);

  await consumer.subscribe({
    topics: [sourceTopic.name],
    // to read records sent before subscriber is created for when the groupId is new
    // and there are no committed offsets. If the groupId is not new, it will start
    // from the last committed offset.
    fromBeginning: true,
  });

  await consumer.run({
    autoCommitInterval: AUTO_COMMIT_INTERVAL_MS,
    eachBatchAutoResolve: true,
    // Enable parallel processing of partitions
    partitionsConsumedConcurrently: PARTITIONS_CONSUMED_CONCURRENTLY, // To be adjusted
    eachBatch: async ({ batch, heartbeat, isRunning, isStale }) => {
      if (!isRunning() || isStale()) {
        return;
      }

      metrics.count_in += batch.messages.length;

      cliLog({
        action: "Received",
        message: `${logger.logPrefix} ${batch.messages.length} message(s)`,
      });
      logger.log(`Received ${batch.messages.length} message(s)`);

      let index = 0;
      const processedMessages: (SlimKafkaMessage[] | undefined)[] =
        await Readable.from(batch.messages)
          .map(
            async (message) => {
              index++;
              if (
                (batch.messages.length > DEFAULT_MAX_STREAMING_CONCURRENCY &&
                  index % DEFAULT_MAX_STREAMING_CONCURRENCY) ||
                index - 1 === batch.messages.length
              ) {
                await heartbeat();
              }
              return handleMessage(logger, streamingFunction, message);
            },
            {
              concurrency: MAX_STREAMING_CONCURRENCY,
            },
          )
          .toArray();

      const filteredMessages = processedMessages
        .flat()
        .filter((msg) => msg !== undefined);

      if (targetTopic === undefined || processedMessages.length === 0) {
        return;
      }

      await heartbeat();

      if (filteredMessages.length > 0) {
        await sendMessages(
          logger,
          metrics,
          targetTopic,
          producer,
          filteredMessages as SlimKafkaMessage[],
        );
      }
    },
  });

  logger.log("Consumer is running...");
};

/**
 * Creates a Logger instance that prefixes all log messages with the source and target topic
 *
 * @param args - The streaming function arguments containing source and target topics
 * @returns A Logger instance with standard log, error and warn methods
 * @example
 * ```ts
 * const logger = buildLogger({sourceTopic: 'source', targetTopic: 'target'});
 * logger.log('message'); // Outputs: "source -> target: message"
 * ```
 */
const buildLogger = (args: StreamingFunctionArgs, workerId: number): Logger => {
  const logPrefix = `${args.sourceTopic.name} -> ${args.targetTopic?.name || "No Target"} - ${workerId}`;
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

/**
 * Initializes and runs a clustered streaming function system that processes messages from Kafka
 *
 * This function:
 * 1. Creates a cluster of workers to handle Kafka message processing
 * 2. Sets up Kafka producers and consumers for each worker
 * 3. Configures logging and metrics collection
 * 4. Handles graceful shutdown on termination
 *
 * The system supports:
 * - Multiple workers processing messages in parallel
 * - Dynamic CPU usage control via maxCpuUsageRatio
 * - SASL authentication for Kafka
 * - Metrics tracking for message counts and bytes processed
 * - Graceful shutdown of Kafka connections
 *
 * @returns Promise that resolves when the cluster is started
 * @throws Will log errors if Kafka connections fail
 *
 * @example
 * ```ts
 * await runStreamingFunctions({
 *   sourceTopic: { name: 'source', partitions: 3, retentionPeriod: 86400, maxMessageBytes: 1048576 },
 *   targetTopic: { name: 'target', partitions: 3, retentionPeriod: 86400, maxMessageBytes: 1048576 },
 *   functionFilePath: './transform.js',
 *   broker: 'localhost:9092',
 *   maxSubscriberCount: 3,
 *   isDmv2: false
 * }); // Starts the streaming function cluster
 * ```
 */
export const runStreamingFunctions = async (
  args: StreamingFunctionArgs,
): Promise<void> => {
  const streamingFuncId = `flow-${args.sourceTopic.name}-${args.targetTopic?.name}`;

  const cluster = new Cluster({
    // This is an arbitrary value, we can adjust it as needed
    // based on the performance of the streaming functions
    // I would like it to be replaced by a value that could be dynamic and controlled
    // by the Rust CLI. Since the Rust CLI is managing all the processes, it could
    // abritrage the resources available between the different services
    // (streaming, consumption api, ingest API).
    maxCpuUsageRatio: 0.5,
    maxWorkerCount: args.maxSubscriberCount,
    workerStart: async (worker, parallelism) => {
      const logger = buildLogger(args, worker.id);

      let metrics = {
        count_in: 0,
        count_out: 0,
        bytes: 0,
      };

      setTimeout(() => sendMessageMetrics(logger, metrics), 1000);

      const clientIdPrefix = HOSTNAME ? `${HOSTNAME}-` : "";
      const processId = clientIdPrefix + streamingFuncId + "-ts-" + worker.id;

      const kafka = new Kafka({
        clientId: processId,
        brokers: [args.broker],
        ssl: args.securityProtocol === "SASL_SSL",
        sasl: buildSaslConfig(logger, args),
        retry: {
          initialRetryTime: RETRY_INITIAL_TIME_MS,
          maxRetryTime: MAX_RETRY_TIME_MS,
          retries: MAX_RETRIES,
        },
      });

      const consumer: Consumer = kafka.consumer({
        groupId: streamingFuncId,
        sessionTimeout: SESSION_TIMEOUT_CONSUMER,
        heartbeatInterval: HEARTBEAT_INTERVAL_CONSUMER,
        retry: {
          retries: MAX_RETRIES_CONSUMER,
        },
      });

      const producer: Producer = kafka.producer({
        transactionalId: processId,
        idempotent: true,
        retry: {
          retries: MAX_RETRIES_PRODUCER,
          factor: RETRY_FACTOR_PRODUCER,
          maxRetryTime: MAX_RETRY_TIME_MS,
        },
      });

      try {
        producer.on(producer.events.REQUEST, (event) => {
          logger.log(`Sending message size with ${event.payload.size}`);
        });

        if (args.targetTopic !== undefined) {
          await startProducer(logger, producer);
        }

        try {
          await startConsumer(
            args.isDmv2,
            logger,
            metrics,
            parallelism,
            args.functionFilePath,
            args.sourceTopic,
            consumer,
            producer,
            streamingFuncId,
            args.targetTopic,
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

      return [logger, producer, consumer] as [Logger, Producer, Consumer];
    },
    workerStop: async ([logger, producer, consumer]) => {
      logger.log(`Received SIGTERM, shutting down ...`);
      await consumer.stop(); // Stop consuming new messages
      // Wait for in-flight messages to complete
      await new Promise((resolve) => setTimeout(resolve, 5000));
      await stopProducer(logger, producer);
      await stopConsumer(logger, consumer);
    },
  });

  cluster.start();
};
