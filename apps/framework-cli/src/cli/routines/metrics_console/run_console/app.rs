use prometheus_parse::HistogramCount;
use ratatui::layout::Rect;

use super::client::{parsing_histogram_data, ParsedMetricsData, PathMetricsData};
use std::{collections::HashMap, error};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum State {
    Main(),
    PathDetails(String),
}

pub struct BytesMetricsData {
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub path_bytes_hashmap: HashMap<String, u64>,
    pub kafka_bytes_out_total: HashMap<String, (String, u64)>,
    pub streaming_functions_bytes: HashMap<String, u64>,
}

pub struct PathBytesParsedData {
    pub previous_bytes: HashMap<String, u64>,
    pub path_bytes_in_per_sec_vec: HashMap<String, u64>,
    pub path_bytes_out_per_sec_vec: HashMap<String, u64>,
}

pub struct MainBytesParsedData {
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub bytes_in_per_sec: u64,
    pub bytes_out_per_sec: u64,
}

pub enum TableState {
    Endpoint,
    Kafka,
    StreamingFunction,
}

pub struct AppKafkaClickHouseSyncMetrics {
    pub kafka_messages_in_total: HashMap<String, (String, f64)>,
    pub kafka_messages_out_total: Vec<(String, String, f64)>,
    pub kafka_messages_out_per_sec: HashMap<String, (String, f64)>,
    pub kafka_bytes_out_total: HashMap<String, (String, u64)>,
    pub kafka_bytes_out_per_sec: HashMap<String, u64>,
}

pub struct AppStreamingFunctionsMetrics {
    pub streaming_functions_in: HashMap<String, f64>,
    pub streaming_functions_out: HashMap<String, f64>,
    pub streaming_functions_in_per_sec: HashMap<String, f64>,
    pub streaming_functions_out_per_sec: HashMap<String, f64>,
    pub streaming_functions_bytes: HashMap<String, u64>,
    pub streaming_functions_bytes_per_sec: HashMap<String, u64>,
}

pub struct AppOverviewMetrics {
    pub average: f64,
    pub total_requests: f64,
    pub requests_per_sec: f64,
    pub summary: Vec<PathMetricsData>,
    pub main_bytes_data: MainBytesParsedData,
}

pub struct AppTableScrollData {
    pub endpoint_starting_row: usize,
    pub kafka_starting_row: usize,
    pub streaming_functions_table_starting_row: usize,
}

pub struct AppPathsData {
    pub path_detailed_data: Option<Vec<HistogramCount>>,
    pub path_requests_per_sec: HashMap<String, f64>,
    pub requests_per_sec_vec: HashMap<String, Vec<u64>>,
    pub parsed_bytes_data: PathBytesParsedData,
}

pub struct App {
    pub table_state: TableState,
    pub viewport: Rect,
    pub state: State,
    pub running: bool,
    pub table_scroll_data: AppTableScrollData,
    pub paths_data: AppPathsData,
    pub kafka_clikhouse_sync_metrics: AppKafkaClickHouseSyncMetrics,
    pub streaming_functions_metrics: AppStreamingFunctionsMetrics,
    pub overview_data: AppOverviewMetrics,
}

