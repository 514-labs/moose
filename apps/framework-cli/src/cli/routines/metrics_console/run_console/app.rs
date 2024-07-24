use prometheus_parse::HistogramCount;
use ratatui::layout::Rect;

use super::client::{parsing_histogram_data, ParsedMetricsData, PathMetricsData};
use std::{collections::HashMap, error};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum State {
    Main(),
    PathDetails(String),
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

pub struct App {
    pub table_state: TableState,
    pub viewport: Rect,
    pub state: State,
    pub running: bool,
    pub average: f64,
    pub requests_per_sec: f64,
    pub total_requests: f64,
    pub summary: Vec<PathMetricsData>,
    pub endpoint_starting_row: usize,
    pub kafka_starting_row: usize,
    pub streaming_functions_table_starting_row: usize,
    pub path_detailed_data: Option<Vec<HistogramCount>>,
    pub path_requests_per_sec: HashMap<String, f64>,
    pub requests_per_sec_vec: HashMap<String, Vec<u64>>,
    pub parsed_bytes_data: PathBytesParsedData,
    pub main_bytes_data: MainBytesParsedData,
    pub kafka_messages_in_total: HashMap<String, (String, f64)>,
    pub kafka_messages_out_total: Vec<(String, String, f64)>,
    pub kafka_messages_out_per_sec: HashMap<String, (String, f64)>,
    pub streaming_functions_in: HashMap<String, f64>,
    pub streaming_functions_out: HashMap<String, f64>,
    pub streaming_functions_in_per_sec: HashMap<String, f64>,
    pub streaming_functions_out_per_sec: HashMap<String, f64>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            table_state: TableState::Endpoint,
            viewport: Rect::new(0, 0, 0, 0),
            state: State::Main(),
            running: true,
            average: 0.0,
            requests_per_sec: 0.0,
            total_requests: 0.0,
            summary: vec![],
            endpoint_starting_row: 0,
            kafka_starting_row: 0,
            streaming_functions_table_starting_row: 0,
            path_detailed_data: vec![].into(),
            path_requests_per_sec: HashMap::new(),
            requests_per_sec_vec: HashMap::new(),
            parsed_bytes_data: PathBytesParsedData {
                previous_bytes: HashMap::new(),
                path_bytes_in_per_sec_vec: HashMap::new(),
                path_bytes_out_per_sec_vec: HashMap::new(),
            },
            main_bytes_data: MainBytesParsedData {
                total_bytes_in: 0,
                total_bytes_out: 0,
                bytes_in_per_sec: 0,
                bytes_out_per_sec: 0,
            },
            kafka_messages_in_total: HashMap::new(),
            kafka_messages_out_total: vec![],
            kafka_messages_out_per_sec: HashMap::new(),
            streaming_functions_in: HashMap::new(),
            streaming_functions_out: HashMap::new(),
            streaming_functions_in_per_sec: HashMap::new(),
            streaming_functions_out_per_sec: HashMap::new(),
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
        self.average = parsed_data.average_latency;
        self.total_requests = parsed_data.total_requests;
        self.summary = parsed_data.paths_data_vec;
        if !self.summary.is_empty() {
            self.path_detailed_data = parsing_histogram_data(
                self.summary[self.endpoint_starting_row].path.clone(),
                parsed_data.histogram_vec,
            )
        }

        self.parsed_bytes_data.previous_bytes = parsed_data.paths_bytes_hashmap;

        self.main_bytes_data.total_bytes_in = parsed_data.total_bytes_in;
        self.main_bytes_data.total_bytes_out = parsed_data.total_bytes_out;
        self.kafka_messages_in_total = parsed_data.kafka_messages_in_total;
        self.kafka_messages_out_total = parsed_data.kafka_messages_out_total;
        self.streaming_functions_in = parsed_data.streaming_functions_in;
        self.streaming_functions_out = parsed_data.streaming_functions_out;
    }

    pub fn endpoint_down(&mut self) {
        if (self.endpoint_starting_row + 1) < self.summary.len() {
            self.endpoint_starting_row += 1;
        }
    }
    pub fn endpoint_up(&mut self) {
        if self.endpoint_starting_row > 0 {
            self.endpoint_starting_row -= 1;
        }
    }

    pub fn kafka_down(&mut self) {
        if (self.kafka_starting_row + 1) < self.kafka_messages_out_total.len() {
            self.kafka_starting_row += 1;
        }
    }
    pub fn kafka_up(&mut self) {
        if self.kafka_starting_row > 0 {
            self.kafka_starting_row -= 1;
        }
    }

    pub fn streaming_functions_down(&mut self) {
        if (self.streaming_functions_table_starting_row + 1) < self.streaming_functions_in.len() {
            self.streaming_functions_table_starting_row += 1;
        }
    }
    pub fn streaming_functions_up(&mut self) {
        if self.streaming_functions_table_starting_row > 0 {
            self.streaming_functions_table_starting_row -= 1;
        }
    }

    pub fn per_sec_metrics(
        &mut self,
        new_total_requests: f64,
        path_metrics: &Vec<PathMetricsData>,
        path_bytes_hashmap: &HashMap<String, u64>,
        total_bytes_in: &u64,
        total_bytes_out: &u64,
        kafka_messages_in_total: &Vec<(String, String, f64)>,
        streaming_functions_in: &HashMap<String, f64>,
        streaming_functions_out: &HashMap<String, f64>,
    ) {
        self.requests_per_sec = new_total_requests - self.total_requests;
        self.main_bytes_data.bytes_in_per_sec =
            total_bytes_in - self.main_bytes_data.total_bytes_in;
        self.main_bytes_data.bytes_out_per_sec =
            total_bytes_out - self.main_bytes_data.total_bytes_out;

        // Initializes variables and vec for each path in summary to keep unwraps in ui.rs safe
        for path in &self.summary {
            if !self.path_requests_per_sec.contains_key(&path.path) {
                self.path_requests_per_sec
                    .insert(path.path.clone(), path.request_count);
            }

            if !self.requests_per_sec_vec.contains_key(&path.path) {
                self.requests_per_sec_vec
                    .insert(path.path.clone(), vec![0; 150]);
            }

            if !self
                .parsed_bytes_data
                .path_bytes_in_per_sec_vec
                .contains_key(&path.path)
            {
                self.parsed_bytes_data
                    .path_bytes_in_per_sec_vec
                    .insert(path.path.clone(), 0);
            }

            if !self
                .parsed_bytes_data
                .path_bytes_out_per_sec_vec
                .contains_key(&path.path)
            {
                self.parsed_bytes_data
                    .path_bytes_out_per_sec_vec
                    .insert(path.path.clone(), 0);
            }

            for item in path_metrics {
                if item.path == path.path {
                    self.path_requests_per_sec
                        .insert(path.path.clone(), item.request_count - path.request_count);
                }
            }
            self.requests_per_sec_vec
                .get_mut(&path.path)
                .unwrap()
                .push(*self.path_requests_per_sec.get(&path.path).unwrap() as u64);
            while self.requests_per_sec_vec.get(&path.path).unwrap().len() > 150 {
                self.requests_per_sec_vec
                    .get_mut(&path.path)
                    .unwrap()
                    .remove(0);
            }
        }
        for path in &self.parsed_bytes_data.previous_bytes {
            match path_bytes_hashmap.get(path.0) {
                Some(value) => {
                    if path.0.starts_with("ingest/") {
                        self.parsed_bytes_data
                            .path_bytes_in_per_sec_vec
                            .insert(path.0.clone(), *value - path.1);
                    } else {
                        self.parsed_bytes_data
                            .path_bytes_out_per_sec_vec
                            .insert(path.0.clone(), *value - path.1);
                    }
                }
                None => {
                    if path.0.starts_with("ingest/") {
                        self.parsed_bytes_data
                            .path_bytes_in_per_sec_vec
                            .insert(path.0.clone(), 0);
                    } else {
                        self.parsed_bytes_data
                            .path_bytes_out_per_sec_vec
                            .insert(path.0.clone(), 0);
                    }
                }
            }

            for item in kafka_messages_in_total {
                for prev_item in &self.kafka_messages_out_total {
                    if item.0 == prev_item.0 && item.1 == prev_item.1 {
                        self.kafka_messages_out_per_sec
                            .insert(item.0.clone(), (item.1.clone(), item.2 - prev_item.2));
                    }
                }
            }

            for item in streaming_functions_in {
                for prev_item in &self.streaming_functions_in {
                    if item.0 == prev_item.0 {
                        self.streaming_functions_in_per_sec
                            .insert(item.0.clone(), item.1 - prev_item.1);
                    }
                }
            }
            for item in streaming_functions_out {
                for prev_item in &self.streaming_functions_out {
                    if item.0 == prev_item.0 {
                        self.streaming_functions_out_per_sec
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
