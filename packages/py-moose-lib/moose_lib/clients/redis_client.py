import atexit
import json
import os
import redis
import threading
from pydantic import BaseModel
from typing import Optional, TypeVar, Type, Union, TypeAlias


T = TypeVar('T')
SupportedValue: TypeAlias = Union[str, BaseModel]

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
    _client: Optional[redis.Redis] = None
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
            self._client = redis.from_url(self._redis_url, decode_responses=True)
            self._is_connected = True
            print("Python Redis client connected")

        self._clear_disconnect_timer()
        self._disconnect_timer.start()

    def set(self, key: str, value: SupportedValue, ttl_seconds: Optional[int] = None) -> None:
        """
        Sets a value in the cache. Only accepts strings or Pydantic models.
        Objects are automatically JSON stringified.

        Args:
            key: The key to store the value under
            value: The value to store. Must be a string or Pydantic model
            ttl_seconds: Optional time-to-live in seconds. If not provided, defaults to 1 hour (3600 seconds).
                       Must be a non-negative number. If 0, the key will expire immediately.

        Example:
            ### Store a string
            cache.set("foo", "bar")

            ### Store a Pydantic model
            class Config(BaseModel):
                baz: int
                qux: bool
            cache.set("foo:config", Config(baz=123, qux=True))
        """
        try:
            # Validate value type
            if not isinstance(value, (str, BaseModel)):
                raise TypeError(
                    f"Value must be a string or Pydantic model. Got {type(value).__name__}"
                )

            # Validate TTL
            if ttl_seconds is not None and ttl_seconds < 0:
                raise ValueError("ttl_seconds must be a non-negative number")

            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key)

            if isinstance(value, str):
                string_value = value
            else:
                string_value = value.model_dump_json()

            # Use provided TTL or default to 1 hour
            ttl = ttl_seconds if ttl_seconds is not None else 3600
            self._client.setex(prefixed_key, ttl, string_value)
        except Exception as e:
            print(f"Error setting cache key {key}: {e}")
            raise

    def get(self, key: str, type_hint: Type[T] = str) -> Optional[T]:
        """
        Retrieves a value from the cache. Only supports strings or Pydantic models.
        The type_hint parameter determines how the value will be parsed and returned.

        Args:
            key: The key to retrieve
            type_hint: Type hint for the return value. Must be str or a Pydantic model class.
                      Defaults to str.

        Returns:
            The value parsed as the specified type. Returns None if key doesn't exist.

        Example:
            ### Get a string (default)
            value = cache.get("foo")

            ### Get and parse as Pydantic model
            class Config(BaseModel):
                baz: int
                qux: bool
            config = cache.get("foo:config", Config)
        """
        try:
            # Validate type_hint
            if not isinstance(type_hint, type):
                raise TypeError("type_hint must be a type")
            if not (type_hint is str or issubclass(type_hint, BaseModel)):
                raise TypeError(
                    "type_hint must be str or a Pydantic model class. "
                    f"Got {type_hint.__name__}"
                )

            self._ensure_connected()
            prefixed_key = self._get_prefixed_key(key)
            value = self._client.get(prefixed_key)

            if value is None:
                return None
            elif type_hint is str:
                return value
            elif isinstance(type_hint, type) and issubclass(type_hint, BaseModel):
                try:
                    parsed = json.loads(value)
                    return type_hint.model_validate(parsed)
                except Exception as e:
                    raise ValueError(f"Failed to validate as {type_hint.__name__}: {e}")
            else:
                raise TypeError(f"Unsupported type_hint: {type_hint}")

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
