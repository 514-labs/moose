pub(super) const NAMESPACE_SEPARATOR: &str = ".";

pub(super) const KAFKA_RETENTION_CONFIG_KEY: &str = "retention.ms";
pub(super) const KAFKA_MAX_MESSAGE_BYTES_CONFIG_KEY: &str = "max.message.bytes";
pub(super) const KAFKA_BOOSTRAP_SERVERS_CONFIG_KEY: &str = "bootstrap.servers";
pub(super) const KAFKA_SASL_USERNAME_CONFIG_KEY: &str = "sasl.username";
pub(super) const KAFKA_SASL_PASSWORD_CONFIG_KEY: &str = "sasl.password";
pub(super) const KAFKA_SASL_MECHANISM_CONFIG_KEY: &str = "sasl.mechanism";
pub(super) const KAFKA_SECURITY_PROTOCOL_CONFIG_KEY: &str = "security.protocol";
pub(super) const KAFKA_MESSAGE_TIMEOUT_MS_CONFIG_KEY: &str = "message.timeout.ms";
pub(super) const KAFKA_ENABLE_IDEMPOTENCE_CONFIG_KEY: &str = "enable.idempotence";
pub(super) const KAFKA_ACKS_CONFIG_KEY: &str = "acks";
pub(super) const KAFKA_ENABLE_GAPLESS_GUARANTEE_CONFIG_KEY: &str = "enable.gapless.guarantee";
pub(super) const KAFKA_RETRIES_CONFIG_KEY: &str = "retries";

pub(super) const KAFKA_SESSION_TIMEOUT_MS_CONFIG_KEY: &str = "session.timeout.ms";
pub(super) const KAFKA_ENABLE_PARTITION_EOF_CONFIG_KEY: &str = "enable.partition.eof";
pub(super) const KAFKA_ENABLE_AUTO_COMMIT_CONFIG_KEY: &str = "enable.auto.commit";
pub(super) const KAFKA_AUTO_COMMIT_INTERVAL_MS_CONFIG_KEY: &str = "auto.commit.interval.ms";
pub(super) const KAFKA_ENABLE_AUTO_OFFSET_STORE_CONFIG_KEY: &str = "enable.auto.offset.store";
pub(super) const KAFKA_AUTO_OFFSET_RESET_CONFIG_KEY: &str = "auto.offset.reset";
pub(super) const KAFKA_GROUP_ID_CONFIG_KEY: &str = "group.id";
pub(super) const KAFKA_ISOLATION_LEVEL_CONFIG_KEY: &str = "isolation.level";

// This is 9 MB
pub(super) const DEFAULT_MAX_MESSAGE_BYTES: usize = 10000;

// This is 10 seconds
pub(super) const DEFAULT_RETENTION_MS: u128 = 10000;
