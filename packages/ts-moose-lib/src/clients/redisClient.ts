import { createClient, RedisClientType } from "redis";

// Module-level singleton instance and initialization promise
let instance: MooseCache | null = null;
let initPromise: Promise<MooseCache> | null = null;

type SupportedTypes = string | object;

export class MooseCache {
  private client: RedisClientType;
  private isConnected: boolean = false;
  private readonly keyPrefix: string;
  private disconnectTimer: NodeJS.Timeout | null = null;
  private readonly idleTimeout: number;
  private connectPromise: Promise<void> | null = null;

  private constructor() {
    const redisUrl =
      process.env.MOOSE_REDIS_CONFIG__URL || "redis://127.0.0.1:6379";
    const prefix = process.env.MOOSE_REDIS_CONFIG__KEY_PREFIX || "MS";
    // 30 seconds of inactivity before disconnecting
    this.idleTimeout =
      parseInt(process.env.MOOSE_REDIS_CONFIG__IDLE_TIMEOUT || "30", 10) * 1000;
    this.keyPrefix = `${prefix}::moosecache::`;

    this.client = createClient({
      url: redisUrl,
    });

    process.on("SIGTERM", this.gracefulShutdown);
    process.on("SIGINT", this.gracefulShutdown);

    this.client.on("error", async (err: Error) => {
      console.error("TS Redis client error:", err);
      await this.disconnect();
    });

    this.client.on("connect", () => {
      this.isConnected = true;
      console.log("TS Redis client connected");
    });

    this.client.on("end", () => {
      this.isConnected = false;
      console.log("TS Redis client disconnected");
      this.clearDisconnectTimer();
    });
  }

  private clearDisconnectTimer(): void {
    if (this.disconnectTimer) {
      clearTimeout(this.disconnectTimer);
      this.disconnectTimer = null;
    }
  }

  private resetDisconnectTimer(): void {
    this.clearDisconnectTimer();
    this.disconnectTimer = setTimeout(async () => {
      if (this.isConnected) {
        console.log("TS Redis client disconnecting due to inactivity");
        await this.disconnect();
      }
    }, this.idleTimeout);
  }

  private async ensureConnected(): Promise<void> {
    if (!this.isConnected) {
      await this.connect();
    }
    this.resetDisconnectTimer();
  }

  private async connect(): Promise<void> {
    // If already connected, return immediately
    if (this.isConnected) {
      return;
    }

    // If connection is in progress, wait for it
    // This prevents race conditions when multiple callers try to reconnect
    // simultaneously after a disconnection
    if (this.connectPromise) {
      return this.connectPromise;
    }

    // Start connection
    this.connectPromise = (async () => {
      try {
        await this.client.connect();
        this.resetDisconnectTimer();
      } catch (error) {
        // Reset the promise on error so retries can work
        this.connectPromise = null;
        throw error;
      }
    })();

    return this.connectPromise;
  }

  private async gracefulShutdown(): Promise<void> {
    if (this.isConnected) {
      await this.disconnect();
    }
    process.exit(0);
  }

  private getPrefixedKey(key: string): string {
    return `${this.keyPrefix}${key}`;
  }

  /**
   * Gets the singleton instance of MooseCache. Creates a new instance if one doesn't exist.
   * The client will automatically connect to Redis and handle reconnection if needed.
   *
   * @returns Promise<MooseCache> The singleton instance of MooseCache
   * @example
   * const cache = await MooseCache.get();
   */
  public static async get(): Promise<MooseCache> {
    // If we already have an instance, return it immediately
    if (instance) {
      return instance;
    }

    // If initialization is already in progress, wait for it
    // This prevents race conditions where multiple concurrent calls to get()
    // would each create their own instance and connection
    //
    // A simple singleton pattern (just checking if instance exists) isn't enough
    // because multiple async calls can check "if (!instance)" simultaneously,
    // find it's null, and each try to create their own instance before any
    // of them finish setting the instance variable
    if (initPromise) {
      return initPromise;
    }

    // Start initialization
    // We store the promise immediately so that any concurrent calls
    // will wait for this same initialization instead of starting their own
    initPromise = (async () => {
      try {
        const newInstance = new MooseCache();
        await newInstance.connect();
        instance = newInstance;
        return newInstance;
      } catch (error) {
        // Reset the promise on error so retries can work
        initPromise = null;
        throw error;
      }
    })();

    return initPromise;
  }

