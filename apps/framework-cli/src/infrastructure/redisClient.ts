import Redis from "redis";
import { v4 as uuidv4 } from "uuid";

const REDIS_KEY_PREFIX = "MS"; // MooSe
const PRESENCE_UPDATE_INTERVAL = 1000; // 1 second
const KEY_EXPIRATION_TTL = 3; // 3 seconds
const LOCK_TTL = 10; // 10 seconds
const LOCK_RENEWAL_INTERVAL = 3000; // 3 seconds

type MessageCallback = (message: string) => void;

let redis: Redis.RedisClientType;
let redisPubSub: Redis.RedisClientType;
let serviceName: string;
let instanceId: string;
let presenceIntervalId: NodeJS.Timeout;
let subscribedChannels: string[] = [];
let broadcastCallback: MessageCallback = () => {};
let instanceCallback: MessageCallback = () => {};

let lockKey = "";
let isLeader = false;
let renewLockIntervalId: NodeJS.Timeout;

/**
 * Initialize the Redis client
 * @param name - The name of the service
 * @returns The instance ID
 */
export async function init(name: string) {
  serviceName = name;
  lockKey = `${REDIS_KEY_PREFIX}::${serviceName}::leader-lock`;
  instanceId = uuidv4();
  console.log(
    `Initializing Redis client for ${serviceName} with instance ID: ${instanceId}`,
  );

  redis = Redis.createClient({
    url: process.env.REDIS_URL,
  });

  redisPubSub = Redis.createClient({
    url: process.env.REDIS_URL,
  });

  redis.on("error", (err) => console.log("Redis Client Error", err));
  redisPubSub.on("error", (err) =>
    console.log("Redis PubSub Client Error", err),
  );

  await redis.connect();
  await redisPubSub.connect();
  console.log("Redis clients connected");
  await createMessageChannels();

  startPeriodicTasks();
  await startLeaderElection();
  return instanceId;
}

/**
 * Cleanup and disconnect the Redis client
 */
export async function disconnectRedis() {
  stopPeriodicTasks();
  await releaseLock();
  await unsubscribeFromChannels();
  if (redis && redis.isOpen) {
    await redis.quit();
  }
  if (redisPubSub && redisPubSub.isOpen) {
    await redisPubSub.quit();
  }
  console.log("Redis clients disconnected");
}

/**
 * Register the channels for the Redis client
 * @param broadcastCb - The callback for the broadcast channel
 * @param instanceCb - The callback for the instance channel
 */
export function registerChannels(
  broadcastCb: MessageCallback,
  instanceCb: MessageCallback,
) {
  broadcastCallback = broadcastCb;
  instanceCallback = instanceCb;
}

/**
 * Create the message channels for the Redis client
 */
async function createMessageChannels() {
  const broadcastChannel = `${REDIS_KEY_PREFIX}::${serviceName}::msgchannel`;
  const instanceChannel = `${REDIS_KEY_PREFIX}::${serviceName}::${instanceId}::msgchannel`;

  await redisPubSub.subscribe(broadcastChannel, (message) => {
    broadcastCallback(message);
  });
  await redisPubSub.subscribe(instanceChannel, (message) => {
    instanceCallback(message);
  });

  subscribedChannels = [broadcastChannel, instanceChannel];
}

/**
 * Unsubscribe from the channels for the Redis client
 */
async function unsubscribeFromChannels() {
  for (const channel of subscribedChannels) {
    await redisPubSub.unsubscribe(channel);
  }
  subscribedChannels = [];
  console.log("Unsubscribed from all channels");
}

/**
 * Send a message to a specific instance
 * @param message - The message to send
 * @param instanceId - The instance ID to send the message to
 */
export async function sendMessageToInstance(
  message: string,
  instanceId: string,
) {
  await redis.publish(
    `${REDIS_KEY_PREFIX}::${serviceName}::${instanceId}::msgchannel`,
    message,
  );
}

/**
 * Broadcast a message to all instances
 * @param message - The message to broadcast
 */
export async function broadcastMessage(message: string) {
  await redis.publish(
    `${REDIS_KEY_PREFIX}::${serviceName}::msgchannel`,
    message,
  );
}

/**
 * Get the instance ID
 * @returns The instance ID
 */
export function getInstanceId() {
  return instanceId;
}

/**
 * Get the service name
 * @returns The service name
 */
export function getServiceName() {
  return serviceName;
}

/**
 * Update the presence of the instance
 */
