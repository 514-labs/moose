import { createClient, RedisClientType } from "redis";

export class MooseCache {
  private static instance: MooseCache;
  private client: RedisClientType;
  private isConnected: boolean = false;
  private readonly keyPrefix: string;
  private disconnectTimer: NodeJS.Timeout | null = null;
  private readonly idleTimeout: number;

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

    this.client.on("error", (err: Error) => {
      console.error("TS Redis client error:", err);
      this.isConnected = false;
      this.clearDisconnectTimer();
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
    if (!this.isConnected) {
      await this.client.connect();
      this.resetDisconnectTimer();
    }
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
    if (!MooseCache.instance) {
      MooseCache.instance = new MooseCache();
      await MooseCache.instance.connect();
    }
    return MooseCache.instance;
  }

  /**
   * Sets a value in the cache. Objects are automatically JSON stringified.
   *
   * @param key - The key to store the value under
   * @param value - The value to store. Can be a string or any object (will be JSON stringified)
   * @param ttlSeconds - Optional time-to-live in seconds. If provided, the key will automatically expire after this duration
   * @example
   * // Store a string
   * await cache.set("foo", "bar");
   *
   * // Store an object with TTL
   * await cache.set("foo:config", { baz: 123, qux: true }, 3600); // expires in 1 hour
   */
  public async set(
    key: string,
    value: string | object,
    ttlSeconds?: number,
  ): Promise<void> {
    try {
      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(key);
      const stringValue =
        typeof value === "object" ? JSON.stringify(value) : value;

      if (ttlSeconds) {
        await this.client.setEx(prefixedKey, ttlSeconds, stringValue);
      } else {
        await this.client.set(prefixedKey, stringValue);
      }
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
  public async get<T = string>(key: string): Promise<T | null> {
    try {
      await this.ensureConnected();
      const prefixedKey = this.getPrefixedKey(key);
      const value = await this.client.get(prefixedKey);

      if (value === null) return null;

      // Try to parse as JSON, if it fails return as string
      try {
        return JSON.parse(value) as T;
      } catch {
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
    if (this.isConnected) {
      await this.client.quit();
    }
  }
}
