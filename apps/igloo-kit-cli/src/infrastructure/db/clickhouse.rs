use clickhouse::Client;
use reqwest::Url;

#[derive(Clone)]
pub struct ClickhouseConfig {
    pub db_name: String, // ex. local
    pub user: String,
    pub password: String,
    pub host: String, // ex. localhost
    pub host_port: i32, // ex. 18123
    pub postgres_port: i32, // ex. 9005
    pub kafka_port: i32, // ex. 9092
    pub cluster_network: String, // ex. panda-house
}

pub struct ConfiguredClient {
    pub client: Client,
    pub config: ClickhouseConfig,
}


pub fn create_client(clickhouse_config: ClickhouseConfig) -> ConfiguredClient {
    ConfiguredClient {
        client: Client::default()
        .with_url(Url::parse(&format!("http://{}:{}", clickhouse_config.host, clickhouse_config.host_port)).unwrap())
        .with_user(format!("{}", clickhouse_config.user))
        .with_password(format!("{}", clickhouse_config.password))
        .with_database(format!("{}", clickhouse_config.db_name)),
        config: clickhouse_config,
    }    
}

// Creates a table in clickhouse from a file name. this table should have a single field that accepts a json blob
pub async fn create_table(table_name: String, topic: String, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let config = &configured_client.config;
    let db_name = &config.db_name;
    let cluster_network = &config.cluster_network;
    let kafka_port = &config.kafka_port;

    // If you want to change the settings when doing a query you can do it as follows: SETTINGS allow_experimental_object_type = 1;
    client.query(format!("CREATE TABLE IF NOT EXISTS {db_name}.{table_name} (data String) ENGINE = Kafka('{cluster_network}:{kafka_port}', '{topic}', 'clickhouse-group', 'JSONEachRow') ;").as_str() ).execute().await
}

pub async fn delete_table(table_name: String, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let client = &configured_client.client;
    let db_name = &configured_client.config.db_name;

    client.query(format!("DROP TABLE {db_name}.{table_name}").as_str()).execute().await
}