async function presenceUpdate() {
  const instanceId = getInstanceId();
  redis.setEx(
    `${REDIS_KEY_PREFIX}::${serviceName}::${instanceId}::presence`,
    KEY_EXPIRATION_TTL,
    Date.now().toString(),
  );
}

/**
 * Renew the leader lock if this instance is the current leader
 */
async function renewLockUpdate() {
  if (isCurrentLeader()) {
    try {
      await renewLock();
    } catch (error) {
      console.error("Error in renewLockUpdate:", error);
    }
  } else {
    await attemptLeadership();
  }
}

/**
 * Start the periodic tasks
 */
function startPeriodicTasks() {
  presenceIntervalId = setInterval(presenceUpdate, PRESENCE_UPDATE_INTERVAL);
  renewLockIntervalId = setInterval(renewLockUpdate, LOCK_RENEWAL_INTERVAL);
  console.log("Periodic tasks started");
}

/**
 * Stop the periodic tasks
 */
function stopPeriodicTasks() {
  if (presenceIntervalId) {
    clearInterval(presenceIntervalId);
  }
  if (renewLockIntervalId) {
    clearInterval(renewLockIntervalId);
  }
  console.log("Periodic tasks stopped");
}

/**
 * Get a message from the queue
 * @returns The message
 */
export async function getQueueMessage() {
  const sourceQueue = `${REDIS_KEY_PREFIX}::${serviceName}::mqrecieved`;
  const destinationQueue = `${REDIS_KEY_PREFIX}::${serviceName}::mqprocess`;
  const message = await redis.rPopLPush(sourceQueue, destinationQueue);
  return message;
}

/**
 * Post a message to the queue
 * @param message - The message to post
 */
export async function postQueueMessage(message: string) {
  const queue = `${REDIS_KEY_PREFIX}::${serviceName}::mqrecieved`;
  await redis.rPush(queue, message);
}

/**
 * Mark a message in the queue as processed
 * @param message - The message to mark
 * @param success - Whether the message was processed successfully
 */
export async function markQueueMessage(message: string, success: boolean) {
  const inProgressQueue = `${REDIS_KEY_PREFIX}::${serviceName}::mqinprogress`;
  const incompleteQueue = `${REDIS_KEY_PREFIX}::${serviceName}::mqincomplete`;

  if (success) {
    // If successful, remove the message from the process queue
    await redis.lRem(inProgressQueue, 0, message);
    console.log(
      `Successfully processed and removed message from ${inProgressQueue}`,
    );
  } else {
    // If not successful, move the message to the incomplete queue
    await redis
      .multi()
      .lRem(inProgressQueue, 0, message)
      .rPush(incompleteQueue, message)
      .exec();
    console.log(
      `Moved unsuccessful message from ${inProgressQueue} to ${incompleteQueue}`,
    );
  }
}

/**
 * Start the leader election
 */
export async function startLeaderElection(): Promise<void> {
  await attemptLeadership();
}

/**
 * Attempt to gain leadership
 */
async function attemptLeadership(): Promise<void> {
  try {
    const result = await redis.set(lockKey, instanceId, {
      NX: true,
      EX: LOCK_TTL,
    });

    if (result === "OK") {
      if (!isLeader) {
        console.log(`Instance ${instanceId} became leader`);
        isLeader = true;
      }
      await renewLock();
    } else if (isLeader) {
      console.log(`Instance ${instanceId} lost leadership`);
      isLeader = false;
    }
  } catch (error) {
    console.error("Error in leader election:", error);
    isLeader = false;
  }
}

/**
 * Renew the leader lock
 */
async function renewLock(): Promise<void> {
  try {
    const extended = await redis.expire(lockKey, LOCK_TTL);
    if (!extended) {
      console.log("Failed to extend leader lock, lost leadership");
      isLeader = false;
    }
  } catch (error) {
    console.error("Error extending leader lock:", error);
    isLeader = false;
  }
}

/**
 * Release the leader lock
 */
async function releaseLock(): Promise<void> {
  const script = `
    if redis.call("get", KEYS[1]) == ARGV[1] then
      return redis.call("del", KEYS[1])
    else
      return 0
    end
  `;

  try {
    await redis.eval(script, {
      keys: [lockKey],
      arguments: [instanceId],
    });
    console.log(`Instance ${instanceId} released leadership`);
    isLeader = false;
  } catch (error) {
    console.error("Error releasing lock:", error);
  }
}

/**
 * Check if the current instance is the leader
 * @returns Whether the current instance is the leader
 */
export function isCurrentLeader(): boolean {
  return isLeader;
}
