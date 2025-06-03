import json
import os
import time
import threading
import atexit
from typing import Any, Optional, TypeVar, Generic, Type
import redis
from redis import Redis

T = TypeVar('T')

class MooseCache:
    """
    A singleton Redis cache client that automatically handles connection management
    and key prefixing.

    Example:
        cache = MooseCache()  # Gets or creates the singleton instance
    """
    _instance = None
    _redis_url: str
    _key_prefix: str
    _client: Optional[Redis] = None
    _is_connected: bool = False
    _disconnect_timer: Optional[threading.Timer] = None
    _idle_timeout: int

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(MooseCache, cls).__new__(cls)
            atexit.register(cls._instance.disconnect)
        return cls._instance

    def __init__(self) -> None:
        if self._client is not None:
            return

        self._redis_url = os.getenv('MOOSE_REDIS_CONFIG__URL', 'redis://127.0.0.1:6379')
        prefix = os.getenv('MOOSE_REDIS_CONFIG__KEY_PREFIX', 'MS')
        # 30 seconds of inactivity before disconnecting
        self._idle_timeout = int(os.getenv('MOOSE_REDIS_CONFIG__IDLE_TIMEOUT', '30'))
        self._key_prefix = f"{prefix}::moosecache::"

        self._ensure_connected()

    def _get_prefixed_key(self, key: str) -> str:
        """Internal method to prefix keys with the configured prefix."""
        return f"{self._key_prefix}{key}"

    def _clear_disconnect_timer(self) -> None:
        """Clear the disconnect timer if it exists and create a new one."""
        if self._disconnect_timer is not None:
            self._disconnect_timer.cancel()
        self._disconnect_timer = threading.Timer(self._idle_timeout, self.disconnect)
        self._disconnect_timer.daemon = True

    def _ensure_connected(self) -> None:
        """Ensure the client is connected and reset the disconnect timer."""
        if not self._is_connected:
            self._client = redis.from_url(self._redis_url)
            self._is_connected = True
            print("Python Redis client connected")

        self._clear_disconnect_timer()
        self._disconnect_timer.start()

    def set(self, key: str, value: Any, ttl_seconds: Optional[int] = None) -> None:
        """
        Sets a value in the cache. Objects are automatically JSON stringified.

        Args:
            key: The key to store the value under
            value: The value to store. Can be a string or any object (will be JSON stringified)
            ttl_seconds: Optional time-to-live in seconds. If provided, the key will automatically expire after this duration

        Example:
            # Store a string
            cache.set("foo", "bar")

            # Store an object with TTL
            cache.set("foo:config", {"baz": 123, "qux": True}, 3600)  # expires in 1 hour
        """
        try:
            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key)
            string_value = json.dumps(value) if isinstance(value, (dict, list)) else str(value)

            if ttl_seconds:
                self._client.setex(prefixed_key, ttl_seconds, string_value)
            else:
                self._client.set(prefixed_key, string_value)
        except Exception as e:
            print(f"Error setting cache key {key}: {e}")
            raise

    def get(self, key: str, type_hint: Optional[Type[T]] = None) -> Optional[T]:
        """
        Retrieves a value from the cache. Attempts to parse the value as JSON if possible.

        Args:
            key: The key to retrieve
            type_hint: Optional type hint for the return value

        Returns:
            The value, parsed as the specified type if it was JSON, or as string if not.
            Returns None if key doesn't exist

        Example:
            # Get a string
            value = cache.get("foo")

            # Get and parse an object with type safety
            from typing import TypedDict
            class Config(TypedDict):
                baz: int
                qux: bool
            config = cache.get("foo:config", Config)
        """
        try:
            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key)
            value = self._client.get(prefixed_key)

            if value is None:
                return None

            # Try to parse as JSON, if it fails return as string
            try:
                parsed = json.loads(value)
                if type_hint:
                    return type_hint(parsed)
                return parsed
            except json.JSONDecodeError:
                return value.decode() if isinstance(value, bytes) else value
        except Exception as e:
            print(f"Error getting cache key {key}: {e}")
            raise

    def delete(self, key: str) -> None:
        """
        Deletes a specific key from the cache.

        Args:
            key: The key to delete

        Example:
            cache.delete("foo")
        """
        try:
            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key)
            self._client.delete(prefixed_key)
        except Exception as e:
            print(f"Error deleting cache key {key}: {e}")
            raise

    def clear_keys(self, key_prefix: str) -> None:
        """
        Deletes all keys that start with the given prefix.

        Args:
            key_prefix: The prefix of keys to delete

        Example:
            # Delete all keys starting with "foo"
            cache.clear_keys("foo")
        """
        try:
            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key_prefix)
            keys = self._client.keys(f"{prefixed_key}*")
            if keys:
                self._client.delete(*keys)
        except Exception as e:
            print(f"Error clearing cache keys with prefix {key_prefix}: {e}")
            raise

    def clear(self) -> None:
        """
        Deletes all keys in the cache

        Example:
            cache.clear()
        """
        try:
            self._ensure_connected()
            keys = self._client.keys(f"{self._key_prefix}*")
            if keys:
                self._client.delete(*keys)
        except Exception as e:
            print(f"Error clearing cache: {e}")
            raise

    def disconnect(self) -> None:
        """
        Manually disconnects the Redis client. The client will automatically reconnect
        when the next operation is performed.

        Example:
            cache.disconnect()
        """
        if self._is_connected and self._client:
            self._client.close()
            self._is_connected = False
            self._clear_disconnect_timer()

        print("Python Redis client disconnected")