impl Default for App {
    fn default() -> Self {
        Self {
            table_state: TableState::Endpoint,
            viewport: Rect::default(),
            state: State::Main(),
            running: true,
            table_scroll_data: AppTableScrollData {
                endpoint_starting_row: 0,
                kafka_starting_row: 0,
                streaming_functions_table_starting_row: 0,
            },
            paths_data: AppPathsData {
                path_detailed_data: None,
                path_requests_per_sec: HashMap::new(),
                requests_per_sec_vec: HashMap::new(),
                parsed_bytes_data: PathBytesParsedData {
                    previous_bytes: HashMap::new(),
                    path_bytes_in_per_sec_vec: HashMap::new(),
                    path_bytes_out_per_sec_vec: HashMap::new(),
                },
            },
            kafka_clikhouse_sync_metrics: AppKafkaClickHouseSyncMetrics {
                kafka_messages_in_total: HashMap::new(),
                kafka_messages_out_total: vec![],
                kafka_messages_out_per_sec: HashMap::new(),
                kafka_bytes_out_total: HashMap::new(),
                kafka_bytes_out_per_sec: HashMap::new(),
            },
            streaming_functions_metrics: AppStreamingFunctionsMetrics {
                streaming_functions_in: HashMap::new(),
                streaming_functions_out: HashMap::new(),
                streaming_functions_in_per_sec: HashMap::new(),
                streaming_functions_out_per_sec: HashMap::new(),
                streaming_functions_bytes: HashMap::new(),
                streaming_functions_bytes_per_sec: HashMap::new(),
            },
            overview_data: AppOverviewMetrics {
                average: 0.0,
                total_requests: 0.0,
                requests_per_sec: 0.0,
                summary: vec![],
                main_bytes_data: MainBytesParsedData {
                    total_bytes_in: 0,
                    total_bytes_out: 0,
                    bytes_in_per_sec: 0,
                    bytes_out_per_sec: 0,
                },
            },
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_metrics(&mut self, parsed_data: ParsedMetricsData) {
        self.overview_data.average = parsed_data.average_latency;
        self.overview_data.total_requests = parsed_data.total_requests;
        self.overview_data.summary = parsed_data.paths_data_vec;
        if !self.overview_data.summary.is_empty() {
            self.paths_data.path_detailed_data = parsing_histogram_data(
                self.overview_data.summary[self.table_scroll_data.endpoint_starting_row]
                    .path
                    .clone(),
                parsed_data.histogram_vec,
            )
        }

        self.paths_data.parsed_bytes_data.previous_bytes = parsed_data.paths_bytes_hashmap;

        self.overview_data.main_bytes_data.total_bytes_in = parsed_data.total_bytes_in;
        self.overview_data.main_bytes_data.total_bytes_out = parsed_data.total_bytes_out;
        self.kafka_clikhouse_sync_metrics.kafka_messages_in_total =
            parsed_data.kafka_messages_in_total;
        self.kafka_clikhouse_sync_metrics.kafka_messages_out_total =
            parsed_data.kafka_messages_out_total;
        self.kafka_clikhouse_sync_metrics.kafka_bytes_out_total = parsed_data.kafka_bytes_out_total;
        self.streaming_functions_metrics.streaming_functions_in =
            parsed_data.streaming_functions_in;
        self.streaming_functions_metrics.streaming_functions_out =
            parsed_data.streaming_functions_out;
        self.streaming_functions_metrics.streaming_functions_bytes =
            parsed_data.streaming_functions_bytes;
    }

    pub fn endpoint_down(&mut self) {
        if (self.table_scroll_data.endpoint_starting_row + 1) < self.overview_data.summary.len() {
            self.table_scroll_data.endpoint_starting_row += 1;
        }
    }
    pub fn endpoint_up(&mut self) {
        if self.table_scroll_data.endpoint_starting_row > 0 {
            self.table_scroll_data.endpoint_starting_row -= 1;
        }
    }

    pub fn kafka_down(&mut self) {
        if (self.table_scroll_data.kafka_starting_row + 1)
            < self
                .kafka_clikhouse_sync_metrics
                .kafka_messages_out_total
                .len()
        {
            self.table_scroll_data.kafka_starting_row += 1;
        }
    }
    pub fn kafka_up(&mut self) {
        if self.table_scroll_data.kafka_starting_row > 0 {
            self.table_scroll_data.kafka_starting_row -= 1;
        }
    }

    pub fn streaming_functions_down(&mut self) {
        if (self
            .table_scroll_data
            .streaming_functions_table_starting_row
            + 1)
            < self
                .streaming_functions_metrics
                .streaming_functions_in
                .len()
        {
            self.table_scroll_data
                .streaming_functions_table_starting_row += 1;
        }
    }
    pub fn streaming_functions_up(&mut self) {
        if self
            .table_scroll_data
            .streaming_functions_table_starting_row
            > 0
        {
            self.table_scroll_data
                .streaming_functions_table_starting_row -= 1;
        }
    }

    pub fn per_sec_metrics(
        &mut self,
        new_total_requests: f64,
        path_metrics: &Vec<PathMetricsData>,
        bytes_data: BytesMetricsData,
        kafka_messages_in_total: &Vec<(String, String, f64)>,
        new_streaming_functions_messages_in: &HashMap<String, f64>,
        new_streaming_functions_messages_out: &HashMap<String, f64>,
    ) {
        let path_bytes_hashmap: HashMap<String, u64> = bytes_data.path_bytes_hashmap;
        let total_bytes_in: u64 = bytes_data.total_bytes_in;
        let total_bytes_out: u64 = bytes_data.total_bytes_out;
        let new_kafka_bytes_out = bytes_data.kafka_bytes_out_total;
        let streaming_functions_bytes: HashMap<String, u64> = bytes_data.streaming_functions_bytes;

        self.overview_data.requests_per_sec =
            new_total_requests - self.overview_data.total_requests;
        self.overview_data.main_bytes_data.bytes_in_per_sec =
            total_bytes_in - self.overview_data.main_bytes_data.total_bytes_in;
        self.overview_data.main_bytes_data.bytes_out_per_sec =
            total_bytes_out - self.overview_data.main_bytes_data.total_bytes_out;

        // Initializes variables and vec for each path in summary to keep unwraps in ui.rs safe
        for path in &self.overview_data.summary {
            if !self
                .paths_data
                .path_requests_per_sec
                .contains_key(&path.path)
            {
                self.paths_data
                    .path_requests_per_sec
                    .insert(path.path.clone(), path.request_count);
            }

            if !self
                .paths_data
                .requests_per_sec_vec
                .contains_key(&path.path)
            {
                self.paths_data
                    .requests_per_sec_vec
                    .insert(path.path.clone(), vec![0; 150]);
            }

            if !self
                .paths_data
                .parsed_bytes_data
                .path_bytes_in_per_sec_vec
                .contains_key(&path.path)
            {
                self.paths_data
                    .parsed_bytes_data
                    .path_bytes_in_per_sec_vec
                    .insert(path.path.clone(), 0);
            }

            if !self
                .paths_data
                .parsed_bytes_data
                .path_bytes_out_per_sec_vec
                .contains_key(&path.path)
            {
                self.paths_data
                    .parsed_bytes_data
                    .path_bytes_out_per_sec_vec
                    .insert(path.path.clone(), 0);
            }

            for item in path_metrics {
                if item.path == path.path {
                    self.paths_data
                        .path_requests_per_sec
                        .insert(path.path.clone(), item.request_count - path.request_count);
                }
            }
            self.paths_data
                .requests_per_sec_vec
                .get_mut(&path.path)
                .unwrap()
                .push(
                    *self
                        .paths_data
                        .path_requests_per_sec
                        .get(&path.path)
                        .unwrap() as u64,
                );
            while self
                .paths_data
                .requests_per_sec_vec
                .get(&path.path)
                .unwrap()
                .len()
                > 150
            {
                self.paths_data
                    .requests_per_sec_vec
                    .get_mut(&path.path)
                    .unwrap()
                    .remove(0);
            }
        }
        for path in &self.paths_data.parsed_bytes_data.previous_bytes {
            match path_bytes_hashmap.get(path.0) {
                Some(value) => {
                    if path.0.starts_with("ingest/") {
                        self.paths_data
                            .parsed_bytes_data
                            .path_bytes_in_per_sec_vec
                            .insert(path.0.clone(), *value - path.1);
                    } else {
                        self.paths_data
                            .parsed_bytes_data
                            .path_bytes_out_per_sec_vec
                            .insert(path.0.clone(), *value - path.1);
                    }
                }
                None => {
                    if path.0.starts_with("ingest/") {
                        self.paths_data
                            .parsed_bytes_data
                            .path_bytes_in_per_sec_vec
                            .insert(path.0.clone(), 0);
                    } else {
                        self.paths_data
                            .parsed_bytes_data
                            .path_bytes_out_per_sec_vec
                            .insert(path.0.clone(), 0);
                    }
                }
            }

            for item in kafka_messages_in_total {
                for prev_item in &self.kafka_clikhouse_sync_metrics.kafka_messages_out_total {
                    if item.0 == prev_item.0 && item.1 == prev_item.1 {
                        self.kafka_clikhouse_sync_metrics
                            .kafka_messages_out_per_sec
                            .insert(item.0.clone(), (item.1.clone(), item.2 - prev_item.2));
                    }
                }
            }
            for item in new_streaming_functions_messages_in {
                for prev_item in &self.streaming_functions_metrics.streaming_functions_in {
                    if item.0 == prev_item.0 {
                        self.streaming_functions_metrics
                            .streaming_functions_out_per_sec
                            .insert(item.0.clone(), item.1 - prev_item.1);
                    }
                }
            }
            for item in new_streaming_functions_messages_out {
                for prev_item in &self.streaming_functions_metrics.streaming_functions_out {
                    if item.0 == prev_item.0 {
                        self.streaming_functions_metrics
                            .streaming_functions_in_per_sec
                            .insert(item.0.clone(), item.1 - prev_item.1);
                    }
                }
            }

            for item in &new_kafka_bytes_out {
                for prev_item in &self.kafka_clikhouse_sync_metrics.kafka_bytes_out_total {
                    if *item.0 == *prev_item.0 {
                        self.kafka_clikhouse_sync_metrics
                            .kafka_bytes_out_per_sec
                            .insert(item.0.clone(), item.1 .1 - prev_item.1 .1);
                    }
                }
            }

            for item in &streaming_functions_bytes {
                for prev_item in &self.streaming_functions_metrics.streaming_functions_bytes {
                    if *item.0 == *prev_item.0 {
                        self.streaming_functions_metrics
                            .streaming_functions_bytes_per_sec
                            .insert(item.0.clone(), item.1 - prev_item.1);
                    }
                }
            }
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}