  /**
   * Sets a value in the cache. Objects are automatically JSON stringified.
   *
   * @param key - The key to store the value under
   * @param value - The value to store. Can be a string or any object (will be JSON stringified)
   * @param ttlSeconds - Optional time-to-live in seconds. If not provided, defaults to 1 hour (3600 seconds).
   *                    Must be a non-negative number. If 0, the key will expire immediately.
   * @example
   * // Store a string
   * await cache.set("foo", "bar");
   *
   * // Store an object with custom TTL
   * await cache.set("foo:config", { baz: 123, qux: true }, 60); // expires in 1 minute
   *
   * // This is essentially a get-set, which returns the previous value if it exists.
   * // You can create logic to only do work for the first time.
   * const value = await cache.set("testSessionId", "true");
   * if (value) {
   *   // Cache was set before, return
   * } else {
   *   // Cache was set for first time, do work
   * }
   */
  public async set(
    key: string,
    value: string | object,
    ttlSeconds?: number,
  ): Promise<string | null> {
    try {
      // Validate TTL
      if (ttlSeconds !== undefined && ttlSeconds < 0) {
        throw new Error("ttlSeconds must be a non-negative number");
      }

      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(key);
      const stringValue =
        typeof value === "object" ? JSON.stringify(value) : value;

      // Use provided TTL or default to 1 hour
      const ttl = ttlSeconds ?? 3600;
      return await this.client.set(prefixedKey, stringValue, {
        EX: ttl,
        GET: true,
      });
    } catch (error) {
      console.error(`Error setting cache key ${key}:`, error);
      throw error;
    }
  }

  /**
   * Retrieves a value from the cache. Attempts to parse the value as JSON if possible.
   *
   * @param key - The key to retrieve
   * @returns Promise<T | null> The value, parsed as type T if it was JSON, or as string if not. Returns null if key doesn't exist
   * @example
   * // Get a string
   * const value = await cache.get("foo");
   *
   * // Get and parse an object with type safety
   * interface Config { baz: number; qux: boolean; }
   * const config = await cache.get<Config>("foo:config");
   */
  public async get<T extends SupportedTypes = string>(
    key: string,
  ): Promise<T | null> {
    try {
      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(key);
      const value = await this.client.get(prefixedKey);

      if (value === null) return null;

      // Note: We can't check if T is string at runtime because TypeScript types are erased.
      // Instead, we try to parse as JSON and return the original string if that fails.
      try {
        const parsed = JSON.parse(value);
        // Only return parsed value if it's an object
        if (typeof parsed === "object" && parsed !== null) {
          return parsed as T;
        }
        // If parsed value isn't an object, return as string
        return value as T;
      } catch {
        // If JSON parse fails, return as string
        return value as T;
      }
    } catch (error) {
      console.error(`Error getting cache key ${key}:`, error);
      throw error;
    }
  }

  /**
   * Deletes a specific key from the cache.
   *
   * @param key - The key to delete
   * @example
   * await cache.delete("foo");
   */
  public async delete(key: string): Promise<void> {
    try {
      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(key);
      await this.client.del(prefixedKey);
    } catch (error) {
      console.error(`Error deleting cache key ${key}:`, error);
      throw error;
    }
  }

  /**
   * Deletes all keys that start with the given prefix.
   *
   * @param keyPrefix - The prefix of keys to delete
   * @example
   * // Delete all keys starting with "foo"
   * await cache.clearKeys("foo");
   */
  public async clearKeys(keyPrefix: string): Promise<void> {
    try {
      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(keyPrefix);
      const keys = await this.client.keys(`${prefixedKey}*`);
      if (keys.length > 0) {
        await this.client.del(keys);
      }
    } catch (error) {
      console.error(
        `Error clearing cache keys with prefix ${keyPrefix}:`,
        error,
      );
      throw error;
    }
  }

  /**
   * Deletes all keys in the cache
   *
   * @example
   * await cache.clear();
   */
  public async clear(): Promise<void> {
    try {
      await this.ensureConnected();
      const keys = await this.client.keys(`${this.keyPrefix}*`);
      if (keys.length > 0) {
        await this.client.del(keys);
      }
    } catch (error) {
      console.error("Error clearing cache:", error);
      throw error;
    }
  }

  /**
   * Manually disconnects the Redis client. The client will automatically reconnect
   * when the next operation is performed.
   *
   * @example
   * await cache.disconnect();
   */
  public async disconnect(): Promise<void> {
    this.clearDisconnectTimer();
    this.connectPromise = null;
    if (this.isConnected) {
      await this.client.quit();
    }
  }
}
