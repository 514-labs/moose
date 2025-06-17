"""
Runtime configuration management for Moose.

This module provides a singleton registry for managing runtime configuration settings,
particularly for ClickHouse connections.
"""
from dataclasses import dataclass
from typing import Optional

@dataclass
class RuntimeClickHouseConfig:
    """Runtime ClickHouse configuration settings."""
    host: str
    port: str
    username: str
    password: str
    database: str
    use_ssl: bool

class ConfigurationRegistry:
    """Singleton registry for managing runtime configuration.

    This class provides a centralized way to manage and access runtime configuration
    settings, with fallback to file-based configuration when runtime settings are not set.
    """
    _instance: Optional['ConfigurationRegistry'] = None
    _clickhouse_config: Optional[RuntimeClickHouseConfig] = None

    @classmethod
    def get_instance(cls) -> 'ConfigurationRegistry':
        """Get the singleton instance of ConfigurationRegistry.

        Returns:
            The singleton ConfigurationRegistry instance.
        """
        if not cls._instance:
            cls._instance = cls()
        return cls._instance

    def set_clickhouse_config(self, config: RuntimeClickHouseConfig) -> None:
        """Set the runtime ClickHouse configuration.

        Args:
            config: The ClickHouse configuration to use.
        """
        self._clickhouse_config = config

    def get_clickhouse_config(self) -> RuntimeClickHouseConfig:
        """Get the current ClickHouse configuration.

        If runtime configuration is not set, falls back to reading from moose.config.toml.

        Returns:
            The current ClickHouse configuration.
        """
        if self._clickhouse_config:
            return self._clickhouse_config

        # Fallback to reading from config file
        from .config_file import read_project_config
        try:
            config = read_project_config()
            return RuntimeClickHouseConfig(
                host=config.clickhouse_config.host,
                port=str(config.clickhouse_config.host_port),
                username=config.clickhouse_config.user,
                password=config.clickhouse_config.password,
                database=config.clickhouse_config.db_name,
                use_ssl=config.clickhouse_config.use_ssl
            )
        except Exception as e:
            raise RuntimeError(f"Failed to get ClickHouse configuration: {e}")

    def has_runtime_config(self) -> bool:
        """Check if runtime configuration is set.

        Returns:
            True if runtime configuration is set, False otherwise.
        """
        return self._clickhouse_config is not None

# Create singleton instance
config_registry = ConfigurationRegistry.get_instance()
